use crate::cmd;
// use crate::errors::ProvError;
// use crate::errors::RunError;
use crate::jail::Jail;
use crate::pkg::Pkg;
use anyhow::Result;
use log::{debug, info};
// use regex::Regex;
use serde::Deserialize;
// use std::collections::HashMap;
// use std::fs::copy;
use std::fs;
// use std::os::unix::prelude::*;
// use std::path::Path;

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
        let pkg_name = format!("puppet{}", self.puppet_version);
        let pkg = Pkg::new(&pkg_name, &jail.mountpoint());

        if !pkg.is_installed()? {
            info!("{}: installing {}", jail.name(), pkg_name);
            pkg.install()?;
        }

        // Copy puppet manifest into the jail
        fs::create_dir_all(&self.tmp_dir)?;
        // todo - set tight permissions on tmp_dir
        let trimmed_path = self.path.trim_end_matches('/');
        let puppet_dir = format!("{}/{}", &jail.mountpoint(), &self.tmp_dir);
        cmd!("rsync", "-r", trimmed_path, puppet_dir)?;

        // Construct puppet command
        // let mut puppet_args = vec![jail.name(), "puppet", "apply"];
        // if let Some(mp) = &self.module_path {
        //     puppet_args.push("--modulepath");
        //     puppet_args.push(mp);
        // }
        // if let Some(hc) = &self.hiera_config {
        //     puppet_args.push("--hiera_config");
        //     puppet_args.push(hc);
        // }
        // for ea in self.extra_args.iter() {
        //     puppet_args.push(&ea);
        // }
        // puppet_args.push(&self.manifest_file);

        // Cmd::new("jexec").args(puppet_args).exec()?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        debug!("validating puppet provisioner");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    // use pretty_assertions::assert_eq;
    use serial_test::serial;
    // use std::os::unix::fs::MetadataExt;

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
