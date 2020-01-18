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

fn create_jail(name: &str, settings: &Settings) -> Result<()> {
    let jail = Jail::new(
        &format!("{}/{}", settings.jails_dataset, name),
        settings.source[&settings.jail[name].source].clone(),
    );

    if jail.exists()? {
        info!("jail '{}' exists already, skipping", jail.name());
    } else {
        info!("Creating jail '{}'", jail.name());
        jail.create()?;
    }

    Ok(())
}

// Create subcommand
fn create(matches: &ArgMatches, settings: Settings) -> Result<()> {
    if matches.is_present("all") {
        for (jname, _) in settings.jail.iter() {
            create_jail(jname, &settings)?
        }
    }
    Ok(())
}

fn destroy(matches: &ArgMatches, settings: Settings) -> Result<()> {
    Ok(())
}

fn make_it_so() -> Result<()> {
    let matches = cli::parse_args();
    let settings = Settings::new(matches.value_of("config").unwrap())?;

    // Create jails root dataset
    let jails_ds = zfs::DataSet::new(&settings.jails_dataset);
    jails_ds.create()?;
    jails_ds.set("mountpoint", &settings.jails_mountpoint)?;

    // Process subcommands
    match matches.subcommand() {
        ("create", Some(sub_matches)) => {
            create(sub_matches, settings)?;
        }
        ("destroy", Some(sub_matches)) => {
            destroy(sub_matches, settings)?;
        }
        _ => (), // handled by clap
    }

    Ok(())
}

fn main() {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed)
        .expect("No interactive terminal");

    make_it_so().unwrap_or_else(|err| {
        error!("{}", err);
        process::exit(1);
    })
}
