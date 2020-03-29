use crate::jail::Jail;
use crate::zfs;
use anyhow::{bail, ensure, Result};
use log::{debug, info};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZfsClone {
    pub path: PathBuf,
}

impl ZfsClone {
    pub fn install(&self, jail: &Jail) -> Result<()> {
        let src_dataset = zfs::DataSet::new(&self.path);
        let dest_dataset = &jail.zfs_ds();

        ensure!(
            src_dataset.exists()?,
            format!(
                "{}: source dataset {} doesn't exist",
                &jail.name(),
                &self.path.display()
            ),
        );

        match src_dataset.last_snap("ready")? {
            Some(snapshot) => {
                info!(
                    "{}: cloning {}@{} to {}{}",
                    &jail.name(),
                    &src_dataset.path().display(),
                    &snapshot,
                    &dest_dataset.path().display(),
                    &jail.noop_suffix(),
                );
                if !jail.noop() {
                    src_dataset.clone(&snapshot, &dest_dataset.path())?;
                }
                Ok(())
            }
            None => {
                bail!(
                    "{}: 'ready' snapshot not found for source dataset: {}",
                    &jail.name(),
                    &self.path.display()
                );
            }
        }
    }

    pub fn validate(&self) -> Result<()> {
        debug!("Validating zfs clone source");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    use crate::zfs::DataSet;
    use pretty_assertions::assert_eq;
    use serial_test::serial;
    use std::path::Path;

    fn cleanup(source_ds: &DataSet, jail: &Jail) -> Result<()> {
        jail.destroy()?;
        if source_ds.exists()? {
            source_ds.destroy_r()?;
        }
        Ok(())
    }

    #[test]
    #[serial]
    fn install() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jails = s.to_jails()?;
        let jail = &jails["clone_test"];
        let source_ds = DataSet::new(Path::new("zroot/rjtest_clone"));

        let clone_source = ZfsClone {
            path: PathBuf::from("zroot/rjtest_clone"),
        };

        cleanup(&source_ds, &jail)?;

        let err = clone_source.install(jail).unwrap_err();
        assert_eq!(
            err.downcast::<String>().unwrap(),
            "clone_test: source dataset zroot/rjtest_clone doesn't exist".to_string()
        );

        source_ds.create()?;

        let err = clone_source.install(jail).unwrap_err();
        assert_eq!(
            err.downcast::<String>().unwrap(),
            "clone_test: 'ready' snapshot not found for source dataset: zroot/rjtest_clone"
                .to_string()
        );

        source_ds.snap("ready")?;
        clone_source.install(jail)?;

        cleanup(&source_ds, &jail)?;
        Ok(())
    }
}
