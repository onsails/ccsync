//! Diff generation with color-coded output

use std::fmt::Write;
use std::fs;
use std::path::Path;

use anyhow::Context;
use similar::{ChangeTag, TextDiff};

use crate::error::Result;

/// Diff generator for creating visual diffs
pub struct DiffGenerator;

impl Default for DiffGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffGenerator {
    /// Create a new diff generator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generate a color-coded unified diff between two files
    ///
    /// # Errors
    ///
    /// Returns an error if files cannot be read.
    pub fn generate(source: &Path, destination: &Path) -> Result<String> {
        let source_content = fs::read_to_string(source)
            .with_context(|| format!("Failed to read source file: {}", source.display()))?;

        let dest_content = fs::read_to_string(destination).with_context(|| {
            format!("Failed to read destination file: {}", destination.display())
        })?;

        Ok(Self::generate_from_content(
            &source_content,
            &dest_content,
            source,
            destination,
        ))
    }

    /// Generate a diff from string contents
    #[must_use]
    pub fn generate_from_content(
        source_content: &str,
        dest_content: &str,
        source_path: &Path,
        dest_path: &Path,
    ) -> String {
        const DIFF_CONTEXT_LINES: usize = 3;

        let diff = TextDiff::from_lines(dest_content, source_content);

        let mut output = String::new();

        writeln!(output, "\x1b[1m--- {}\x1b[0m", dest_path.display())
            .expect("Writing to String should never fail");
        writeln!(output, "\x1b[1m+++ {}\x1b[0m", source_path.display())
            .expect("Writing to String should never fail");

        for (idx, group) in diff.grouped_ops(DIFF_CONTEXT_LINES).iter().enumerate() {
            if idx > 0 {
                output.push_str("...\n");
            }

            for op in group {
                for change in diff.iter_changes(op) {
                    let (sign, color) = match change.tag() {
                        ChangeTag::Delete => ("-", "\x1b[31m"), // Red
                        ChangeTag::Insert => ("+", "\x1b[32m"), // Green
                        ChangeTag::Equal => (" ", "\x1b[0m"),   // No color
                    };

                    let newline = if change.value().ends_with('\n') {
                        ""
                    } else {
                        "\n"
                    };

                    write!(output, "{color}{sign}{}{newline}\x1b[0m", change.value())
                        .expect("Writing to String should never fail");
                }
            }
        }

        output
    }

    /// Generate a simple line-by-line diff without colors (for testing)
    ///
    /// # Errors
    ///
    /// Returns an error if files cannot be read.
    pub fn generate_plain(source: &Path, destination: &Path) -> Result<String> {
        let source_content = fs::read_to_string(source)
            .with_context(|| format!("Failed to read source file: {}", source.display()))?;

        let dest_content = fs::read_to_string(destination).with_context(|| {
            format!("Failed to read destination file: {}", destination.display())
        })?;

        let diff = TextDiff::from_lines(&dest_content, &source_content);
        let mut output = String::new();

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };

            write!(output, "{sign}{}", change.value())
                .expect("Writing to String should never fail");
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_diff_identical_files() {
        let tmp = TempDir::new().unwrap();
        let file1 = tmp.path().join("file1.txt");
        let file2 = tmp.path().join("file2.txt");

        let content = "line 1\nline 2\nline 3\n";
        fs::write(&file1, content).unwrap();
        fs::write(&file2, content).unwrap();

        let _generator = DiffGenerator::new();
        let diff = DiffGenerator::generate_plain(&file1, &file2).unwrap();

        // All lines should be equal (prefixed with space)
        assert!(diff.lines().all(|line| line.starts_with(' ')));
    }

    #[test]
    fn test_diff_different_files() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.txt");
        let dest = tmp.path().join("dest.txt");

        fs::write(&dest, "line 1\nline 2\nline 3\n").unwrap();
        fs::write(&source, "line 1\nmodified line 2\nline 3\n").unwrap();

        let _generator = DiffGenerator::new();
        let diff = DiffGenerator::generate_plain(&source, &dest).unwrap();

        // Should contain deletions and insertions
        assert!(diff.contains("-line 2"));
        assert!(diff.contains("+modified line 2"));
    }

    #[test]
    fn test_diff_with_colors() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.txt");
        let dest = tmp.path().join("dest.txt");

        fs::write(&dest, "old line\n").unwrap();
        fs::write(&source, "new line\n").unwrap();

        let _generator = DiffGenerator::new();
        let diff = DiffGenerator::generate(&source, &dest).unwrap();

        // Should contain ANSI color codes
        assert!(diff.contains("\x1b[31m")); // Red for deletions
        assert!(diff.contains("\x1b[32m")); // Green for insertions
        assert!(diff.contains("\x1b[0m")); // Reset
    }

    #[test]
    fn test_diff_added_lines() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.txt");
        let dest = tmp.path().join("dest.txt");

        fs::write(&dest, "line 1\n").unwrap();
        fs::write(&source, "line 1\nline 2\nline 3\n").unwrap();

        let _generator = DiffGenerator::new();
        let diff = DiffGenerator::generate_plain(&source, &dest).unwrap();

        assert!(diff.contains("+line 2"));
        assert!(diff.contains("+line 3"));
    }

    #[test]
    fn test_diff_removed_lines() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.txt");
        let dest = tmp.path().join("dest.txt");

        fs::write(&dest, "line 1\nline 2\nline 3\n").unwrap();
        fs::write(&source, "line 1\n").unwrap();

        let _generator = DiffGenerator::new();
        let diff = DiffGenerator::generate_plain(&source, &dest).unwrap();

        assert!(diff.contains("-line 2"));
        assert!(diff.contains("-line 3"));
    }

    #[test]
    fn test_diff_unicode_content() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.txt");
        let dest = tmp.path().join("dest.txt");

        fs::write(&dest, "Hello 世界\n").unwrap();
        fs::write(&source, "Hello World\n").unwrap();

        let _generator = DiffGenerator::new();
        let diff = DiffGenerator::generate_plain(&source, &dest);

        assert!(diff.is_ok());
    }

    #[test]
    fn test_diff_empty_files() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.txt");
        let dest = tmp.path().join("dest.txt");

        fs::write(&source, "").unwrap();
        fs::write(&dest, "").unwrap();

        let _generator = DiffGenerator::new();
        let diff = DiffGenerator::generate_plain(&source, &dest).unwrap();

        // Empty files should have empty diff
        assert!(diff.is_empty() || diff.trim().is_empty());
    }
}
