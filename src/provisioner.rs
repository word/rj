#![allow(dead_code)]
use anyhow::Result;
use serde::Deserialize;

mod exec;
mod file;
mod test;
use crate::jail::Jail;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Provisioner {
    #[serde(alias = "file")]
    File(file::ProvFile),
    #[serde(alias = "exec")]
    Exec(exec::Exec),
    #[serde(alias = "test")]
    Test(test::Test),
}

impl Provisioner {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        match self {
            Provisioner::File(p) => p.provision(jail),
            Provisioner::Exec(p) => p.provision(jail),
            Provisioner::Test(p) => p.provision(jail),
        }
    }
    pub fn validate(&self) -> Result<()> {
        match self {
            Provisioner::File(p) => p.validate(),
            Provisioner::Exec(p) => p.validate(),
            Provisioner::Test(p) => p.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn provision() -> Result<()> {
        let s = Settings::new("testdata/config.toml")?;
        let jails = s.to_jails()?;
        Provisioner::Test(test::Test).provision(&jails["test1"])
    }
}
