use anyhow::Result;
use log::info;
use serde::Deserialize;

use crate::jail::Jail;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exec {
    path: String,
}

impl Exec {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: exec provisioner running", jail.name());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn provision() -> Result<()> {
        let s = Settings::new("config.toml")?;
        let jails = s.to_jails()?;

        Ok(())
    }
}
