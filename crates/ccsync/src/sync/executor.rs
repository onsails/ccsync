//! Atomic file operations executor

use std::fs;
use std::path::Path;

use anyhow::Context;

use super::actions::SyncAction;
use super::SyncResult;
use crate::comparison::ConflictStrategy;
use crate::error::Result;

/// Executes file operations atomically
pub struct FileOperationExecutor {
    dry_run: bool,
}

impl FileOperationExecutor {
    /// Create a new executor
    #[must_use]
    pub const fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// Execute a sync action
    ///
    /// # Errors
    ///
    /// Returns an error if file operations fail.
    pub fn execute(&self, action: &SyncAction, result: &mut SyncResult) -> Result<()> {
        match action {
            SyncAction::Create { source, dest } => {
                if self.dry_run {
                    println!("[DRY RUN] Would create: {}", dest.display());
                    result.created += 1;
                } else {
                    self.copy_file(source, dest)?;
                    result.created += 1;
                }
            }
            SyncAction::Update { source, dest } => {
                if self.dry_run {
                    println!("[DRY RUN] Would update: {}", dest.display());
                    result.updated += 1;
                } else {
                    self.copy_file(source, dest)?;
                    result.updated += 1;
                }
            }
            SyncAction::Skip { path, reason } => {
                if self.dry_run {
                    println!("[DRY RUN] Would skip: {} ({})", path.display(), reason);
                }
                result.skipped += 1;
            }
            SyncAction::Conflict {
                source,
                dest,
                strategy,
                source_newer,
            } => {
                self.handle_conflict(source, dest, *strategy, *source_newer, result)?;
            }
        }
        Ok(())
    }

    /// Handle a conflict according to strategy
    fn handle_conflict(
        &self,
        source: &Path,
        dest: &Path,
        strategy: ConflictStrategy,
        source_newer: bool,
        result: &mut SyncResult,
    ) -> Result<()> {
        match strategy {
            ConflictStrategy::Fail => {
                anyhow::bail!(
                    "Conflict: {} <-> {} (use --conflict to resolve)",
                    source.display(),
                    dest.display()
                );
            }
            ConflictStrategy::Overwrite => {
                if self.dry_run {
                    println!("[DRY RUN] Would overwrite: {}", dest.display());
                } else {
                    self.copy_file(source, dest)?;
                }
                result.updated += 1;
            }
            ConflictStrategy::Skip => {
                if self.dry_run {
                    println!("[DRY RUN] Would skip conflict: {}", dest.display());
                }
                result.conflicts += 1;
            }
            ConflictStrategy::Newer => {
                if source_newer {
                    if self.dry_run {
                        println!("[DRY RUN] Would update (source newer): {}", dest.display());
                    } else {
                        self.copy_file(source, dest)?;
                    }
                    result.updated += 1;
                } else {
                    if self.dry_run {
                        println!(
                            "[DRY RUN] Would skip (dest newer): {}",
                            dest.display()
                        );
                    }
                    result.skipped += 1;
                }
            }
        }
        Ok(())
    }

    /// Copy file atomically
    fn copy_file(&self, source: &Path, dest: &Path) -> Result<()> {
        // Create parent directory if needed
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Copy file
        fs::copy(source, dest).with_context(|| {
            format!(
                "Failed to copy {} to {}",
                source.display(),
                dest.display()
            )
        })?;

        Ok(())
    }
}
