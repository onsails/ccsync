//! Bidirectional synchronization engine
//!
//! This module implements the core sync logic for to-local and to-global operations.
//! Interactive prompts are NOT implemented here - they will be added in Task 4.
//! The sync engine uses ConflictStrategy from config/CLI flags directly.

mod actions;
mod executor;
mod orchestrator;
mod reporting;



/// Synchronization result with statistics
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    /// Files created
    pub created: usize,
    /// Files updated
    pub updated: usize,
    /// Files deleted
    pub deleted: usize,
    /// Files skipped
    pub skipped: usize,
    /// Conflicts encountered
    pub conflicts: usize,
    /// Errors encountered
    pub errors: Vec<String>,
}

impl SyncResult {
    /// Total operations performed
    #[must_use]
    pub const fn total_operations(&self) -> usize {
        self.created + self.updated + self.deleted
    }

    /// Whether sync was successful (no errors)
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}
