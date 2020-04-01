use crate::cmd;
use crate::cmd::Cmd;
use crate::cmd_stream;
use crate::jail::Jail;
use crate::pkg::Pkg;
use anyhow::{bail, Result};
use log::{debug, info};
use serde::Deserialize;
use std::fs;
use std::fs::{set_permissions, Permissions};
use std::os::unix::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Puppet {
    #[serde(skip)] // set in Settings based on the IndexMap key
    pub name: String,
    pub path: PathBuf,
    #[serde(default = "default_manifest_file")]
    pub manifest_file: PathBuf,
    #[serde(default)]
    pub module_path: Option<String>,
    #[serde(default)]
    pub hiera_config: Option<PathBuf>,
    #[serde(default = "default_extra_args")]
    pub extra_args: Vec<String>,
    #[serde(default = "default_tmp_dir")]
    pub tmp_dir: PathBuf,
    #[serde(default = "default_version")]
    pub puppet_version: String,
}

fn default_manifest_file() -> PathBuf {
    PathBuf::from("init.pp")
}

fn default_tmp_dir() -> PathBuf {
    PathBuf::from("/var/rj")
}

fn default_version() -> String {
    "6".to_string()
}

fn default_extra_args() -> Vec<String> {
    vec![]
}

impl Puppet {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: puppet provisioner running", jail.name());

        let dest_path = jail.mountpoint().join(self.tmp_dir.strip_prefix("/")?);
        self.install_puppet(jail)?;
        self.copy_manifest(&self.path, &dest_path)?;

        let manifest_inside_path = &self.tmp_dir.join(self.path.file_name().unwrap());
        let manifest_outside_path = jail
            .mountpoint()
            .join(&manifest_inside_path.strip_prefix("/")?);
        let wrapper_outside_path = manifest_outside_path.join("wrapper.sh");
        let wrapper_inside_path = manifest_inside_path.join("wrapper.sh");

        self.make_wrapper(&manifest_inside_path, &wrapper_outside_path)?;
        if manifest_outside_path.join("Puppetfile").is_file() {
            self.run_r10k(jail, &wrapper_inside_path)?;
        }
        self.run_puppet(jail, &wrapper_inside_path)?;

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

    // Wrapper changes into the root of the manifest directory before executing a command.  Used for executing puppet and r10k.
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
        if *jail.noop() {
            cmd.arg("--noop");
        }
        cmd.arg(&self.manifest_file);
        debug!("{:?}", &cmd);
        cmd.stream()?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        debug!("validating puppet provisioner");
        self.validate_path()?;
        self.validate_manifest_file()?;
        self.validate_hiera_config()?;
        self.validate_puppet_version()?;
        Ok(())
    }

    fn validate_path(&self) -> Result<()> {
        if self.path.is_dir() {
            Ok(())
        } else {
            bail!(
                "puppet provisioner, path: {} doesn't exist or is not a directory",
                &self.path.display()
            )
        }
    }

    fn validate_manifest_file(&self) -> Result<()> {
        let manifest_file_path = self.path.join(&self.manifest_file);
        if manifest_file_path.is_file() {
            Ok(())
        } else {
            bail!(
                "puppet provisioner, manifest file doesn't exist in: {} or is not a file",
                manifest_file_path.display()
            )
        }
    }

    fn validate_hiera_config(&self) -> Result<()> {
        if let Some(h) = &self.hiera_config {
            let hiera_config_path = self.path.join(h);
            if hiera_config_path.is_file() {
                return Ok(());
            } else {
                bail!(
                    "puppet provisioner, hiera config file doesn't exist in: {} or is not a file",
                    hiera_config_path.display()
                )
            }
        }

        Ok(())
    }

    fn validate_puppet_version(&self) -> Result<()> {
        if ["5", "6"].contains(&self.puppet_version.as_str()) {
            Ok(())
        } else {
            bail!(
                "puppet provisioner, puppet version: {} is not supported",
                &self.puppet_version
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provisioner::test_helpers::setup;
    use crate::settings::Settings;
    use pretty_assertions::assert_eq;
    use serial_test::serial;

    #[test]
    #[serial]
    fn provision() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jail = setup(&s, "puppet_test")?;

        jail.apply()?;

        let testfile_path = &jail.mountpoint().join("tmp/puppet_testfile");
        let testfile = fs::read(testfile_path)?;
        let testfile_content = String::from_utf8_lossy(&testfile);
        assert_eq!(&testfile_content, "/root");

        jail.destroy()?;
        Ok(())
    }

    #[test]
    #[serial]
    fn provision_noop() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jail = setup(&s, "puppet_test")?;

        jail.apply()?;
        // remove the created test file
        let path = Path::new(jail.mountpoint()).join("tmp/puppet_testfile");
        fs::remove_file(&path)?;

        // make a noop jail
        let s_noop = Settings::new("testdata/config.toml", true)?;
        let jail_noop = setup(&s_noop, "puppet_test")?;

        // on noop provision it shoudn't re-create the file
        jail_noop.provision()?;
        assert_eq!(path.is_file(), false);

        jail.destroy()?;
        Ok(())
    }

    // Test the simplest case where only init.pp is defined.
    #[test]
    #[serial]
    fn provision_simple() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jail = setup(&s, "puppet_simple_test")?;

        jail.apply()?;

        let testfile_path = &jail.mountpoint().join("tmp/puppet_simple_testfile");
        let testfile = fs::read(testfile_path)?;
        let testfile_content = String::from_utf8_lossy(&testfile);
        assert_eq!(&testfile_content, "simple");

        jail.destroy()?;
        Ok(())
    }

    #[test]
    fn validate() {
        let mut puppet = Puppet {
            name: "test".to_owned(),
            path: PathBuf::from("testdata/provisioners/puppet"),
            manifest_file: PathBuf::from("manifests/site.pp"),
            module_path: Some("site-modules:modules".to_string()),
            hiera_config: Some(PathBuf::from("hiera.yaml")),
            extra_args: vec![],
            tmp_dir: default_tmp_dir(),
            puppet_version: default_version(),
        };

        assert!(puppet.validate().is_ok());
        puppet.path = PathBuf::from("nonexistent");
        assert!(puppet.validate().is_err());
        puppet.path = PathBuf::from("testdata/provisioners/puppet");
        puppet.manifest_file = PathBuf::from("nonexistent");
        assert!(puppet.validate().is_err());
        puppet.manifest_file = PathBuf::from("manifests/site.pp");
        puppet.hiera_config = Some(PathBuf::from("nonexistent"));
        assert!(puppet.validate().is_err());
        puppet.hiera_config = Some(PathBuf::from("hiera.yaml"));
        puppet.puppet_version = "23".to_string();
        assert!(puppet.validate().is_err());
        puppet.puppet_version = default_version();
        assert!(puppet.validate().is_ok());
    }
}
