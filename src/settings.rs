use anyhow::Result;
use indexmap::IndexMap; // like HashMap but preserves insertion order
use serde::Deserialize;
use std::fs;
use toml;

use super::Jail;
use super::Provisioner;
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
#[serde(deny_unknown_fields)]
pub struct JailSettings {
    pub source: String,
    #[serde(default = "default_true")]
    pub start: bool,
    #[serde(default)]
    pub conf: IndexMap<String, JailConfValue>,
    #[serde(default)]
    pub provisioners: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Settings {
    pub jails_dataset: String,
    pub jails_mountpoint: String,
    #[serde(default)]
    pub jail_conf_defaults: IndexMap<String, JailConfValue>,
    pub jail: IndexMap<String, JailSettings>,
    pub source: IndexMap<String, Source>,
    pub provisioner: IndexMap<String, Provisioner>,
}

#[allow(dead_code)]
impl Settings {
    pub fn new(config_file: &str) -> Result<Self> {
        let settings: Settings = toml::from_str(&fs::read_to_string(config_file)?)?;

        // Validate sources
        for (_, source) in settings.source.iter() {
            source.validate()?;
        }

        // Validate provisioners
        for (_, provisioner) in settings.provisioner.iter() {
            provisioner.validate()?;
        }

        Ok(settings)
    }

    pub fn to_jails(&self) -> Result<IndexMap<String, Jail>> {
        let mut jails = IndexMap::new();

        for (jname, jsettings) in &mut self.jail.iter() {
            // gather jail provisioners
            let mut provisioners = vec![];
            for p in jsettings.provisioners.iter() {
                provisioners.push(&self.provisioner[p]);
            }

            // make jail
            let jail = Jail::new(
                // data set path
                &format!("{}/{}", &self.jails_dataset, &jname),
                // jail source
                &self.source[&jsettings.source],
                // jail conf
                &jsettings,
                &self.jail_conf_defaults,
                provisioners,
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
    use crate::provisioner::Provisioner;
    use pretty_assertions::assert_eq;

    #[test]
    fn deserialize() {
        let s = Settings::new("testdata/config.toml").unwrap();
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

        // test provisioners

        if let Provisioner::Exec(prov) = &s.provisioner["exec"] {
            assert_eq!(prov.cmd, "touch /tmp/exec_test".to_string());
        }

        if let Provisioner::Puppet(prov) = &s.provisioner["puppet"] {
            assert_eq!(prov.path, "testdata/provisioners/puppet".to_string());
        }

        // test sources

        if let Source::FreeBSD(src) = &s.source["freebsd12"] {
            assert_eq!(src.mirror, "ftp.uk.freebsd.org".to_string());
        }

        if let Source::ZfsClone(src) = &s.source["base"] {
            assert_eq!(src.path, "zroot/jails/base".to_string());
        }

        // test 'enabled' option

        assert_eq!(s.jail["base"].start, false);
        assert!(s.jail["test1"].start);
        assert!(s.jail["test2"].start);
    }

    #[test]
    fn to_jail() -> Result<()> {
        let s = Settings::new("testdata/config.toml")?;
        let jails = s.to_jails()?;
        assert_eq!(jails["base"].name(), "base");
        assert_eq!(jails["test1"].name(), "test1");
        assert_eq!(jails["test2"].name(), "test2");
        Ok(())
    }

    #[test]
    #[should_panic]
    fn unknown_field() {
        let mut config = fs::read_to_string("testdata/config.toml").unwrap();
        let slice = "[jail.test1]";
        let pos = config.rfind(slice).unwrap() + slice.len();
        config.insert_str(pos, "\nunknown = whatever");
        let _s: Settings = toml::from_str(&config).unwrap();
    }
}
