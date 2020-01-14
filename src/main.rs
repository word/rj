use anyhow::Result;
use std::process;

mod cli;
mod cmd;
mod errors;
mod settings;
mod zfs;
use clap::ArgMatches;
use rj::{Jail, Source};
use settings::Settings;

// Creates all jails
fn create_all(settings: Settings) -> Result<()> {
    for (jname, jconf) in settings.jail.iter() {
        let jail = Jail::new(
            &format!("{}/{}", settings.jails_dataset, jname),
            settings.source[&jconf.source].clone(),
        );

        if jail.exists()? {
            println!("jail '{}' exists already, skipping", jail.name());
        } else {
            println!("Creating jail '{}'", jail.name());
            jail.create()?;
        }
    }

    Ok(())
}

// Create subcommand
fn create(matches: &ArgMatches, settings: Settings) -> Result<()> {
    if matches.is_present("all") {
        return create_all(settings);
    }
    Ok(())
}

fn make_it_so() -> Result<()> {
    let matches = cli::parse_args();

    // FIXME - sort this durig load earlier so it can be immutable
    let mut settings = Settings::new("config.toml")?;

    // Sort jails by 'order' field
    settings
        .jail
        .sort_by(|_, av, _, bv| av.order.cmp(&bv.order));

    // Create jails root dataset
    let jails_ds = zfs::DataSet::new(&settings.jails_dataset);
    jails_ds.create()?;
    jails_ds.set("mountpoint", &settings.jails_mountpoint)?;

    // Process subcommands
    match matches.subcommand() {
        ("create", Some(create_matches)) => {
            create(create_matches, settings)?;
        }
        _ => println!("fuckall"),
    }

    Ok(())
}

fn main() {
    make_it_so().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    })
}
