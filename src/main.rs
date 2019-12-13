// use std::process;
use std::process::Command;
use std::io::{self, Write};

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

fn main() {
    let _release = "12-RELEASE";

    // let out = run("ls", &["/"]).unwrap();
    let out = run("ls -la /").unwrap();

    io::stdout().write_all(&out.stdout).unwrap();

    // println!("{:?}", test);
}
