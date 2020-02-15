use crate::errors::RunError;
use anyhow::Result;
use log::{error, info};
use std::ffi::OsString;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::thread;

pub fn run(command: &mut Command) -> Result<String> {
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

// Run a command and stream stdout and stderr into the logger
// Fail on command status other than 0
pub fn stream<T>(program: &str, args: T) -> Result<()>
where
    T: IntoIterator,
    T::Item: Into<OsString>,
{
    let mut argv_vec = Vec::new();
    argv_vec.extend(args.into_iter().map(Into::<OsString>::into));

    let mut child = Command::new(program)
        .args(argv_vec)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stderr_handle = thread::spawn(|| {
        for line in BufReader::new(stderr).lines() {
            error!("{}", line.unwrap());
        }
    });

    for line in BufReader::new(stdout).lines() {
        info!("{}", line?);
    }

    stderr_handle.join().unwrap();
    let status = child.wait()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use simplelog::{Config, LevelFilter, WriteLogger};
    use tempfile::NamedTempFile;

    #[test]
    fn streamer() -> Result<()> {
        let test_script = r#"
            echo out1
            sleep 1
            echo err1 >&2
            sleep 1
            echo out2
            sleep 1
            echo err2 >&2
        "#;

        let logfile = NamedTempFile::new()?;
        let mut outfile = logfile.reopen()?;
        WriteLogger::init(LevelFilter::Info, Config::default(), logfile)?;
        stream("sh", &["-c", test_script])?;
        let mut output = String::new();
        outfile.read_to_string(&mut output)?;

        let mut valid_lines = [
            "[ INFO] out1",
            "[ERROR] err1",
            "[ INFO] out2",
            "[ERROR] err2",
        ]
        .iter();

        for line in output.lines() {
            assert!(line.contains(valid_lines.next().unwrap()));
        }
        Ok(())
    }
}
