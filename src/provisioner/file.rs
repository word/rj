use crate::cmd;
use crate::jail::Jail;
use anyhow::{bail, Result};
use log::{debug, info};
use regex::Regex;
use serde::Deserialize;
use std::fs::copy;
use std::fs::{set_permissions, Permissions};
use std::os::unix::prelude::*;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvFile {
    #[serde(skip)] // set in Settings based on the IndexMap key
    pub name: String,
    source: PathBuf,
    dest: PathBuf,
    #[serde(default = "default_mode")]
    mode: String,
    #[serde(default = "default_user")]
    owner: String,
    #[serde(default = "default_group")]
    group: String,
}

fn default_user() -> String {
    "root".to_string()
}

fn default_group() -> String {
    "wheel".to_string()
}

fn default_mode() -> String {
    "644".to_string()
}

impl ProvFile {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: running file provisioner: {}", jail.name(), self.name);

        let full_dest = jail.mountpoint().join(&self.dest.strip_prefix("/")?);

        // copy the file
        info!(
            "{}: {} -> {}{}",
            jail.name(),
            self.source.display(),
            full_dest.display(),
            jail.noop_suffix(),
        );
        if !jail.noop() {
            copy(&self.source, &full_dest)?;
        }

        // set permissions
        let mode_u32 = u32::from_str_radix(&self.mode, 8)?;
        debug!(
            "{}: mode -> {:o}{}",
            jail.name(),
            mode_u32,
            jail.noop_suffix()
        );
        if !jail.noop() {
            set_permissions(&full_dest, Permissions::from_mode(mode_u32))?;
        }

        // set ownership
        let user_group = format!("{}:{}", self.owner, self.group);
        debug!(
            "{}: user:group -> {}{}",
            jail.name(),
            user_group,
            jail.noop_suffix()
        );
        if !jail.noop() {
            cmd!("chroot", jail.mountpoint(), "chown", user_group, &self.dest)?;
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        debug!("validating file provisioner: {}", self.name);
        self.validate_source()?;
        self.validate_dest()?;
        self.validate_mode()
    }

    fn validate_source(&self) -> Result<()> {
        if self.source.is_file() {
            Ok(())
        } else {
            bail!(
                "file provisioner {}, invalid source: {}",
                self.name,
                self.source.display()
            );
        }
    }

    fn validate_dest(&self) -> Result<()> {
        if self.dest.is_absolute() {
            Ok(())
        } else {
            bail!(
                "file provisioner {}, dest path must be absolute: {}",
                self.name,
                self.dest.display()
            )
        }
    }

    fn validate_mode(&self) -> Result<()> {
        let re = Regex::new(r"^(0|1|2|4)?[0-7]{3}$")?;
        if re.is_match(&self.mode) {
            Ok(())
        } else {
            bail!(
                "file provisioner {}, invalid file mode: {}",
                self.name,
                self.mode
            )
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
    use std::fs;
    use std::os::unix::fs::MetadataExt;

    #[test]
    #[serial]
    fn provision() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jail = setup(&s, "file_test")?;

        jail.apply()?;

        let path = jail.mountpoint().join("tmp/file.txt");
        assert!(path.is_file());
        let metadata = std::fs::metadata(path)?;
        let mode_s = format!("{:o}", metadata.mode());
        assert_eq!(mode_s, "100640");
        assert_eq!(metadata.uid(), 65534);
        assert_eq!(metadata.gid(), 65534);

        jail.destroy()?;
        Ok(())
    }

    #[test]
    #[serial]
    fn provision_noop() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
        let jail = setup(&s, "file_test")?;

        jail.apply()?;
        // remove the copied test file
        let path = jail.mountpoint().join("tmp/file.txt");
        assert_eq!(path.is_file(), true);
        fs::remove_file(&path)?;

        // make a noop jail
        let s_noop = Settings::new("testdata/config.toml", true)?;
        let jail_noop = setup(&s_noop, "file_test")?;

        // on provision it shoudn't re-create the file
        jail_noop.provision()?;
        assert_eq!(path.is_file(), false);

        jail.destroy()?;
        Ok(())
    }

    #[test]
    fn validate() {
        let mut file = ProvFile {
            name: "test".to_owned(),
            source: PathBuf::from("/tmp/whatever123"),
            dest: PathBuf::from("/tmp/desttest"),
            mode: "640".to_string(),
            owner: "root".to_string(),
            group: "wheel".to_string(),
        };
        assert!(file.validate().is_err());
        file.source = PathBuf::from("/etc/hosts");
        assert!(file.validate().is_ok());
        file.dest = PathBuf::from("tmp/desttest");
        assert!(file.validate().is_err());
        file.dest = PathBuf::from("/tmp/desttest");
        file.mode = "999".to_string();
        assert!(file.validate().is_err());
        file.mode = "3644".to_string();
        assert!(file.validate().is_err());
        file.mode = "0644".to_string();
        assert!(file.validate().is_ok());
    }
}
