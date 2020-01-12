use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;
// use std::env;

#[derive(Debug, Deserialize)]
struct Release {
    release: String,
    mirror: String,
    dists: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Jail {
    kind: String,
    release: Option<String>,
    basejail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    debug: bool,
    jails_dataset: String,
    jails_mountpoint: String,
    jail: HashMap<String, Jail>,
    release: HashMap<String, Release>,
}

#[allow(dead_code)]
impl Settings {
    pub fn new(config_file: &str) -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Start off by merging in the "default" configuration file
        s.merge(File::with_name(config_file))?;

        // Add in the current environment file
        // Default to 'development' env
        // Note that this file is _optional_
        // let env = env::var("RUN_MODE").unwrap_or("development".into());
        // s.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        // s.merge(File::with_name("config/local").required(false))?;

        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("rj"))?;

        // You may also programmatically change settings
        // s.set("database.url", "postgres://")?;

        // Now that we're done, let's access our configuration
        // println!("debug: {:?}", s.get_bool("debug"));
        // println!("database: {:?}", s.get::<String>("database.url"));

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_settings() -> () {
        let settings = Settings::new("config.toml").unwrap();
        println!("{:?}", settings);
        assert_eq!(settings.debug, false);
        assert_eq!(settings.release["12"].release, "12.0-RELEASE");
        assert_eq!(settings.release["12"].mirror, "ftp.uk.freebsd.org");
        assert_eq!(settings.release["12"].dists, vec!["base", "lib32"]);
        assert_eq!(settings.jail["base"].kind, "full");
        assert_eq!(settings.jail["base"].release, Some("12".to_string()));
        assert_eq!(settings.jail["base"].basejail, None);
        assert_eq!(settings.jail["example"].kind, "clone");
        assert_eq!(settings.jail["example"].basejail, Some("base".to_string()));
        assert_eq!(settings.jail["example"].release, None);
    }
}
