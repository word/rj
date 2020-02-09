#![allow(dead_code)]
use anyhow::Result;
use indexmap::{indexmap, IndexMap};
use log::{debug, info};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::cmd;
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
}

impl Jail<'_> {
    // read only aliases for attributes
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
    ) -> Jail<'a> {
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
        }
    }

    pub fn create(&self) -> Result<()> {
        if self.exists()? {
            info!("jail '{}' exists already, skipping", self.name());
            return Ok(());
        };

        info!("Creating jail '{}'", self.name());
        // install the jail using whatever source
        self.source.install(&self.mountpoint, &self.zfs_ds)?;
        self.configure()?;
        self.provision()?;

        if self.settings.start {
            self.enable()?;
            self.start()?;
        }

        Ok(())
    }

    pub fn destroy(&self) -> Result<()> {
        if self.exists()? {
            info!("Destroying jail '{}'", &self.name);
        } else {
            info!("Jail '{}' doesn't exist, skipping", &self.name);
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
            debug!("Removing config file: {}", &self.conf_path);
            fs::remove_file(&self.conf_path)?;
        }

        // remove zfs snapshots
        let snaps = self.zfs_ds.list_snaps()?;
        if !(snaps.is_empty()) {
            debug!("Destroying snapshots");
            for snap in snaps {
                self.zfs_ds.snap_destroy(&snap)?;
            }
        }

        // destroy zfs dataset
        self.zfs_ds.destroy()
    }

    pub fn configure(&self) -> Result<()> {
        info!("Writing config to: {}", &self.conf_path);

        // add "path" jail parameter
        let extra_conf = indexmap! {
            "path".to_string() => JailConfValue::String(self.mountpoint.to_string()),
        };

        fs::write(
            &self.conf_path,
            templates::render_jail_conf(
                &self.name,
                &self.conf_defaults,
                &self.settings.conf,
                &extra_conf,
            )?,
        )?;
        Ok(())
    }

    pub fn update(&self) -> Result<()> {
        todo!()
    }

    pub fn rollback(&self) -> Result<()> {
        todo!()
    }

    pub fn start(&self) -> Result<()> {
        info!("Starting jail {}", &self.name);

        let mut service = Command::new("service");
        service.arg("jail").arg("start").arg(&self.name);
        cmd::run(&mut service)?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        info!("Stopping jail {}", &self.name);

        let mut service = Command::new("service");
        service.arg("jail").arg("stop").arg(&self.name);
        cmd::run(&mut service)?;
        Ok(())
    }

    pub fn is_running(&self) -> Result<bool> {
        let output = Command::new("jls").arg("-j").arg(&self.name).output()?;
        Ok(output.status.success())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        // let mut sysrc = Command::new("sysrc");
        // sysrc.arg("-n").arg("jails_list");
        let enabled_jails = cmd::run(&mut Command::new("sysrc").arg("-n").arg("jails_list"))?;
        Ok(enabled_jails.contains(&self.name))
    }

    pub fn enable(&self) -> Result<()> {
        info!("Enabling jail {} in rc.conf", &self.name);

        let mut sysrc = Command::new("sysrc");
        sysrc.arg(format!("jails_list+={}", &self.name));
        cmd::run(&mut sysrc)?;
        Ok(())
    }

    pub fn disable(&self) -> Result<()> {
        info!("Disabling in rc.conf");

        let mut sysrc = Command::new("sysrc");
        sysrc.arg(format!("jails_list-={}", &self.name));
        cmd::run(&mut sysrc)?;
        Ok(())
    }

    pub fn provision(&self) -> Result<()> {
        self.zfs_ds.snap("ready")
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
    use std::path::Path;
    use std::sync::Once;

    static INIT: Once = Once::new();
    lazy_static! {
        static ref S: Settings = Settings::new("config.toml").unwrap();
    }

    // Initialise and create test jails from example config file
    pub fn setup_once<'a>() -> IndexMap<String, Jail<'a>> {
        let jails = S.to_jails().unwrap();

        INIT.call_once(|| {
            // enable log messages for debugging
            // TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed).unwrap();
            // cleanup before all
            // jails_ds.destroy_r().unwrap();

            // Initialise
            crate::init(&S).unwrap();

            // Create test jails
            for (_, jail) in jails.iter() {
                if jail.exists().unwrap() {
                    // Tidy up any jails that may be left over from failed tests.
                    // Leave 'base' around because it takes log time to extract.
                    if jail.name() != "base" {
                        jail.destroy().unwrap();
                        jail.create().unwrap_or_else(|e| {
                            panic!("Failed creating jail {}: {}", &jail.name(), e)
                        });
                    }
                } else {
                    jail.create()
                        .unwrap_or_else(|e| panic!("Failed creating jail {}: {}", &jail.name(), e));
                }
            }
        });
        jails
    }

    #[test]
    fn test_jail_mountpoint() {
        let jails = setup_once();
        assert_eq!(jails["base"].mountpoint, "/jails/base");
    }

    #[test]
    fn test_jail_name() {
        let jails = setup_once();
        assert_eq!(jails["base"].name, "base");
    }

    // Trying to create an already created jail should just skip it without an error.
    #[test]
    fn test_jail_create_existing() {
        let jails = setup_once();
        let result = jails["base"].create();
        assert!(result.is_ok());
    }

    #[test]
    fn test_jail_create_destroy() -> Result<()> {
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
        let jail_conf_path = "/etc/jail.test2.conf";
        assert!(Path::new(jail_conf_path).is_file());
        assert_eq!(ok_jail_conf, fs::read_to_string(jail_conf_path)?);

        // Check jail is enabled in rc.conf
        assert!(jail2.is_enabled().unwrap());
        // let mut sysrc = Command::new("sysrc");
        // sysrc.arg("-n").arg("jails_list");
        // let enabled_jails = cmd::run(&mut sysrc)?;
        // assert!(enabled_jails.contains("test2"));

        // Check that it's running
        assert!(jail2.is_running().unwrap());

        jail2.destroy()?;
        // make sure all resources are cleaned up after destroy
        // check config file is gone
        assert_eq!(Path::new(jail_conf_path).is_file(), false);
        // check jail is disabled in rc.conf
        let mut sysrc = Command::new("sysrc");
        sysrc.arg("-n").arg("jails_list");
        let enabled_jails = cmd::run(&mut sysrc)?;
        assert!(!enabled_jails.contains("test2"));

        jail1.destroy()?;
        let enabled_jails = cmd::run(&mut sysrc)?;
        assert!(!enabled_jails.contains("test1"));

        Ok(())
    }

    #[test]
    fn test_jail_disabled() -> Result<()> {
        setup_once();
        // Check that the base jail isn't enabled in rc.conf
        let mut sysrc = Command::new("sysrc");
        sysrc.arg("-n").arg("-q").arg("jails_list");
        let enabled_jails = cmd::run(&mut sysrc)?;
        assert!(!enabled_jails.contains("base"));

        Ok(())
    }
}