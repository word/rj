use anyhow::Result;
use std::process;

mod cmd;
mod errors;
mod lib;
mod zfs;

fn make_it_so() -> Result<()> {
    let j_ds = String::from("zroot/jails");
    let bj_ds = format!("{}/basejail", &j_ds);
    let bj_dir = String::from("/jails/basejail");
    let mirror = String::from("ftp.uk.freebsd.org");
    let release = String::from("12.0-RELEASE");
    let dists = ["base", "lib32"];

    // Create ZFS data sets for jails
    for set in vec![&j_ds, &bj_ds] {
        println!("Creating zfs data set {}", set);
        zfs::create_ds(&set)?;
    }

    // process::exit(0);

    // Extract FreeBSD base jail
    for dist in &dists {
        println!("Extracing {} to {}", &dist, &bj_dir);

        let url = format!(
            "http://{}/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz",
            mirror, release, dist
        );
        lib::fetch_extract(&url, &bj_dir)?;
    }

    Ok(())
}

fn main() {
    make_it_so().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    })
}
