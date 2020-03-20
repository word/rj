use crate::cmd::Cmd;
use crate::jail::Jail;
use anyhow::Result;
use log::{debug, info};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exec {
    pub cmd: String,
    #[serde(default = "default_mode")]
    pub exec_mode: ExecMode,
}

#[derive(Clone, Debug, Deserialize)]
pub enum ExecMode {
    #[serde(alias = "jexec")]
    Jexec,
    #[serde(alias = "chroot")]
    Chroot,
}

fn default_mode() -> ExecMode {
    ExecMode::Jexec
}

impl Exec {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: exec provisioner running", jail.name());
        info!("{}: running command: {}", jail.name(), &self.cmd);

        let mut args: Vec<&str> = self.cmd.split(' ').collect();

        match self.exec_mode {
            ExecMode::Jexec => {
                args.insert(0, jail.name());
                Cmd::new("jexec").args(args).stream()
            }
            ExecMode::Chroot => {
                args.insert(0, jail.mountpoint());
                Cmd::new("chroot").args(args).stream()
            }
        }
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
    fn provision_jexec() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jails = s.to_jails()?;
        let jail = &jails["exec_test"];
        let basejail = &jails["base"];

        if !basejail.exists()? {
            crate::init(&s).unwrap();
            basejail.apply()?;
        }

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

    #[test]
    #[serial]
    fn provision_chroot() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jails = s.to_jails()?;
        let jail = &jails["exec_chroot_test"];
        let basejail = &jails["base"];

        if !basejail.exists()? {
            crate::init(&s).unwrap();
            basejail.apply()?;
        }

        // clean up if left over from a failed test
        if jail.exists()? {
            jail.destroy()?;
        }
        jail.apply()?;

        let full_dest = format!("{}/tmp/exec_chroot_test", jail.mountpoint());
        let metadata = std::fs::metadata(full_dest)?;
        assert!(metadata.is_file());

        jail.destroy()?;
        Ok(())
    }
}
