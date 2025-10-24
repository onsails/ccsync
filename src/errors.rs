//! Centralized error handling module.
//!
//! This module defines custom error types and error handling logic for the application.
//! It provides clear, user-friendly error messages and appropriate exit codes.

use std::fmt;

/// Main error type for ccsync operations.
#[derive(Debug)]
pub enum CcsyncError {
    /// IO operation failed
    Io(std::io::Error),
    /// Configuration parsing error
    Config(String),
    /// File sync error
    Sync(String),
    /// Path validation error
    Path(String),
}

impl fmt::Display for CcsyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CcsyncError::Io(err) => write!(f, "IO error: {}", err),
            CcsyncError::Config(msg) => write!(f, "Configuration error: {}", msg),
            CcsyncError::Sync(msg) => write!(f, "Sync error: {}", msg),
            CcsyncError::Path(msg) => write!(f, "Path error: {}", msg),
        }
    }
}

impl std::error::Error for CcsyncError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CcsyncError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CcsyncError {
    fn from(err: std::io::Error) -> Self {
        CcsyncError::Io(err)
    }
}

/// Result type alias for ccsync operations.
pub type Result<T> = std::result::Result<T, CcsyncError>;
