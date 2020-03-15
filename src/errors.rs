use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct CmdError {
    pub code: Option<i32>,
    pub message: String,
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.code {
            Some(code) => write!(f, "({}) {}", code, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl error::Error for CmdError {
    fn description(&self) -> &str {
        &self.message
    }
}
