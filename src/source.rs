use crate::zfs;
use anyhow::Result;
use serde::Deserialize;

mod freebsd;
mod zfs_clone;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Source {
    #[serde(alias = "freebsd")]
    FreeBSD(freebsd::FreeBSD),
    #[serde(alias = "clone")]
    ZfsClone(zfs_clone::ZfsClone),
}

impl Source {
    pub fn install(&self, dest_path: &str, dest_dataset: &zfs::DataSet) -> Result<()> {
        match self {
            Source::FreeBSD(s) => s.install(dest_path, dest_dataset),
            Source::ZfsClone(s) => s.install(dest_path, dest_dataset),
        }
    }
}
