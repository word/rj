use std::process::Command;

use anyhow::Result;
use crate::cmd;

#[derive(Debug)]
pub struct DataSet {
    pub path: String,
    pub mount_point: String,
}

impl DataSet {

    pub fn new(path: String) -> Self {
        DataSet { path: path, mount_point: String::from("") }
    }

    pub fn create(&self) -> Result<()> {
        match self.exists() {
            Ok(true) => {
                println!("Data set already exists, skipping");
                Ok(())
            }
            Ok(false) => {
                let mut zfs = Command::new("zfs");
                zfs.arg("create").arg(&self.path);
                cmd::run(&mut zfs)?;
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    pub fn exists(&self) -> Result<bool> {
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
