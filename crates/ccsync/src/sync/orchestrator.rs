//! Sync orchestration - coordinates the sync workflow

use std::path::Path;

use anyhow::Context;

use super::actions::SyncActionResolver;
use super::executor::FileOperationExecutor;
use super::SyncResult;
use crate::comparison::{ConflictStrategy, FileComparator};
use crate::config::{Config, PatternMatcher};
use crate::error::Result;
use crate::scanner::{FileFilter, Scanner};

/// Sync direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    /// Sync from global (~/.claude) to local (.claude)
    ToLocal,
    /// Sync from local (.claude) to global (~/.claude)
    ToGlobal,
}

/// Main sync engine
pub struct SyncEngine {
    config: Config,
    direction: SyncDirection,
}

impl SyncEngine {
    /// Create a new sync engine
    #[must_use]
    pub const fn new(config: Config, direction: SyncDirection) -> Self {
        Self { config, direction }
    }

    /// Execute the sync operation
    ///
    /// # Errors
    ///
    /// Returns an error if sync fails.
    pub fn sync(&self, source_root: &Path, dest_root: &Path) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        // Scan source directory
        let filter = FileFilter::new();
        let scanner = Scanner::new(filter, self.config.preserve_symlinks == Some(true));
        let scan_result = scanner.scan(source_root);

        // Apply pattern filters
        let pattern_matcher = if !self.config.ignore.is_empty() || !self.config.include.is_empty()
        {
            Some(PatternMatcher::with_patterns(
                &self.config.ignore,
                &self.config.include,
            )?)
        } else {
            None
        };

        // Process each scanned file
        let executor = FileOperationExecutor::new(self.config.dry_run == Some(true));

        for file in &scan_result.files {
            // Apply pattern filter
            if let Some(ref matcher) = pattern_matcher
                && !matcher.should_include(&file.path, false) {
                    result.skipped += 1;
                    continue;
                }

            // Get relative path
            let rel_path = file
                .path
                .strip_prefix(source_root)
                .with_context(|| format!("Failed to strip prefix from {}", file.path.display()))?;

            let dest_path = dest_root.join(rel_path);

            // Compare files
            let comparison = FileComparator::compare(
                &file.path,
                &dest_path,
                Self::get_conflict_strategy(),
            )?;

            // Determine action
            let action = SyncActionResolver::resolve(
                file.path.clone(),
                dest_path,
                &comparison,
                Self::get_conflict_strategy(),
            );

            // Execute action
            if let Err(e) = executor.execute(&action, &mut result) {
                result.errors.push(e.to_string());
            }
        }

        // Log warnings from scanner
        for warning in &scan_result.warnings {
            eprintln!("Warning: {warning}");
        }

        Ok(result)
    }

    /// Get conflict strategy from config or use default
    const fn get_conflict_strategy() -> ConflictStrategy {
        // Default to Fail if not specified
        ConflictStrategy::Fail
    }
}
