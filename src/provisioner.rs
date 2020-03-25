#![allow(dead_code)]
use crate::jail::Jail;
use anyhow::Result;
use serde::Deserialize;

pub(crate) mod exec;
pub(crate) mod file;
pub(crate) mod puppet;
pub(crate) mod test;
mod test_helpers;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Provisioner {
    #[serde(alias = "file")]
    File(file::ProvFile),
    #[serde(alias = "exec")]
    Exec(exec::Exec),
    #[serde(alias = "test")]
    Test(test::Test),
    #[serde(alias = "puppet")]
    Puppet(puppet::Puppet),
}

impl Provisioner {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        match self {
            Provisioner::File(p) => p.provision(jail),
            Provisioner::Exec(p) => p.provision(jail),
            Provisioner::Test(p) => p.provision(jail),
            Provisioner::Puppet(p) => p.provision(jail),
        }
    }
    pub fn validate(&self) -> Result<()> {
        match self {
            Provisioner::File(p) => p.validate(),
            Provisioner::Exec(p) => p.validate(),
            Provisioner::Test(p) => p.validate(),
            Provisioner::Puppet(p) => p.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn provision() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jails = s.to_jails()?;
        Provisioner::Test(test::Test).provision(&jails["test1"])
    }
}
