use anyhow::Result;
use std::process;

mod cmd;
mod errors;
mod settings;
mod zfs;
use rj::{Jail, Source};
use settings::Settings;

fn make_it_so() -> Result<()> {
    let mut settings = Settings::new("config.toml")?;

    // Sort jails by 'order' field
    settings
        .jail
        .sort_by(|_, av, _, bv| av.order.cmp(&bv.order));

    // Create jails root dataset
    let jails_ds = zfs::DataSet::new(&settings.jails_dataset);
    jails_ds.create()?;
    jails_ds.set("mountpoint", &settings.jails_mountpoint)?;

    // Create jails
    for (jname, jconf) in settings.jail.iter() {
        let jail = Jail::new(
            &format!("{}/{}", settings.jails_dataset, jname),
            &settings.source[&jconf.source],
        );
        jail.create()?;
    }

    Ok(())
}

fn main() {
    make_it_so().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    })
}
