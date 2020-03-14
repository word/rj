use crate::errors::SourceError;
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
    pub fn install(&self, dest_path: &str, dest_dataset: &zfs::DataSet) -> Result<()> {
        let src_dataset = zfs::DataSet::new(&self.path);
        match src_dataset.last_snap("ready")? {
            Some(s) => {
                info!(
                    "cloning {}@{} to {}",
                    &src_dataset.path(),
                    &s,
                    dest_dataset.path()
                );
                src_dataset.clone(&s, dest_dataset.path())?;
                Ok(())
            }
            None => {
                let msg = format!(
                    "'ready' snapshot not found in source dataset: {}",
                    &self.path
                );
                Err(anyhow::Error::new(SourceError(msg)))
            }
        }
    }
}
