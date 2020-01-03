use anyhow::Result;
use std::process;

mod cmd;
mod errors;
mod lib;
mod zfs;

struct FbsdRel {
    release: String,
    dists: Vec<String>,
    mirror: String,
    path: String,
}

impl FbsdRel {
    pub fn extract(&self) -> Result<()> {
        for dist in &self.dists {
            println!("Extracing {} to {}", &dist, &self.path);

            let url = format!(
                "http://{}/pub/FreeBSD/releases/amd64/amd64/{}/{}.txz",
                &self.mirror, &self.release, dist
            );
            lib::fetch_extract(&url, &self.path)?;
        }
        Ok(())
    }
}

fn make_it_so() -> Result<()> {
    let jails_mount = "/jails";

    let basejail_rel = FbsdRel {
        release: String::from("12.0-RELEASE"),
        mirror: String::from("ftp.uk.freebsd.org"),
        dists: vec!["base".to_string(), "lib32".to_string()],
        path: format!("{}/basejail", jails_mount),
    };

    // Create base data sets
    let jails_ds = zfs::DataSet::new("zroot/jails")?;
    jails_ds.set("mountpoint", &jails_mount)?;
    let basejail_ds = zfs::DataSet::new(&format!("{}/basejail", &jails_ds.get_path()))?;
    // let basejail_mount = basejail_ds.get("mountpoint")?;

    // process::exit(0);

    // Extract FreeBSD base jail
    basejail_rel.extract()?;
    basejail_ds.snap("ready")?;

    Ok(())
}

fn main() {
    make_it_so().unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    })
}
