use anyhow::{bail, Result};
use indexmap::IndexMap; // like HashMap but preserves insertion order
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use toml;

use super::Jail;
use super::Provisioner;
use super::Source;
use super::Volume;

// Represents the different types of values a jail.conf option can have.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JailConfValue {
    String(String),
    Bool(bool),
    Vec(Vec<String>),
    Int(i32),
    Path(PathBuf),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JailSettings {
    pub source: String,
    #[serde(default = "default_true")]
    pub start: bool,
    #[serde(default = "default_true")]
    pub enable: bool,
    #[serde(default)]
    pub conf: IndexMap<String, JailConfValue>,
    #[serde(default)]
    pub provisioners: Vec<String>,
    #[serde(default)]
    pub volumes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub jails_dataset: PathBuf,
    pub jails_mountpoint: PathBuf,
    #[serde(default)]
    pub jail_conf_defaults: IndexMap<String, JailConfValue>,
    pub jail: IndexMap<String, JailSettings>,
    pub source: IndexMap<String, Source>,
    #[serde(default)]
    pub provisioner: IndexMap<String, Provisioner>,
    #[serde(default)]
    pub volume: IndexMap<String, Volume>,
    #[serde(default)] // false
    pub noop: bool,
}

fn default_true() -> bool {
    true
}

#[allow(dead_code)]
impl Settings {
    pub fn new(config_file: &str, noop: bool) -> Result<Self> {
        let mut settings: Settings = toml::from_str(&fs::read_to_string(config_file)?)?;

        settings.noop = noop;

        // Validate sources
        for (_, source) in settings.source.iter() {
            source.validate()?;
        }

        for (prov_name, provisioner) in settings.provisioner.iter_mut() {
            // Set provisioner name
            provisioner.name(prov_name);
            // Validate provisioner
            provisioner.validate()?;
        }

        Ok(settings)
    }

    pub fn to_jails(&self) -> Result<IndexMap<String, Jail>> {
        let mut jails = IndexMap::new();

        for (jail_name, jail_settings) in &mut self.jail.iter() {
            if !&self.source.contains_key(&jail_settings.source) {
                bail!("{}: unknown source: {}", jail_name, jail_settings.source);
            }

            // gather jail provisioners
            let mut provisioners = Vec::new();
            for p in jail_settings.provisioners.iter() {
                // error if the provisioner is not defined
                if !&self.provisioner.contains_key(p) {
                    bail!("{}: unknown provisioner: {}", jail_name, p);
                }
                provisioners.push(&self.provisioner[p]);
            }

            // gather volumes
            let mut volumes = vec![];
            for v in jail_settings.volumes.iter() {
                // error if the volume is not defined
                if !&self.volume.contains_key(v) {
                    bail!("{}: unknown volume: {}", jail_name, v);
                }
                volumes.push(&self.volume[v]);
            }

            // make jails
            let jail = Jail::new(
                jail_name,
                &self.jails_mountpoint,
                &self.jails_dataset,
                // jail source
                &self.source[&jail_settings.source],
                &jail_settings,
                &self.jail_conf_defaults,
                provisioners,
                &self.noop,
                volumes,
            );
            jails.insert(jail_name.to_owned(), jail);
        }
        Ok(jails)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provisioner::Provisioner;
    use pretty_assertions::assert_eq;

    #[test]
    fn deserialize() {
        let s = Settings::new("testdata/config.toml", false).unwrap();
        println!("{:?}", s);

        assert_eq!(s.jails_dataset, PathBuf::from("zroot/jails"));
        assert_eq!(s.jails_mountpoint, PathBuf::from("/jails"));
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
            assert_eq!(prov.name, "exec".to_string());
            assert_eq!(prov.cmd, "touch /tmp/exec_test".to_string());
        }

        if let Provisioner::Puppet(prov) = &s.provisioner["puppet"] {
            assert_eq!(prov.name, "puppet".to_string());
            assert_eq!(prov.path, PathBuf::from("testdata/provisioners/puppet"));
        }

        if let Provisioner::Puppet(prov) = &s.provisioner["puppet_simple"] {
            assert_eq!(prov.name, "puppet_simple".to_string());
            assert_eq!(
                prov.path,
                PathBuf::from("testdata/provisioners/puppet_simple")
            );
        }

        // test sources

        if let Source::FreeBSD(src) = &s.source["freebsd12"] {
            assert_eq!(src.mirror, "ftp.uk.freebsd.org".to_string());
        }

        if let Source::ZfsClone(src) = &s.source["base"] {
            assert_eq!(src.path, PathBuf::from("zroot/jails/base"));
        }

        // test 'enabled' option

        assert_eq!(s.jail["base"].start, false);
        assert!(s.jail["test1"].start);
        assert!(s.jail["test2"].start);
    }

    #[test]
    fn to_jail() -> Result<()> {
        let s = Settings::new("testdata/config.toml", false)?;
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

    #[test]
    fn unknown_source() {
        let mut s = Settings::new("testdata/config.toml", false).unwrap();
        s.jail["test1"].source = "nope".to_owned();

        let err = s.to_jails().unwrap_err();
        assert_eq!(
            err.downcast::<String>().unwrap(),
            "test1: unknown source: nope".to_string()
        )
    }

    #[test]
    fn unknown_provisioner() {
        let mut s = Settings::new("testdata/config.toml", false).unwrap();
        s.jail["test1"].provisioners = vec!["nope".to_owned()];

        let err = s.to_jails().unwrap_err();
        assert_eq!(
            err.downcast::<String>().unwrap(),
            "test1: unknown provisioner: nope".to_string()
        )
    }
}
