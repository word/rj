use crate::errors::SourceError;
use crate::jail::Jail;
use crate::zfs;
use anyhow::Result;
use log::info;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZfsClone {
    path: String,
}

impl ZfsClone {
    pub fn install(&self, jail: &Jail) -> Result<()> {
        let src_dataset = zfs::DataSet::new(&self.path);
        let dest_dataset = &jail.zfs_ds();

        match src_dataset.last_snap("ready")? {
            Some(snapshot) => {
                info!(
                    "{}: cloning {}@{} to {}",
                    &jail.name(),
                    &src_dataset.path(),
                    &snapshot,
                    &dest_dataset.path()
                );
                src_dataset.clone(&snapshot, &dest_dataset.path())?;
                Ok(())
            }
            None => {
                let msg = format!(
                    "{}: 'ready' snapshot not found in source dataset: {}",
                    &jail.name(),
                    &self.path
                );
                Err(anyhow::Error::new(SourceError(msg)))
            }
        }
    }
}
