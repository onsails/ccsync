//! File timestamp comparison for determining recency

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use anyhow::Context;

use crate::error::Result;

/// Timestamp comparator
pub struct TimestampComparator;

impl TimestampComparator {
    /// Create a new timestamp comparator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Check if source file is newer than destination file
    ///
    /// # Errors
    ///
    /// Returns an error if file metadata cannot be read.
    pub fn is_newer(source: &Path, destination: &Path) -> Result<bool> {
        let source_time = Self::get_modified_time(source)?;
        let dest_time = Self::get_modified_time(destination)?;

        Ok(source_time > dest_time)
    }

    /// Get the modification time of a file
    pub fn get_modified_time(path: &Path) -> Result<SystemTime> {
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to read metadata for: {}", path.display()))?;

        metadata
            .modified()
            .with_context(|| format!("Failed to get modification time for: {}", path.display()))
    }

    /// Compare modification times and return ordering
    ///
    /// # Errors
    ///
    /// Returns an error if file metadata cannot be read.
    pub fn compare_times(
        source: &Path,
        destination: &Path,
    ) -> Result<std::cmp::Ordering> {
        let source_time = Self::get_modified_time(source)?;
        let dest_time = Self::get_modified_time(destination)?;

        Ok(source_time.cmp(&dest_time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_identical_timestamps() {
        let tmp = TempDir::new().unwrap();
        let file1 = tmp.path().join("file1.txt");
        let file2 = tmp.path().join("file2.txt");

        fs::write(&file1, "content").unwrap();

        // Copy file - this may create identical or newer timestamps depending on filesystem
        fs::copy(&file1, &file2).unwrap();

        let _comparator = TimestampComparator::new();
        let ordering = TimestampComparator::compare_times(&file1, &file2).unwrap();

        // On some filesystems, copy creates identical timestamps (Equal)
        // On others, copy creates a newer file (Less, because file1 < file2)
        // We verify both files exist and comparison works without panicking
        assert!(
            matches!(ordering, std::cmp::Ordering::Equal | std::cmp::Ordering::Less),
            "Expected Equal or Less for copied file timestamps, got {:?}",
            ordering
        );
    }

    #[test]
    fn test_source_newer() {
        let tmp = TempDir::new().unwrap();
        let file1 = tmp.path().join("old.txt");
        let file2 = tmp.path().join("new.txt");

        // Create old file
        fs::write(&file1, "old content").unwrap();

        // Sleep to ensure time difference
        thread::sleep(Duration::from_millis(10));

        // Create new file
        fs::write(&file2, "new content").unwrap();

        let _comparator = TimestampComparator::new();
        let is_newer = TimestampComparator::is_newer(&file2, &file1).unwrap();

        assert!(is_newer, "file2 should be newer than file1");
    }

    #[test]
    fn test_destination_newer() {
        let tmp = TempDir::new().unwrap();
        let file1 = tmp.path().join("new.txt");
        let file2 = tmp.path().join("old.txt");

        // Create old file
        fs::write(&file2, "old content").unwrap();

        // Sleep to ensure time difference
        thread::sleep(Duration::from_millis(10));

        // Create new file
        fs::write(&file1, "new content").unwrap();

        let _comparator = TimestampComparator::new();
        let is_newer = TimestampComparator::is_newer(&file1, &file2).unwrap();

        assert!(is_newer, "file1 should be newer than file2");
    }

    #[test]
    fn test_get_modified_time() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("file.txt");
        fs::write(&file, "content").unwrap();

        let time = TimestampComparator::get_modified_time(&file);
        assert!(time.is_ok());
    }

    #[test]
    fn test_nonexistent_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("nonexistent.txt");

        let result = TimestampComparator::get_modified_time(&file);
        assert!(result.is_err());
    }
}
