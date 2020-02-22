#![allow(dead_code)]
use anyhow::Result;
use difference::Changeset;
use indexmap::{indexmap, IndexMap};
use log::{debug, info};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::cmd;
use crate::cmd_capture;
use crate::provisioner::Provisioner;
use crate::settings;
use crate::source::Source;
use crate::templates;
use crate::zfs;

use settings::{JailConfValue, JailSettings};

pub struct Jail<'a> {
    name: String,
    mountpoint: String,
    source: &'a Source,
    zfs_ds_path: String,
    zfs_ds: zfs::DataSet,
    settings: &'a JailSettings,
    conf_defaults: &'a IndexMap<String, JailConfValue>,
    conf_path: String,
    provisioners: Vec<&'a Provisioner>,
}

impl Jail<'_> {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn mountpoint(&self) -> &String {
        &self.mountpoint
    }

    pub fn new<'a>(
        ds_path: &str,
        source: &'a Source,
        settings: &'a JailSettings,
        conf_defaults: &'a IndexMap<String, JailConfValue>,
        provisioners: Vec<&'a Provisioner>,
    ) -> Jail<'a> {
        // Workout jail name from data set path (fixme: just get it from settings)
        let mut components: Vec<&str> = ds_path.split('/').collect();
        components.remove(0); // remove the zfs pool name
        let name = (*components.last().unwrap()).to_string();

        Jail {
            name: name.clone(),
            mountpoint: format!("/{}", components.join("/")),
            source,
            zfs_ds_path: ds_path.to_string(),
            zfs_ds: zfs::DataSet::new(ds_path),
            settings,
            conf_defaults,
            conf_path: format!("/etc/jail.{}.conf", name),
            provisioners,
        }
    }

    pub fn apply(&self) -> Result<()> {
        info!("{}: applying changes", self.name());

        if !self.exists()? {
            self.install()?;
        }
        self.configure()?;
        if self.settings.start {
            if !(self.is_enabled()?) {
                info!("{}: disabled -> enabled", &self.name);
                self.enable()?
            }
            if !(self.is_running()?) {
                info!("{}: stopped -> running", &self.name);
                self.start()?
            }
        }

        // Only provision is not already provisioned or if previous provisioning attempt did not succeed.
        if self.zfs_ds.last_snap("ready")?.is_none() {
            self.provision()?;
        }
        Ok(())
    }

    pub fn destroy(&self) -> Result<()> {
        if self.exists()? {
            info!("{}: destroying", &self.name);
        } else {
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
        if Path::new(&self.conf_path).is_file() {
            debug!("{}: removing config file: {}", &self.name, &self.conf_path);
            fs::remove_file(&self.conf_path)?;
        }

        // remove zfs snapshots
        let snaps = self.zfs_ds.list_snaps()?;
        if !(snaps.is_empty()) {
            debug!("{}: destroying snapshots", &self.name);
            for snap in snaps {
                self.zfs_ds.snap_destroy(&snap)?;
            }
        }

        // destroy zfs dataset
        self.zfs_ds.destroy()
    }

    pub fn install(&self) -> Result<()> {
        self.source.install(&self.mountpoint, &self.zfs_ds)
    }

    pub fn configure(&self) -> Result<()> {
        // add any additional config params
        let extra_conf = indexmap! {
            "path".to_string() => JailConfValue::String(self.mountpoint.to_string()),
        };

        let rendered = templates::render_jail_conf(
            &self.name,
            &self.conf_defaults,
            &self.settings.conf,
            &extra_conf,
        )?;

        // check if a config file exists already
        if Path::is_file(Path::new(&self.conf_path)) {
            let current = fs::read_to_string(&self.conf_path)?;
            if current != rendered {
                let diff = Changeset::new(&current, &rendered, "");
                info!("{}: updating {}\n{}", &self.name, &self.conf_path, &diff);
                fs::write(&self.conf_path, &rendered)?;
            }
        } else {
            info!("{}: creating {}", &self.name, &self.conf_path);
            fs::write(&self.conf_path, &rendered)?;
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
        info!("{}: starting", &self.name);
        cmd!("service", "jail", "start", &self.name)
    }

    pub fn stop(&self) -> Result<()> {
        info!("{}: stopping", &self.name);
        cmd!("service", "jail", "stop", &self.name)
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
        info!("{}: enabling in rc.conf", &self.name);
        let arg = format!("jail_list+={}", &self.name);
        cmd!("sysrc", arg)
    }

    fn disable(&self) -> Result<()> {
        info!("{}: disabling in rc.conf", &self.name);
        let arg = format!("jail_list-={}", &self.name);
        cmd!("sysrc", arg)
    }

    pub fn provision(&self) -> Result<()> {
        for p in self.provisioners.iter() {
            p.provision(&self)?;
        }
        self.zfs_ds.snap_with_time("ready")
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
    use settings::Settings;
    // use simplelog::*;
    use serial_test::serial;
    use std::fs;
    use std::path::Path;
    use std::sync::Once;

    static INIT: Once = Once::new();
    lazy_static! {
        static ref S: Settings = Settings::new("testdata/config.toml").unwrap();
    }

    // Initialise and create test jails from an example config file
    pub fn setup_once<'a>() -> IndexMap<String, Jail<'a>> {
        let jails = S.to_jails().unwrap();

        INIT.call_once(|| {
            // enable log messages for debugging
            // TermLogger::init(LevelFilter::Debug, Config::default(),
            // TerminalMode::Mixed).unwrap();

            // Initialise
            crate::init(&S).unwrap();

            // clean up test jails that may be left over from a failed run
            if jails["test1"].exists().unwrap() {
                jails["test1"].destroy().unwrap();
            }
            if jails["test2"].exists().unwrap() {
                jails["test2"].destroy().unwrap();
            }

            if !(jails["base"].exists().unwrap()) {
                jails["base"].apply().unwrap();
            }
            jails["test1"].apply().unwrap();
            jails["test2"].apply().unwrap();
        });
        jails
    }

    #[test]
    fn mountpoint() {
        let jails = setup_once();
        assert_eq!(jails["base"].mountpoint, "/jails/base");
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

        // Check jail is enabled in rc.conf
        assert!(jail1.is_enabled()?);
        assert!(jail2.is_enabled()?);

        // Check that it's running
        assert!(jail1.is_running()?);
        assert!(jail2.is_running()?);

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
        fs::write(&jail2.conf_path, "local change")?;
        jail2.apply()?;
        assert_eq!(ok_jail_conf, fs::read_to_string(jail2_conf_path)?);

        // make sure all resources are cleaned up after destroy
        jail1.destroy()?;
        // check config file is gone
        assert_eq!(Path::new(jail1_conf_path).is_file(), false);
        // check jail is disabled in rc.conf
        assert_eq!(jail1.is_enabled()?, false);
        // check jail is dead
        assert_eq!(jail1.is_running()?, false);

        jail2.destroy()?;
        assert_eq!(Path::new(jail2_conf_path).is_file(), false);
        assert_eq!(jail2.is_enabled()?, false);
        assert_eq!(jail2.is_running()?, false);

        Ok(())
    }

    #[test]
    fn base_disabled() -> Result<()> {
        let jails = setup_once();
        let basejail = &jails["base"];
        assert_eq!(basejail.is_enabled().unwrap(), false);
        assert_eq!(basejail.is_running().unwrap(), false);
        Ok(())
    }
}
