#![allow(dead_code)]
use anyhow::Result;
use std::fs;
use std::path::Path;
use tar::Archive;
use xz2::read::XzDecoder;

mod cmd;
mod errors;
mod zfs;

use crate::errors::JailError;

pub struct Jail {
    name: String,
    mountpoint: String,
    release: Release,
    zfs_ds_path: String,
    zfs_ds: zfs::DataSet,
}

impl Jail {
    pub fn new(path: &str, release: Release) -> Jail {
        let mut components: Vec<&str> = path.split("/").collect();
        components.remove(0); // remove the zfs pool name

        Jail {
            name: components.last().unwrap().to_string(),
            mountpoint: format!("/{}", components.join("/")),
            release: release,
            zfs_ds_path: path.to_string(),
            zfs_ds: zfs::DataSet::new(path),
        }
    }

    pub fn create(&self) -> Result<()> {
        if self.exists()? {
            let err = JailError(format!(
                "Can\'t create jail: {}, already exists",
                &self.zfs_ds_path
            ));
            return Err(anyhow::Error::new(err));
        };
        match &self.release {
            Release::FreeBSDFull(r) => {
                self.zfs_ds.create()?;
                if !(&self.zfs_ds.snap_exists("base")?) {
                    r.extract(&self.mountpoint)?;
                    &self.zfs_ds.snap("base")?;
                };
            }
            Release::ZfsTemplate(src_dataset) => {
                src_dataset.clone(&"ready", self.zfs_ds.get_path())?;
            }
        };
        // TODO - should be moved to provisioner maybe perhaps
        self.zfs_ds.snap("ready")
    }

    pub fn destroy(&self) -> Result<()> {
        let snaps = self.zfs_ds.list_snaps()?;
        if !(snaps.is_empty()) {
            for snap in snaps {
                self.zfs_ds.snap_destroy(&snap)?;
            }
        }
        self.zfs_ds.destroy()
    }
    pub fn update() {}
    pub fn snapshot() {}
    pub fn start() {}
    pub fn stop() {}
    pub fn enable() {}
    pub fn provision() {}
    pub fn exists(&self) -> Result<bool> {
        self.zfs_ds.exists()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;
    use std::sync::Once;

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
        let release = Release::ZfsTemplate(basejail.zfs_ds);
        let jail = Jail::new("zroot/jails/thinjail", release);
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
        assert!(result.is_err(), "{:?}", result);

        // let error = result.unwrap_err();
        assert_eq!(
            result.unwrap_err().to_string(),
            format!(
                "Can't create jail: {}, already exists",
                basejail.zfs_ds_path
            )
        )
    }

    static INIT: Once = Once::new();

    pub fn setup_once() -> Jail {
        // Setup the basejail
        let release = Release::FreeBSDFull(FreeBSDFullRel {
            release: "12.0-RELEASE".to_string(),
            mirror: "ftp.uk.freebsd.org".to_string(),
            dists: vec!["base".to_string(), "lib32".to_string()],
            // dists: vec![], // extracts quicker...
        });
        let basejail = Jail::new("zroot/jails/basejail", release);
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
}

pub enum Release {
    FreeBSDFull(FreeBSDFullRel),
    ZfsTemplate(zfs::DataSet),
}

pub struct FreeBSDFullRel {
    pub release: String,
    pub dists: Vec<String>,
    pub mirror: String,
}

impl FreeBSDFullRel {
    pub fn extract(&self, dest: &str) -> Result<()> {
        for dist in &self.dists {
            println!("Extracing {} to {}", &dist, &dest);

            let url = format!(
                "http://{}/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz",
                &self.mirror, &self.release, dist
            );
            fetch_extract(&url, &dest)?;
        }
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
