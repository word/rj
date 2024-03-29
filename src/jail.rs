#![allow(dead_code)]
use crate::cmd;
use crate::cmd_capture;
use crate::provisioner::Provisioner;
use crate::settings;
use crate::source::Source;
use crate::template::fstab::Fstab;
use crate::template::jail_conf::JailConf;
use crate::volumes::Volume;
use crate::zfs;
use anyhow::Result;
use askama::Template;
use difference::Changeset;
use indexmap::{indexmap, IndexMap};
use log::info;
use settings::{JailConfValue, JailSettings};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

enum Change {
    Created,
    Modified,
    None,
}

#[derive(Clone, Debug)]
pub struct Jail<'a> {
    jail_conf_defaults: &'a IndexMap<String, JailConfValue>,
    jail_conf_path: PathBuf,
    fstab_path: PathBuf,
    mountpoint: PathBuf,
    name: String,
    noop: &'a bool,
    noop_suffix: String,
    provisioners: Vec<&'a Provisioner>,
    jail_settings: &'a JailSettings,
    source: &'a Source,
    volumes: Vec<&'a Volume>,
    zfs_ds: zfs::DataSet,
    zfs_ds_path: PathBuf,
}

impl Jail<'_> {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn mountpoint(&self) -> &PathBuf {
        &self.mountpoint
    }

    pub fn zfs_ds(&self) -> &zfs::DataSet {
        &self.zfs_ds
    }

    pub fn noop(&self) -> &bool {
        &self.noop
    }

    pub fn noop_suffix(&self) -> &String {
        &self.noop_suffix
    }

    pub fn new<'a>(
        name: &str,
        jails_mountpoint: &Path,
        jails_dataset: &Path,
        source: &'a Source,
        jail_settings: &'a JailSettings,
        jail_conf_defaults: &'a IndexMap<String, JailConfValue>,
        provisioners: Vec<&'a Provisioner>,
        noop: &'a bool,
        volumes: Vec<&'a Volume>,
    ) -> Jail<'a> {
        //
        // Set the noop suffix which is displayed in log messages when noop is set

        let zfs_dataset_path = jails_dataset.join(name);

        Jail {
            name: name.to_owned(),
            mountpoint: jails_mountpoint.join(name),
            source,
            zfs_ds_path: zfs_dataset_path.to_owned(),
            zfs_ds: zfs::DataSet::new(&zfs_dataset_path),
            jail_settings,
            jail_conf_defaults,
            jail_conf_path: PathBuf::from(format!("/etc/jail.{}.conf", name)),
            fstab_path: PathBuf::from(format!("/etc/fstab.{}", name)),
            provisioners,
            noop,
            noop_suffix: Self::make_noop_suffix(noop),
            volumes,
        }
    }

    fn make_noop_suffix(noop: &bool) -> String {
        let mut noop_suffix = String::new();
        if *noop {
            noop_suffix = " (noop)".to_owned();
        }
        noop_suffix
    }

    pub fn apply(&self) -> Result<()> {
        info!("{}: applying changes", self.name());

        let mut restart = false;

        if !self.exists()? {
            self.install()?;
        }

        if let Change::Modified = self.configure()? {
            restart = true;
        }

        // FIXME - what if all volumes are removed?
        if !self.volumes.is_empty() {
            self.write_fstab()?;
        }

        if self.jail_settings.enable {
            if !(self.is_enabled()?) {
                self.enable()?
            }
        } else {
            if self.is_enabled()? {
                self.disable()?
            }
        }

        if self.jail_settings.start {
            if !(self.is_running()?) {
                self.start()?
            }
        } else {
            if self.is_running()? {
                self.stop()?
            }
        }

        self.provision()?;

        if self.is_running()? && restart {
            self.stop()?;
            self.start()?;
        }

        Ok(())
    }

    pub fn destroy(&self) -> Result<()> {
        if !self.exists()? {
            info!("{}: doesn't exist, skipping", &self.name);
            return Ok(());
        }

        if self.is_running()? {
            self.stop()?;
        }

        // disable in rc.conf
        if self.is_enabled()? {
            self.disable()?;
        }

        // remove jail config file
        if Path::new(&self.jail_conf_path).is_file() {
            info!(
                "{}: removing config file: {}{}",
                &self.name,
                &self.jail_conf_path.display(),
                &self.noop_suffix
            );
            if !self.noop {
                fs::remove_file(&self.jail_conf_path)?;
            }
        }

        // remove fstab
        if Path::new(&self.fstab_path).is_file() {
            info!(
                "{}: removing fstab: {}{}",
                &self.name,
                &self.fstab_path.display(),
                &self.noop_suffix
            );
            if !self.noop {
                fs::remove_file(&self.fstab_path)?;
            }
        }

        // remove zfs snapshots
        let snaps = self.zfs_ds.list_snaps()?;
        if !(snaps.is_empty()) {
            info!("{}: destroying snapshots{}", &self.name, &self.noop_suffix);
            if !self.noop {
                for snap in snaps {
                    self.zfs_ds.snap_destroy(&snap)?;
                }
            }
        }

        // destroy zfs dataset
        info!("{}: destroying dataset{}", &self.name, &self.noop_suffix);
        if !self.noop {
            self.zfs_ds.destroy()?;
        }
        Ok(())
    }

    fn install(&self) -> Result<()> {
        self.source.install(&self)
    }

    fn configure(&self) -> Result<Change> {
        // add any additional config params
        let mut extra_conf = indexmap! {
            "path".to_owned() => JailConfValue::Path(self.mountpoint.to_owned()),
        };

        if !&self.volumes.is_empty() {
            extra_conf.insert(
                "mount.fstab".to_owned(),
                JailConfValue::Path(self.fstab_path.to_owned()),
            );
        }

        let jail_conf_template = JailConf::new(
            &self.name,
            &self.jail_conf_defaults,
            &self.jail_settings.conf,
            &extra_conf,
        )?;
        let rendered = jail_conf_template.render()?;
        let mut change = Change::None;

        // FIXME - DRY this up
        if self.jail_conf_path.is_file() {
            let current = fs::read_to_string(&self.jail_conf_path)?;
            if current != rendered {
                change = Change::Modified;
                let diff = Changeset::new(&current, &rendered, "");
                info!(
                    "{}: updating {}{}\n{}",
                    &self.name,
                    &self.jail_conf_path.display(),
                    &self.noop_suffix,
                    &diff
                );
            }
        } else {
            change = Change::Created;
            info!(
                "{}: creating {}{}",
                &self.name,
                &self.jail_conf_path.display(),
                &self.noop_suffix
            );
        }

        if !self.noop {
            fs::write(&self.jail_conf_path, &rendered)?;
        }

        Ok(change)
    }

    fn write_fstab(&self) -> Result<()> {
        let fstab = Fstab {
            volumes: &self.volumes,
            jail_mountpoint: &self.mountpoint,
        };
        let rendered = fstab.render()?;

        // FIXME - DRY this up
        if self.fstab_path.is_file() {
            let current = fs::read_to_string(&self.fstab_path)?;
            if current != rendered {
                let diff = Changeset::new(&current, &rendered, "");
                info!(
                    "{}: updating {}{}\n{}",
                    &self.name,
                    &self.fstab_path.display(),
                    &self.noop_suffix,
                    &diff
                );
            }
        } else {
            info!(
                "{}: creating {}{}",
                &self.name,
                &self.fstab_path.display(),
                &self.noop_suffix
            );
        }

        if !self.noop {
            fs::write(&self.fstab_path, &rendered)?;
        }

        Ok(())
    }

    pub fn upgrade(&self) -> Result<()> {
        todo!()
    }

    pub fn rollback(&self) -> Result<()> {
        todo!()
    }

    pub fn start(&self) -> Result<()> {
        info!("{}: starting{}", &self.name, &self.noop_suffix);
        if !self.noop {
            cmd!("service", "jail", "start", &self.name)?;
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        info!("{}: stopping{}", &self.name, &self.noop_suffix);
        if !self.noop {
            cmd!("service", "jail", "stop", &self.name)?;
        }
        Ok(())
    }

    pub fn is_running(&self) -> Result<bool> {
        let output = Command::new("jls").arg("-j").arg(&self.name).output()?;
        Ok(output.status.success())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        let enabled_jails = cmd_capture!("sysrc", "-n", "jail_list")?;
        Ok(enabled_jails.contains(&self.name))
    }

    fn enable(&self) -> Result<()> {
        info!("{}: enabling in rc.conf{}", &self.name, &self.noop_suffix);
        let arg = format!("jail_list+={}", &self.name);
        if !self.noop {
            cmd!("sysrc", arg)?;
        }
        Ok(())
    }

    fn disable(&self) -> Result<()> {
        info!("{}: disabling in rc.conf{}", &self.name, &self.noop_suffix);
        let arg = format!("jail_list-={}", &self.name);
        if !self.noop {
            cmd!("sysrc", arg)?;
        }
        Ok(())
    }

    pub fn provision(&self) -> Result<()> {
        if !self.provisioners.is_empty() {
            self.snap("pre-provision")?;

            if !self.is_running()? {
                self.start()?;
            }
        }

        info!("{}: provisioning{}", &self.name, &self.noop_suffix);
        // TODO - in case of noop before the jail exists, show a list of provisioners that would be run.
        if self.exists()? {
            for p in self.provisioners.iter() {
                // provisioners implement noop themselves
                p.provision(&self)?;
            }
        }

        self.snap("ready")?;

        if self.jail_settings.stop_after {
            if self.is_running()? {
                self.stop()?;
            }
        }

        Ok(())
    }

    fn snap(&self, snap_name: &str) -> Result<()> {
        info!(
            "{}: creating '{}' snapshot{}",
            &self.name, snap_name, &self.noop_suffix
        );
        if !self.noop {
            self.zfs_ds.snap_with_time(snap_name)?;
        }
        Ok(())
    }

    pub fn exists(&self) -> Result<bool> {
        self.zfs_ds.exists()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use indoc::indoc;
    use lazy_static::lazy_static;
    use pretty_assertions::assert_eq;
    use regex::Regex;
    use serial_test::serial;
    use settings::Settings;
    // use simplelog::*;
    use std::fs;
    use std::path::Path;
    use std::sync::Once;

    static INIT: Once = Once::new();
    lazy_static! {
        static ref S: Settings = Settings::new("testdata/config.toml", false).unwrap();
    }

    // Initialise and create test jails from an example config file
    pub fn setup_once<'a>() -> IndexMap<String, Jail<'a>> {
        let jails = S.to_jails().unwrap();

        INIT.call_once(|| {
            // enable log messages for debugging
            // TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed).unwrap();

            // Initialise
            crate::init(&S).unwrap();

            // clean up test jails that may be left over from a failed run
            if jails["test1"].exists().unwrap() {
                jails["test1"].destroy().unwrap();
            }
            if jails["test2"].exists().unwrap() {
                jails["test2"].destroy().unwrap();
            }
            if jails["stopped"].exists().unwrap() {
                jails["stopped"].destroy().unwrap();
            }

            jails["base"].apply().unwrap();
            jails["test1"].apply().unwrap();
            jails["test2"].apply().unwrap();
            jails["stopped"].apply().unwrap();
        });
        jails
    }

    #[test]
    fn mountpoint() {
        let jails = setup_once();
        assert_eq!(jails["base"].mountpoint, PathBuf::from("/jails/base"));
    }

    #[test]
    fn name() {
        let jails = setup_once();
        assert_eq!(jails["base"].name, "base");
    }

    #[test]
    #[serial]
    fn apply_and_destroy() -> Result<()> {
        let jails = setup_once();
        let jail1 = &jails["test1"];
        let jail2 = &jails["test2"];

        assert!(jail1.exists()?);

        // Check jail conf is created correctly
        let ok_jail_conf = indoc!(
            r#"
            exec.start = "/bin/sh /etc/rc";
            exec.stop = "/bin/sh /etc/rc.shutdown";
            exec.clean = true;
            mount.devfs = true;

            test2 {
                path = "/jails/test2";
                host.hostname = "test2.jail";
                allow.set_hostname = 1;
                allow.raw_sockets = 1;
                ip4.addr = "lo0|10.11.11.2/32";
                ip4.addr += "lo0|10.23.23.2/32";
                allow.mount = true;
            }
            "#
        );
        let jail1_conf_path = "/etc/jail.test1.conf";
        let jail2_conf_path = "/etc/jail.test2.conf";
        assert!(Path::new(jail1_conf_path).is_file());
        assert!(Path::new(jail2_conf_path).is_file());
        assert_eq!(ok_jail_conf, fs::read_to_string(jail2_conf_path)?);

        // check volumes are set up correctly
        let ok_fstab = indoc!(
            r#"
            /usr/local/share/examples /jails/test1/mnt nullfs rw 0 0
            /usr/local/share/examples /jails/test1/media nullfs ro 0 0
            "#
        );
        let jail1_fstab_path = Path::new(&jail1.fstab_path);
        assert!(jail1_fstab_path.is_file());
        assert_eq!(ok_fstab, fs::read_to_string(jail1_fstab_path)?);

        // Check jail is enabled in rc.conf
        assert!(jail1.is_enabled()?);
        assert!(jail2.is_enabled()?);

        // Check that it's running
        assert!(jail1.is_running()?);
        assert!(jail2.is_running()?);

        // check the volumes are mounted
        let mounted_filesystems = cmd_capture!("mount")?;
        assert!(mounted_filesystems
            .contains("/usr/local/share/examples on /jails/test1/mnt (nullfs, local)"));
        assert!(mounted_filesystems.contains(
            "/usr/local/share/examples on /jails/test1/media (nullfs, local, read-only)"
        ));

        // Check that snapshots were created
        let re_preprov = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{3}_pre-provision$")?;
        let re_ready = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{3}_ready$")?;

        // jail1 doesn't have any provisioners configured so should not contain a pre-provision snapshot
        for snap in jail1.zfs_ds.list_snaps()?.iter() {
            assert_eq!(re_preprov.is_match(snap), false);
        }

        // jail2 should contain both snapshots in correct order
        let s = jail2.zfs_ds.list_snaps()?;
        let mut snaps_iter = s.iter();
        for re in &[re_preprov, re_ready] {
            assert!(re.is_match(snaps_iter.next().unwrap()));
        }

        // Test updating and correcting drift
        // Stopped
        jail2.stop()?;
        jail2.apply()?;
        assert!(jail2.is_running()?);
        // Disabled
        jail2.disable()?;
        jail2.apply()?;
        assert!(jail2.is_enabled()?);
        // Config changes
        fs::write(&jail2.jail_conf_path, "local change")?;
        jail2.apply()?;
        assert_eq!(ok_jail_conf, fs::read_to_string(jail2_conf_path)?);

        // make sure all resources are cleaned up after destroy
        jail1.destroy()?;
        // check the ds is gone
        assert_eq!(jail1.exists()?, false);
        // check config file is gone
        assert_eq!(Path::new(jail1_conf_path).is_file(), false);
        // check jail is disabled in rc.conf
        assert_eq!(jail1.is_enabled()?, false);
        // check jail is dead
        assert_eq!(jail1.is_running()?, false);

        jail2.destroy()?;
        assert_eq!(jail2.exists()?, false);
        assert_eq!(Path::new(jail2_conf_path).is_file(), false);
        assert_eq!(jail2.is_enabled()?, false);
        assert_eq!(jail2.is_running()?, false);

        Ok(())
    }

    #[test]
    #[serial]
    fn noop_apply_and_destroy() -> Result<()> {
        // set up noop jail
        let s_noop = Settings::new("testdata/config.toml", true)?;
        let jails_noop = s_noop.to_jails()?;
        let jail_noop = &jails_noop["test2"];

        // set up same jail without noop (so we can create one for testing destroy)
        let s = Settings::new("testdata/config.toml", false)?;
        let jails = s.to_jails()?;
        let jail = &jails["test2"];

        // test noop apply
        jail_noop.apply()?;
        assert_eq!(jail_noop.exists()?, false);
        assert_eq!(Path::new("/etc/jail.test2.conf").is_file(), false);
        assert_eq!(jail_noop.is_enabled()?, false);
        assert_eq!(jail_noop.is_running()?, false);

        // actually create it this time
        jail.apply()?;

        // test noop destroy - should have no effect
        jail_noop.destroy()?;
        assert!(jail_noop.exists()?);
        assert!(Path::new("/etc/jail.test2.conf").is_file());
        assert!(jail_noop.is_enabled()?);
        assert!(jail_noop.is_running()?);

        // cleanup
        jail.destroy()
    }

    #[test]
    fn stop_after() -> Result<()> {
        let jails = setup_once();
        let basejail = &jails["base"];
        assert_eq!(basejail.is_enabled().unwrap(), false);
        assert_eq!(basejail.is_running().unwrap(), false);
        Ok(())
    }

    #[test]
    #[serial]
    fn stopped() -> Result<()> {
        let jails = setup_once();
        let jail = &jails["stopped"];
        // should be disabled and stopped
        assert_eq!(jail.is_enabled().unwrap(), false);
        assert_eq!(jail.is_running().unwrap(), false);

        // enable and start the jail
        jail.enable()?;
        jail.start()?;
        jail.apply()?;
        // the apply should disable and stop it
        assert_eq!(jail.is_enabled().unwrap(), false);
        assert_eq!(jail.is_running().unwrap(), false);
        Ok(())
    }

    #[test]
    fn make_noop_suffix() -> () {
        assert_eq!(Jail::make_noop_suffix(&true), String::from(" (noop)"));
        assert_eq!(Jail::make_noop_suffix(&false), String::new());
    }
}
