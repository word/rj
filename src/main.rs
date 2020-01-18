use anyhow::Result;
use clap::ArgMatches;
use log::{error, info};
use simplelog::*;
use std::process;

mod cli;
mod cmd;
mod errors;
mod settings;
mod zfs;
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
            info!("jail '{}' exists already, skipping", jail.name());
        } else {
            info!("Creating jail '{}'", jail.name());
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
    //TODO - set default config to /usr/local/etc/rj.toml
    //       and allow changing via a cli flag
    let settings = Settings::new("config.toml")?;

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
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed)
        .expect("No interactive terminal");

    make_it_so().unwrap_or_else(|err| {
        error!("ERROR: {}", err);
        process::exit(1);
    })
}
