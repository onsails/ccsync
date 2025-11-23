//! Sync orchestration - coordinates the sync workflow

use std::path::Path;

use anyhow::Context;

use super::SyncResult;
use super::actions::{SyncAction, SyncActionResolver};
use super::executor::FileOperationExecutor;
use crate::comparison::{ConflictStrategy, DirectoryComparator, FileComparator};
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

            // Determine action based on whether it's a file or directory
            let action = Self::determine_sync_action(&file.path, &dest_path, is_dir, conflict_strategy)?;

            // Skip actions don't need approval (they're automatic decisions)
            if matches!(action, super::actions::SyncAction::Skip { .. }) {
                if let Err(e) = executor.execute(&action, &mut result) {
                    eprintln!("Error: {e}");
                    result.errors.push(e.to_string());
                }
                continue;
            }

            // Check approval if callback provided (only for Create and Conflict actions)
            match Self::apply_approval(&action, &mut approver, &mut result) {
                Ok(Some(action_to_execute)) => {
                    // Execute action
                    if let Err(e) = executor.execute(&action_to_execute, &mut result) {
                        eprintln!("Error: {e}");
                        result.errors.push(e.to_string());
                    }
                }
                Ok(None) => {
                    // User skipped - move to next file
                }
                Err(e) => {
                    // User aborted or error in approval
                    return Err(e);
                }
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

    /// Determine the sync action for a file or directory
    fn determine_sync_action(
        source_path: &Path,
        dest_path: &Path,
        is_dir: bool,
        conflict_strategy: ConflictStrategy,
    ) -> Result<SyncAction> {
        if is_dir {
            // Handle directory syncing
            if dest_path.exists() {
                // Both exist - compare directories
                let dir_comparison = DirectoryComparator::compare(source_path, dest_path)?;

                if dir_comparison.is_identical() {
                    Ok(SyncAction::Skip {
                        path: source_path.to_path_buf(),
                        reason: "identical content".to_string(),
                    })
                } else {
                    // Directories differ - check if source is newer
                    let source_newer = DirectoryComparator::is_source_newer(source_path, dest_path)?;
                    Ok(SyncAction::DirectoryConflict {
                        source: source_path.to_path_buf(),
                        dest: dest_path.to_path_buf(),
                        strategy: conflict_strategy,
                        source_newer,
                    })
                }
            } else {
                // Destination doesn't exist - create it
                Ok(SyncAction::CreateDirectory {
                    source: source_path.to_path_buf(),
                    dest: dest_path.to_path_buf(),
                })
            }
        } else {
            // Handle file syncing
            let comparison = FileComparator::compare(source_path, dest_path, conflict_strategy)?;
            Ok(SyncActionResolver::resolve(
                source_path.to_path_buf(),
                dest_path.to_path_buf(),
                &comparison,
            ))
        }
    }

    /// Apply approval logic to a sync action
    /// Returns Ok(Some(action)) if approved, Ok(None) if user skipped, or Err if aborted
    fn apply_approval(
        action: &SyncAction,
        approver: &mut Option<ApprovalCallback>,
        result: &mut SyncResult,
    ) -> Result<Option<SyncAction>> {
        if let Some(approve) = approver {
            match approve(action) {
                Ok(true) => {
                    // Approved - if this is a Fail conflict, treat as Overwrite
                    Ok(Some(match action {
                        SyncAction::Conflict {
                            source,
                            dest,
                            strategy: ConflictStrategy::Fail,
                            source_newer,
                        } => SyncAction::Conflict {
                            source: source.clone(),
                            dest: dest.clone(),
                            strategy: ConflictStrategy::Overwrite,
                            source_newer: *source_newer,
                        },
                        SyncAction::DirectoryConflict {
                            source,
                            dest,
                            strategy: ConflictStrategy::Fail,
                            source_newer,
                        } => SyncAction::DirectoryConflict {
                            source: source.clone(),
                            dest: dest.clone(),
                            strategy: ConflictStrategy::Overwrite,
                            source_newer: *source_newer,
                        },
                        _ => action.clone(),
                    }))
                }
                Ok(false) => {
                    // Skipped by user
                    result.skipped += 1;
                    *result
                        .skip_reasons
                        .entry("user skipped".to_string())
                        .or_insert(0) += 1;
                    Ok(None)
                }
                Err(e) => Err(e),
            }
        } else {
            Ok(Some(action.clone()))
        }
    }
}
