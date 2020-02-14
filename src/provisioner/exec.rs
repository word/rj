use anyhow::Result;
use log::info;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exec {
    path: String,
}

impl Exec {
    pub fn provision(&self) -> Result<()> {
        info!("Exec provisioner running");
        Ok(())
    }
}
