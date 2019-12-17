use std::process::Command;
use anyhow::Result;
use crate::cmd;

pub fn ds_exist(ds: &str) -> Result<bool> {
    let mut zfs = Command::new("zfs");
    zfs.arg("list").arg(&ds);

    match cmd::run(&mut zfs) {
        Ok(_) => Ok(true),
        Err(e) => {
            let not_exists = format!("cannot open \'{}\': dataset does not exist\n", &ds);
            if e.to_string() == not_exists {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}

pub fn create_ds(ds: &str) -> Result<()> {
    match ds_exist(&ds) {
        Ok(true) => {
            println!("Data set already exists, skipping");
            Ok(())
        }
        Ok(false) => {
            let mut zfs = Command::new("zfs");
            zfs.arg("create").arg(&ds);
            cmd::run(&mut zfs)?;
            Ok(())
        }
        Err(e) => Err(e)
    }
}
