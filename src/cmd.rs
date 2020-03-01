use crate::errors::RunError;
use anyhow::Result;
use log::{error, info};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::thread;

pub struct Cmd {
    command: Command,
}

impl Cmd {
    pub fn new(program: &str) -> Cmd {
        Cmd {
            command: Command::new(program.to_string()),
        }
    }

    #[allow(dead_code)]
    pub fn args<I, S>(&mut self, args: I) -> &mut Cmd
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }

    #[allow(dead_code)]
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Cmd
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.envs(vars);
        self
    }

    #[allow(dead_code)]
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Cmd {
        self.command.current_dir(dir);
        self
    }

    pub fn exec(&mut self) -> Result<()> {
        let output = self.command.output()?;
        Self::check_exit_status(&self, output)?;
        Ok(())
    }

    pub fn capture(&mut self) -> Result<String> {
        let output = self.command.output()?;
        Self::check_exit_status(&self, output)
    }

    // Run a command and stream stdout and stderr into the logger
    // Fail on exit status other than 0
    pub fn stream(&mut self) -> Result<()> {
        let mut child = self
            .command
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
                message: format!("Failed command: {:?}", &self.command),
            };
            Err(anyhow::Error::new(err))
        }
    }

    // Return Err if exit status is not 0
    #[allow(dead_code)]
    fn check_exit_status(&self, output: Output) -> Result<String> {
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        if output.status.success() {
            Ok(stdout)
        } else {
            let cmd_err = RunError {
                code: output.status.code(),
                message: format!("Failed command: '{:?}', stderr: {}", &self.command, stderr),
            };
            Err(anyhow::Error::new(cmd_err))
        }
    }
}

// Run a command and stream stdout and stderr into the logger
// Fail on exit status other than 0
pub fn stream<T, U>(program: U, args: T) -> Result<()>
where
    U: AsRef<OsStr>,
    U: AsRef<str>,
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
            message: format!(
                "Failed command: {} {}",
                AsRef::<str>::as_ref(&program),
                argv_vec.join(" ")
            ),
        };
        Err(anyhow::Error::new(err))
    }
}

#[macro_export]
macro_rules! cmd {
    ( $program:expr $(, $arg:expr )* $(,)? ) => {
        {
            let args: &[String] = &[$( Into::<String>::into($arg) ),*];
            let mut c = $crate::cmd::Cmd::new($program);
            c.args(args);
            c.exec()
        }
    };
}

#[macro_export]
macro_rules! cmd_capture {
    ( $program:expr $(, $arg:expr )* $(,)? ) => {
        {
            let args: &[String] = &[$( Into::<String>::into($arg) ),*];
            let mut c = $crate::cmd::Cmd::new($program);
            c.args(args);
            c.capture()
        }
    };
}

#[macro_export]
macro_rules! cmd_stream {
    ( $program:expr $(, $arg:expr )* $(,)? ) => {
        {
            let args: &[String] = &[$( Into::<String>::into($arg) ),*];
            let mut c = $crate::cmd::Cmd::new($program);
            c.args(args);
            c.stream()
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
    fn cmd_run() -> Result<()> {
        cmd!("echo", "hello")?;
        Ok(())
    }

    #[test]
    fn cmd_run_noargs() -> Result<()> {
        cmd!("echo")?;
        Ok(())
    }

    #[test]
    fn cmd_with_env() -> Result<()> {
        let mut env = HashMap::new();
        env.insert("TESTENV", "testie");
        Cmd::new("printenv").args(&["TESTENV"]).envs(env).exec()
    }

    #[test]
    fn cmd_error() {
        assert!(cmd!("cat", "nonexistent").is_err());
    }

    #[test]
    fn capture_output() -> Result<()> {
        let output = cmd_capture!("echo", "hello")?;
        assert_eq!(output, "hello\n");
        Ok(())
    }

    #[test]
    fn capture_error() {
        assert!(cmd_capture!("cat", "nonexistent").is_err());
    }

    // This test doesn't play well with others because logger is initialised
    // globally
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
        cmd_stream!("sh", "-c", test_script)?;
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
