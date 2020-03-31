use anyhow::Result;
use askama::Template;
use indexmap::IndexMap;

use crate::settings::JailConfValue;

#[derive(Template)]
#[template(path = "jail.conf", escape = "none")]
pub struct JailConf {
    name: String,
    defaults: Vec<String>,
    conf: Vec<String>,
    extra_conf: Vec<String>,
}

impl JailConf {
    pub fn new(
        name: &str,
        defaults: &IndexMap<String, JailConfValue>,
        conf: &IndexMap<String, JailConfValue>,
        extra_conf: &IndexMap<String, JailConfValue>,
    ) -> Result<JailConf> {
        //
        let defaults = Self::format_lines(&defaults)?;
        let conf = Self::format_lines(&conf)?;
        let extra_conf = Self::format_lines(&extra_conf)?;

        let jail_template = JailConf {
            name: name.to_owned(),
            defaults,
            conf,
            extra_conf,
        };

        Ok(jail_template)
    }

    // Converts the jail config IndexMap into a vector of strings.
    // The each line format depends on what type it's JailConfValue is.
    fn format_lines(map: &IndexMap<String, JailConfValue>) -> Result<Vec<String>> {
        let mut lines = vec![];
        for (k, v) in map {
            let key = k.replacen("_", ".", 1);

            match v {
                JailConfValue::String(v) => {
                    lines.push(format!("{} = \"{}\";", key, v));
                }
                JailConfValue::Bool(v) => {
                    lines.push(format!("{} = {};", key, v));
                }
                JailConfValue::Int(v) => {
                    lines.push(format!("{} = {};", key, v));
                }
                JailConfValue::Path(v) => {
                    lines.push(format!("{} = \"{}\";", key, v.display()));
                }
                JailConfValue::Vec(v) => {
                    for item in v.iter().enumerate() {
                        if item.0 == 0 {
                            lines.push(format!("{} = \"{}\";", key, item.1));
                        } else {
                            lines.push(format!("{} += \"{}\";", key, item.1));
                        }
                    }
                }
            }
        }
        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::indexmap;
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn test_render_jail_conf() -> Result<()> {
        let name = "prison";
        let defaults = indexmap! {
            "exec_start".to_string() =>
                JailConfValue::String("/bin/sh /etc/rc".to_string()),
            "exec_stop".to_string() =>
                JailConfValue::String("/bin/sh /etc/rc.shutdown".to_string()),
        };
        let conf = indexmap! {
            "host_hostname".to_string() =>
                JailConfValue::String("prison.example".to_string()),
            "allow_set_hostname".to_string() =>
                JailConfValue::Int(1),
            "mount.fstab".to_string() =>
                JailConfValue::Path(PathBuf::from("/etc/fstab.prison")),
            "ip4_addr".to_string() =>
                JailConfValue::Vec(vec![
                    "lo0|10.11.11.2/32".to_string(),
                    "lo0|10.23.23.2/32".to_string(),
                ]),
        };
        let extra_conf = indexmap! {
            "path".to_string() => JailConfValue::String("/jails/prison".to_string()),
        };

        let jail_conf_template = JailConf::new(&name, &defaults, &conf, &extra_conf)?;
        let rendered = jail_conf_template.render()?;

        let ok = indoc!(
            r#"
            exec.start = "/bin/sh /etc/rc";
            exec.stop = "/bin/sh /etc/rc.shutdown";

            prison {
                path = "/jails/prison";
                host.hostname = "prison.example";
                allow.set_hostname = 1;
                mount.fstab = "/etc/fstab.prison";
                ip4.addr = "lo0|10.11.11.2/32";
                ip4.addr += "lo0|10.23.23.2/32";
            }
            "#
        );
        println!("{:?}", ok);
        assert_eq!(rendered, ok);
        Ok(())
    }
}
