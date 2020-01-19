use anyhow::Result;
use askama::Template;
use log::debug;

#[derive(Template)]
#[template(path = "jail.conf", escape = "none")]
struct JailTemplate<'a> {
    name: &'a str,
    defaults: &'a [String],
    conf: &'a [String],
}

fn render_jail_conf(name: &str, defaults: &[String], conf: &[String]) -> Result<String> {
    debug!("Rendering jail template");

    let jail_template = JailTemplate {
        name,
        defaults,
        conf,
    };

    let rendered = jail_template.render()?;

    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_render_jail_conf() -> Result<()> {
        let name = "Bob";
        let defaults = [
            "exec.start = /bin/sh /etc/rc".to_string(),
            "exec.stop = /bin/sh /etc/rc.shutdown".to_string(),
        ];
        let conf = [
            "host.hostname = test.example.com".to_string(),
            "ip4.addr = lo0|10.23.23.1".to_string(),
        ];
        let rendered = render_jail_conf(&name, &defaults, &conf)?;
        let ok = "Hello, Bob!";
        assert_eq!(rendered, ok);
        Ok(())
    }
}
