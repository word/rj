use std::process;

mod cmd;
mod errors;
mod zfs;

use anyhow::Result;
use reqwest;
use std::fs;
use std::path::Path;
use tar::Archive;
use xz2::read::XzDecoder;

fn fetch_extract(url: &str, dest: &str) -> Result<()> {
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

fn main() {
    let j_ds = String::from("zroot/jails");
    let bj_ds = format!("{}/basejail", &j_ds);
    let bj_dir = String::from("/jails/basejail");
    let mirror = String::from("ftp.uk.freebsd.org");
    let release = String::from("12.0-RELEASE");
    let dists = ["base", "lib32"];

    for set in vec![&j_ds, &bj_ds] {
        println!("Creating jail data set {}", set);
        zfs::create_ds(&set).unwrap_or_else(|err| {
            eprintln!("ERROR: {}", err);
            process::exit(1);
        });
    }

    for dist in &dists {
        println!("Extracing {} to {}", &dist, &bj_dir);

        let url = format!(
            "http://{}/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz",
            mirror, release, dist
        );
        fetch_extract(&url, &bj_dir).unwrap_or_else(|err| {
            eprintln!("ERROR: {:?}", err);
            process::exit(1);
        });
    }
}
