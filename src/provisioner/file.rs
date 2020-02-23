use crate::cmd;
use crate::jail::Jail;
use anyhow::Result;
use log::{debug, info};
use serde::Deserialize;
use std::fs::copy;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvFile {
    source: String,
    dest: String,
    mode: String,
}

impl ProvFile {
    pub fn provision(&self, jail: &Jail) -> Result<()> {
        info!("{}: file provisioner running", jail.name());

        let d = Path::new(&self.dest).strip_prefix("/")?;
        let jail_dest = Path::new(&jail.mountpoint()).join(d);

        println!("{:?}", &jail.mountpoint());
        println!("{:?}", &jail_dest);
        copy(&self.source, &jail_dest)?;
        let f = File::create(&jail_dest)?;
        let metadata = f.metadata()?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(u32::from_str_radix(&self.mode, 8)?);
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
    use serial_test::serial;

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

        cmd!("jexec", jail.name(), "test", "-f", "/tmp/file.txt")?;

        jail.destroy()?;
        Ok(())
    }
}
