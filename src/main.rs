use std::process;
use std::process::Command;
use std::str;
use std::fmt;

// These type aliases make it possible to propagate different types of errors a function using
// Result.
type GenError = Box<dyn std::error::Error>;
type GenResult<T> = Result<T, GenError>;

#[derive(Debug, Clone)]
pub struct RunError {
    code: Option<i32>,
    message: String,
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.code {
            Some(code) => write!(f, "{}, {}", code, self.message),
            None       => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for RunError {
    fn description(&self) -> &str {
        &self.message
    }
}

fn run(command:&mut Command) -> GenResult<(String)> {
    // execute the command
    let output = command.output()?;

    // check the status
    if output.status.success() {
        return Ok(String::from_utf8(output.stdout)?);
    } else {
        let cmd_err = GenError::from(RunError {
            code: output.status.code(),
            message: String::from_utf8(output.stderr)?,
        });
        return Err(cmd_err);
    }
}

fn zfs_ds_exist(ds: &str) -> GenResult<bool> {

    let mut zfs = Command::new("zfs");
    zfs.arg("list").arg(&ds);

    match run(&mut zfs) {
        Ok(_) => return Ok(true),
        Err(e) => {
            let not_exists = format!("cannot open \'{}\': dataset does not exist\n", &ds);
            if e.to_string() == not_exists {
                return Ok(false)
            } else {
                return Err(GenError::from(e))
            }
        }
    }
}

fn zfs_create_ds(ds: &str) -> GenResult<()> {
    match zfs_ds_exist(&ds) {
        Ok(true) => {
            println!("Data set already exists, skipping");
            return Ok(())
        }
        Ok(false) => {
            let mut zfs = Command::new("zfs");
            zfs.arg("create").arg(&ds);
            run(&mut zfs)?;
            return Ok(())
        }
        Err(e) => return Err(GenError::from(e))
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
