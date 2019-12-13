// use std::process;
use std::process::Command;
use std::io::{self, Write};

fn run(cmd: &str, args: &[&str]) -> Result<std::process::Output, io::Error> {

    let output = Command::new(cmd).args(args).output()?;
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

    let out = run("ls", &["/"]).unwrap();

    io::stdout().write_all(&out.stdout).unwrap();

    // println!("{:?}", test);
}
