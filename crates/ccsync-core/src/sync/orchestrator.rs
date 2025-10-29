//! Sync orchestration - coordinates the sync workflow

use std::path::Path;

use anyhow::Context;

use super::SyncResult;
use super::actions::{SyncAction, SyncActionResolver};
use super::executor::FileOperationExecutor;
use crate::comparison::{ConflictStrategy, FileComparator};
use crate::config::{Config, PatternMatcher, SyncDirection};
use crate::error::Result;
use crate::scanner::{FileFilter, Scanner};

/// Approval callback for interactive sync operations
pub type ApprovalCallback = Box<dyn FnMut(&SyncAction) -> Result<bool>>;

/// Main sync engine
pub struct SyncEngine {
    config: Config,
    /// Sync direction (currently unused, will be used for direction-specific rules and reporting)
    #[allow(dead_code)]
    direction: SyncDirection,
    pattern_matcher: Option<PatternMatcher>,
}

impl SyncEngine {
    /// Create a new sync engine
    ///
    /// # Errors
    ///
    /// Returns an error if pattern compilation fails.
    pub fn new(config: Config, direction: SyncDirection) -> Result<Self> {
        // Compile pattern matcher once during construction
        let pattern_matcher = if !config.ignore.is_empty() || !config.include.is_empty() {
            Some(PatternMatcher::with_patterns(
                &config.ignore,
                &config.include,
            )?)
        } else {
            None
        };

        Ok(Self {
            config,
            direction,
            pattern_matcher,
        })
    }

    /// Execute the sync operation
    ///
    /// # Errors
    ///
    /// Returns an error if sync fails.
    pub fn sync(&self, source_root: &Path, dest_root: &Path) -> Result<SyncResult> {
        self.sync_with_approver(source_root, dest_root, None)
    }

    /// Execute the sync operation with an optional approval callback
    ///
    /// The approver callback is called before executing each action.
    /// It should return Ok(true) to proceed, Ok(false) to skip, or Err to abort.
    ///
    /// # Errors
    ///
    /// Returns an error if sync fails or approver returns an error.
    pub fn sync_with_approver(
        &self,
        source_root: &Path,
        dest_root: &Path,
        mut approver: Option<ApprovalCallback>,
    ) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        // Scan source directory
        let filter = FileFilter::new();
        let scanner = Scanner::new(filter, self.config.preserve_symlinks == Some(true));
        let scan_result = scanner.scan(source_root);

        // Process each scanned file
        let executor = FileOperationExecutor::new(self.config.dry_run == Some(true));
        let conflict_strategy = self.get_conflict_strategy();

        for file in &scan_result.files {
            // Get relative path first (needed for pattern matching)
            let rel_path = file
                .path
                .strip_prefix(source_root)
                .with_context(|| format!("Failed to strip prefix from {}", file.path.display()))?;

            // Apply pattern filter to relative path
            let is_dir = file.path.is_dir();
            if let Some(ref matcher) = self.pattern_matcher
                && !matcher.should_include(rel_path, is_dir)
            {
                result.skipped += 1;
                continue;
            }

            let dest_path = dest_root.join(rel_path);

            // Compare files
            let comparison = FileComparator::compare(&file.path, &dest_path, conflict_strategy)?;

            // Determine action
            let action = SyncActionResolver::resolve(file.path.clone(), dest_path, &comparison);

            // Skip actions don't need approval (they're automatic decisions)
            if matches!(action, super::actions::SyncAction::Skip { .. }) {
                if let Err(e) = executor.execute(&action, &mut result) {
                    eprintln!("Error: {e}");
                    result.errors.push(e.to_string());
                }
                continue;
            }

            // Check approval if callback provided (only for Create and Conflict actions)
            if let Some(ref mut approve) = approver {
                match approve(&action) {
                    Ok(true) => {
                        // Approved - continue to execution
                    }
                    Ok(false) => {
                        // Skipped by user
                        result.skipped += 1;
                        *result
                            .skip_reasons
                            .entry("user skipped".to_string())
                            .or_insert(0) += 1;
                        continue;
                    }
                    Err(e) => {
                        // User aborted or error in approval
                        return Err(e);
                    }
                }
            }

            // Execute action
            if let Err(e) = executor.execute(&action, &mut result) {
                eprintln!("Error: {e}");
                result.errors.push(e.to_string());
            }
        }

        // Log warnings from scanner
        for warning in &scan_result.warnings {
            eprintln!("Warning: {warning}");
        }

        // Fail fast if any errors occurred
        if !result.errors.is_empty() {
            anyhow::bail!(
                "Sync failed with {} error(s):\n  - {}",
                result.errors.len(),
                result.errors.join("\n  - ")
            );
        }

        Ok(result)
    }

    /// Get conflict strategy from config or use default
    const fn get_conflict_strategy(&self) -> ConflictStrategy {
        match self.config.conflict_strategy {
            Some(strategy) => strategy,
            None => ConflictStrategy::Fail,
        }
    }
}
