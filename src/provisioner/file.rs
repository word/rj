use crate::cmd;
use crate::errors::ProvError;
use crate::jail::Jail;
use anyhow::Result;
use log::{debug, info};
use regex::Regex;
use serde::Deserialize;
use std::fs::copy;
use std::fs::{set_permissions, Permissions};
use std::os::unix::prelude::*;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvFile {
    source: String,
    dest: String,
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

impl ProvFile {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: file provisioner running", jail.name());

        let d = Path::new(&self.dest).strip_prefix("/")?;
        let full_dest = Path::new(&jail.mountpoint()).join(d);

        info!(
            "{}: {} -> {}",
            jail.name(),
            &self.source,
            full_dest.display()
        );
        copy(&self.source, &full_dest)?;
        let mode_u32 = u32::from_str_radix(&self.mode, 8)?;
        debug!("{}: mode -> {:o}", jail.name(), mode_u32);
        set_permissions(&full_dest, Permissions::from_mode(mode_u32))?;
        let user_group = format!("{}:{}", &self.owner, &self.group);
        debug!("{}: user:group -> {}", jail.name(), user_group);
        cmd!("chroot", jail.mountpoint(), "chown", user_group, &self.dest)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        debug!("validating file provisioner");
        self.validate_source()?;
        self.validate_dest()?;
        self.validate_mode()
    }

    fn validate_source(&self) -> Result<()> {
        if Path::new(&self.source).is_file() {
            Ok(())
        } else {
            let msg = format!("file provisioner, invalid source: {}", &self.source);
            Err(anyhow::Error::new(ProvError(msg)))
        }
    }

    fn validate_dest(&self) -> Result<()> {
        if Path::new(&self.dest).is_absolute() {
            Ok(())
        } else {
            let msg = format!(
                "file provisioner, dest path must be absolute: {}",
                &self.dest
            );
            Err(anyhow::Error::new(ProvError(msg)))
        }
    }

    fn validate_mode(&self) -> Result<()> {
        let re = Regex::new(r"^(0|1|2|4)?[0-7]{3}$")?;
        if re.is_match(&self.mode) {
            Ok(())
        } else {
            let msg = format!("file provisioner, invalid file mode: {}", &self.mode);
            Err(anyhow::Error::new(ProvError(msg)))
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
        let jail = &jails["file_test"];

        // clean up if left over from a failed test
        if jail.exists()? {
            jail.destroy()?;
        }
        jail.apply()?;

        let full_dest = format!("{}/tmp/file.txt", jail.mountpoint());
        let metadata = std::fs::metadata(full_dest)?;
        assert!(metadata.is_file());
        let mode_s = format!("{:o}", metadata.mode());
        assert_eq!(mode_s, "100640");
        assert_eq!(metadata.uid(), 65534);
        assert_eq!(metadata.gid(), 65534);

        jail.destroy()?;
        Ok(())
    }

    #[test]
    fn validation() {
        let mut file = ProvFile {
            source: "/tmp/whatever123".to_string(),
            dest: "/tmp/desttest".to_string(),
            mode: "640".to_string(),
            owner: "root".to_string(),
            group: "wheel".to_string(),
        };
        assert!(file.validate().is_err());
        file.source = "/etc/hosts".to_string();
        assert!(file.validate().is_ok());
        file.dest = "tmp/desttest".to_string();
        assert!(file.validate().is_err());
        file.dest = "/tmp/desttest".to_string();
        file.mode = "999".to_string();
        assert!(file.validate().is_err());
        file.mode = "3644".to_string();
        assert!(file.validate().is_err());
        file.mode = "0644".to_string();
        assert!(file.validate().is_ok());
    }
}
