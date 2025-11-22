//! Atomic file operations executor

use std::fs;
use std::path::Path;

use anyhow::Context;

use super::SyncResult;
use super::actions::SyncAction;
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
                    eprintln!("[DRY RUN] Would create: {}", dest.display());
                } else {
                    Self::copy_file(source, dest)?;
                }
                result.created += 1;
            }
            SyncAction::CreateDirectory { source, dest } => {
                if self.dry_run {
                    eprintln!("[DRY RUN] Would create directory: {}", dest.display());
                } else {
                    Self::copy_directory(source, dest)?;
                }
                result.created += 1;
            }
            SyncAction::Skip { path, reason } => {
                if self.dry_run {
                    eprintln!("[DRY RUN] Would skip: {} ({})", path.display(), reason);
                }
                result.skipped += 1;
                *result.skip_reasons.entry(reason.clone()).or_insert(0) += 1;
            }
            SyncAction::Conflict {
                source,
                dest,
                strategy,
                source_newer,
            } => {
                self.handle_conflict(source, dest, *strategy, *source_newer, result)?;
            }
            SyncAction::DirectoryConflict {
                source,
                dest,
                strategy,
                source_newer,
            } => {
                self.handle_directory_conflict(source, dest, *strategy, *source_newer, result)?;
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
                    eprintln!("[DRY RUN] Would overwrite: {}", dest.display());
                } else {
                    Self::copy_file(source, dest)?;
                }
                result.updated += 1;
            }
            ConflictStrategy::Skip => {
                if self.dry_run {
                    eprintln!("[DRY RUN] Would skip conflict: {}", dest.display());
                }
                result.conflicts += 1;
            }
            ConflictStrategy::Newer => {
                if source_newer {
                    if self.dry_run {
                        eprintln!("[DRY RUN] Would update (source newer): {}", dest.display());
                    } else {
                        Self::copy_file(source, dest)?;
                    }
                    result.updated += 1;
                } else {
                    if self.dry_run {
                        eprintln!("[DRY RUN] Would skip (dest newer): {}", dest.display());
                    }
                    result.skipped += 1;
                }
            }
        }
        Ok(())
    }

    /// Copy file atomically
    fn copy_file(source: &Path, dest: &Path) -> Result<()> {
        // Create parent directory if needed
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Copy file
        fs::copy(source, dest).with_context(|| {
            format!("Failed to copy {} to {}", source.display(), dest.display())
        })?;

        Ok(())
    }

    /// Handle a directory conflict according to strategy
    fn handle_directory_conflict(
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
                    "Directory conflict: {} <-> {} (use --conflict to resolve)",
                    source.display(),
                    dest.display()
                );
            }
            ConflictStrategy::Overwrite => {
                if self.dry_run {
                    eprintln!("[DRY RUN] Would overwrite directory: {}", dest.display());
                } else {
                    // Remove destination and copy source
                    if dest.exists() {
                        fs::remove_dir_all(dest)?;
                    }
                    Self::copy_directory(source, dest)?;
                }
                result.updated += 1;
            }
            ConflictStrategy::Skip => {
                if self.dry_run {
                    eprintln!("[DRY RUN] Would skip directory conflict: {}", dest.display());
                }
                result.conflicts += 1;
            }
            ConflictStrategy::Newer => {
                if source_newer {
                    if self.dry_run {
                        eprintln!(
                            "[DRY RUN] Would update directory (source newer): {}",
                            dest.display()
                        );
                    } else {
                        // Remove destination and copy source
                        if dest.exists() {
                            fs::remove_dir_all(dest)?;
                        }
                        Self::copy_directory(source, dest)?;
                    }
                    result.updated += 1;
                } else if self.dry_run {
                    eprintln!("[DRY RUN] Would skip directory (dest newer): {}", dest.display());
                    result.skipped += 1;
                } else {
                    result.skipped += 1;
                }
            }
        }
        Ok(())
    }

    /// Copy directory recursively
    ///
    /// # Errors
    ///
    /// Returns an error if directory operations fail.
    pub fn copy_directory(source: &Path, dest: &Path) -> Result<()> {
        // Create destination directory
        fs::create_dir_all(dest)
            .with_context(|| format!("Failed to create directory: {}", dest.display()))?;

        // Recursively copy contents
        Self::copy_directory_contents(source, dest)?;

        Ok(())
    }

    /// Recursively copy directory contents
    fn copy_directory_contents(source: &Path, dest: &Path) -> Result<()> {
        for entry in fs::read_dir(source)
            .with_context(|| format!("Failed to read directory: {}", source.display()))?
        {
            let entry =
                entry.with_context(|| format!("Failed to read entry in: {}", source.display()))?;
            let path = entry.path();
            let file_name = path.file_name().unwrap();
            let dest_path = dest.join(file_name);

            if path.is_dir() {
                Self::copy_directory(&path, &dest_path)?;
            } else if path.is_file() {
                Self::copy_file(&path, &dest_path)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_copy_directory_basic() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir(&src).unwrap();
        fs::write(src.join("file1.txt"), "content1").unwrap();
        fs::write(src.join("file2.txt"), "content2").unwrap();

        FileOperationExecutor::copy_directory(&src, &dst).unwrap();

        assert!(dst.exists());
        assert!(dst.join("file1.txt").exists());
        assert!(dst.join("file2.txt").exists());
        assert_eq!(fs::read_to_string(dst.join("file1.txt")).unwrap(), "content1");
        assert_eq!(fs::read_to_string(dst.join("file2.txt")).unwrap(), "content2");
    }

    #[test]
    fn test_copy_directory_nested() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir(&src).unwrap();
        let subdir = src.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(src.join("root.txt"), "root").unwrap();
        fs::write(subdir.join("nested.txt"), "nested").unwrap();

        FileOperationExecutor::copy_directory(&src, &dst).unwrap();

        assert!(dst.exists());
        assert!(dst.join("root.txt").exists());
        assert!(dst.join("subdir").exists());
        assert!(dst.join("subdir/nested.txt").exists());
        assert_eq!(fs::read_to_string(dst.join("subdir/nested.txt")).unwrap(), "nested");
    }

    #[test]
    fn test_copy_directory_empty() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir(&src).unwrap();

        FileOperationExecutor::copy_directory(&src, &dst).unwrap();

        assert!(dst.exists());
        assert!(dst.is_dir());
    }
}
