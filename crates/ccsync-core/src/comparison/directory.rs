//! Directory comparison for recursive syncing
//!
//! This module provides recursive directory comparison to identify
//! files that are added, modified, removed, or unchanged between
//! source and destination directories.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;

use super::hash::FileHasher;
use super::timestamp::TimestampComparator;

/// Result of comparing two directories recursively
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryComparison {
    /// Files present in source but not in destination
    pub added: Vec<PathBuf>,
    /// Files with different content between source and destination
    pub modified: Vec<PathBuf>,
    /// Files present in destination but not in source
    pub removed: Vec<PathBuf>,
    /// Files with identical content in both locations
    pub unchanged: Vec<PathBuf>,
}

impl DirectoryComparison {
    /// Check if directories are identical (no changes)
    #[must_use]
    pub const fn is_identical(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.removed.is_empty()
    }

    /// Count total number of changes
    #[must_use]
    pub const fn change_count(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len()
    }
}

/// Directory comparator for recursive comparison
pub struct DirectoryComparator;

impl DirectoryComparator {
    /// Compare two directories recursively
    ///
    /// Returns paths relative to the source/destination roots.
    ///
    /// # Errors
    ///
    /// Returns an error if directory traversal or file operations fail.
    pub fn compare(source: &Path, destination: &Path) -> Result<DirectoryComparison> {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut removed = Vec::new();
        let mut unchanged = Vec::new();

        // Collect all files in source
        let source_files = Self::collect_files(source)?;
        let dest_files = if destination.exists() {
            Self::collect_files(destination)?
        } else {
            HashSet::new()
        };

        // Files in source
        for rel_path in &source_files {
            let source_file = source.join(rel_path);
            let dest_file = destination.join(rel_path);

            if dest_files.contains(rel_path) {
                // File exists in both - check if modified
                let source_hash = FileHasher::hash(&source_file)?;
                let dest_hash = FileHasher::hash(&dest_file)?;

                if source_hash == dest_hash {
                    unchanged.push(rel_path.clone());
                } else {
                    modified.push(rel_path.clone());
                }
            } else {
                // File only in source
                added.push(rel_path.clone());
            }
        }

        // Files only in destination
        for rel_path in &dest_files {
            if !source_files.contains(rel_path) {
                removed.push(rel_path.clone());
            }
        }

        Ok(DirectoryComparison {
            added,
            modified,
            removed,
            unchanged,
        })
    }

    /// Determine if source directory is newer than destination
    ///
    /// Uses the newest file in each directory tree for comparison.
    ///
    /// # Errors
    ///
    /// Returns an error if file metadata cannot be read.
    pub fn is_source_newer(source: &Path, destination: &Path) -> Result<bool> {
        let source_newest = Self::find_newest_file(source)?;
        let dest_newest = Self::find_newest_file(destination)?;

        match (source_newest, dest_newest) {
            (Some(src), Some(dst)) => TimestampComparator::is_newer(&src, &dst),
            (Some(_), None) => Ok(true), // Source exists, dest doesn't
            (None, Some(_) | None) => Ok(false), // Dest exists or both empty
        }
    }

    /// Collect all files in a directory tree (relative paths)
    fn collect_files(dir: &Path) -> Result<HashSet<PathBuf>> {
        let mut files = HashSet::new();
        Self::collect_files_recursive(dir, dir, &mut files)?;
        Ok(files)
    }

    /// Recursively collect files, storing relative paths
    fn collect_files_recursive(
        base: &Path,
        current: &Path,
        files: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                Self::collect_files_recursive(base, &path, files)?;
            } else if path.is_file() {
                let rel_path = path.strip_prefix(base).unwrap().to_path_buf();
                files.insert(rel_path);
            }
        }
        Ok(())
    }

    /// Find the newest file in a directory tree
    fn find_newest_file(dir: &Path) -> Result<Option<PathBuf>> {
        if !dir.exists() {
            return Ok(None);
        }

        let files = Self::collect_files(dir)?;
        if files.is_empty() {
            return Ok(None);
        }

        let mut newest: Option<PathBuf> = None;

        for rel_path in files {
            let full_path = dir.join(&rel_path);
            newest = match newest {
                None => Some(full_path),
                Some(ref current_newest) => {
                    if TimestampComparator::is_newer(&full_path, current_newest)? {
                        Some(full_path)
                    } else {
                        Some(current_newest.clone())
                    }
                }
            };
        }

        Ok(newest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_compare_identical_directories() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir(&src).unwrap();
        fs::create_dir(&dst).unwrap();

        fs::write(src.join("file1.txt"), "content").unwrap();
        fs::write(dst.join("file1.txt"), "content").unwrap();

        let result = DirectoryComparator::compare(&src, &dst).unwrap();

        assert!(result.is_identical());
        assert_eq!(result.unchanged.len(), 1);
        assert!(result.unchanged.iter().any(|p| p == Path::new("file1.txt")));
    }

    #[test]
    fn test_compare_added_files() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir(&src).unwrap();
        fs::create_dir(&dst).unwrap();

        fs::write(src.join("new.txt"), "new content").unwrap();

        let result = DirectoryComparator::compare(&src, &dst).unwrap();

        assert_eq!(result.added.len(), 1);
        assert!(result.added.iter().any(|p| p == Path::new("new.txt")));
        assert_eq!(result.change_count(), 1);
    }

    #[test]
    fn test_compare_modified_files() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir(&src).unwrap();
        fs::create_dir(&dst).unwrap();

        fs::write(src.join("file.txt"), "new content").unwrap();
        fs::write(dst.join("file.txt"), "old content").unwrap();

        let result = DirectoryComparator::compare(&src, &dst).unwrap();

        assert_eq!(result.modified.len(), 1);
        assert!(result.modified.iter().any(|p| p == Path::new("file.txt")));
    }

    #[test]
    fn test_compare_removed_files() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir(&src).unwrap();
        fs::create_dir(&dst).unwrap();

        fs::write(dst.join("old.txt"), "old").unwrap();

        let result = DirectoryComparator::compare(&src, &dst).unwrap();

        assert_eq!(result.removed.len(), 1);
        assert!(result.removed.iter().any(|p| p == Path::new("old.txt")));
    }

    #[test]
    fn test_compare_nested_directories() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir(&src).unwrap();
        fs::create_dir(&dst).unwrap();

        let src_sub = src.join("subdir");
        let dst_sub = dst.join("subdir");
        fs::create_dir(&src_sub).unwrap();
        fs::create_dir(&dst_sub).unwrap();

        fs::write(src_sub.join("nested.txt"), "content").unwrap();
        fs::write(dst_sub.join("nested.txt"), "content").unwrap();

        let result = DirectoryComparator::compare(&src, &dst).unwrap();

        assert!(result.is_identical());
        assert_eq!(result.unchanged.len(), 1);
        assert!(result
            .unchanged
            .iter()
            .any(|p| p == Path::new("subdir/nested.txt")));
    }

    #[test]
    fn test_compare_destination_does_not_exist() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir(&src).unwrap();
        fs::write(src.join("file.txt"), "content").unwrap();

        let result = DirectoryComparator::compare(&src, &dst).unwrap();

        assert_eq!(result.added.len(), 1);
        assert_eq!(result.removed.len(), 0);
    }
}
