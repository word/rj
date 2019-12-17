use std::fmt;

#[derive(Debug, Clone)]
pub struct RunError {
    pub code: Option<i32>,
    pub message: String,
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.code {
            Some(code) => write!(f, "({}) {}", code, self.message),
            None       => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for RunError {
    fn description(&self) -> &str {
        &self.message
    }
}
