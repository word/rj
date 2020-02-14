#![allow(dead_code)]
use anyhow::Result;
use serde::Deserialize;

mod exec;
mod file;
mod test;

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
    fn provision(&self) -> Result<()> {
        match self {
            Provisioner::File(p) => p.provision(),
            Provisioner::Exec(p) => p.provision(),
            Provisioner::Test(p) => p.provision(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{test, Provisioner};
    // use pretty_assertions::assert_eq;

    #[test]
    fn provision() {
        Provisioner::Test(test::Test).provision().unwrap();
    }
}
