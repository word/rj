use log::info;
use std::process::Command;

use super::cmd;
use anyhow::Result;

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

    // create the zfs data set if it doesn't exist already
    pub fn create(&self) -> Result<bool> {
        if self.exists()? {
            info!("Dataset {} already exists, skipping", &self.path);
            Ok(false)
        } else {
            info!("Creating zfs dataset {}", &self.path);
            let mut zfs = Command::new("zfs");
            zfs.arg("create");
            zfs.arg(&self.path);
            cmd::run(&mut zfs)?;
            Ok(true)
        }
    }

    #[allow(dead_code)]
    pub fn get_path(&self) -> &String {
        &self.path
    }

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
        info!("Destroying zfs dataset: {}", &self.path);
        let mut zfs = Command::new("zfs");
        zfs.arg("destroy");
        zfs.arg(&self.path);
        cmd::run(&mut zfs)?;
        Ok(())
    }

    // destroy recursively
    #[allow(dead_code)]
    pub fn destroy_r(&self) -> Result<()> {
        info!("Destroying zfs dataset recursively: {}", &self.path);
        let mut zfs = Command::new("zfs");
        zfs.arg("destroy");
        zfs.arg("-r");
        zfs.arg(&self.path);
        cmd::run(&mut zfs)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn snap(&self, snap_name: &str) -> Result<()> {
        let snap_path = format!("{}@{}", &self.path, &snap_name);
        info!("Creating snapshot: {}", &snap_path);
        let mut zfs = Command::new("zfs");
        zfs.arg("snapshot");
        zfs.arg(&snap_path);
        cmd::run(&mut zfs)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn clone(&self, snap: &str, dest: &str) -> Result<DataSet> {
        info!("Cloning {} to {}", &self.path, &dest);
        let mut zfs = Command::new("zfs");
        zfs.arg("clone");
        zfs.arg(format!("{}@{}", &self.path, snap));
        zfs.arg(&dest);
        cmd::run(&mut zfs)?;
        Ok(DataSet::new(dest))
    }

    pub fn exists(&self) -> Result<bool> {
        self.ds_exists(&self.path)
    }

    #[allow(dead_code)]
    pub fn snap_exists(&self, snap_name: &str) -> Result<bool> {
        self.ds_exists(&format!("{}@{}", &self.path, snap_name))
    }

    #[allow(dead_code)]
    pub fn list_snaps(&self) -> Result<Vec<String>> {
        let mut zfs = Command::new("zfs");
        zfs.arg("list")
            .arg("-H")
            .arg("-o")
            .arg("name")
            .arg("-t")
            .arg("snap");
        let output = cmd::run(&mut zfs)?;
        let snaps = output
            .lines()
            .filter(|s| s.starts_with(&self.path))
            .map(|s| s.split('@').last().unwrap().to_string())
            .collect::<Vec<String>>();
        Ok(snaps)
    }

    #[allow(dead_code)]
    pub fn snap_destroy(&self, snap_name: &str) -> Result<()> {
        info!("Destroying snapshot {}@{}", &self.path, snap_name);
        let mut zfs = Command::new("zfs");
        zfs.arg("destroy")
            .arg(&format!("{}@{}", self.path, snap_name));
        cmd::run(&mut zfs)?;
        Ok(())
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
        assert!(ds.create().is_ok());

        // Run the test closure but catch the panic so that the teardown section below can run.
        let result = panic::catch_unwind(AssertUnwindSafe(|| test(&mut ds)));

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
    fn test_ds_exists() -> Result<()> {
        run_test(|ds| ds.exists())
    }

    #[test]
    fn test_ds_double_destroy() {
        let ds = DataSet::new("zroot/rjtest");
        assert!(ds.create().is_ok());
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
    fn test_ds_invalid_path() {
        let inv_ds = DataSet::new("noexist/rjtest");
        assert!(inv_ds.create().is_err());
    }

    #[test]
    fn test_ds_snap() -> Result<()> {
        run_test(|ds| {
            assert!(ds.snap("testsnap").is_ok());
            assert!(ds.snap_exists("testsnap")?);
            Ok(())
        })
    }

    #[test]
    fn test_ds_snap_already_exists() -> Result<()> {
        run_test(|ds| {
            ds.snap("testsnap")?;
            assert!(ds.snap("testsnap").is_err());
            Ok(())
        })
    }

    #[test]
    fn test_ds_snap_invalid_name() -> Result<()> {
        run_test(|ds| {
            assert!(ds.snap("test&snap").is_err());
            Ok(())
        })
    }

    #[test]
    fn test_ds_clone() -> Result<()> {
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
    fn test_ds_invalid_clone() -> Result<()> {
        run_test(|ds| {
            assert!(ds.clone("noexist", "zroot/test").is_err());
            Ok(())
        })
    }

    #[test]
    fn test_ds_list_snaps() -> Result<()> {
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
    fn test_ds_snap_destroy() -> Result<()> {
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
