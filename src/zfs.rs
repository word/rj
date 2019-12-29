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
    pub fn create(&self) -> Result<()> {
        match self.exists() {
            Ok(true) => {
                println!("Data set {} already exists, skipping", &self.path);
                Ok(())
            }
            Ok(false) => {
                println!("Creating zfs data set {}", &self.path);
                let mut zfs = Command::new("zfs");
                zfs.arg("create");
                if self.mount_point.is_some() {
                    let mount_point = self.mount_point.as_ref();
                    zfs.arg("-o");
                    zfs.arg(format!("mountpoint={}", mount_point.unwrap()));
                }
                zfs.arg(&self.path);
                cmd::run(&mut zfs)?;
                Ok(())
            }
            Err(e) => Err(e)
        }
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
