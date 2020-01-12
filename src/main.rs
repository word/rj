use anyhow::Result;
use std::process;

mod cmd;
mod errors;
mod settings;
mod zfs;
use rj::{FreeBSDFullRel, Jail, Release};

fn make_it_so() -> Result<()> {
    let jails_mount = "/jails";
    let basejail_name = "basejail";
    let basejail_release = "12.0-RELEASE";
    let freebsd_mirror = "ftp.uk.freebsd.org";

    let basejail_rel = Release::FreeBSDFull(FreeBSDFullRel {
        release: basejail_release.to_string(),
        mirror: freebsd_mirror.to_string(),
        dists: vec!["base".to_string(), "lib32".to_string()],
    });

    // Create jail data set
    let jails_ds = zfs::DataSet::new("zroot/jails");
    jails_ds.create()?;
    jails_ds.set("mountpoint", &jails_mount)?;

    // Create basejail
    let basejail = Jail::new(
        &format!("{}/{}", &jails_ds.get_path(), basejail_name),
        basejail_rel,
    );
    match basejail.exists()? {
        true => println!("Basejail '{}' already exists, skipping", &basejail.name()),
        false => {
            basejail.create()?;
        }
    };

    Ok(())
}

fn main() {
    make_it_so().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    })
}
