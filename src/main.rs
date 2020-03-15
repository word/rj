use anyhow::{Error, Result};
use clap::ArgMatches;
use log::{debug, error, info};
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};
use std::process;

mod cli;
mod cmd;
mod errors;
mod jail;
mod pkg;
mod provisioner;
mod settings;
mod source;
mod templates;
mod util;
mod zfs;

use errors::ArgError;
use jail::Jail;
use provisioner::Provisioner;
use settings::Settings;
use source::Source;

fn jail_action(action: &str, jail: &Jail) -> Result<()> {
    debug!("action {}", action);
    match action {
        "apply" => jail.apply(),
        "destroy" => jail.destroy(),
        "provision" => jail.provision(),
        _ => panic!("unknown action {}", action),
    }
}

// process the subcommand
fn subcommand(sub_name: &str, sub_matches: &ArgMatches, settings: Settings) -> Result<()> {
    let jails = &settings.to_jails()?;

    if sub_name == "init" {
        init(&settings)?;
        return Ok(());
    }

    // work on all jails if --all is set
    if sub_matches.is_present("all") {
        debug!("working on all jails");
        if sub_name == "destroy" {
            // destroy jails in reverse order
            for (_, jail) in jails.iter().rev() {
                jail_action(&sub_name, &jail)?
            }
        } else {
            for (_, jail) in jails.iter() {
                jail_action(&sub_name, &jail)?
            }
        }
        return Ok(());
    }

    let jname = sub_matches.value_of("jail_name").unwrap();
    debug!("jail name: {}", jname);

    if jails.contains_key(jname) {
        jail_action(&sub_name, &jails[jname])
    } else {
        let msg = format!("jail '{}' is not defined", jname);
        let err = Error::new(ArgError(msg));
        Err(err)
    }
}

fn init(settings: &Settings) -> Result<()> {
    info!("initializing");
    // Create jails root ZFS dataset
    let jails_ds = zfs::DataSet::new(&settings.jails_dataset);
    jails_ds.create()?;
    if jails_ds.get("mountpoint")? != settings.jails_mountpoint {
        jails_ds.set("mountpoint", &settings.jails_mountpoint)?;
    }
    // enable jails in rc.conf
    cmd!("sysrc", "jail_enable=YES")
}

fn make_it_so(matches: ArgMatches) -> Result<()> {
    // Load settings
    let settings = Settings::new(matches.value_of("config").unwrap())?;

    // Execute the subcommand
    if let (sub_name, Some(sub_matches)) = matches.subcommand() {
        subcommand(sub_name, sub_matches, settings)?;
    }

    Ok(())
}

fn main() {
    let matches = cli::parse_args();

    // TODO: set log level using env
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
    });

    info!("done");
}
