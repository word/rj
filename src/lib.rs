#![allow(dead_code)]
use anyhow::Result;
use log::info;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tar::Archive;
use xz2::read::XzDecoder;

mod cmd;
mod errors;
mod settings;
mod zfs;

pub struct Jail {
    name: String,
    mountpoint: String,
    source: Source,
    zfs_ds_path: String,
    zfs_ds: zfs::DataSet,
    order: i16,
}

impl Jail {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn mountpoint(&self) -> &String {
        &self.mountpoint
    }

    pub fn order(&self) -> &i16 {
        &self.order
    }

    pub fn new(ds_path: &str, source: Source, order: i16) -> Jail {
        let mut components: Vec<&str> = ds_path.split('/').collect();
        components.remove(0); // remove the zfs pool name

        Jail {
            name: components.last().unwrap().to_string(),
            mountpoint: format!("/{}", components.join("/")),
            source,
            zfs_ds_path: ds_path.to_string(),
            zfs_ds: zfs::DataSet::new(ds_path),
            order,
        }
    }

    pub fn create(&self) -> Result<()> {
        if self.exists()? {
            info!("jail '{}' exists already, skipping", self.name());
            return Ok(());
        };

        info!("Creating jail '{}'", self.name());
        // install the jail using whatever source
        self.source.install(&self.mountpoint, &self.zfs_ds)?;
        // TODO - should be moved to provisioner maybe perhaps
        self.zfs_ds.snap("ready")?;
        Ok(())
    }

    pub fn destroy(&self) -> Result<()> {
        if self.exists()? {
            info!("Destroying jail '{}'", &self.name);
        } else {
            info!("Jail '{}' doesn't exist, skipping", &self.name);
            return Ok(());
        }

        let snaps = self.zfs_ds.list_snaps()?;
        if !(snaps.is_empty()) {
            for snap in snaps {
                self.zfs_ds.snap_destroy(&snap)?;
            }
        }
        self.zfs_ds.destroy()
    }
    pub fn update() {}
    pub fn start() {}
    pub fn stop() {}
    pub fn enable() {}
    pub fn provision() {}
    pub fn exists(&self) -> Result<bool> {
        self.zfs_ds.exists()
    }
}

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
            fetch_extract(&url, &dest)?;
        }
        Ok(())
    }

    fn install_clone(&self, path: &str, dest_dataset: &zfs::DataSet) -> Result<()> {
        let src_dataset = zfs::DataSet::new(path);
        src_dataset.clone(&"ready", dest_dataset.get_path())?;
        Ok(())
    }
}

// fetch an xz archive using http and extract it to a destination directory
pub fn fetch_extract(url: &str, dest: &str) -> Result<()> {
    let response = reqwest::get(url)?;
    let decompressor = XzDecoder::new(response);
    let mut archive = Archive::new(decompressor);
    archive.set_preserve_permissions(true);

    // Remove any matching hard/soft links at the destination before extracting files.  This is
    // necessary because Tar won't overwrite links and panics instead.  This effectively will
    // overwrite any existing files and links at the destination.
    for (_, file) in archive.entries()?.enumerate() {
        let mut file = file?;
        let file_dest = Path::new(dest).join(file.path()?);

        // Check if the archived file is a link (hard and soft)
        if file.header().link_name().unwrap().is_some() {
            // Check if the link exist at the destination already.  Using symlink_metadata here
            // becasue is_file returns false for symlinks.  This will match any file, symlink or
            // hardlink.
            if file_dest.symlink_metadata().is_ok() {
                // remove the link so it can be extracted later
                // println!("removing link {:?}", &file_dest);
                fs::remove_file(&file_dest)?;
            }
        }
        file.unpack_in(dest)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn setup_once() -> Jail {
        // Setup the basejail
        let source = Source::FreeBSD {
            release: "12.0-RELEASE".to_string(),
            mirror: "ftp.uk.freebsd.org".to_string(),
            dists: vec!["base".to_string(), "lib32".to_string()],
            // dists: vec![], // extracts quicker...
        };
        let basejail = Jail::new("zroot/jails/basejail", source, 0);
        let jails_ds = zfs::DataSet::new("zroot/jails");

        INIT.call_once(|| {
            // cleanup before all
            // jails_ds.destroy_r().unwrap();
            jails_ds.create().unwrap();
            jails_ds.set("mountpoint", "/jails").unwrap();
            if !(basejail.exists().unwrap()) {
                basejail.create().unwrap();
            }
        });

        basejail
    }

    #[test]
    fn test_jail_mountpoint() {
        let basejail = setup_once();
        assert_eq!(basejail.mountpoint, "/jails/basejail");
    }

    #[test]
    fn test_jail_name() {
        let basejail = setup_once();
        assert_eq!(basejail.name, "basejail");
    }

    #[test]
    fn test_jail_thin_create_destroy() -> Result<()> {
        let basejail = setup_once();
        let source = Source::Cloned {
            path: basejail.zfs_ds.get_path().to_string(),
        };
        let jail = Jail::new("zroot/jails/thinjail", source, 0);
        jail.create()?;
        assert!(jail.exists()?);
        jail.destroy()?;
        assert!(!jail.exists()?);
        Ok(())
    }

    #[test]
    fn test_jail_create_existing() {
        let basejail = setup_once();
        let result = basejail.create();
        assert!(result.is_ok());
    }
}
