//! File comparison, diff generation, and conflict detection
//!
//! This module provides read-only analysis of files to determine:
//! - Content differences via SHA-256 hashing
//! - Which file is newer via timestamp comparison
//! - Visual diffs for changed files
//! - Conflict classification and resolution strategy determination

mod diff;
mod directory;
mod hash;
mod timestamp;

#[cfg(test)]
mod integration_tests;

use std::path::Path;

use serde::{Deserialize, Serialize};

pub use diff::DiffGenerator;
pub use directory::{DirectoryComparator, DirectoryComparison};
pub use hash::FileHasher;
pub use timestamp::TimestampComparator;

use crate::error::Result;

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConflictStrategy {
    /// Abort on conflict
    Fail,
    /// Overwrite destination with source
    Overwrite,
    /// Skip conflicting files
    Skip,
    /// Keep the newer file based on modification time
    Newer,
}

/// Result of comparing two files
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonResult {
    /// Files have identical content
    Identical,
    /// Only source file exists
    SourceOnly,
    /// Only destination file exists
    DestinationOnly,
    /// Both files exist with different content (conflict)
    Conflict {
        /// Whether source is newer than destination
        source_newer: bool,
        /// Chosen resolution strategy
        strategy: ConflictStrategy,
    },
}

/// File comparator that combines hashing, timestamps, and diff generation
pub struct FileComparator;

impl Default for FileComparator {
    fn default() -> Self {
        Self::new()
    }
}

impl FileComparator {
    /// Create a new file comparator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Compare two file paths and determine the comparison result
    ///
    /// # Errors
    ///
    /// Returns an error if file I/O operations fail.
    pub fn compare(
        source: &Path,
        destination: &Path,
        strategy: ConflictStrategy,
    ) -> Result<ComparisonResult> {
        let source_exists = source.exists();
        let dest_exists = destination.exists();

        match (source_exists, dest_exists) {
            (false, false) => {
                anyhow::bail!(
                    "Neither source nor destination file exists: source={}, dest={}",
                    source.display(),
                    destination.display()
                )
            }
            (true, false) => Ok(ComparisonResult::SourceOnly),
            (false, true) => Ok(ComparisonResult::DestinationOnly),
            (true, true) => {
                // Both exist - check if content differs
                let source_hash = FileHasher::hash(source)?;
                let dest_hash = FileHasher::hash(destination)?;

                if source_hash == dest_hash {
                    Ok(ComparisonResult::Identical)
                } else {
                    // Conflict - both exist with different content
                    let source_newer = TimestampComparator::is_newer(source, destination)?;
                    Ok(ComparisonResult::Conflict {
                        source_newer,
                        strategy,
                    })
                }
            }
        }
    }

    /// Generate a colored diff between two files
    ///
    /// # Errors
    ///
    /// Returns an error if file reading fails.
    pub fn generate_diff(source: &Path, destination: &Path) -> Result<String> {
        DiffGenerator::generate(source, destination)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_strategy_types() {
        assert_eq!(ConflictStrategy::Fail, ConflictStrategy::Fail);
        assert_ne!(ConflictStrategy::Fail, ConflictStrategy::Overwrite);
    }

    #[test]
    fn test_comparison_result_types() {
        let identical = ComparisonResult::Identical;
        let source_only = ComparisonResult::SourceOnly;
        assert_ne!(identical, source_only);
    }
}
