use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use indexmap::IndexMap;
use serde::Deserialize; // like HashMap but preserves order

use super::Source;

#[derive(Debug, Deserialize)]
pub struct JailSettings {
    pub source: String,
    pub order: i16,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub jails_dataset: String,
    pub jails_mountpoint: String,
    pub jail: IndexMap<String, JailSettings>,
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

        let mut settings: Self;

        // You can deserialize (and thus freeze) the entire configuration as
        match s.try_into() {
            Ok(s) => settings = s,
            Err(e) => return Err(e),
        }

        // Sort jails by 'order' field
        settings
            .jail
            .sort_by(|_, av, _, bv| av.order.cmp(&bv.order));
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_settings() {
        let s = Settings::new("config.toml").unwrap();
        println!("{:?}", s);
        assert_eq!(s.debug, false);
        assert_eq!(s.jail["base"].source, "freebsd12");
        assert_eq!(s.jail["base"].order, 10);
        assert_eq!(s.jail["example"].source, "base");
        assert_eq!(s.jail["example"].order, 20);

        if let Source::FreeBSD {
            release,
            mirror,
            dists,
        } = &s.source["freebsd12"]
        {
            assert_eq!(release, "12.0-RELEASE");
            assert_eq!(mirror, "ftp.uk.freebsd.org");
            assert_eq!(dists, &vec!["base".to_string(), "lib32".to_string()]);
        }

        if let Source::Cloned { path } = &s.source["base"] {
            assert_eq!(path, "zroot/jails/base");
        }
    }
}
