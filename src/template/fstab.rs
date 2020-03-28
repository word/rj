use crate::Volume;
use askama::Template;

#[derive(Clone, Debug, Template)]
#[template(path = "fstab", escape = "none")]
pub struct Fstab<'a> {
    pub volumes: Vec<&'a Volume>,
    pub jail_mountpoint: String,
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

        let fstab = Fstab {
            volumes: vec![&volume1, &volume2],
            jail_mountpoint: "/jails/jail".to_owned(),
        };
        let rendered = fstab.render()?;

        let ok = indoc!(
            r#"
            /tmp/test /jails/jail/mnt/test nullfs rw 0 0
            /tmp/test2 /jails/jail/mnt/test2 nullfs ro 0 0
           "#
        );

        assert_eq!(rendered, ok);

        Ok(())
    }
}
