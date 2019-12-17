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
    let _release = "12-RELEASE";
    let jdataset = "zroot/jails";
    let bjdataset = format!("{}/basejail", &jdataset);

    println!("Creating jail data set {}", &jdataset);
    zfs_create_ds(&jdataset).unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    });

    println!("Creating base jail data set {}", &bjdataset);
    zfs_create_ds(&bjdataset).unwrap_or_else(|err| {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    });

    // match zfs_ds_exist(&bjdataset) {
    //     Ok(true) => println!("Data set exists already, skipping"),
    //     Ok(false) => {
    //         zfs_create(&bjdataset).unwrap_or_else(|err| {
    //             eprintln!("ERROR: {}", err);
    //             process::exit(1);
    //         });
    //     }
    //     Err(e) => {
    //         eprintln!("ERROR: {}", e);
    //         process::exit(1);
    //     }
    // }



    // let mut zfs = Command::new("zfs");
    // zfs.arg("list").arg(&bjdataset);

    // run(&mut zfs).unwrap_or_else(|err| {
    //     eprintln!("ERROR: {}", err);
    //     process::exit(1);
    // });

    // process::exit(0);
    // let output = Command::new("zfs").args(&["list", &bjdataset]).output().unwrap();

    // if !output.status.success() {

    //     let err_msg = str::from_utf8(&output.stderr).unwrap();
    //     let not_exist_msg = format!("cannot open \'{}\': dataset does not exist\n", &bjdataset);

    //     if err_msg == not_exist_msg {
    //         // Create the data set
    //         if let Err(err) = Command::new("zfs").args(&["create", &bjdataset]).status() {
    //             eprintln!("Error creating data set: {}", &err);
    //         }
    //     } else {
    //         eprintln!("Unexpected error: {}", &err_msg);
    //         process::exit(1);
    //     };

    //     // match err {
    //     //     "dataset does not exist" => println!("no exist"),
    //     //     _ => println!("big trouble"),
    //     // }
    //     // println!("{:?}", &err);
    // } else {
    //     println!("Dataset exists already, skipping");
    // };

    // let cmd = format!("zfs list {}", &bjdataset);
    // let status = run(&cmd).unwrap().status;
    // if !status.success() {
    //     cmd = format!("zfs cateate {}/basejail"
    // }


    // if run(&check_basejail[..]).unwrap().status.success();

    // let out = run("ls", &["/"]).unwrap();
    // let out = run("ls -la /").unwrap();

    // io::stdout().write_all(&out.stdout).unwrap();
    // io::stdout().write_all(&out.stderr).unwrap();
    // println!("{}", &out.status.success());
    // println!("{}", &out.status.code().unwrap());
}
