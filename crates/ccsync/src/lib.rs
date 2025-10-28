//! # ccsync
//!
//! Core library for Claude Configuration Synchronization Tool.
//!
//! This library provides the core functionality for synchronizing
//! agents, skills, and commands between global (~/.claude) and
//! project-specific (.claude) directories.

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Core error types for the ccsync library
pub mod error {
    /// Result type alias using `anyhow::Error`
    pub type Result<T> = anyhow::Result<T>;
}

/// File scanning functionality
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) mod scanner;

/// File comparison and conflict detection
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) mod comparison;

/// Configuration file parsing and management
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) mod config;

/// Bidirectional synchronization engine
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) mod sync;
