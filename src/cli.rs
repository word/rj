use clap::{crate_description, crate_name, crate_version, AppSettings, Arg, SubCommand};

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
                .value_name("FILE")
                .help("Config file path")
                .takes_value(true)
                .default_value("/usr/local/etc/rj.conf"),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("Show debug messages"),
        )
        .subcommand(
            SubCommand::with_name("apply")
                .about("Apply changes")
                .arg(
                    Arg::with_name("jail_name")
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
                .about("Destroys jails")
                .arg(
                    Arg::with_name("jail_name")
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
}

// Parses the command line arguments and returns the matches.
pub fn parse_args<'a>() -> clap::ArgMatches<'a> {
    create_app().get_matches()
}
