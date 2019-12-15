use std::process;
use std::process::Command;
use std::io::{Error, ErrorKind};
use std::io;
use std::str;

// fn run(cmd: &str, args: &[&str]) -> Result<std::process::Output, io::Error> {
// fn run(cmd: &str) -> Result<std::process::Output, io::Error> {

//     // split the command into a vector
//     let mut cmd_vec: Vec<&str> = cmd.split_whitespace().collect();

//     // use the first part as thecommand and the rest as arguments
//     let output = Command::new(cmd_vec.remove(0)).args(cmd_vec).output()?;
//     Ok(output)

//     // {
//     //     Ok(output)  => output,
//     //     Err(e)      => {
//     //         eprintln!("muchas problemos: {}", e);
//     //         process::exit(1);
//     //     },
//     // };

//     // return test
// }

type GenError = Box<std::error::Error>;
type GenResult<T> = Result<T, GenError>;

fn run(command:&mut Command) -> GenResult<String> {
    // execute the command
    let output = command.output()?;

    // check the status
    if output.status.success() {
        return Ok(String::from_utf8(output.stdout)?);
    } else {
        let cmd_error = Error::new(ErrorKind::Other, String::from_utf8(output.stderr)?);
        return Err(GenError::from(cmd_error));
    }

    // todo - would be good to return the status too.  Probably needs to return a struct instead of
    // a String
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
                return Err(GenError::from(e));
            }
        }
    }

}

fn main() {
    let _release = "12-RELEASE";
    let jdataset = "zroot/jails";
    let bjdataset = format!("{}/basejail", &jdataset);

    println!("Creating base jail data set {}", &bjdataset);

    match zfs_ds_exist(&bjdataset) {
        Ok(true) => println!("Data set exists already, skipping"),
        Ok(false) => println!("TODO: create data set"),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            process::exit(1);
        }
    }



    // let mut zfs = Command::new("zfs");
    // zfs.arg("list").arg(&bjdataset);

    // run(&mut zfs).unwrap_or_else(|err| {
    //     eprintln!("ERROR: {}", err);
    //     process::exit(1);
    // });

    process::exit(0);
    let output = Command::new("zfs").args(&["list", &bjdataset]).output().unwrap();

    if !output.status.success() {

        let err_msg = str::from_utf8(&output.stderr).unwrap();
        let not_exist_msg = format!("cannot open \'{}\': dataset does not exist\n", &bjdataset);

        if err_msg == not_exist_msg {
            // Create the data set
            if let Err(err) = Command::new("zfs").args(&["create", &bjdataset]).status() {
                eprintln!("Error creating data set: {}", &err);
            }
        } else {
            eprintln!("Unexpected error: {}", &err_msg);
            process::exit(1);
        };

        // match err {
        //     "dataset does not exist" => println!("no exist"),
        //     _ => println!("big trouble"),
        // }
        // println!("{:?}", &err);
    } else {
        println!("Dataset exists already, skipping");
    };

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
