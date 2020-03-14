use crate::jail::Jail;
use crate::util;
use anyhow::Result;
use log::info;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreeBSD {
    release: String,
    mirror: String,
    dists: Vec<String>,
}

impl FreeBSD {
    pub fn install(&self, jail: &Jail) -> Result<()> {
        &jail.zfs_ds().create()?;
        info!("{}: installing FreeBSD: {}", &jail.name(), &self.release);

        for dist in &self.dists {
            info!(
                "{}: extracing {} to {}",
                &jail.name(),
                &dist,
                &jail.mountpoint()
            );

            let url = format!(
                "http://{}/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz",
                &self.mirror, &self.release, dist
            );
            util::fetch_extract(&url, &jail.mountpoint())?;
        }
        Ok(())
    }
}
