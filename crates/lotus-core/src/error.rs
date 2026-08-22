use std::fmt;

#[derive(Debug)]
pub enum LotusError {
    Io(std::io::Error),
    Manifest(String),
    Trust(String),
    NotFound(String),
    State(String),
    Conflict(String),
    Unsupported(&'static str),
}

impl fmt::Display for LotusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LotusError::Io(e) => write!(f, "io error: {e}"),
            LotusError::Manifest(msg) => write!(f, "manifest error:\n{msg}"),
            LotusError::Trust(msg) => write!(f, "trust error: {msg}"),
            LotusError::NotFound(msg) => write!(f, "not found: {msg}"),
            LotusError::State(msg) => write!(f, "state error: {msg}"),
            LotusError::Conflict(msg) => write!(f, "conflict: {msg}"),
            LotusError::Unsupported(msg) => write!(f, "unsupported on this platform: {msg}"),
        }
    }
}

impl std::error::Error for LotusError {}

impl From<std::io::Error> for LotusError {
    fn from(e: std::io::Error) -> Self {
        LotusError::Io(e)
    }
}

impl From<serde_json::Error> for LotusError {
    fn from(e: serde_json::Error) -> Self {
        LotusError::State(format!("json: {e}"))
    }
}

pub type Result<T> = std::result::Result<T, LotusError>;

