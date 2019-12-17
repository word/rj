use anyhow::Result;
use std::process::Command;
use crate::errors::RunError;

pub fn run(command:&mut Command) -> Result<(String)> {
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
