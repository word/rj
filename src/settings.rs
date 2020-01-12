use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
// use std::collections::HashMap;
use indexmap::IndexMap; // like HashMap but preserves insertion order
                        // use std::env;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Source {
    #[serde(alias = "freebsd")]
    FreeBSD {
        release: String,
        mirror: String,
        dists: Vec<String>,
    },
    #[serde(rename = "clone")]
    Cloned { path: String },
}

#[derive(Debug, Deserialize)]
pub struct Jail {
    pub source: String,
    pub order: i16,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub jails_dataset: String,
    pub jails_mountpoint: String,
    pub jail: IndexMap<String, Jail>,
    pub source: IndexMap<String, Source>,
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
        assert_eq!(s.jail["base"].source, "freebsd12");
        assert_eq!(s.jail["base"].order, 10);
        assert_eq!(s.jail["example"].source, "base");
        assert_eq!(s.jail["example"].order, 20);

        match &s.source["freebsd12"] {
            Source::FreeBSD {
                release,
                mirror,
                dists,
            } => {
                assert_eq!(release, "12.0-RELEASE");
                assert_eq!(mirror, "ftp.uk.freebsd.org");
                assert_eq!(dists, &vec!["base".to_string(), "lib32".to_string()]);
            }
            _ => {}
        }

        match &s.source["base"] {
            Source::Cloned { path } => {
                assert_eq!(path, "zroot/jails/base");
            }
            _ => {}
        }
    }
}
