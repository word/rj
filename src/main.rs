use anyhow::Result;
use std::process;

mod cmd;
mod errors;
mod lib;
mod zfs;

fn make_it_so() -> Result<()> {
    let mirror = String::from("ftp.uk.freebsd.org");
    let release = String::from("12.0-RELEASE");
    let dists = ["base", "lib32"];
    let jails_mount = "/jails";

    let jails_ds = zfs::DataSet::new("zroot/jails")?;
    jails_ds.set("mountpoint", &jails_mount)?;
    let basejail_ds = zfs::DataSet::new(&format!("{}/basejail", &jails_ds.get_path()))?;
    let basejail_mount = basejail_ds.get("mountpoint")?;

    // process::exit(0);

    // Extract FreeBSD base jail
    for dist in &dists {
        println!("Extracing {} to {}", &dist, &basejail_mount);

        let url = format!(
            "http://{}/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz",
            mirror, release, dist
        );
        lib::fetch_extract(&url, &basejail_mount)?;
    }

    Ok(())
}

fn main() {
    make_it_so().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    })
}
