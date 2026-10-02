use std::fmt;

#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub message: String,
}

impl ErrorInfo {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }

    pub fn permission_denied(resource: impl Into<String>) -> Self {
        Self { message: format!("Permission denied: {}", resource.into()) }
    }

    pub fn not_supported(what: impl Into<String>) -> Self {
        Self { message: format!("Not supported: {}", what.into()) }
    }

    pub fn io_error(path: impl Into<String>, err: std::io::Error) -> Self {
        Self { message: format!("{}: {}", path.into(), err) }
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
