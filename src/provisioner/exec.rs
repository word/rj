use crate::cmd::Cmd;
use crate::jail::Jail;
use anyhow::Result;
use log::{debug, info};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exec {
    pub cmd: String,
}

impl Exec {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: exec provisioner running", jail.name());
        info!("{}: executing: {}", jail.name(), &self.cmd,);

        let mut args: Vec<&str> = self.cmd.split(' ').collect();
        args.insert(0, jail.name());
        Cmd::new("jexec").args(args).stream()
    }

    pub fn validate(&self) -> Result<()> {
        debug!("validating exec provisioner");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    use serial_test::serial;

    #[test]
    #[serial]
    fn provision() -> Result<()> {
        let s = Settings::new("testdata/config.toml")?;
        let jails = s.to_jails()?;
        let jail = &jails["exec_test"];

        // clean up if left over from a failed test
        if jail.exists()? {
            jail.destroy()?;
        }
        jail.apply()?;

        let full_dest = format!("{}/tmp/exec_test", jail.mountpoint());
        let metadata = std::fs::metadata(full_dest)?;
        assert!(metadata.is_file());

        jail.destroy()?;
        Ok(())
    }
}
