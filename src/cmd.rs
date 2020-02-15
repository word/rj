use crate::errors::RunError;
use anyhow::Result;
use log::{error, info};
use std::io::prelude::*;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::thread;

// Deprecated Command wrapper
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

// Wrapper around Command which checks the exist status and returns an Err if it isn't success.
pub fn cmd<T>(program: &str, args: T) -> Result<String>
where
    T: IntoIterator,
    T::Item: ToString,
{
    let mut argv_vec = Vec::new();
    argv_vec.extend(args.into_iter().map(|s| s.to_string()));

    let output = Command::new(&program).args(&argv_vec).output()?;

    // check the status
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        let stderr_out = String::from_utf8(output.stderr)?;
        let cmd_err = RunError {
            code: output.status.code(),
            message: format!(
                "Failed running: '{} {}', stderr: {}",
                program,
                argv_vec.join(" "),
                stderr_out
            ),
        };
        Err(anyhow::Error::new(cmd_err))
    }
}

// Run a command and stream stdout and stderr into the logger
// Fail on exit status other than 0
pub fn stream<T>(program: &str, args: T) -> Result<()>
where
    T: IntoIterator,
    T::Item: ToString,
{
    let mut argv_vec = Vec::new();
    argv_vec.extend(args.into_iter().map(|s| s.to_string()));

    let mut child = Command::new(&program)
        .args(&argv_vec)
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

    if status.success() {
        Ok(())
    } else {
        let err = RunError {
            code: status.code(),
            message: format!("Failed running: {} {}", program, argv_vec.join(" ")),
        };
        Err(anyhow::Error::new(err))
    }
}

#[macro_export]
macro_rules! cmd {
    ( $program:expr $(, $arg:expr )* $(,)? ) => {
        {
            $crate::cmd::cmd($program, &[$( $arg ),*])
        }
    };
}

#[macro_export]
macro_rules! cmd_stream {
    ( $program:expr $(, $arg:expr )* $(,)? ) => {
        {
            $crate::cmd::stream($program, &[$( $arg ),*])
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use simplelog::{Config, LevelFilter, WriteLogger};
    use tempfile::NamedTempFile;

    #[test]
    fn cmd_output() -> Result<()> {
        let output = cmd!("echo", "hello")?;
        assert_eq!(output, "hello\n");
        Ok(())
    }

    #[test]
    fn cmd_error() {
        assert!(cmd!("cat", "nonexistent").is_err());
    }

    // This test doesn't play well with others because logger is initialised globally
    #[test]
    #[ignore]
    fn stream_output() -> Result<()> {
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
        let mut output_iter = output.lines();

        let valid_lines = [
            "[ INFO] out1",
            "[ERROR] err1",
            "[ INFO] out2",
            "[ERROR] err2",
        ];

        println!("{}", output);

        for line in valid_lines.iter() {
            assert!(output_iter.next().unwrap().contains(line));
        }
        Ok(())
    }

    #[test]
    fn stream_error() {
        assert!(cmd_stream!("cat", "nonexistent").is_err());
    }
}
