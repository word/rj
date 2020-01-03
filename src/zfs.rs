use std::process::Command;

use crate::cmd;
use anyhow::Result;

#[derive(Debug)]
pub struct DataSet {
    path: String,
}

impl DataSet {
    pub fn new(path: &str) -> Result<Self> {
        let mut ds = DataSet {
            path: path.to_string(),
        };

        ds.create()?;
        Ok(ds)
    }

    // create the zfs data set if it doesn't exist already
    // returns false if it already exists otherwise returns true
    fn create(&mut self) -> Result<bool> {
        match self.exists()? {
            true => {
                println!("Data set {} already exists, skipping", &self.path);
                Ok(false)
            }
            false => {
                println!("Creating zfs data set {}", &self.path);
                let mut zfs = Command::new("zfs");
                zfs.arg("create");
                zfs.arg(&self.path);
                cmd::run(&mut zfs)?;
                Ok(true)
            }
        }
    }

    pub fn get_path(&self) -> &String {
        &self.path
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

    // destroy recursively
    #[allow(dead_code)]
    pub fn destroy_r(&self) -> Result<()> {
        let mut zfs = Command::new("zfs");
        zfs.arg("destroy");
        zfs.arg("-r");
        zfs.arg(&self.path);
        cmd::run(&mut zfs)?;
        Ok(())
    }

    // Todo - shuld err if it already exists
    #[allow(dead_code)]
    pub fn snap(&self, snap_name: &str) -> Result<()> {
        let snap_path = format!("{}@{}", &self.path, snap_name);
        println!("Creating snapshot: {}", &snap_path);
        let mut zfs = Command::new("zfs");
        zfs.arg("snapshot");
        zfs.arg(&snap_path);
        cmd::run(&mut zfs)?;
        Ok(())
    }

    pub fn exists(&self) -> Result<bool> {
        self.ds_exists(&self.path)
    }

    #[allow(dead_code)]
    pub fn snap_exists(&self, snap_name: &str) -> Result<bool> {
        self.ds_exists(&format!("{}@{}", &self.path, snap_name))
    }

    // checks if data set exists
    fn ds_exists(&self, ds_path: &str) -> Result<bool> {
        let mut zfs = Command::new("zfs");
        zfs.arg("list").arg(&ds_path);

        match cmd::run(&mut zfs) {
            Ok(_) => Ok(true),
            Err(e) => {
                let not_exists =
                    format!("(1) cannot open \'{}\': dataset does not exist\n", &ds_path);
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
    use pretty_assertions::assert_eq;
    use std::panic::{self, AssertUnwindSafe};

    fn run_test<T, R>(test: T) -> Result<()>
    where
        T: Fn(&mut DataSet) -> Result<R>,
        R: std::fmt::Debug,
    {
        // Set up
        let mut ds = DataSet::new("zroot/rjtest")?;

        // Run the test closure but catch the panic so that the teardown section below can run.
        let result = panic::catch_unwind(AssertUnwindSafe(|| test(&mut ds))).unwrap();

        // Teardown
        let destroy_result = ds.destroy_r();
        // Output is displayed in case of panic
        println!("{:?}", &destroy_result);
        assert!(destroy_result.is_ok());

        // Check result
        println!("{:?}", &result);
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_ds_create() -> Result<()> {
        run_test(|ds| ds.exists())
    }

    #[test]
    fn test_ds_double_destroy() -> () {
        let ds = DataSet::new("zroot/rjtest").unwrap();
        assert!(ds.destroy().is_ok());
        assert!(ds.destroy().is_err());
    }

    #[test]
    fn test_ds_set() -> Result<()> {
        run_test(|ds| {
            ds.set("atime", "off")?;
            assert_eq!(ds.get("atime")?, "off");
            Ok(())
        })
    }

    #[test]
    fn test_ds_invalid_set() -> Result<()> {
        run_test(|ds| {
            assert!(ds.set("noexist", "nope").is_err());
            Ok(())
        })
    }

    // Creating a DS that already exists shoudn't fail.  It becomes a representation of the
    // existing DS.
    #[test]
    fn test_ds_already_exists() -> Result<()> {
        run_test(|ds| {
            let result = ds.create();
            // should not error
            assert!(result.is_ok());
            // should return false if DS already exists
            assert!(!result.unwrap());
            Ok(())
        })
    }

    #[test]
    fn test_ds_invalid_path() -> () {
        assert!(DataSet::new("noexist/rjtest").is_err());
    }

    #[test]
    fn test_snap() -> Result<()> {
        run_test(|ds| {
            assert!(ds.snap("testsnap").is_ok());
            assert!(ds.snap_exists("testsnap")?);
            Ok(())
        })
    }

    #[test]
    fn test_snap_already_exists() -> Result<()> {
        run_test(|ds| {
            ds.snap("testsnap")?;
            assert!(ds.snap("testsnap").is_err());
            Ok(())
        })
    }

    #[test]
    fn test_snap_invalid_name() -> Result<()> {
        run_test(|ds| {
            assert!(ds.snap("test&snap").is_err());
            Ok(())
        })
    }
}
