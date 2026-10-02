use std::fmt;

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub message: String,
}

impl std::error::Error for ErrorInfo {}

impl fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub type Result<T> = std::result::Result<T, ErrorInfo>;
