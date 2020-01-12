use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;
// use std::env;

#[derive(Debug, Deserialize)]
pub struct Release {
    pub release: String,
    pub mirror: String,
    pub dists: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Jail {
    pub kind: String,
    pub release: Option<String>,
    pub basejail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub jails_dataset: String,
    pub jails_mountpoint: String,
    pub jail: HashMap<String, Jail>,
    pub release: HashMap<String, Release>,
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
        let s = Settings::new("config.toml").unwrap();
        println!("{:?}", s);
        assert_eq!(s.debug, false);
        assert_eq!(s.release["12"].release, "12.0-RELEASE");
        assert_eq!(s.release["12"].mirror, "ftp.uk.freebsd.org");
        assert_eq!(s.release["12"].dists, vec!["base", "lib32"]);
        assert_eq!(s.jail["base"].kind, "full");
        assert_eq!(s.jail["base"].release, Some("12".to_string()));
        assert_eq!(s.jail["base"].basejail, None);
        assert_eq!(s.jail["example"].kind, "clone");
        assert_eq!(s.jail["example"].basejail, Some("base".to_string()));
        assert_eq!(s.jail["example"].release, None);
    }
}
