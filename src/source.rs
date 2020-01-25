use anyhow::Result;
use log::info;
use serde::Deserialize;

use crate::util;
use crate::zfs;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Source {
    #[serde(alias = "freebsd")]
    FreeBSD {
        release: String,
        mirror: String,
        dists: Vec<String>,
    },
    #[serde(rename = "clone")]
    Cloned { path: String },
}

impl Source {
    pub fn install(&self, dest_path: &str, dest_dataset: &zfs::DataSet) -> Result<()> {
        match &self {
            Source::FreeBSD {
                release,
                mirror,
                dists,
            } => {
                dest_dataset.create()?;
                if !(&dest_dataset.snap_exists("base")?) {
                    self.install_freebsd(release, mirror, dists, dest_path)?;
                    dest_dataset.snap("base")?;
                };
            }
            Source::Cloned { path } => self.install_clone(path, dest_dataset)?,
        }

        Ok(())
    }

    fn install_freebsd(
        &self,
        release: &str,
        mirror: &str,
        dists: &[String],
        dest: &str,
    ) -> Result<()> {
        for dist in dists {
            info!("Extracing {} to {}", &dist, &dest);

            let url = format!(
                "http://{}/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz",
                mirror, release, dist
            );
            util::fetch_extract(&url, &dest)?;
        }
        Ok(())
    }

    fn install_clone(&self, path: &str, dest_dataset: &zfs::DataSet) -> Result<()> {
        let src_dataset = zfs::DataSet::new(path);
        src_dataset.clone(&"ready", dest_dataset.get_path())?;
        Ok(())
    }
}
