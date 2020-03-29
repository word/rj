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
        info!(
            "{}: running command: {}{}",
            jail.name(),
            &self.cmd,
            jail.noop_suffix()
        );

        let args: Vec<&str> = self.cmd.split(' ').collect();

        match self.exec_mode {
            ExecMode::Jexec => {
                if !jail.noop() {
                    Cmd::new("jexec").arg(jail.name()).args(args).stream()?;
                }
            }
            ExecMode::Chroot => {
                if !jail.noop() {
                    Cmd::new("chroot")
                        .arg(jail.mountpoint())
                        .args(args)
                        .stream()?;
                }
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        debug!("validating exec provisioner");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provisioner::test_helpers::setup;
    use crate::settings::Settings;
    use serial_test::serial;
    use std::fs;
    use std::path::Path;

    #[test]
    #[serial]
    fn provision_jexec() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jail = setup(&s, "exec_test")?;

        jail.apply()?;

        let path = Path::new(jail.mountpoint()).join("tmp/exec_test");
        assert!(path.is_file());

        jail.destroy()?;
        Ok(())
    }

    #[test]
    #[serial]
    fn provision_jexec_noop() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jail = setup(&s, "exec_test")?;

        // create the test jail
        jail.apply()?;
        // remove the created test file
        let path = Path::new(jail.mountpoint()).join("tmp/exec_test");
        fs::remove_file(&path)?;

        // make a noop jail
        let s_noop = Settings::new("testdata/config.toml", true)?;
        let jail_noop = setup(&s_noop, "exec_chroot_test")?;

        // on provision it shoudn't re-create the file
        jail_noop.provision()?;
        assert_eq!(path.is_file(), false);

        jail.destroy()?;
        Ok(())
    }

    #[test]
    #[serial]
    fn provision_chroot() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jail = setup(&s, "exec_chroot_test")?;

        jail.apply()?;

        let path = Path::new(jail.mountpoint()).join("tmp/exec_chroot_test");
        assert!(path.is_file());

        jail.destroy()?;
        Ok(())
    }

    #[test]
    #[serial]
    fn provision_chroot_noop() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jail = setup(&s, "exec_chroot_test")?;

        // create the test jail
        jail.apply()?;
        // remove created test file
        let path = Path::new(jail.mountpoint()).join("tmp/exec_chroot_test");
        fs::remove_file(&path)?;

        // make a noop jail
        let s_noop = Settings::new("testdata/config.toml", true)?;
        let jail_noop = setup(&s_noop, "exec_chroot_test")?;

        // on provision it shoudn't re-create the file
        jail_noop.provision()?;
        assert_eq!(path.is_file(), false);

        jail.destroy()?;
        Ok(())
    }
}
