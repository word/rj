use anyhow::Result;
use clap::ArgMatches;
use log::{debug, error, info};
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

    jail.create()?;

    Ok(())
}

// Create subcommand
fn create(matches: &ArgMatches, settings: Settings) -> Result<()> {
    if matches.is_present("all") {
        for (jname, _) in settings.jail.iter() {
            create_jail(jname, &settings)?
        }
        return Ok(());
    }

    let jname = matches.value_of("jail_name").unwrap();
    debug!("jail name: {}", jname);

    if settings.jail.contains_key(jname) {
        create_jail(jname, &settings)
    } else {
        Err(anyhow::Error::new(errors::ArgError(format!(
            "Jail '{}' is not defined",
            jname
        ))))
    }
}

fn destroy(_matches: &ArgMatches, _settings: Settings) -> Result<()> {
    Ok(())
}

fn make_it_so(matches: ArgMatches) -> Result<()> {
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
        _ => {} // handled by clap
    }

    Ok(())
}

fn main() {
    let matches = cli::parse_args();

    if matches.is_present("debug") {
        TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed)
            .expect("No interactive terminal");
    } else {
        TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed)
            .expect("No interactive terminal");
    }

    make_it_so(matches).unwrap_or_else(|err| {
        error!("{}", err);
        process::exit(1);
    })
}
