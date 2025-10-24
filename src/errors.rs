//! Centralized error handling module.
//!
//! This module defines custom error types and error handling logic for the application.
//! It provides clear, user-friendly error messages and appropriate exit codes.
//!
//! Uses `thiserror` for library error types following Rust best practices.

use thiserror::Error;

/// Main error type for ccsync operations.
///
/// This type will be used in future tasks for error handling across the codebase.
#[derive(Error, Debug)]
#[allow(dead_code)] // Will be used in Task 3+
pub enum CcsyncError {
    /// IO operation failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias for ccsync operations.
///
/// This type will be used in future tasks for function return types.
#[allow(dead_code)] // Will be used in Task 3+
pub type Result<T> = std::result::Result<T, CcsyncError>;
