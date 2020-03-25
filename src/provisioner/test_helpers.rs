use crate::jail::Jail;
use crate::settings::Settings;
use anyhow::Result;

// set up a jail for provisioner testing
pub fn setup<'a>(s: &'a Settings, name: &str) -> Result<Jail<'a>> {
    let jails = s.to_jails()?;
    let jail = &jails[name];
    let basejail = &jails["base"];

    // make sure we have a basejail set up
    if !basejail.exists()? {
        crate::init(&s).unwrap();
        basejail.apply()?;
    }

    // clean up if left over from a failed test
    if jail.exists()? {
        jail.destroy()?;
    }
    Ok(jail.to_owned())
}
