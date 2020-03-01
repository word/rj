use crate::cmd::Cmd;
use crate::errors::RunError;
use anyhow::Result;
use log::debug;
use std::collections::HashMap;

pub struct Pkg {
    name: String,
    chroot: String,
    env: HashMap<String, String>,
}

impl Pkg {
    pub fn new(name: &str, chroot: &str) -> Pkg {
        let mut env = HashMap::new();
        env.insert("ASSUME_ALWAYS_YES".to_string(), "yes".to_string());
        env.insert("DEFAULT_ALWAYS_YES".to_string(), "yes".to_string());

        Pkg {
            name: name.to_string(),
            chroot: chroot.to_string(),
            env: env,
        }
    }

    pub fn install(&self) -> Result<()> {
        debug!("pkg install {} in {}", &self.name, &self.chroot);
        Cmd::new("pkg")
            .args(&["-c", &self.chroot, "install", &self.name])
            .envs(&self.env)
            .exec()
    }

    // check if package is installed
    pub fn is_installed(&self) -> Result<bool> {
        debug!(
            "checking if pkg {} is installed in {}",
            &self.name, &self.chroot
        );
        let result = Cmd::new("pkg")
            .args(&["-c", &self.chroot, "info", &self.name])
            .envs(&self.env)
            .exec();

        match result {
            Ok(_) => return Ok(true),
            Err(e) => match e.downcast_ref::<RunError>() {
                Some(re) => {
                    // package not installed or no packages installed
                    if re.code.unwrap() == 70 || re.code.unwrap() == 69 {
                        return Ok(false);
                    } else {
                        return Err(e);
                    }
                }
                None => return Err(e),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    use pretty_assertions::assert_eq;
    use serial_test::serial;

    #[test]
    #[serial]
    fn pkg() -> Result<()> {
        let s = Settings::new("testdata/config.toml")?;
        let jails = s.to_jails()?;
        let jail = &jails["pkg_test"];
        // clean up if left over from a failed test
        if jail.exists()? {
            jail.destroy()?;
        }
        jail.apply()?;

        let pkg = Pkg::new("tokei", jail.mountpoint());
        assert_eq!(pkg.is_installed()?, false);
        pkg.install()?;
        assert!(pkg.is_installed()?);

        jail.destroy()?;
        Ok(())
    }
}
