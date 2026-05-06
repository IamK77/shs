use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ShsError {
    Io(io::Error),
    Json(serde_json::Error),
    Inquire(inquire::InquireError),
    /// User declined an interactive prompt or otherwise opted out.
    Aborted(String),
    /// A non-IO problem with config/state (missing host, bad JSON, missing env var).
    Config(String),
}

impl fmt::Display for ShsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShsError::Io(e) => write!(f, "I/O error: {}", e),
            ShsError::Json(e) => write!(f, "JSON error: {}", e),
            ShsError::Inquire(e) => write!(f, "prompt error: {}", e),
            ShsError::Aborted(msg) | ShsError::Config(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ShsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ShsError::Io(e) => Some(e),
            ShsError::Json(e) => Some(e),
            ShsError::Inquire(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ShsError {
    fn from(e: io::Error) -> Self {
        ShsError::Io(e)
    }
}

impl From<serde_json::Error> for ShsError {
    fn from(e: serde_json::Error) -> Self {
        ShsError::Json(e)
    }
}

impl From<inquire::InquireError> for ShsError {
    fn from(e: inquire::InquireError) -> Self {
        ShsError::Inquire(e)
    }
}

pub type Result<T> = std::result::Result<T, ShsError>;
