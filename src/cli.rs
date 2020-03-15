use clap::{crate_description, crate_name, crate_version, AppSettings, Arg, SubCommand};
use std::path::Path;

// Create a clap app
fn create_app<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .set_term_width(80)
        .setting(AppSettings::InferSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .env("RJ_CONFIG")
                .value_name("FILE")
                .help("Config file path")
                .takes_value(true)
                .validator(is_valid_config)
                .default_value("rj.conf"),
        )
        .arg(
            Arg::with_name("debug")
                .env("RJ_DEBUG")
                .short("d")
                .long("debug")
                .help("Show debug messages"),
        )
        .subcommand(
            SubCommand::with_name("apply")
                .about("Apply changes")
                .arg(
                    Arg::with_name("jail_name")
                        .multiple(true)
                        .help("Name of the jail to apply changes to")
                        .index(1)
                        .required_unless("all"),
                )
                .arg(
                    Arg::with_name("all")
                        .short("a")
                        .long("all")
                        .help("Apply to all jails"),
                ),
        )
        .subcommand(
            SubCommand::with_name("destroy")
                .about("Destroy jails")
                .arg(
                    Arg::with_name("jail_name")
                        .multiple(true)
                        .help("Name of the jail to destroy")
                        .index(1)
                        .required_unless("all"),
                )
                .arg(
                    Arg::with_name("all")
                        .short("a")
                        .long("all")
                        .help("Destroy all jails"),
                ),
        )
        .subcommand(
            SubCommand::with_name("provision")
                .about("Provision jails")
                .arg(
                    Arg::with_name("jail_name")
                        .multiple(true)
                        .help("Name of the jail to provision")
                        .index(1)
                        .required_unless("all"),
                )
                .arg(
                    Arg::with_name("all")
                        .short("a")
                        .long("all")
                        .help("Provision all jails"),
                ),
        )
        .subcommand(SubCommand::with_name("init").about("Initialise rj"))
}

// Parses the command line arguments and returns the matches.
pub fn parse_args<'a>() -> clap::ArgMatches<'a> {
    create_app().get_matches()
}

fn is_valid_config(s: String) -> Result<(), String> {
    if !Path::new(&s).is_file() {
        return Err(format!("file not found: {}", &s));
    }
    Ok(())
}
