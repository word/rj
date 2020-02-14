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
    File(file::File),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    use std::fs;
    // use pretty_assertions::assert_eq;

    #[test]
    fn provision() -> Result<()> {
        let s = Settings::new(fs::read_to_string("config.toml")?)?;
        let jails = s.to_jails()?;
        Provisioner::Test(test::Test).provision(&jails["test1"])
    }
}
