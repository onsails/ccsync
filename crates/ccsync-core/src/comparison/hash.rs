//! File hashing for content comparison using SHA-256

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use anyhow::Context;
use sha2::{Digest, Sha256};

use crate::error::Result;

/// File hash result
pub type FileHash = [u8; 32];

/// File hasher
pub struct FileHasher;

impl Default for FileHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl FileHasher {
    /// Create a new file hasher
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Compute SHA-256 hash of a file by streaming its contents
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    pub fn hash(path: &Path) -> Result<FileHash> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open file for hashing: {}", path.display()))?;

        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192]; // 8KB buffer for streaming

        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        Ok(hasher.finalize().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_hash_identical_files() {
        let tmp = TempDir::new().unwrap();
        let file1 = tmp.path().join("file1.txt");
        let file2 = tmp.path().join("file2.txt");

        fs::write(&file1, "same content").unwrap();
        fs::write(&file2, "same content").unwrap();

        let _hasher = FileHasher::new();
        let hash1 = FileHasher::hash(&file1).unwrap();
        let hash2 = FileHasher::hash(&file2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_different_files() {
        let tmp = TempDir::new().unwrap();
        let file1 = tmp.path().join("file1.txt");
        let file2 = tmp.path().join("file2.txt");

        fs::write(&file1, "content 1").unwrap();
        fs::write(&file2, "content 2").unwrap();

        let _hasher = FileHasher::new();
        let hash1 = FileHasher::hash(&file1).unwrap();
        let hash2 = FileHasher::hash(&file2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_large_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("large.bin");

        // Create a 1MB file
        let content = vec![0u8; 1024 * 1024];
        fs::write(&file, &content).unwrap();

        let _hasher = FileHasher::new();
        let hash = FileHasher::hash(&file);

        assert!(hash.is_ok());
    }

    #[test]
    fn test_hash_empty_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("empty.txt");
        fs::write(&file, "").unwrap();

        let _hasher = FileHasher::new();
        let hash = FileHasher::hash(&file);

        assert!(hash.is_ok());
    }
}
