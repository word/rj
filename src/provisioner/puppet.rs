#[allow(dead_code)]
#[allow(unused_imports)]
use crate::cmd;
use crate::errors::ProvError;
use crate::errors::RunError;
use crate::jail::Jail;
use anyhow::Result;
use log::{debug, info};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::copy;
use std::fs::{set_permissions, Permissions};
use std::os::unix::prelude::*;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Puppet {
    path: String,
    #[serde(default = "default_manifest_file")]
    manifest_file: String,
    #[serde(default = "default_none")]
    module_path: Option<String>,
    #[serde(default = "default_none")]
    hiera_config: Option<String>,
    #[serde(default = "default_extra_args")]
    extra_args: Vec<String>,
    #[serde(default = "default_tmp_dir")]
    tmp_dir: String,
    #[serde(default = "default_version")]
    puppet_version: String,
}

fn default_manifest_file() -> String {
    "init.pp".to_string()
}

fn default_tmp_dir() -> String {
    "/var/rj".to_string()
}

fn default_version() -> String {
    "6".to_string()
}

fn default_none() -> Option<String> {
    None
}

fn default_extra_args() -> Vec<String> {
    vec![]
}

impl Puppet {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: puppet provisioner running", jail.name());
        let puppet_pkg = format!("puppet{}", self.puppet_version);

        if !Self::is_installed(&jail, &puppet_pkg)? {
            info!("{}: installing {}", jail.name(), puppet_pkg);
            let mut env = HashMap::new();
            env.insert("TESTENV", "testie");
            cmd::cmd_env(
                "jexec",
                &[jail.name(), "pkg", "install", "-y", &puppet_pkg],
                env,
            )?;
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        debug!("validating puppet provisioner");
        Ok(())
    }

    // check if pkg is installed
    fn is_installed(jail: &Jail, pkg_name: &str) -> Result<bool> {
        let mut env = HashMap::new();
        env.insert("ASSUME_ALWAYS_YES", "yes");
        let result = cmd::cmd_env("jexec", &[jail.name(), "pkg", "info", pkg_name], env);

        match result {
            Ok(_) => return Ok(true),
            Err(e) => match e.downcast_ref::<RunError>() {
                Some(re) => {
                    if re.code.unwrap() == 70 {
                        return Ok(false);
                    } else {
                        return Err(e);
                    }
                }
                None => return Err(e),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    use pretty_assertions::assert_eq;
    use serial_test::serial;
    use std::os::unix::fs::MetadataExt;

    #[test]
    #[serial]
    fn provision() -> Result<()> {
        let s = Settings::new("testdata/config.toml")?;
        let jails = s.to_jails()?;
        let jail = &jails["puppet_test"];

        // clean up if left over from a failed test
        if jail.exists()? {
            // jail.destroy()?;
        }
        jail.apply()?;

        // jail.destroy()?;
        Ok(())
    }

    #[test]
    fn validation() {}
}
