#![allow(dead_code)]
use anyhow::Result;
use log::info;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Provisioner {
    #[serde(alias = "file")]
    File(ProFile),
    #[serde(alias = "exec")]
    Exec(ProExec),
}

impl Provisioner {
    fn provision(&self) -> Result<()> {
        match self {
            Provisioner::File(p) => p.provision()?,
            Provisioner::Exec(p) => p.provision()?,
        };
        Ok(())
    }
}

impl ProFile {
    fn provision(&self) -> Result<()> {
        println!("ProFile running!");
        Ok(())
    }
}

impl ProExec {
    fn provision(&self) -> Result<()> {
        println!("ProExec running!");
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProFile {
    source: String,
    dest: String,
    mode: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProExec {
    path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn prov() {
        let exec = ProExec {
            path: "/path".to_string(),
        };
        let prov = Provisioner::Exec(exec);
        prov.provision().unwrap();
    }
}
