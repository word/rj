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
                .help("Path to a custom config file")
                .takes_value(true)
                .default_value("/usr/local/etc/rj.conf"),
        )
        .subcommand(
            SubCommand::with_name("create")
                .about("Creates jails")
                .arg(
                    Arg::with_name("jail_name")
                        .help("Name of the jail to create")
                        .index(1)
                        .required_unless("all"),
                )
                .arg(
                    Arg::with_name("all")
                        .short("a")
                        .long("all")
                        .help("Create all jails"),
                ),
        )
}

// Parses the command line arguments and returns the matches.
pub fn parse_args<'a>() -> clap::ArgMatches<'a> {
    create_app().get_matches()
}
