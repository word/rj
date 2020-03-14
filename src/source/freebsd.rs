use crate::util;
use crate::zfs;
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
    pub fn install(&self, dest_path: &str, dest_dataset: &zfs::DataSet) -> Result<()> {
        dest_dataset.create()?;
        info!("installing FreeBSD: {}", &self.release);

        for dist in &self.dists {
            info!("extracing {} to {}", &dist, &dest_path);

            let url = format!(
                "http://{}/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz",
                &self.mirror, &self.release, dist
            );
            util::fetch_extract(&url, &dest_path)?;
        }
        Ok(())
    }
}
