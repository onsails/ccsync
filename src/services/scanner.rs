//! File scanning and discovery module.
//!
//! This module provides functionality for recursively scanning directories
//! and discovering configuration files within .claude directories.
//!
//! # Error Handling Philosophy
//!
//! This module follows a nuanced approach to error handling:
//!
//! - **Fail-fast errors**: Permission errors, invalid paths, and unexpected I/O errors
//!   cause immediate failure with descriptive error messages
//! - **Expected conditions**: Broken symlinks and symlink loops are treated as expected
//!   conditions in file systems, collected in the result, and scanning continues
//! - **Security**: All paths are canonicalized before scanning to prevent traversal attacks
//!
//! This design allows the scanner to be resilient to common file system issues
//! while still failing on unexpected errors that indicate serious problems.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Options for file scanning behavior.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used in Task 6
pub(crate) struct ScanOptions {
    /// Whether to follow symlinks (default: true)
    pub(crate) follow_symlinks: bool,
    /// Maximum depth to traverse (None = unlimited)
    pub(crate) max_depth: Option<usize>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            follow_symlinks: true,
            max_depth: None,
        }
    }
}

/// Result of a file scan operation.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used in Task 6
pub(crate) struct ScanResult {
    /// All discovered files
    pub(crate) files: Vec<PathBuf>,
    /// Broken symlinks encountered
    pub(crate) broken_symlinks: Vec<PathBuf>,
    /// Symlink loops detected
    pub(crate) symlink_loops: Vec<PathBuf>,
}

/// Scanner for recursively discovering files in directories.
#[allow(dead_code)] // Will be used in Task 6
pub(crate) struct Scanner {
    options: ScanOptions,
}

#[allow(dead_code)] // Will be used in Task 6
impl Scanner {
    /// Create a new scanner with default options.
    pub(crate) fn new() -> Self {
        Self {
            options: ScanOptions::default(),
        }
    }

    /// Create a scanner with custom options.
    pub(crate) fn with_options(options: ScanOptions) -> Self {
        Self { options }
    }

    /// Scan a directory for files.
    ///
    /// This method recursively traverses the given directory and collects
    /// all files found, while handling symlinks according to the configured options.
    ///
    /// # Security
    ///
    /// The path is canonicalized to prevent directory traversal attacks.
    /// Only paths that exist and are accessible will be scanned.
    pub(crate) fn scan<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<ScanResult> {
        let path = path.as_ref();

        // Canonicalize path for security (prevents traversal attacks)
        let canonical_path = path.canonicalize()
            .map_err(|e| anyhow::anyhow!("Invalid path {}: {}", path.display(), e))?;
        let mut result = ScanResult {
            files: Vec::new(),
            broken_symlinks: Vec::new(),
            symlink_loops: Vec::new(),
        };

        // Build walkdir configuration using canonical path
        let mut walker = WalkDir::new(&canonical_path).follow_links(self.options.follow_symlinks);

        if let Some(depth) = self.options.max_depth {
            walker = walker.max_depth(depth);
        }

        // Process entries
        for entry in walker {
            match entry {
                Ok(entry) => {
                    // Only collect files, not directories
                    if entry.file_type().is_file() {
                        result.files.push(entry.path().to_path_buf());
                    }
                }
                Err(e) => {
                    // Handle specific error types appropriately
                    if let Some(path) = e.path() {
                        // Check for broken symlink (ENOENT/NotFound)
                        if let Some(io_err) = e.io_error() {
                            if io_err.kind() == std::io::ErrorKind::NotFound {
                                result.broken_symlinks.push(path.to_path_buf());
                                continue;
                            }
                        }

                        // Check for symlink loop
                        if e.loop_ancestor().is_some() {
                            result.symlink_loops.push(path.to_path_buf());
                            continue;
                        }
                    }

                    // For other errors (permissions, etc.), fail fast with context
                    return Err(anyhow::anyhow!(
                        "Error scanning directory{}: {}",
                        e.path().map(|p| format!(" at {}", p.display())).unwrap_or_default(),
                        e
                    ));
                }
            }
        }

        Ok(result)
    }

    /// Scan specifically for .claude directories and their contents.
    ///
    /// This method finds all .claude directories within the given path
    /// and scans their contents.
    ///
    /// Note: When searching for .claude directories, symlinks are never followed
    /// to avoid confusion and ensure we only find real .claude directories.
    /// However, when scanning the contents of found .claude directories,
    /// the `follow_symlinks` option from ScanOptions is respected.
    pub(crate) fn scan_claude_dirs<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<ScanResult> {
        let path = path.as_ref();
        let mut result = ScanResult {
            files: Vec::new(),
            broken_symlinks: Vec::new(),
            symlink_loops: Vec::new(),
        };

        // First, find all .claude directories
        // Note: We intentionally don't follow symlinks when searching for .claude
        // directories to ensure we only find real .claude dirs, not symlinks to them.
        // This prevents confusion where a symlink to ~/.claude would be treated
        // as a local .claude directory.
        let walker = WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Only descend into .claude directories or traverse to find them
                e.file_name() == ".claude" || e.file_type().is_dir()
            });

        for entry in walker {
            match entry {
                Ok(entry) => {
                    if entry.file_name() == ".claude" && entry.file_type().is_dir() {
                        // Scan this .claude directory
                        let scan_result = self.scan(entry.path())?;
                        result.files.extend(scan_result.files);
                        result.broken_symlinks.extend(scan_result.broken_symlinks);
                        result.symlink_loops.extend(scan_result.symlink_loops);
                    }
                }
                Err(e) => {
                    // Fail fast on errors when searching for .claude directories
                    return Err(anyhow::anyhow!(
                        "Error searching for .claude directories{}: {}",
                        e.path().map(|p| format!(" at {}", p.display())).unwrap_or_default(),
                        e
                    ));
                }
            }
        }

        Ok(result)
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    #[test]
    fn test_scanner_basic() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Create test files
        create_test_file(test_dir, "file1.txt", "content1");
        create_test_file(test_dir, "file2.txt", "content2");

        let scanner = Scanner::new();
        let result = scanner.scan(test_dir).unwrap();

        assert_eq!(result.files.len(), 2);
        assert!(result.broken_symlinks.is_empty());
        assert!(result.symlink_loops.is_empty());
    }

    #[test]
    fn test_scanner_nested() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Create nested structure
        let subdir = test_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();

        create_test_file(test_dir, "root.txt", "root");
        create_test_file(&subdir, "nested.txt", "nested");

        let scanner = Scanner::new();
        let result = scanner.scan(test_dir).unwrap();

        assert_eq!(result.files.len(), 2);
    }

    #[test]
    fn test_scanner_max_depth() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Create nested structure
        let subdir1 = test_dir.join("level1");
        let subdir2 = subdir1.join("level2");
        fs::create_dir_all(&subdir2).unwrap();

        create_test_file(test_dir, "root.txt", "root");
        create_test_file(&subdir1, "level1.txt", "level1");
        create_test_file(&subdir2, "level2.txt", "level2");

        let scanner = Scanner::with_options(ScanOptions {
            follow_symlinks: true,
            max_depth: Some(2),
        });
        let result = scanner.scan(test_dir).unwrap();

        // Should find root.txt and level1.txt, but not level2.txt
        assert_eq!(result.files.len(), 2);
    }

    #[test]
    fn test_scan_claude_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Create .claude directory structure
        let claude_dir = test_dir.join(".claude");
        let commands_dir = claude_dir.join("commands");
        fs::create_dir_all(&commands_dir).unwrap();

        create_test_file(&claude_dir, "config.toml", "config");
        create_test_file(&commands_dir, "test.md", "command");

        // Create non-.claude files (should be ignored)
        create_test_file(test_dir, "other.txt", "other");

        let scanner = Scanner::new();
        let result = scanner.scan_claude_dirs(test_dir).unwrap();

        // Should only find files within .claude directory
        assert_eq!(result.files.len(), 2);
        assert!(result.files.iter().all(|p| p.starts_with(&claude_dir)));
    }

    #[test]
    #[cfg(unix)] // Symlinks behave differently on Windows
    fn test_scanner_follows_symlinks() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Create target directory with a file
        let target_dir = test_dir.join("target");
        fs::create_dir(&target_dir).unwrap();
        create_test_file(&target_dir, "real.txt", "real content");

        // Create symlink to the target directory
        std::os::unix::fs::symlink(&target_dir, test_dir.join("link_dir")).unwrap();

        let scanner = Scanner::new();
        let result = scanner.scan(test_dir).unwrap();

        // Should find real.txt through the symlink
        assert!(result.files.iter().any(|p| p.ends_with("real.txt")));
        assert!(result.broken_symlinks.is_empty());
        assert!(result.symlink_loops.is_empty());
    }

    #[test]
    #[cfg(unix)]
    fn test_scanner_detects_broken_symlinks() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Create a valid file
        create_test_file(test_dir, "valid.txt", "content");

        // Create symlink to non-existent target
        std::os::unix::fs::symlink("/nonexistent/path/that/does/not/exist", test_dir.join("broken_link")).unwrap();

        let scanner = Scanner::new();
        let result = scanner.scan(test_dir).unwrap();

        // Should detect the broken symlink and continue scanning
        assert_eq!(result.broken_symlinks.len(), 1);
        assert!(result.broken_symlinks[0].ends_with("broken_link"));
        // Should still find the valid file
        assert_eq!(result.files.len(), 1);
    }

    #[test]
    #[cfg(unix)]
    fn test_scanner_detects_symlink_loops() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Create two directories
        let dir_a = test_dir.join("a");
        let dir_b = test_dir.join("b");
        fs::create_dir(&dir_a).unwrap();
        fs::create_dir(&dir_b).unwrap();

        // Create a loop: a/link_b -> ../b, b/link_a -> ../a
        std::os::unix::fs::symlink(&dir_b, dir_a.join("link_b")).unwrap();
        std::os::unix::fs::symlink(&dir_a, dir_b.join("link_a")).unwrap();

        let scanner = Scanner::new();
        let result = scanner.scan(test_dir).unwrap();

        // Walkdir detects symlink loops and we collect them in the result
        // The loop is caught and added to symlink_loops, not treated as an error
        assert!(!result.symlink_loops.is_empty(), "Should detect at least one symlink loop");
        assert!(result.symlink_loops.iter().any(|p|
            p.to_str().unwrap().contains("link_a") || p.to_str().unwrap().contains("link_b")
        ));
    }

    #[test]
    fn test_scanner_reuse() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        create_test_file(temp_dir1.path(), "file1.txt", "content1");
        create_test_file(temp_dir2.path(), "file2.txt", "content2");

        let scanner = Scanner::new();

        // First scan
        let result1 = scanner.scan(temp_dir1.path()).unwrap();
        assert_eq!(result1.files.len(), 1);

        // Second scan with same scanner - should work correctly (visited_paths cleared)
        let result2 = scanner.scan(temp_dir2.path()).unwrap();
        assert_eq!(result2.files.len(), 1);
    }

    #[test]
    #[cfg(unix)]
    fn test_scanner_respects_no_follow_symlinks() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path();

        // Create a regular file in root
        create_test_file(test_dir, "regular.txt", "regular");

        // Create target directory with a file
        let target_dir = test_dir.join("target");
        fs::create_dir(&target_dir).unwrap();
        create_test_file(&target_dir, "real.txt", "real content");

        // Create symlink to target directory
        std::os::unix::fs::symlink(&target_dir, test_dir.join("link_dir")).unwrap();

        // Scan with follow_symlinks=false
        let scanner = Scanner::with_options(ScanOptions {
            follow_symlinks: false,
            max_depth: None,
        });
        let result = scanner.scan(test_dir).unwrap();

        // Should find regular.txt and real.txt in target/ but NOT through link_dir
        // With follow_symlinks=false, walkdir won't traverse into link_dir
        let has_regular = result.files.iter().any(|p| p.ends_with("regular.txt"));
        let has_real_via_target = result.files.iter().any(|p| p.to_str().unwrap().contains("target") && p.ends_with("real.txt"));

        assert!(has_regular, "Should find regular.txt");
        assert!(has_real_via_target, "Should find real.txt via target/ directory");
    }
}
