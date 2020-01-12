use anyhow::Result;
use std::process;

mod cmd;
mod errors;
mod settings;
mod zfs;
use rj::{Jail, Source};
use settings::Settings;

fn make_it_so() -> Result<()> {
    let settings = settings::Settings::new("config.toml")?;

    // Create jails root dataset
    let jails_ds = zfs::DataSet::new(&settings.jails_dataset);
    jails_ds.create()?;
    jails_ds.set("mountpoint", &settings.jails_mountpoint)?;

    for (key, val) in settings.jail.iter() {
        println!("key: {} val: {:?}", key, val);
    }

    // Create basejail
    // let basejail_rel = Release::FreeBSDFull(FreeBSDFullRel::new(settings.release["12"]));

    // let basejail = Jail::new(
    //     &format!("{}/{}", &settings.jails_mountpoint, "base"),
    //     basejail_rel,
    // );
    // match basejail.exists()? {
    //     true => println!("Basejail '{}' already exists, skipping", &basejail.name()),
    //     false => {
    //         basejail.create()?;
    //     }
    // };

    Ok(())
}

fn main() {
    make_it_so().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    })
}
