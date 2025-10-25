//! File diff generation for displaying changes between files.
//!
//! This module provides functionality to generate unified diffs between
//! file contents using the `similar` crate.

use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during diff generation
#[derive(Error, Debug)]
pub enum DiffError {
    /// Error reading file for diff
    /// This includes permission errors and binary file errors (non-UTF8 content)
    #[error("Failed to read file {path} for diff: {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Type of file change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// File was added (doesn't exist in source)
    Added,
    /// File was deleted (doesn't exist in destination)
    Deleted,
    /// File was modified (exists in both but content differs)
    Modified,
    /// Files are identical
    Unchanged,
}

/// Result of comparing two files
#[derive(Debug, Clone)]
pub struct FileDiff {
    /// Path being compared
    pub path: PathBuf,
    /// Type of change
    pub change_type: ChangeType,
    /// Unified diff output (None for binary files or unchanged)
    pub diff: Option<String>,
}

impl FileDiff {
    /// Generate a diff between two files
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to source file (e.g., global ~/.claude)
    /// * `dest_path` - Path to destination file (e.g., local ./.claude)
    /// * `relative_path` - Relative path for display in diff
    ///
    /// # Returns
    ///
    /// FileDiff showing the type of change and unified diff
    pub fn compare_files(
        source_path: &Path,
        dest_path: &Path,
        relative_path: &Path,
    ) -> Result<Self, DiffError> {
        let source_exists = source_path.exists();
        let dest_exists = dest_path.exists();

        // Handle existence cases
        match (source_exists, dest_exists) {
            (false, false) => {
                // Neither exists - no change
                Ok(Self {
                    path: relative_path.to_path_buf(),
                    change_type: ChangeType::Unchanged,
                    diff: None,
                })
            }
            (true, false) => {
                // Source exists, dest doesn't - file to be added
                let content = fs::read_to_string(source_path).map_err(|e| {
                    DiffError::ReadError {
                        path: source_path.to_path_buf(),
                        source: e,
                    }
                })?;

                Ok(Self {
                    path: relative_path.to_path_buf(),
                    change_type: ChangeType::Added,
                    diff: Some(Self::format_added(&content, relative_path)),
                })
            }
            (false, true) => {
                // Dest exists, source doesn't - file to be deleted
                let content = fs::read_to_string(dest_path).map_err(|e| DiffError::ReadError {
                    path: dest_path.to_path_buf(),
                    source: e,
                })?;

                Ok(Self {
                    path: relative_path.to_path_buf(),
                    change_type: ChangeType::Deleted,
                    diff: Some(Self::format_deleted(&content, relative_path)),
                })
            }
            (true, true) => {
                // Both exist - compare content
                Self::compare_existing_files(source_path, dest_path, relative_path)
            }
        }
    }

    /// Compare two existing files
    fn compare_existing_files(
        source_path: &Path,
        dest_path: &Path,
        relative_path: &Path,
    ) -> Result<Self, DiffError> {
        // Read both files
        let source_content = fs::read_to_string(source_path).map_err(|e| {
            DiffError::ReadError {
                path: source_path.to_path_buf(),
                source: e,
            }
        })?;

        let dest_content = fs::read_to_string(dest_path).map_err(|e| DiffError::ReadError {
            path: dest_path.to_path_buf(),
            source: e,
        })?;

        // Check if content is identical
        if source_content == dest_content {
            return Ok(Self {
                path: relative_path.to_path_buf(),
                change_type: ChangeType::Unchanged,
                diff: None,
            });
        }

        // Generate unified diff
        let diff = TextDiff::from_lines(&dest_content, &source_content);
        let unified_diff = Self::format_unified_diff(&diff, relative_path);

        Ok(Self {
            path: relative_path.to_path_buf(),
            change_type: ChangeType::Modified,
            diff: Some(unified_diff),
        })
    }

    /// Format unified diff output
    fn format_unified_diff(diff: &TextDiff<'_, '_, '_, str>, path: &Path) -> String {
        let mut output = String::new();

        // Add header
        output.push_str(&format!("--- a/{}\n", path.display()));
        output.push_str(&format!("+++ b/{}\n", path.display()));

        // Add hunks
        for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
            if idx > 0 {
                output.push('\n');
            }

            let old_start = group[0].old_range().start;
            let new_start = group[0].new_range().start;

            output.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                old_start + 1,
                group.iter().map(|op| op.old_range().len()).sum::<usize>(),
                new_start + 1,
                group.iter().map(|op| op.new_range().len()).sum::<usize>(),
            ));

            for op in group {
                for change in diff.iter_changes(op) {
                    let prefix = match change.tag() {
                        ChangeTag::Delete => "-",
                        ChangeTag::Insert => "+",
                        ChangeTag::Equal => " ",
                    };
                    output.push_str(&format!("{}{}", prefix, change));
                }
            }
        }

        output
    }

    /// Format added file in standard unified diff format
    fn format_added(content: &str, path: &Path) -> String {
        let mut output = String::new();
        let line_count = content.lines().count();

        output.push_str("--- /dev/null\n");
        output.push_str(&format!("+++ b/{}\n", path.display()));
        output.push_str(&format!("@@ -0,0 +1,{} @@\n", line_count));

        for line in content.lines() {
            output.push_str(&format!("+{}\n", line));
        }
        output
    }

    /// Format deleted file in standard unified diff format
    fn format_deleted(content: &str, path: &Path) -> String {
        let mut output = String::new();
        let line_count = content.lines().count();

        output.push_str(&format!("--- a/{}\n", path.display()));
        output.push_str("+++ /dev/null\n");
        output.push_str(&format!("@@ -1,{} +0,0 @@\n", line_count));

        for line in content.lines() {
            output.push_str(&format!("-{}\n", line));
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_compare_both_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        let diff = FileDiff::compare_files(&source, &dest, Path::new("test.txt")).unwrap();

        assert_eq!(diff.change_type, ChangeType::Unchanged);
        assert!(diff.diff.is_none());
    }

    #[test]
    fn test_compare_added_file() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        fs::write(&source, "new content\n").unwrap();

        let diff = FileDiff::compare_files(&source, &dest, Path::new("test.txt")).unwrap();

        assert_eq!(diff.change_type, ChangeType::Added);
        assert!(diff.diff.is_some());
        let diff_text = diff.diff.unwrap();
        assert!(diff_text.contains("--- /dev/null"));
        assert!(diff_text.contains("+++ b/test.txt"));
        assert!(diff_text.contains("@@ -0,0 +1,"));
        assert!(diff_text.contains("+new content"));
    }

    #[test]
    fn test_compare_deleted_file() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        fs::write(&dest, "old content\n").unwrap();

        let diff = FileDiff::compare_files(&source, &dest, Path::new("test.txt")).unwrap();

        assert_eq!(diff.change_type, ChangeType::Deleted);
        assert!(diff.diff.is_some());
        let diff_text = diff.diff.unwrap();
        assert!(diff_text.contains("--- a/test.txt"));
        assert!(diff_text.contains("+++ /dev/null"));
        assert!(diff_text.contains("@@ -1,"));
        assert!(diff_text.contains("-old content"));
    }

    #[test]
    fn test_compare_identical_files() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        fs::write(&source, "same content\n").unwrap();
        fs::write(&dest, "same content\n").unwrap();

        let diff = FileDiff::compare_files(&source, &dest, Path::new("test.txt")).unwrap();

        assert_eq!(diff.change_type, ChangeType::Unchanged);
        assert!(diff.diff.is_none());
    }

    #[test]
    fn test_compare_modified_file() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        fs::write(&dest, "line 1\nline 2\nline 3\n").unwrap();
        fs::write(&source, "line 1\nmodified line 2\nline 3\n").unwrap();

        let diff = FileDiff::compare_files(&source, &dest, Path::new("test.txt")).unwrap();

        assert_eq!(diff.change_type, ChangeType::Modified);
        assert!(diff.diff.is_some());
        let diff_text = diff.diff.unwrap();
        assert!(diff_text.contains("@@ "));
        assert!(diff_text.contains("-line 2"));
        assert!(diff_text.contains("+modified line 2"));
    }

    #[test]
    fn test_unified_diff_format() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        fs::write(&dest, "old line\n").unwrap();
        fs::write(&source, "new line\n").unwrap();

        let diff = FileDiff::compare_files(&source, &dest, Path::new("file.txt")).unwrap();

        assert_eq!(diff.change_type, ChangeType::Modified);
        let diff_text = diff.diff.unwrap();

        // Check unified diff format
        assert!(diff_text.contains("--- a/file.txt"));
        assert!(diff_text.contains("+++ b/file.txt"));
        assert!(diff_text.contains("@@"));
    }

    #[test]
    fn test_multiline_diff() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        let old_content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
        let new_content = "line 1\nmodified 2\nline 3\nnew line 4\nline 5\n";

        fs::write(&dest, old_content).unwrap();
        fs::write(&source, new_content).unwrap();

        let diff = FileDiff::compare_files(&source, &dest, Path::new("test.txt")).unwrap();

        assert_eq!(diff.change_type, ChangeType::Modified);
        let diff_text = diff.diff.unwrap();

        assert!(diff_text.contains("-line 2"));
        assert!(diff_text.contains("+modified 2"));
        assert!(diff_text.contains("-line 4"));
        assert!(diff_text.contains("+new line 4"));
    }

    #[test]
    fn test_empty_file_diff() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        fs::write(&source, "").unwrap();
        fs::write(&dest, "").unwrap();

        let diff = FileDiff::compare_files(&source, &dest, Path::new("test.txt")).unwrap();

        assert_eq!(diff.change_type, ChangeType::Unchanged);
        assert!(diff.diff.is_none());
    }

    #[test]
    fn test_added_to_empty() {
        let temp_dir = TempDir::new().unwrap();
        let source = temp_dir.path().join("source.txt");
        let dest = temp_dir.path().join("dest.txt");

        fs::write(&dest, "").unwrap();
        fs::write(&source, "new content\n").unwrap();

        let diff = FileDiff::compare_files(&source, &dest, Path::new("test.txt")).unwrap();

        assert_eq!(diff.change_type, ChangeType::Modified);
        assert!(diff.diff.is_some());
    }
}
