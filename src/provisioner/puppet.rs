use crate::cmd;
use crate::cmd::Cmd;
use crate::cmd_stream;
// use crate::errors::ProvError;
// use crate::errors::RunError;
use crate::jail::Jail;
use crate::pkg::Pkg;
use anyhow::Result;
use log::{debug, info};
use serde::Deserialize;
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

        // TODO: can we trim paths on loading?
        let path_tr = self.path.trim_end_matches('/');
        let tmp_dir_tr = self.tmp_dir.trim_end_matches('/');
        let src_path = Path::new(path_tr);
        let dest_path = Path::new(&jail.mountpoint()).join(tmp_dir_tr);

        self.install_puppet(jail)?;
        self.copy_manifest(&src_path, &dest_path)?;

        let src_dirname = Path::new(&src_path).file_name().unwrap();
        let manifest_path = Path::new(&self.tmp_dir).join(src_dirname);
        let wrapper_path = Path::new(jail.mountpoint())
            .join(&manifest_path)
            .join("puppet_wrapper.sh");

        self.make_wrapper(&manifest_path, &wrapper_path)?;
        // TODO - only if Puppetfile exists
        self.run_r10k(jail, &wrapper_path)?;
        self.run_puppet(jail, &wrapper_path)?;

        Ok(())
    }

    fn install_puppet(&self, jail: &Jail) -> Result<()> {
        let pkg_name = format!("puppet{}", &self.puppet_version);
        let pkg = Pkg::new(&pkg_name, &jail.mountpoint());

        if !pkg.is_installed()? {
            info!("{}: installing {}", jail.name(), pkg_name);
            pkg.install()?;
        }
        Ok(())
    }

    fn copy_manifest(&self, src: &Path, dest: &Path) -> Result<()> {
        fs::create_dir_all(&dest)?;
        set_permissions(&dest, Permissions::from_mode(0o700))?;
        cmd!(
            "rsync",
            "-r",
            "--delete",
            src.to_str().unwrap(),
            dest.to_str().unwrap()
        )
    }

    fn make_wrapper(&self, manifest_path: &Path, wrapper_path: &Path) -> Result<()> {
        let wrapper = format!("#!/bin/sh\ncd {} && $@\n", &manifest_path.to_str().unwrap());
        fs::write(&wrapper_path, wrapper)?;
        set_permissions(&wrapper_path, Permissions::from_mode(0o755))?;
        Ok(())
    }

    fn run_r10k(&self, jail: &Jail, wrapper_path: &Path) -> Result<()> {
        let pkg = Pkg::new("rubygem-r10k", &jail.mountpoint());
        if !pkg.is_installed()? {
            info!("{}: installing {}", jail.name(), "rubygem-r10k");
            pkg.install()?;
        }

        cmd_stream!(
            "jexec",
            jail.name(),
            wrapper_path.to_str().unwrap(),
            "r10k",
            "puppetfile",
            "install"
        )
    }

    fn run_puppet(&self, jail: &Jail, wrapper_path: &Path) -> Result<()> {
        // Construct puppet command
        let mut cmd = Cmd::new("jexec");
        cmd.arg(jail.name())
            .arg(&wrapper_path)
            .arg("puppet")
            .arg("apply");

        if let Some(mp) = &self.module_path {
            cmd.arg("--modulepath").arg(mp);
        }
        if let Some(hc) = &self.hiera_config {
            cmd.arg("--hiera_config").arg(hc);
        }
        for ea in self.extra_args.iter() {
            cmd.arg(ea);
        }
        cmd.arg(&self.manifest_file);
        debug!("{:?}", &cmd);
        cmd.stream()?;
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
            // jail.destroy()?;
        }
        jail.apply()?;

        // FIXME - remove
        jail.provision()?;

        let testfile_path = format!("{}/tmp/puppet_testfile", &jail.mountpoint());
        let testfile = fs::read(testfile_path)?;
        let testfile_content = String::from_utf8_lossy(&testfile);
        assert_eq!(&testfile_content, "/root");

        // jail.destroy()?;
        Ok(())
    }

    #[test]
    fn validation() {}
}
