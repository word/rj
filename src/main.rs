use std::process;
use std::process::Command;
use std::io::{self, Write};

fn main() {
    let _release = "12-RELEASE";

    let test = match Command::new("/bin/lsss").args(&["/"]).output() {
        Ok(output)  => output,
        Err(e)      => {
            eprintln!("muchas problemos: {}", e);
            process::exit(1);
        },
    };

    io::stdout().write_all(&test.stdout).unwrap();

    // println!("{:?}", test);
}
