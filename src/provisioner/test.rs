use anyhow::Result;
use serde::Deserialize;

// This is used only for testing the Provisioner interface

#[derive(Clone, Debug, Deserialize)]
pub struct Test;

impl Test {
    pub fn provision(&self) -> Result<()> {
        Ok(())
    }
}
