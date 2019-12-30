use std::process::Command;

use anyhow::Result;
use crate::cmd;

#[derive(Debug)]
pub struct DataSet {
    pub path: String,
    pub mountpoint: String,
}

impl DataSet {

    pub fn new(path: String, mountpoint: String) -> Result<Self> {
        let mut ds = DataSet {
            path: String::from(path),
            mountpoint: mountpoint,
        };

        ds.create()?;
        Ok(ds)
    }

    // create the zfs data set if it doesn't exist already
    fn create(&mut self) -> Result<()> {
        match self.exists() {
            Ok(true) => {
                println!("Data set {} already exists, skipping", &self.path);
                Ok(())
            }
            Ok(false) => {
                println!("Creating zfs data set {}", &self.path);
                let mut zfs = Command::new("zfs");
                zfs.arg("create");
                zfs.arg("-o");
                zfs.arg(format!("mountpoint={}", &self.mountpoint));
                zfs.arg(&self.path);
                cmd::run(&mut zfs)?;
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    #[allow(dead_code)]
    pub fn set(&self, property: &str, value: &str) -> Result<()> {
        let mut zfs = Command::new("zfs");
        zfs.arg("set");
        zfs.arg(format!("{}={}", property, value));
        zfs.arg(&self.path);
        cmd::run(&mut zfs)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get(&self, property: &str) -> Result<String> {
        // zfs get -H -o value mountpoint zroot/jails
        let mut zfs = Command::new("zfs");
        zfs.args(&["get", "-H", "-o", "value"]);
        zfs.arg(property);
        zfs.arg(&self.path);
        cmd::run(&mut zfs)
    }

    #[allow(dead_code)]
    pub fn destroy(&self) -> Result<()> {
        let mut zfs = Command::new("zfs");
        zfs.arg("destroy");
        zfs.arg(&self.path);
        cmd::run(&mut zfs)?;
        Ok(())
    }

    // check if data set exists
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

#[cfg(test)]
mod tests {
    // import names from outer (for mod tests) scope.
    use super::*;
    use std::panic;

    fn run_test<T>(test: T) -> Result<()>
        where T: Fn(DataSet) -> Result<()> + panic::RefUnwindSafe
    {
        let ds = DataSet::new(
            "zroot/rjtest".to_string(),
            "/rjtest".to_string(),
        )?;

        let result = panic::catch_unwind(|| {
            test(ds)
        });

        // test(ds)?;
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_ds_create_destroy() -> Result<()> {
        run_test(|ds| {
            assert!(ds.exists()?);
            Ok(())
        })
        // assert!(ds.exists()?);
        // ds.destroy()?;
        // assert!(!ds.exists()?);
        // Ok(())
    }
}
