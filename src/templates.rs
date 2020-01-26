#![allow(dead_code)]
use anyhow::Result;
use askama::Template;
use indexmap::IndexMap;
use log::debug;

use crate::settings::JailConfValue;

#[derive(Template)]
#[template(path = "jail.conf", escape = "none")]
struct JailTemplate<'a> {
    name: &'a str,
    defaults: &'a Vec<String>,
    conf: &'a Vec<String>,
    extra_conf: &'a Vec<String>,
}

// Converts the jail config IndexMap into vector of strings.
// The format depends on what type JailConfValue is.
fn prepare_lines(map: &IndexMap<String, JailConfValue>) -> Result<Vec<String>> {
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

pub fn render_jail_conf(
    name: &str,
    defaults_map: &IndexMap<String, JailConfValue>,
    conf_map: &IndexMap<String, JailConfValue>,
    extra_conf_map: &IndexMap<String, JailConfValue>,
) -> Result<String> {
    debug!("Rendering jail template");

    let defaults = prepare_lines(&defaults_map)?;
    let conf = prepare_lines(&conf_map)?;
    let extra_conf = prepare_lines(&extra_conf_map)?;

    let jail_template = JailTemplate {
        name,
        defaults: &defaults,
        conf: &conf,
        extra_conf: &extra_conf,
    };

    let mut rendered = jail_template.render()?;
    rendered.push_str("\n");
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::indexmap;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_render_jail_conf() {
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
            "ip4_addr".to_string() =>
                JailConfValue::Vec(vec![
                    "lo0|10.11.11.2/32".to_string(),
                    "lo0|10.23.23.2/32".to_string(),
                ]),
        };
        let extra_conf = indexmap! {
            "path".to_string() => JailConfValue::String("/jails/prison".to_string()),
        };

        let rendered = render_jail_conf(&name, &defaults, &conf, &extra_conf).unwrap();
        let ok = indoc!(
            r#"
            exec.start = "/bin/sh /etc/rc";
            exec.stop = "/bin/sh /etc/rc.shutdown";

            prison {
                path = "/jails/prison";
                host.hostname = "prison.example";
                allow.set_hostname = 1;
                ip4.addr = "lo0|10.11.11.2/32";
                ip4.addr += "lo0|10.23.23.2/32";
            }
            "#
        );
        println!("{:?}", ok);
        assert_eq!(rendered, ok);
    }
}
