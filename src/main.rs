use anyhow::Result;
use std::process;

mod cmd;
mod errors;
mod lib;
mod zfs;

use lib::FreeBSDFullRel;

fn make_it_so() -> Result<()> {
    let jails_mount = "/jails";
    let basejail_name = "basejail";
    let basejail_release = "12.0-RELEASE";
    let freebsd_mirror = "ftp.uk.freebsd.org";

    let basejail_rel = FreeBSDFullRel {
        release: basejail_release.to_string(),
        mirror: freebsd_mirror.to_string(),
        dists: vec!["base".to_string(), "lib32".to_string()],
    };

    // Create base data sets
    let jails_ds = zfs::DataSet::new("zroot/jails")?;
    let basejail_ds = zfs::DataSet::new(&format!("{}/basejail", &jails_ds.get_path()))?;
    jails_ds.set("mountpoint", &jails_mount)?;

    if !(basejail_ds.snap_exists("ready")?) {
        // Extract FreeBSD base jail
        basejail_rel.extract(&format!("{}/{}", jails_mount, basejail_name))?;
        basejail_ds.snap("ready")?;
    };

    Ok(())
}

fn main() {
    make_it_so().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    })
}
