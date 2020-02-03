use anyhow::Result;
use indexmap::IndexMap; // like HashMap but preserves insertion order
use serde::Deserialize;
use std::fs;
use toml;

use super::Jail;
use super::Source;

// Represents the different types of values a jail.conf option can have.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JailConfValue {
    String(String),
    Bool(bool),
    Vec(Vec<String>),
    Int(i32),
}

#[derive(Clone, Debug, Deserialize)]
pub struct JailSettings {
    pub source: String,
    #[serde(default = "default_true")]
    pub start: bool,
    #[serde(default)]
    pub conf: IndexMap<String, JailConfValue>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub jails_dataset: String,
    pub jails_mountpoint: String,
    #[serde(default)]
    pub jail_conf_defaults: IndexMap<String, JailConfValue>,
    pub jail: IndexMap<String, JailSettings>,
    pub source: IndexMap<String, Source>,
}

#[allow(dead_code)]
impl Settings {
    pub fn new(config_file: &str) -> Result<Self> {
        let settings: Settings = toml::from_str(&fs::read_to_string(config_file)?)?;

        // Sort jails by 'order' field
        // settings
        //     .jail
        //     .sort_by(|_, av, _, bv| av.order.cmp(&bv.order));

        Ok(settings)
    }

    pub fn to_jails(&self) -> Result<IndexMap<String, Jail>> {
        let mut jails = IndexMap::new();
        for (jname, jsettings) in &mut self.jail.iter() {
            let jail = Jail::new(
                // data set path
                &format!("{}/{}", &self.jails_dataset, &jname),
                // jail source
                &self.source[&jsettings.source],
                // jail conf
                &jsettings,
                &self.jail_conf_defaults,
            );
            jails.insert(jname.to_string(), jail);
        }
        Ok(jails)
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_settings() {
        let s = Settings::new("config.toml").unwrap();
        println!("{:?}", s);

        assert_eq!(s.jails_dataset, "zroot/jails");
        assert_eq!(s.jails_mountpoint, "/jails");
        assert_eq!(
            s.jail_conf_defaults["exec_start"],
            JailConfValue::String("/bin/sh /etc/rc".to_string())
        );

        assert_eq!(s.jail["base"].source, "freebsd12");
        assert_eq!(s.jail["test1"].source, "base");

        // test 'conf' option

        assert_eq!(
            s.jail["test2"].conf["host_hostname"],
            JailConfValue::String("test2.jail".to_string())
        );
        assert_eq!(
            s.jail["test2"].conf["allow_mount"],
            JailConfValue::Bool(true)
        );
        assert_eq!(
            s.jail["test2"].conf["allow_raw_sockets"],
            JailConfValue::Int(1)
        );
        assert_eq!(
            s.jail["test2"].conf["ip4_addr"],
            JailConfValue::Vec(vec![
                "lo0|10.11.11.2/32".to_string(),
                "lo0|10.23.23.2/32".to_string()
            ])
        );

        // test sources

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

        // test 'enabled' option

        assert_eq!(s.jail["base"].start, false);
        assert!(s.jail["test1"].start);
        assert!(s.jail["test2"].start);
    }

    #[test]
    fn test_settings_to_jail() {
        let s = Settings::new("config.toml").unwrap();
        let jails = s.to_jails().unwrap();
        assert_eq!(jails["base"].name(), "base");
        assert_eq!(jails["test1"].name(), "test1");
        assert_eq!(jails["test2"].name(), "test2");
    }
}
