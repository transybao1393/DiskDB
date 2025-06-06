use std::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum DiskDBError {
    Io(std::io::Error),
    Database(String),
    Protocol(String),
    Tls(native_tls::Error),
    InvalidCommand(String),
    KeyNotFound(String),
    ConnectionClosed,
    Config(String),
}

impl fmt::Display for DiskDBError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiskDBError::Io(e) => write!(f, "IO error: {}", e),
            DiskDBError::Database(msg) => write!(f, "Database error: {}", msg),
            DiskDBError::Protocol(msg) => write!(f, "Protocol error: {}", msg),
            DiskDBError::Tls(e) => write!(f, "TLS error: {}", e),
            DiskDBError::InvalidCommand(cmd) => write!(f, "Invalid command: {}", cmd),
            DiskDBError::KeyNotFound(key) => write!(f, "Key not found: {}", key),
            DiskDBError::ConnectionClosed => write!(f, "Connection closed"),
            DiskDBError::Config(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl StdError for DiskDBError {}

impl From<std::io::Error> for DiskDBError {
    fn from(err: std::io::Error) -> Self {
        DiskDBError::Io(err)
    }
}

impl From<native_tls::Error> for DiskDBError {
    fn from(err: native_tls::Error) -> Self {
        DiskDBError::Tls(err)
    }
}

impl From<rocksdb::Error> for DiskDBError {
    fn from(err: rocksdb::Error) -> Self {
        DiskDBError::Database(err.to_string())
    }
}


pub type Result<T> = std::result::Result<T, DiskDBError>;