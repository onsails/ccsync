//! File metadata comparison for efficient difference detection.
//!
//! This module provides functionality to compare files based on metadata
//! (existence, size, modification time) without reading full file contents.
//! This enables quick filtering before expensive content comparisons.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;

/// Errors that can occur during metadata operations
#[derive(Error, Debug)]
pub enum MetadataError {
    /// Error reading file metadata
    #[error("Failed to read metadata for {path}: {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// File metadata for comparison
///
/// Stores essential metadata for quick file comparison without
/// reading full file contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    /// File path
    pub path: PathBuf,
    /// Whether the file exists
    pub exists: bool,
    /// File size in bytes (None if file doesn't exist)
    pub size: Option<u64>,
    /// Modification time (None if file doesn't exist)
    pub modified: Option<SystemTime>,
}

impl FileMetadata {
    /// Get metadata for a file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file
    ///
    /// # Returns
    ///
    /// FileMetadata with existence, size, and modification time
    pub fn from_path(path: &Path) -> Result<Self, MetadataError> {
        let path_buf = path.to_path_buf();

        match fs::metadata(path) {
            Ok(metadata) => {
                let modified_time = metadata.modified().map_err(|e| MetadataError::ReadError {
                    path: path_buf.clone(),
                    source: e,
                })?;

                Ok(Self {
                    path: path_buf,
                    exists: true,
                    size: Some(metadata.len()),
                    modified: Some(modified_time),
                })
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                path: path_buf,
                exists: false,
                size: None,
                modified: None,
            }),
            Err(e) => Err(MetadataError::ReadError {
                path: path_buf,
                source: e,
            }),
        }
    }

    /// Check if this file differs from another based on metadata
    ///
    /// Returns true if files are **guaranteed** to differ based on metadata alone.
    /// Returns false if uncertain (might be same or different - needs content check).
    ///
    /// # Difference Detection Rules:
    ///
    /// - If one exists and the other doesn't → DIFFERENT (returns true)
    /// - If both don't exist → SAME (returns false)
    /// - If sizes differ → DIFFERENT (returns true)
    /// - If sizes match → UNCERTAIN (returns false, needs content check)
    ///
    /// Note: Modification time is NOT used for difference detection because:
    /// - Files can be touched without content changes (false positive)
    /// - Files can be modified with same mtime on some filesystems
    /// - Size mismatch is already a reliable indicator
    ///
    /// # Arguments
    ///
    /// * `other` - Other file metadata to compare against
    pub fn differs_from(&self, other: &FileMetadata) -> bool {
        // Different existence states
        if self.exists != other.exists {
            return true;
        }

        // Both don't exist - they're the same
        if !self.exists && !other.exists {
            return false;
        }

        // Both exist - compare size (reliable indicator)
        if self.size != other.size {
            return true;
        }

        // Sizes match - cannot determine from metadata alone
        // Return false to indicate "uncertain, needs content comparison"
        false
    }

    /// Check if files might be the same based on metadata
    ///
    /// Returns true if metadata suggests files could be identical.
    /// This is an optimization hint - caller should still verify with content comparison.
    pub fn might_be_same(&self, other: &FileMetadata) -> bool {
        self.exists
            && other.exists
            && self.size == other.size
            && self.modified == other.modified
    }

    /// Check if file exists
    pub fn exists(&self) -> bool {
        self.exists
    }

    /// Get file size (if exists)
    pub fn size(&self) -> Option<u64> {
        self.size
    }

    /// Get modification time (if exists)
    pub fn modified(&self) -> Option<SystemTime> {
        self.modified
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_metadata_from_nonexistent_file() {
        let metadata = FileMetadata::from_path(Path::new("/nonexistent/file.txt")).unwrap();

        assert!(!metadata.exists);
        assert_eq!(metadata.size, None);
        assert_eq!(metadata.modified, None);
    }

    #[test]
    fn test_metadata_from_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let metadata = FileMetadata::from_path(&file_path).unwrap();

        assert!(metadata.exists);
        assert_eq!(metadata.size, Some(12)); // "test content" = 12 bytes
        assert!(metadata.modified.is_some());
    }

    #[test]
    fn test_differs_one_exists_other_doesnt() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("exists.txt");
        fs::write(&file1, "content").unwrap();

        let meta1 = FileMetadata::from_path(&file1).unwrap();
        let meta2 = FileMetadata::from_path(Path::new("/nonexistent.txt")).unwrap();

        assert!(meta1.differs_from(&meta2));
        assert!(meta2.differs_from(&meta1));
    }

    #[test]
    fn test_differs_both_dont_exist() {
        let meta1 = FileMetadata::from_path(Path::new("/nonexistent1.txt")).unwrap();
        let meta2 = FileMetadata::from_path(Path::new("/nonexistent2.txt")).unwrap();

        assert!(!meta1.differs_from(&meta2));
    }

    #[test]
    fn test_differs_different_sizes() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        fs::write(&file1, "short").unwrap();
        fs::write(&file2, "much longer content").unwrap();

        let meta1 = FileMetadata::from_path(&file1).unwrap();
        let meta2 = FileMetadata::from_path(&file2).unwrap();

        assert!(meta1.differs_from(&meta2));
    }

    #[test]
    fn test_same_size_different_mtime() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        fs::write(&file1, "content").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(&file2, "content").unwrap();

        let meta1 = FileMetadata::from_path(&file1).unwrap();
        let meta2 = FileMetadata::from_path(&file2).unwrap();

        // Same size, different mtime -> uncertain, returns false (needs content check)
        assert!(!meta1.differs_from(&meta2));
        // but might_be_same should also return false since mtimes differ
        assert!(!meta1.might_be_same(&meta2));
    }

    #[test]
    fn test_might_be_same() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file.txt");
        fs::write(&file1, "content").unwrap();

        let meta1 = FileMetadata::from_path(&file1).unwrap();
        let meta2 = FileMetadata::from_path(&file1).unwrap();

        // Same file read twice - metadata should match
        assert!(meta1.might_be_same(&meta2));
        assert!(!meta1.differs_from(&meta2));
    }

    #[test]
    fn test_might_be_same_false_when_one_missing() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("exists.txt");
        fs::write(&file1, "content").unwrap();

        let meta1 = FileMetadata::from_path(&file1).unwrap();
        let meta2 = FileMetadata::from_path(Path::new("/nonexistent.txt")).unwrap();

        assert!(!meta1.might_be_same(&meta2));
    }

    #[test]
    fn test_accessor_methods() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        let metadata = FileMetadata::from_path(&file_path).unwrap();

        assert!(metadata.exists());
        assert_eq!(metadata.size(), Some(4));
        assert!(metadata.modified().is_some());
    }

    #[test]
    fn test_accessor_methods_nonexistent() {
        let metadata = FileMetadata::from_path(Path::new("/nonexistent.txt")).unwrap();

        assert!(!metadata.exists());
        assert_eq!(metadata.size(), None);
        assert_eq!(metadata.modified(), None);
    }
}
