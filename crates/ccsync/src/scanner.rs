//! File scanning functionality for Claude Code configuration directories
//!
//! This module provides directory-specific scanning patterns for:
//! - `agents/`: Flat directory scanning (*.md files only)
//! - `skills/`: One level subdirectory scanning (skills/*/SKILL.md pattern)
//! - `commands/`: Fully recursive scanning (commands/**/*.md)

mod agents;
mod commands;
mod filters;
mod skills;
mod symlinks;

use std::path::{Path, PathBuf};

pub use filters::FileFilter;
pub use symlinks::{ResolvedPath, SymlinkResolver};

use crate::error::Result;

/// Type of directory scanning to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanMode {
    /// Flat directory scan (agents/)
    Flat,
    /// One level of subdirectories (skills/)
    OneLevel,
    /// Recursive directory scan (commands/)
    Recursive,
}

/// A scanned file with metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedFile {
    /// Original path found during scanning
    pub path: PathBuf,
    /// Scan mode used to find this file
    pub mode: ScanMode,
}

/// Main scanner coordinator
pub struct Scanner {
    filter: FileFilter,
    symlink_resolver: SymlinkResolver,
}

impl Scanner {
    /// Create a new scanner with the given configuration
    #[must_use]
    pub const fn new(filter: FileFilter, preserve_symlinks: bool) -> Self {
        Self {
            filter,
            symlink_resolver: SymlinkResolver::new(preserve_symlinks),
        }
    }

    /// Scan a base directory for Claude Code configuration files
    ///
    /// # Errors
    ///
    /// Returns an error if directory traversal fails or if there are
    /// permission issues reading directories.
    pub fn scan(&mut self, base_path: &Path) -> Result<Vec<ScannedFile>> {
        let mut files = Vec::new();

        // Scan each directory type with appropriate mode
        match Self::scan_directory(&base_path.join("agents"), ScanMode::Flat) {
            Ok(agents) => files.extend(agents),
            Err(e) => eprintln!("Warning: Failed to scan agents directory: {e}"),
        }

        match Self::scan_directory(&base_path.join("skills"), ScanMode::OneLevel) {
            Ok(skills) => files.extend(skills),
            Err(e) => eprintln!("Warning: Failed to scan skills directory: {e}"),
        }

        match Self::scan_directory(&base_path.join("commands"), ScanMode::Recursive) {
            Ok(commands) => files.extend(commands),
            Err(e) => eprintln!("Warning: Failed to scan commands directory: {e}"),
        }

        // Apply filtering and symlink resolution
        let mut resolved_files = Vec::new();
        for file in files {
            if self.filter.should_include(&file.path) {
                match self.symlink_resolver.resolve(&file.path) {
                    Ok(resolved) => {
                        resolved_files.push(ScannedFile {
                            path: resolved.into_path(),
                            mode: file.mode,
                        });
                    }
                    Err(e) => {
                        eprintln!("Warning: {e}");
                    }
                }
            }
        }

        Ok(resolved_files)
    }

    /// Scan a directory with the specified mode
    fn scan_directory(path: &Path, mode: ScanMode) -> Result<Vec<ScannedFile>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let paths = match mode {
            ScanMode::Flat => agents::scan(path)?,
            ScanMode::OneLevel => skills::scan(path)?,
            ScanMode::Recursive => commands::scan(path),
        };

        Ok(paths
            .into_iter()
            .map(|p| ScannedFile { path: p, mode })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_mode_types() {
        assert_eq!(ScanMode::Flat, ScanMode::Flat);
        assert_ne!(ScanMode::Flat, ScanMode::OneLevel);
    }
}
