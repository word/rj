use crate::cmd;
use crate::cmd::Cmd;
use crate::cmd_stream;
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
        let pkg_name = format!("puppet{}", self.puppet_version);
        let pkg = Pkg::new(&pkg_name, &jail.mountpoint());

        if !pkg.is_installed()? {
            info!("{}: installing {}", jail.name(), pkg_name);
            pkg.install()?;
        }

        // Copy puppet manifest into the jail
        // FIXME - improve variable naming (in/out)
        let trimmed_src_path = Path::new(self.path.trim_end_matches('/'));
        let src_path_dirname = Path::new(&trimmed_src_path).file_name().unwrap();
        let dst_path = Path::new(&jail.mountpoint()).join(&self.tmp_dir.trim_start_matches('/'));
        let puppet_dir = Path::new(&self.tmp_dir).join(src_path_dirname);

        fs::create_dir_all(&dst_path)?;
        set_permissions(&dst_path, Permissions::from_mode(0o700))?;
        cmd!(
            "rsync",
            "-r",
            "--delete",
            trimmed_src_path.to_str().unwrap(),
            dst_path.to_str().unwrap()
        )?;

        info!("Puppet dir: {}", puppet_dir.to_str().unwrap());
        info!("dst path: {}", dst_path.to_str().unwrap());
        info!("jail mountpoint: {}", jail.mountpoint());

        // Make exec wrapper
        let puppet_wrapper = format!("#!/bin/sh\ncd {} && $@\n", puppet_dir.to_str().unwrap());
        let out_wrapper_path = Path::new(&dst_path).join("puppet_wrapper.sh");
        let in_wrapper_path = Path::new(&self.tmp_dir).join("puppet_wrapper.sh");
        fs::write(&out_wrapper_path, puppet_wrapper)?;
        set_permissions(&out_wrapper_path, Permissions::from_mode(0o755))?;

        // Run r10k
        // todo - only if puppetfile exists
        let r10k_pkg = Pkg::new("rubygem-r10k", &jail.mountpoint());
        if !r10k_pkg.is_installed()? {
            info!("{}: installing {}", jail.name(), "rubygem-r10k");
            r10k_pkg.install()?;
        }
        cmd_stream!(
            "jexec",
            jail.name(),
            in_wrapper_path.to_str().unwrap(),
            "r10k",
            "puppetfile",
            "install"
        )?;

        // Construct puppet command
        let mut puppet_cmd = Cmd::new("jexec");
        puppet_cmd
            .arg(jail.name())
            .arg(&in_wrapper_path)
            .arg("puppet")
            .arg("apply");

        if let Some(mp) = &self.module_path {
            puppet_cmd.arg("--modulepath").arg(mp);
        }
        if let Some(hc) = &self.hiera_config {
            puppet_cmd.arg("--hiera_config").arg(hc);
        }
        for ea in self.extra_args.iter() {
            puppet_cmd.arg(ea);
        }
        puppet_cmd.arg(&self.manifest_file);
        debug!("{:?}", &puppet_cmd);
        puppet_cmd.stream()?;

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
    use pretty_assertions::assert_eq;
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
            jail.destroy()?;
        }
        jail.apply()?;

        let testfile_path = format!("{}/tmp/puppet_testfile", &jail.mountpoint());
        let testfile = fs::read(testfile_path)?;
        let testfile_content = String::from_utf8_lossy(&testfile);
        assert_eq!(&testfile_content, "/root");

        jail.destroy()?;
        Ok(())
    }

    #[test]
    fn validation() {}
}
