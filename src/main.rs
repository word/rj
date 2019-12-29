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

    let j_ds = zfs::DataSet::new(
        "zroot/jails".to_string(),
        "/jails".to_string(),
    )?;
    let bj_ds = zfs::DataSet::new(
        format!("{}/basejail", &j_ds.path),
        format!("{}/basejail", &j_ds.mountpoint),
    )?;

    // process::exit(0);

    // Extract FreeBSD base jail
    for dist in &dists {
        println!("Extracing {} to {}", &dist, &bj_ds.mountpoint);

        let url = format!(
            "http://{}/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz",
            mirror, release, dist
        );
        lib::fetch_extract(&url, &bj_ds.mountpoint)?;
    }

    Ok(())
}

fn main() {
    make_it_so().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    })
}
