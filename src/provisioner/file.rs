use anyhow::Result;
use log::info;
use serde::Deserialize;

use crate::jail::Jail;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    source: String,
    dest: String,
    mode: String,
}

impl File {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: file provisioner running", jail.name());
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        info!("Validating file provisioner");
        Ok(())
    }
}
