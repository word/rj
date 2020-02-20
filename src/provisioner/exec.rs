use anyhow::Result;
use log::info;
use serde::Deserialize;
use std::fs::copy;
use std::path::Path;

use crate::cmd;
use crate::cmd_capture;
use crate::cmd_stream;
use crate::errors::ProvError;
use crate::jail::Jail;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exec {
    path: String,
}

impl Exec {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: exec provisioner running", jail.name());
        self.validate()?;
        info!("{}: executing: {}", jail.name(), &self.path);
        let exe_filename = Path::new(&self.path).file_name().unwrap();
        let exe_tmp_path = cmd_capture!(
            "jexec",
            jail.name(),
            "mktemp",
            "-t",
            exe_filename.to_str().unwrap(),
        )?;
        copy(&self.path, format!("{}{}", jail.mountpoint(), exe_tmp_path))?;
        cmd!("jexec", jail.name(), "chmod", "0700", &exe_tmp_path)?;
        cmd_stream!("jexec", jail.name(), &exe_tmp_path)?;
        cmd!("jexec", jail.name(), "rm", &exe_tmp_path)
    }

    fn validate(&self) -> Result<()> {
        if Path::new(&self.path).is_file() {
            Ok(())
        } else {
            let msg = format!("Invalid exec path: {}", &self.path);
            Err(anyhow::Error::new(ProvError(msg)))
        }
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

        cmd!("jexec", jail.name(), "cat", "/tmp/exec_test")?;

        jail.destroy()?;
        Ok(())
    }

    #[test]
    fn invalid_path() {
        let exec = Exec {
            path: "/tmp/doesnotexist23".to_string(),
        };
        assert!(exec.validate().is_err());
    }

    fn valid_path() {
        let exec = Exec {
            path: "/etc/hosts".to_string(),
        };
        assert!(exec.validate().is_ok());
    }
}
