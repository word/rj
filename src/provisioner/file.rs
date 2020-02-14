use anyhow::Result;
use log::info;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    source: String,
    dest: String,
    mode: String,
}

impl File {
    pub fn provision(&self) -> Result<()> {
        info!("File provisioner running");
        Ok(())
    }
}
