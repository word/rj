use anyhow::Result;
use duct::cmd;
use log::{error, info};
use serde::Deserialize;
use std::io::prelude::*;
use std::io::BufReader;

use crate::jail::Jail;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exec {
    path: String,
}

impl Exec {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: exec provisioner running", jail.name());

        let cmd = cmd!("find", "/usrrrr");
        let reader = cmd.reader()?;
        for line in BufReader::new(reader).lines() {
            info!("{}", line?);
        }

        Ok(())
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
        // TODO: init
        let jail = &jails["exec_test"];
        jail.apply()?;

        cmd!("chroot", jail.mountpoint(), "cat", "/tmp/exec_test").read()?;

        jails["exec_test"].destroy()?;
        Ok(())
    }
}
