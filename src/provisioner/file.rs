use crate::cmd;
use crate::jail::Jail;
use anyhow::Result;
use log::{debug, info};
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

        copy(&self.source, &full_dest)?;
        let mode_u32 = u32::from_str_radix(&self.mode, 8)?;
        set_permissions(&full_dest, Permissions::from_mode(mode_u32))?;
        let user_group = format!("{}:{}", &self.owner, &self.group);
        cmd!("chroot", jail.mountpoint(), "chown", user_group, &self.dest)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        debug!("validating file provisioner");
        // check source exists
        // check dest is absolute path
        // check mode is valid (regex)
        Ok(())
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
}
