use askama::Template;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Volume {
    device: String,
    mountpoint: String,
    fs_type: String,
    #[serde(default)]
    options: String,
    #[serde(default)]
    dump: i8,
    #[serde(default)]
    pass: i8,
}

#[derive(Clone, Debug, Template)]
#[template(path = "fstab", escape = "none")]
pub struct Volumes {
    volumes: Vec<Volume>,
}

impl Volumes {
    pub fn new(volumes: Vec<Volume>) -> Volumes {
        Volumes { volumes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn render_fstab() -> Result<()> {
        let volume1 = Volume {
            device: "/tmp/test".to_string(),
            mountpoint: "/mnt/test".to_string(),
            fs_type: "nullfs".to_string(),
            options: "rw".to_string(),
            dump: 0,
            pass: 0,
        };

        let volume2 = Volume {
            device: "/tmp/test2".to_string(),
            mountpoint: "/mnt/test2".to_string(),
            fs_type: "nullfs".to_string(),
            options: "ro".to_string(),
            dump: 0,
            pass: 0,
        };

        let volumes = Volumes::new(vec![volume1, volume2]);
        let rendered = volumes.render()?;

        let ok = indoc!(
            r#"
            /tmp/test /mnt/test nullfs rw 0 0
            /tmp/test2 /mnt/test2 nullfs ro 0 0
           "#
        );

        assert_eq!(rendered, ok);

        Ok(())
    }
}
