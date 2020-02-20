use anyhow::Result;
use log::info;
use serde::Deserialize;

use crate::jail::Jail;

// This is used only for testing the Provisioner interface

#[derive(Clone, Debug, Deserialize)]
pub struct Test;

impl Test {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: test provisioner running", jail.name());
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        info!("Validating test provisioner");
        Ok(())
    }
}
