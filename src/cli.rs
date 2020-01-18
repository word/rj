use clap::{crate_authors, crate_description, crate_name, crate_version, Arg, SubCommand};

// Create a clap app
fn create_app<'a, 'b>() -> clap::App<'a, 'b> {
    clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .set_term_width(80)
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("path to a custom config file")
                .takes_value(true)
                .default_value("/usr/local/etc/rj.conf")
                // .validator(is_valid_file),
        )
        .subcommand(
            SubCommand::with_name("create")
                .about("Creates jails")
                .arg(Arg::with_name("all").long("all").help("create all fails")),
        )
}

// Parses the command line arguments and returns the matches.
pub fn parse_args<'a>() -> clap::ArgMatches<'a> {
    create_app().get_matches()
}
