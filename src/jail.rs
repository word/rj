#![allow(dead_code)]
use anyhow::Result;
use indexmap::IndexMap;
use log::info;
use std::fs;

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
}

impl Jail<'_> {
    // read only aliases for attributes
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn mountpoint(&self) -> &String {
        &self.mountpoint
    }

    pub fn order(&self) -> &i16 {
        &self.settings.order
    }

    pub fn new<'a>(
        ds_path: &str,
        source: &'a Source,
        settings: &'a JailSettings,
        conf_defaults: &'a IndexMap<String, JailConfValue>,
    ) -> Jail<'a> {
        let mut components: Vec<&str> = ds_path.split('/').collect();
        components.remove(0); // remove the zfs pool name

        Jail {
            name: components.last().unwrap().to_string(),
            mountpoint: format!("/{}", components.join("/")),
            source,
            zfs_ds_path: ds_path.to_string(),
            zfs_ds: zfs::DataSet::new(ds_path),
            settings,
            conf_defaults,
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
        self.provision()
    }

    pub fn destroy(&self) -> Result<()> {
        if self.exists()? {
            info!("Destroying jail '{}'", &self.name);
        } else {
            info!("Jail '{}' doesn't exist, skipping", &self.name);
            return Ok(());
        }

        let snaps = self.zfs_ds.list_snaps()?;
        if !(snaps.is_empty()) {
            for snap in snaps {
                self.zfs_ds.snap_destroy(&snap)?;
            }
        }
        self.zfs_ds.destroy()
    }

    pub fn configure(&self) -> Result<()> {
        let conf =
            templates::render_jail_conf(&self.name, &self.conf_defaults, &self.settings.conf)?;
        let conf_path = format!("/etc/jail.{}.conf", &self.name);

        fs::write(conf_path, conf)?;

        Ok(())
    }

    pub fn update() {}
    pub fn start() {}
    pub fn stop() {}
    pub fn enable() {}
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
    use std::path::Path;
    use std::sync::Once;

    static INIT: Once = Once::new();
    lazy_static! {
        static ref S: Settings = Settings::new("config.toml").unwrap();
    }

    pub fn setup_once<'a>() -> Jail<'a> {
        // Setup the basejail
        let jails_ds = zfs::DataSet::new("zroot/jails");
        let basejail = Jail::new(
            "zroot/jails/basejail",
            &S.source["freebsd12"],
            &S.jail["base"],
            &S.jail_conf_defaults,
        );

        INIT.call_once(|| {
            // cleanup before all
            // jails_ds.destroy_r().unwrap();
            jails_ds.create().unwrap();
            jails_ds.set("mountpoint", "/jails").unwrap();
            if !(basejail.exists().unwrap()) {
                basejail.create().unwrap();
            }
        });

        basejail
    }

    #[test]
    fn test_jail_mountpoint() {
        let basejail = setup_once();
        assert_eq!(basejail.mountpoint, "/jails/basejail");
    }

    #[test]
    fn test_jail_name() {
        let basejail = setup_once();
        assert_eq!(basejail.name, "basejail");
    }

    #[test]
    fn test_jail_thin_create_destroy() -> Result<()> {
        setup_once();
        let jail = Jail::new(
            "zroot/jails/thin",
            &S.source["base"],
            &S.jail["test1"],
            &S.jail_conf_defaults,
        );
        jail.create()?;
        assert!(jail.exists()?);
        jail.destroy()?;
        assert!(!jail.exists()?);
        Ok(())
    }

    #[test]
    fn test_jail_create_existing() {
        let basejail = setup_once();
        let result = basejail.create();
        assert!(result.is_ok());
    }

    #[test]
    fn test_jail_configure() -> Result<()> {
        setup_once();
        let jail = Jail::new(
            "zroot/jails/test2",
            &S.source["base"],
            &S.jail["test2"],
            &S.jail_conf_defaults,
        );
        jail.create()?;
        jail.configure()?;

        let ok_jail_conf = indoc!(
            r#"
            exec.start = "/bin/sh /etc/rc";
            exec.stop = "/bin/sh /etc/rc.shutdown";
            exec.clean = true;
            mount.devfs = true;

            test2 {
                host.hostname = "test2.jail";
                allow.set_hostname = 1;
                allow.raw_sockets = 1;
                ip4.addr = "lo0|10.11.11.2/32";
                ip4.addr += "lo0|10.23.23.2/32";
                allow.mount = true;
            }"#
        );
        let jail_conf_path = "/etc/jail.test2.conf";

        assert!(Path::new(jail_conf_path).is_file());
        assert_eq!(ok_jail_conf, fs::read_to_string(jail_conf_path)?);

        jail.destroy()?;
        Ok(())
    }
}
