use std::process;
use std::process::Command;
use std::io::{self, Write};
use std::str;

// fn run(cmd: &str, args: &[&str]) -> Result<std::process::Output, io::Error> {
fn run(cmd: &str) -> Result<std::process::Output, io::Error> {

    // split the command into a vector
    let mut cmd_vec: Vec<&str> = cmd.split_whitespace().collect();

    // use the first part as thecommand and the rest as arguments
    let output = Command::new(cmd_vec.remove(0)).args(cmd_vec).output()?;
    Ok(output)

    // {
    //     Ok(output)  => output,
    //     Err(e)      => {
    //         eprintln!("muchas problemos: {}", e);
    //         process::exit(1);
    //     },
    // };

    // return test
}

fn mk_dataset() {

}

fn main() {
    let _release = "12-RELEASE";
    let jdataset = "zroot/jails";
    let bjdataset = format!("{}/basejail", &jdataset);

    println!("Creating base jail dataset {}", &bjdataset);
    let out = Command::new("zfs").args(&["list", &bjdataset]).output().unwrap();

    if !out.status.success() {

        let err = str::from_utf8(&out.stderr).unwrap();
        if err == format!("cannot open \'{}\': dataset does not exist\n", &bjdataset) {
            Command::new("zfs").args(&["create", &bjdataset]).status();
            println!("boo");
        } else {
            eprintln!("Unexpected error: {}", err);
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
