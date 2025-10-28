//! Sync action determination logic

use std::path::PathBuf;

use crate::comparison::{ComparisonResult, ConflictStrategy};

/// Sync action to perform
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncAction {
    /// Create new file at destination
    Create {
        /// Source file path
        source: PathBuf,
        /// Destination file path
        dest: PathBuf,
    },
    /// Skip this file (no action needed)
    Skip {
        /// File path being skipped
        path: PathBuf,
        /// Reason for skipping
        reason: String,
    },
    /// Conflict requiring resolution
    Conflict {
        /// Source file path
        source: PathBuf,
        /// Destination file path
        dest: PathBuf,
        /// Conflict resolution strategy
        strategy: ConflictStrategy,
        /// Whether source is newer than destination
        source_newer: bool,
    },
}

/// Resolves comparison results into sync actions
pub struct SyncActionResolver;

impl SyncActionResolver {
    /// Determine sync action from comparison result
    #[must_use]
    pub fn resolve(source: PathBuf, dest: PathBuf, comparison: &ComparisonResult) -> SyncAction {
        match comparison {
            ComparisonResult::Identical => SyncAction::Skip {
                path: source,
                reason: "identical content".to_string(),
            },
            ComparisonResult::SourceOnly => SyncAction::Create { source, dest },
            ComparisonResult::DestinationOnly => SyncAction::Skip {
                path: dest,
                reason: "source doesn't exist".to_string(),
            },
            ComparisonResult::Conflict {
                source_newer,
                strategy,
            } => SyncAction::Conflict {
                source,
                dest,
                strategy: *strategy,
                source_newer: *source_newer,
            },
        }
    }
}
