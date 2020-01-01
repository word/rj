use std::process::Command;

use crate::cmd;
use anyhow::Result;

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
    // returns false if skipped because it exists already otherwise returns true
    fn create(&mut self) -> Result<bool> {
        match self.exists() {
            Ok(true) => {
                println!("Data set {} already exists, skipping", &self.path);
                Ok(false)
            }
            Ok(false) => {
                println!("Creating zfs data set {}", &self.path);
                let mut zfs = Command::new("zfs");
                zfs.arg("create");
                zfs.arg("-o");
                zfs.arg(format!("mountpoint={}", &self.mountpoint));
                zfs.arg(&self.path);
                cmd::run(&mut zfs)?;
                Ok(true)
            }
            Err(e) => Err(e),
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
        match cmd::run(&mut zfs) {
            Ok(out) => {
                let mut trim = out;
                trim.pop();
                Ok(trim)
            }
            Err(e) => Err(e),
        }
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
                let not_exists = format!(
                    "(1) cannot open \'{}\': dataset does not exist\n",
                    &self.path
                );
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
    // import names from outer scope.
    use super::*;
    use std::panic::{self, AssertUnwindSafe};
    use pretty_assertions::assert_eq;

    fn run_test<T, R>(test: T) -> Result<()>
    where
        T: Fn(&mut DataSet) -> Result<R>,
        R: std::fmt::Debug,
    {

        // Set up
        let mut ds = DataSet::new("zroot/rjtest".to_string(), "/rjtest".to_string())?;

        // Run the test closure
        let result = panic::catch_unwind(AssertUnwindSafe(|| test(&mut ds))).unwrap();

        // Teardown
        assert!(ds.destroy().is_ok());

        // Check result
        println!("{:?}", result);
        assert!(&result.is_ok());

        Ok(())
    }

    #[test]
    fn test_ds_create() -> () {
        run_test(|ds| ds.exists()).unwrap()
    }

    #[test]
    #[should_panic]
    fn test_ds_double_destroy() -> () {
        run_test(|ds| ds.destroy()).unwrap()
    }

    #[test]
    fn test_ds_set() -> () {
        run_test(|ds| {
            ds.set("atime", "off")?;
            assert_eq!(ds.get("atime")?, "off");
            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic]
    fn test_ds_invalid_set() -> () {
        run_test(|ds| ds.set("noexist", "nope")).unwrap()
    }

    #[test]
    fn test_ds_already_exists() -> () {
        run_test(|ds| {
            let result = ds.create();
            // should not error
            assert!(result.is_ok());
            // should return false if DS already exists
            assert!(!result.unwrap());
            Ok(())
        })
        .unwrap()
    }
}
