use std::fmt;

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub message: String,
}

impl ErrorInfo {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }

    pub fn permission_denied(resource: &str) -> Self {
        Self { message: format!("Permission denied: {}", resource) }
    }

    pub fn not_supported(what: &str) -> Self {
        Self { message: format!("Not supported: {}", what) }
    }

    pub fn io_error(path: &str, err: std::io::Error) -> Self {
        Self { message: format!("{}: {}", path, err) }
    }
}

impl std::error::Error for ErrorInfo {}

impl fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<std::io::Error> for ErrorInfo {
    fn from(err: std::io::Error) -> Self {
        Self { message: err.to_string() }
    }
}

pub type Result<T> = std::result::Result<T, ErrorInfo>;
