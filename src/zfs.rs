use anyhow::Result;
use chrono::Local;
use log::info;

use crate::cmd;
use crate::cmd_capture;

#[derive(Debug)]
pub struct DataSet {
    path: String,
}

impl DataSet {
    pub fn new(path: &str) -> DataSet {
        DataSet {
            path: path.to_string(),
        }
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    // create the zfs data set if it doesn't exist already
    pub fn create(&self) -> Result<bool> {
        if self.exists()? {
            info!("Dataset {} already exists, skipping", &self.path);
            Ok(false)
        } else {
            info!("Creating zfs dataset {}", &self.path);
            cmd!("zfs", "create", &self.path)?;
            Ok(true)
        }
    }

    pub fn set(&self, property: &str, value: &str) -> Result<()> {
        let prop = format!("{}={}", property, value);
        cmd!("zfs", "set", &prop, &self.path)
    }

    pub fn get(&self, property: &str) -> Result<String> {
        // zfs get -H -o value mountpoint zroot/jails
        let value = cmd_capture!("zfs", "get", "-H", "-o", "value", property, &self.path)?;
        Ok(value.trim().to_string())
    }

    pub fn destroy(&self) -> Result<()> {
        info!("Destroying zfs dataset: {}", &self.path);
        cmd!("zfs", "destroy", &self.path)
    }

    // destroy recursively
    #[allow(dead_code)]
    pub fn destroy_r(&self) -> Result<()> {
        info!("Destroying zfs dataset recursively: {}", &self.path);
        cmd!("zfs", "destroy", "-r", &self.path)
    }

    pub fn snap(&self, snap_name: &str) -> Result<()> {
        let snap_path = format!("{}@{}", &self.path, &snap_name);
        info!("Creating snapshot: {}", &snap_path);
        cmd!("zfs", "snapshot", &snap_path)
    }

    pub fn snap_with_time(&self, snap_name: &str) -> Result<()> {
        let ts = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let snap_path = format!("{}@{}_{}", &self.path, &snap_name, &ts);
        cmd!("zfs", "snapshot", &snap_path)
    }

    pub fn clone(&self, snap: &str, dest: &str) -> Result<DataSet> {
        info!("Cloning {} to {}", &self.path, &dest);
        let snap_name = format!("{}@{}", &self.path, snap);
        cmd!("zfs", "clone", &snap_name, &dest)?;
        Ok(DataSet::new(dest))
    }

    pub fn exists(&self) -> Result<bool> {
        self.ds_exists(&self.path)
    }

    pub fn snap_exists(&self, snap_name: &str) -> Result<bool> {
        self.ds_exists(&format!("{}@{}", &self.path, snap_name))
    }

    pub fn list_snaps(&self) -> Result<Vec<String>> {
        let output = cmd_capture!("zfs", "list", "-H", "-o", "name", "-t", "snap")?;
        let snaps = output
            .lines()
            .filter(|s| s.starts_with(&self.path))
            .map(|s| s.split('@').last().unwrap().to_string())
            .collect::<Vec<String>>();
        Ok(snaps)
    }

    pub fn snap_destroy(&self, snap_name: &str) -> Result<()> {
        info!("Destroying snapshot {}@{}", &self.path, snap_name);
        let snap_full_name = format!("{}@{}", self.path, snap_name);
        cmd!("zfs", "destroy", &snap_full_name)
    }

    // checks if data set exists.
    // this is used by self.exists() and self.snap_exists()
    fn ds_exists(&self, ds_path: &str) -> Result<bool> {
        let msg_pattern = format!("cannot open \'{}\': dataset does not exist\n", &ds_path);

        match cmd_capture!("zfs", "list", &ds_path) {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains(&msg_pattern) {
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
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::panic::{self, AssertUnwindSafe};

    fn run_test<T, R>(test: T) -> Result<()>
    where
        T: Fn(&mut DataSet) -> Result<R>,
        R: std::fmt::Debug,
    {
        // Set up
        let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(5).collect();
        let mut ds = DataSet::new(&format!("zroot/rjtest-{}", rand_string));
        ds.create()?;

        // Run the test closure but catch the panic so that the teardown section below
        // can run.
        let result = panic::catch_unwind(AssertUnwindSafe(|| test(&mut ds)));

        // Teardown
        ds.destroy_r()?;

        // Check result
        println!("{:?}", &result);
        // Check if there was a panic
        assert!(result.is_ok());
        // Check if there was an error
        assert!(result.unwrap().is_ok());

        Ok(())
    }

    #[test]
    fn ds_exists() -> Result<()> {
        run_test(|ds| ds.exists())
    }

    #[test]
    fn ds_double_destroy() {
        let ds = DataSet::new("zroot/rjtest");
        assert!(ds.create().is_ok());
        assert!(ds.destroy().is_ok());
        assert!(ds.destroy().is_err());
    }

    #[test]
    fn ds_set() -> Result<()> {
        run_test(|ds| {
            ds.set("atime", "off")?;
            assert_eq!(ds.get("atime")?, "off");
            Ok(())
        })
    }

    #[test]
    fn ds_invalid_set() -> Result<()> {
        run_test(|ds| {
            assert!(ds.set("noexist", "nope").is_err());
            Ok(())
        })
    }

    // Creating a DS that already exists shoudn't fail.  It becomes a representation
    // of the existing DS.
    #[test]
    fn ds_already_exists() -> Result<()> {
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
    fn ds_invalid_path() {
        let inv_ds = DataSet::new("noexist/rjtest");
        assert!(inv_ds.create().is_err());
    }

    #[test]
    fn ds_snap() -> Result<()> {
        run_test(|ds| {
            assert!(ds.snap("testsnap").is_ok());
            assert!(ds.snap_exists("testsnap")?);
            Ok(())
        })
    }

    #[test]
    fn ds_snap_with_time() -> Result<()> {
        run_test(|ds| {
            let ts = Local::now().format("%Y-%m-%dT%H").to_string();
            let s = "testsnaptime";
            ds.snap_with_time(&s)?;
            let pattern = format!("{}_{}", &s, &ts);
            let mut exists = false;
            for snap in ds.list_snaps()?.iter() {
                if snap.contains(&pattern) {
                    exists = true;
                    break;
                }
            }
            assert!(exists);
            Ok(())
        })
    }

    #[test]
    fn ds_snap_already_exists() -> Result<()> {
        run_test(|ds| {
            ds.snap("testsnap")?;
            assert!(ds.snap("testsnap").is_err());
            Ok(())
        })
    }

    #[test]
    fn ds_snap_invalid_name() -> Result<()> {
        run_test(|ds| {
            assert!(ds.snap("test&snap").is_err());
            Ok(())
        })
    }

    #[test]
    fn ds_clone() -> Result<()> {
        run_test(|ds| {
            ds.snap("test")?;
            let cloned = ds.clone("test", "zroot/test")?;
            let result = panic::catch_unwind(|| assert!(cloned.exists().unwrap()));
            cloned.destroy()?;
            assert!(result.is_ok());
            Ok(())
        })
    }

    #[test]
    fn ds_invalid_clone() -> Result<()> {
        run_test(|ds| {
            assert!(ds.clone("noexist", "zroot/test").is_err());
            Ok(())
        })
    }

    #[test]
    fn ds_list_snaps() -> Result<()> {
        run_test(|ds| {
            ds.snap("test1")?;
            ds.snap("test2")?;
            let snaps = ds.list_snaps()?;
            assert_eq!(snaps.len(), 2);
            assert_eq!(snaps[0], "test1");
            assert_eq!(snaps[1], "test2");
            Ok(())
        })
    }

    #[test]
    fn ds_snap_destroy() -> Result<()> {
        run_test(|ds| {
            ds.snap("test1")?;
            ds.snap("test2")?;
            ds.snap_destroy("test1")?;
            ds.snap_destroy("test2")?;
            assert!(ds.list_snaps()?.is_empty());
            Ok(())
        })
    }
}
