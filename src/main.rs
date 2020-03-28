use anyhow::{bail, Result};
use clap::ArgMatches;
use log::{debug, error, info};
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};
use std::process;
use text_io::read;

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
mod volume;
mod zfs;

use jail::Jail;
use provisioner::Provisioner;
use settings::Settings;
use source::Source;
use volume::Volume;

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
    if sub_name == "init" {
        init(&settings)?;
        return Ok(());
    } else {
        check_init(&settings)?
    }

    // Workout which jails to operate on

    let jails = settings.to_jails()?;
    let mut selected_jails = Vec::new();

    if sub_matches.is_present("all") {
        if sub_name == "destroy" {
            // order jails in reverse when destroying all
            for (_, jail) in jails.iter().rev() {
                selected_jails.push(jail);
            }
        } else {
            for (_, jail) in jails.iter() {
                selected_jails.push(jail);
            }
        }
    } else {
        for jail_name in sub_matches.values_of("jail_name").unwrap() {
            if !jails.contains_key(jail_name) {
                bail!("jail '{}' is not defined", jail_name);
            }
            selected_jails.push(&jails[jail_name]);
        }
    }

    // Confirm before destroying

    if sub_name == "destroy" && !sub_matches.is_present("auto-approve") {
        info!(
            "You are about to destroy jails: {}",
            selected_jails
                .iter()
                .map(|j| j.name().to_owned())
                .collect::<Vec<String>>()
                .join(", ")
        );
        info!("Are you sure? [y/n]");
        let answer: String = read!("{}\n");
        if answer != "y" {
            bail!("aborting");
        }
    }

    // run actions on selected jails

    for jail in selected_jails.iter() {
        jail_action(&sub_name, &jail)?
    }

    Ok(())
}

// check that rj has been initialised properly
fn check_init(settings: &Settings) -> Result<()> {
    debug!("checking init");
    let jails_ds = zfs::DataSet::new(&settings.jails_dataset);
    let mut error_msgs: Vec<String> = Vec::new();

    if !jails_ds.exists()? {
        error_msgs.push(format!("jails dataset: {} doesn't exist.", jails_ds.path()));
    }

    if cmd!("sysrc", "-c", "jail_enable=YES").is_err() {
        error_msgs.push("jails not enabled in rc.conf.".to_string());
    }

    if !error_msgs.is_empty() {
        error_msgs.push("Run 'init' to fix".to_string());
        bail!(error_msgs.join(" "));
    }

    Ok(())
}

// initialise rj - currently it creates the jails root dataset and enables jails in rc.conf
fn init(settings: &Settings) -> Result<()> {
    info!("initializing");
    // Create jails root ZFS dataset
    let jails_ds = zfs::DataSet::new(&settings.jails_dataset);
    jails_ds.create()?;
    if jails_ds.get("mountpoint")? != settings.jails_mountpoint {
        jails_ds.set("mountpoint", &settings.jails_mountpoint)?;
    }
    info!("enabling jails in rc.conf");
    cmd!("sysrc", "jail_enable=YES")
}

fn make_it_so(matches: ArgMatches) -> Result<()> {
    // Load settings
    let conf_file = matches.value_of("config").unwrap();
    let noop = matches.is_present("noop");
    let settings = Settings::new(conf_file, noop)?;

    // Execute the subcommand
    if let (sub_name, Some(sub_matches)) = matches.subcommand() {
        subcommand(sub_name, sub_matches, settings)?;
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
    });

    info!("done");
}
