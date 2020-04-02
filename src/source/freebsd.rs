use crate::jail::Jail;
use crate::util;
use anyhow::Result;
use log::{debug, info};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreeBSD {
    #[serde(skip)] // set in Settings based on the IndexMap key
    pub name: String,
    pub release: String,
    pub mirror: String,
    pub dists: Vec<String>,
}

impl FreeBSD {
    pub fn install(&self, jail: &Jail) -> Result<()> {
        info!(
            "{}: installing FreeBSD from source: {}{}",
            &jail.name(),
            self.name,
            &jail.noop_suffix()
        );
        if !jail.noop() {
            &jail.zfs_ds().create()?;
        }

        for dist in &self.dists {
            info!(
                "{}: fetching and extracting {} to {}{}",
                &jail.name(),
                &dist,
                &jail.mountpoint().display(),
                &jail.noop_suffix(),
            );

            let url = format!(
                "http://{}/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz",
                &self.mirror, &self.release, dist
            );
            if !jail.noop() {
                util::fetch_extract(&url, &jail.mountpoint())?;
            }
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        debug!("Validating FreeBSD source: {}", self.name);
        Ok(())
    }
}
