use crate::jail::Jail;
use anyhow::Result;
use serde::Deserialize;

pub(crate) mod freebsd;
pub(crate) mod zfs_clone;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Source {
    #[serde(alias = "freebsd")]
    FreeBSD(freebsd::FreeBSD),
    #[serde(alias = "clone")]
    ZfsClone(zfs_clone::ZfsClone),
}

impl Source {
    pub fn install(&self, jail: &Jail) -> Result<()> {
        match self {
            Source::FreeBSD(s) => s.install(jail),
            Source::ZfsClone(s) => s.install(jail),
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Source::FreeBSD(s) => s.validate(),
            Source::ZfsClone(s) => s.validate(),
        }
    }
}
