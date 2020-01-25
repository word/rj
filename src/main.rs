use anyhow::Result;
use clap::ArgMatches;
use indexmap::IndexMap;
use log::{debug, error};
use simplelog::*;
use std::process;

mod cli;
mod cmd;
mod errors;
mod jail;
mod settings;
mod templates;
mod zfs;

use jail::{Jail, Source};
use settings::Settings;

fn jail_action(action: &str, jail: &Jail) -> Result<()> {
    match action {
        "create" => jail.create(),
        "destroy" => jail.destroy(),
        _ => panic!("unknown action {}", action),
    }
}

// process the subcommand
fn subcommand(
    sub_name: &str,
    sub_matches: &ArgMatches,
    mut jails: IndexMap<String, Jail>,
) -> Result<()> {
    if sub_matches.is_present("all") {
        // destroy jails in reverse order
        if sub_name == "destroy" {
            jails.sort_by(|_, av, _, bv| bv.order().cmp(&av.order()));
        }

        for (_, jail) in jails.iter() {
            jail_action(&sub_name, &jail)?
        }
        return Ok(());
    }

    let jname = sub_matches.value_of("jail_name").unwrap();
    debug!("jail name: {}", jname);

    if jails.contains_key(jname) {
        jail_action(&sub_name, &jails[jname])
    } else {
        Err(anyhow::Error::new(errors::ArgError(format!(
            "Jail '{}' is not defined",
            jname
        ))))
    }
}

fn make_it_so(matches: ArgMatches) -> Result<()> {
    let settings = Settings::new(matches.value_of("config").unwrap())?;

    // Create an IndexMap of jails from settings
    let mut jails = IndexMap::new();
    for (jname, jsettings) in settings.jail.iter() {
        let jail = Jail::new(
            // data set path
            &format!("{}/{}", &settings.jails_dataset, &jname),
            // jail source
            &settings.source[&jsettings.source],
            // jail conf
            &jsettings,
            &settings.jail_conf_defaults,
        );
        jails.insert(jname.to_string(), jail);
    }

    // Create jails root ZFS dataset
    let jails_ds = zfs::DataSet::new(&settings.jails_dataset);
    jails_ds.create()?;
    jails_ds.set("mountpoint", &settings.jails_mountpoint)?;

    // Execute the subcommand
    if let (sub_name, Some(sub_matches)) = matches.subcommand() {
        subcommand(sub_name, sub_matches, jails)?;
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
    })
}
