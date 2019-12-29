use std::process::Command;

use anyhow::Result;
use crate::cmd;

#[derive(Debug)]
pub struct DataSet {
    pub path: String,
    pub mount_point: Option<String>,
}

impl DataSet {

    // create the zfs data set if it doesn't exist already
    pub fn create(&mut self) -> Result<()> {
        match self.exists() {
            Ok(true) => {
                println!("Data set {} already exists, skipping", &self.path);
                Ok(())
            }
            Ok(false) => {
                println!("Creating zfs data set {}", &self.path);
                let mut zfs = Command::new("zfs");
                zfs.arg("create");
                zfs.arg(&self.path);
                cmd::run(&mut zfs)?;

                // either set the mount point if provided or get it if not
                match &self.mount_point {
                    Some(m) => {
                        self.set("mountpoint", m)?;
                    },
                    None => {
                        let mount_point = self.get("mountpoint")?;
                        if mount_point != "none" {
                            self.mount_point = Some(mount_point);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    pub fn set(&self, property: &str, value: &str) -> Result<()> {
        let mut zfs = Command::new("zfs");
        zfs.arg("set");
        zfs.arg(format!("{}={}", property, value));
        zfs.arg(&self.path);
        cmd::run(&mut zfs)?;
        Ok(())
    }

    pub fn get(&self, property: &str) -> Result<String> {
        // zfs get -H -o value mountpoint zroot/jails
        let mut zfs = Command::new("zfs");
        zfs.args(&["get", "-H", "-o", "value"]);
        zfs.arg(property);
        zfs.arg(&self.path);
        cmd::run(&mut zfs)
    }

    // check if data set exists
    fn exists(&self) -> Result<bool> {
        let mut zfs = Command::new("zfs");
        zfs.arg("list").arg(&self.path);

        match cmd::run(&mut zfs) {
            Ok(_) => Ok(true),
            Err(e) => {
                let not_exists = format!("(1) cannot open \'{}\': dataset does not exist\n", &self.path);
                if e.to_string() == not_exists {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }

}
