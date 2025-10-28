//! Sync action determination logic

use std::path::PathBuf;

use crate::comparison::{ComparisonResult, ConflictStrategy};

/// Sync action to perform
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncAction {
    /// Create new file at destination
    Create { source: PathBuf, dest: PathBuf },
    /// Update existing file at destination
    Update { source: PathBuf, dest: PathBuf },
    /// Skip this file (no action needed)
    Skip { path: PathBuf, reason: String },
    /// Conflict requiring resolution
    Conflict {
        source: PathBuf,
        dest: PathBuf,
        strategy: ConflictStrategy,
        source_newer: bool,
    },
}

/// Resolves comparison results into sync actions
pub struct SyncActionResolver;

impl SyncActionResolver {
    /// Create a new action resolver
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Determine sync action from comparison result
    #[must_use]
    pub fn resolve(
        source: PathBuf,
        dest: PathBuf,
        comparison: &ComparisonResult,
        _default_strategy: ConflictStrategy,
    ) -> SyncAction {
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
