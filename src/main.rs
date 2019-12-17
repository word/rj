use std::process;
use std::process::Command;
use std::str;

mod errors;
use errors::RunError;

use anyhow::Result;

fn run(command:&mut Command) -> Result<(String)> {
    // execute the command
    let output = command.output()?;

    // check the status
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        let cmd_err = RunError {
            code: output.status.code(),
            message: String::from_utf8(output.stderr)?,
        };
        Err(anyhow::Error::new(cmd_err))
    }
}

fn zfs_ds_exist(ds: &str) -> Result<bool> {
    let mut zfs = Command::new("zfs");
    zfs.arg("list").arg(&ds);

    match run(&mut zfs) {
        Ok(_) => Ok(true),
        Err(e) => {
            let not_exists = format!("cannot open \'{}\': dataset does not exist\n", &ds);
            if e.to_string() == not_exists {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}

fn zfs_create_ds(ds: &str) -> Result<()> {
    match zfs_ds_exist(&ds) {
        Ok(true) => {
            println!("Data set already exists, skipping");
            Ok(())
        }
        Ok(false) => {
            let mut zfs = Command::new("zfs");
            zfs.arg("create").arg(&ds);
            run(&mut zfs)?;
            Ok(())
        }
        Err(e) => Err(e)
    }
}

fn main() {
    let _release = String::from("12-RELEASE");
    let jdataset = String::from("zroot/jails");
    let bjdataset = format!("{}/basejail", &jdataset);

    for set in vec![&jdataset, &bjdataset] {
        println!("Creating jail data set {}", set);
        zfs_create_ds(&set).unwrap_or_else(|err| {
            eprintln!("ERROR: {}", err);
            process::exit(1);
        });
    };

}
