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
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for RunError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone)]
pub struct JailError(pub String);

impl fmt::Display for JailError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for JailError {
    fn description(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct SourceError(pub String);

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SourceError {
    fn description(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ProvError(pub String);

impl fmt::Display for ProvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ProvError {
    fn description(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ArgError(pub String);

impl fmt::Display for ArgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ArgError {
    fn description(&self) -> &str {
        &self.0
    }
}
