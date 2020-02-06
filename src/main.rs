use anyhow::Result;
use clap::ArgMatches;
use indexmap::IndexMap;
use log::{debug, error};
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};
use std::process;
use std::process::Command;

mod cli;
mod cmd;
mod errors;
mod jail;
mod settings;
mod source;
mod templates;
mod util;
mod zfs;

use jail::Jail;
use settings::Settings;
use source::Source;

fn jail_action(action: &str, jail: &Jail) -> Result<()> {
    debug!("action {}", action);
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
    jails: IndexMap<String, Jail>,
) -> Result<()> {
    if sub_matches.is_present("all") {
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
        Err(anyhow::Error::new(errors::ArgError(format!(
            "Jail '{}' is not defined",
            jname
        ))))
    }
}

fn init(settings: &Settings) -> Result<()> {
    // Create jails root ZFS dataset
    let jails_ds = zfs::DataSet::new(&settings.jails_dataset);
    jails_ds.create()?;
    if jails_ds.get("mountpoint")? != settings.jails_mountpoint {
        jails_ds.set("mountpoint", &settings.jails_mountpoint)?;
    }
    // enable jails in rc.conf
    let mut sysrc = Command::new("sysrc");
    sysrc.arg("jail_enable=YES");
    cmd::run(&mut sysrc)?;
    Ok(())
}

fn make_it_so(matches: ArgMatches) -> Result<()> {
    let settings = Settings::new(matches.value_of("config").unwrap())?;
    let jails = settings.to_jails()?;

    // TODO - put this behind a subcommand
    init(&settings)?;

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
