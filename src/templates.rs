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
}

fn render_jail_conf(
    name: &str,
    defaults_map: &IndexMap<&str, JailConfValue>,
    conf_map: &IndexMap<&str, JailConfValue>,
) -> Result<String> {
    debug!("Rendering jail template");

    let mut defaults = vec![];
    let mut conf = vec![];

    for (k, v) in defaults_map {
        match v {
            JailConfValue::String(v) => {
                defaults.push(format!("{} = \"{}\";", k, v));
            }
            JailConfValue::Bool(v) => {
                defaults.push(format!("{} = {};", k, v));
            }
            JailConfValue::Int(v) => {
                defaults.push(format!("{} = {};", k, v));
            }
            JailConfValue::Vec(v) => if v.len() > 1 {},
        }
    }

    let jail_template = JailTemplate {
        name,
        defaults: &defaults,
        conf: &conf,
    };

    let rendered = jail_template.render()?;

    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_render_jail_conf() {
        let name = "prison";
        let mut defaults = IndexMap::new();
        defaults.insert(
            "exec_start",
            JailConfValue::String("/bin/sh /etc/rc".to_string()),
        );
        defaults.insert(
            "exec_stop",
            JailConfValue::String("/bin/sh /etc/rc.shutdown".to_string()),
        );
        let mut conf = IndexMap::new();
        conf.insert(
            "host_hostname",
            JailConfValue::String("prison.example".to_string()),
        );
        let rendered = render_jail_conf(&name, &defaults, &conf).unwrap();
        let ok = indoc!(
            r#"
            exec.start = "/bin/sh /etc/rc";
            exec.stop = "/bin/sh /etc/rc.shutdown";

            prison {
                host.hostname = "prison.example";
            }
        "#
        );
        println!("{:?}", ok);
        assert_eq!(rendered, ok);
    }
}
