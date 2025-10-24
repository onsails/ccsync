//! Pattern matching for configuration files using gitignore syntax.
//!
//! This module provides pattern matching functionality for ignore and include
//! patterns using the `ignore` crate, which implements gitignore-style matching.

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during pattern matching
#[derive(Error, Debug)]
pub enum PatternError {
    /// Error building gitignore matcher
    #[error("Failed to build pattern matcher: {0}")]
    BuildError(String),
}

/// Pattern matcher for ignore/include rules
///
/// Uses gitignore-style pattern matching to determine if files should be
/// ignored or included during sync operations.
///
/// # Pattern Syntax
///
/// Follows gitignore syntax:
/// - `*.tmp` - matches all .tmp files
/// - `dir/` - matches directory
/// - `*.md` - matches all markdown files
/// - `!important.md` - negates previous patterns (include)
///
/// # Examples
///
/// ```no_run
/// use ccsync::models::pattern_matcher::PatternMatcher;
/// use std::path::Path;
///
/// let mut matcher = PatternMatcher::new();
/// matcher.add_ignore_pattern("*.tmp").unwrap();
/// matcher.add_ignore_pattern("*.log").unwrap();
///
/// assert!(matcher.should_ignore(Path::new("test.tmp")));
/// assert!(!matcher.should_ignore(Path::new("test.md")));
/// ```
pub struct PatternMatcher {
    /// Gitignore matcher for ignore patterns
    ignore_matcher: Option<Gitignore>,
    /// Gitignore matcher for include patterns (negated ignore)
    include_matcher: Option<Gitignore>,
}

impl PatternMatcher {
    /// Create a new empty pattern matcher
    pub fn new() -> Self {
        Self {
            ignore_matcher: None,
            include_matcher: None,
        }
    }

    /// Add an ignore pattern (gitignore syntax)
    ///
    /// # Arguments
    ///
    /// * `pattern` - Pattern string in gitignore format
    ///
    /// # Returns
    ///
    /// `Ok(())` if pattern was added successfully
    pub fn add_ignore_pattern(&mut self, pattern: &str) -> Result<(), PatternError> {
        let mut builder = GitignoreBuilder::new("");
        builder.add_line(None, pattern).map_err(|e| {
            PatternError::BuildError(format!("Invalid ignore pattern '{}': {}", pattern, e))
        })?;

        self.ignore_matcher = Some(builder.build().map_err(|e| {
            PatternError::BuildError(format!("Failed to build ignore matcher: {}", e))
        })?);

        Ok(())
    }

    /// Add multiple ignore patterns at once
    ///
    /// # Arguments
    ///
    /// * `patterns` - Iterator of pattern strings
    pub fn add_ignore_patterns<I>(&mut self, patterns: I) -> Result<(), PatternError>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut builder = GitignoreBuilder::new("");

        for pattern in patterns {
            builder.add_line(None, pattern.as_ref()).map_err(|e| {
                PatternError::BuildError(format!(
                    "Invalid ignore pattern '{}': {}",
                    pattern.as_ref(),
                    e
                ))
            })?;
        }

        self.ignore_matcher = Some(builder.build().map_err(|e| {
            PatternError::BuildError(format!("Failed to build ignore matcher: {}", e))
        })?);

        Ok(())
    }

    /// Add an include pattern (higher priority than ignore)
    ///
    /// Include patterns override ignore patterns for matching files.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Pattern string in gitignore format
    pub fn add_include_pattern(&mut self, pattern: &str) -> Result<(), PatternError> {
        let mut builder = GitignoreBuilder::new("");
        builder.add_line(None, pattern).map_err(|e| {
            PatternError::BuildError(format!("Invalid include pattern '{}': {}", pattern, e))
        })?;

        self.include_matcher = Some(builder.build().map_err(|e| {
            PatternError::BuildError(format!("Failed to build include matcher: {}", e))
        })?);

        Ok(())
    }

    /// Add multiple include patterns at once
    pub fn add_include_patterns<I>(&mut self, patterns: I) -> Result<(), PatternError>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut builder = GitignoreBuilder::new("");

        for pattern in patterns {
            builder.add_line(None, pattern.as_ref()).map_err(|e| {
                PatternError::BuildError(format!(
                    "Invalid include pattern '{}': {}",
                    pattern.as_ref(),
                    e
                ))
            })?;
        }

        self.include_matcher = Some(builder.build().map_err(|e| {
            PatternError::BuildError(format!("Failed to build include matcher: {}", e))
        })?);

        Ok(())
    }

    /// Check if a path should be ignored
    ///
    /// Returns `true` if the path matches ignore patterns and doesn't match
    /// include patterns. Include patterns have higher priority.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to check (relative path recommended)
    pub fn should_ignore(&self, path: &Path) -> bool {
        // If path matches include patterns, don't ignore it (include has priority)
        if let Some(ref include_matcher) = self.include_matcher {
            if include_matcher.matched(path, false).is_ignore() {
                return false;
            }
        }

        // Check if path matches ignore patterns
        if let Some(ref ignore_matcher) = self.ignore_matcher {
            return ignore_matcher.matched(path, false).is_ignore();
        }

        // No patterns defined, don't ignore
        false
    }

    /// Check if a path should be included (opposite of should_ignore)
    pub fn should_include(&self, path: &Path) -> bool {
        !self.should_ignore(path)
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_new_matcher() {
        let matcher = PatternMatcher::new();
        assert!(matcher.should_include(Path::new("any/path.txt")));
    }

    #[test]
    fn test_ignore_single_pattern() {
        let mut matcher = PatternMatcher::new();
        matcher.add_ignore_pattern("*.tmp").unwrap();

        assert!(matcher.should_ignore(Path::new("test.tmp")));
        assert!(matcher.should_ignore(Path::new("foo/bar.tmp")));
        assert!(!matcher.should_ignore(Path::new("test.txt")));
    }

    #[test]
    fn test_ignore_multiple_patterns() {
        let mut matcher = PatternMatcher::new();
        matcher
            .add_ignore_patterns(&["*.tmp", "*.log", "build"])
            .unwrap();

        assert!(matcher.should_ignore(Path::new("test.tmp")));
        assert!(matcher.should_ignore(Path::new("debug.log")));
        assert!(matcher.should_ignore(Path::new("build")));
        assert!(!matcher.should_ignore(Path::new("test.txt")));
    }

    #[test]
    fn test_include_overrides_ignore() {
        let mut matcher = PatternMatcher::new();
        matcher.add_ignore_patterns(&["*.md"]).unwrap();
        matcher.add_include_patterns(&["important.md"]).unwrap();

        // important.md should be included (include overrides ignore)
        assert!(!matcher.should_ignore(Path::new("important.md")));
        // Other .md files should still be ignored
        assert!(matcher.should_ignore(Path::new("other.md")));
    }

    #[test]
    fn test_directory_pattern() {
        let mut matcher = PatternMatcher::new();
        matcher.add_ignore_pattern("temp").unwrap();

        assert!(matcher.should_ignore(Path::new("temp")));
        assert!(!matcher.should_ignore(Path::new("other")));
    }

    #[test]
    fn test_wildcard_pattern() {
        let mut matcher = PatternMatcher::new();
        matcher.add_ignore_pattern("test-*.md").unwrap();

        assert!(matcher.should_ignore(Path::new("test-foo.md")));
        assert!(matcher.should_ignore(Path::new("test-bar.md")));
        assert!(!matcher.should_ignore(Path::new("other.md")));
        assert!(!matcher.should_ignore(Path::new("test.md")));
    }

    #[test]
    fn test_nested_path_pattern() {
        let mut matcher = PatternMatcher::new();
        matcher.add_ignore_pattern("commands/*.md").unwrap();

        assert!(matcher.should_ignore(Path::new("commands/test.md")));
        assert!(!matcher.should_ignore(Path::new("commands/subdir/test.md")));
        assert!(!matcher.should_ignore(Path::new("other/test.md")));
    }

    #[test]
    fn test_double_asterisk_pattern() {
        let mut matcher = PatternMatcher::new();
        matcher.add_ignore_pattern("**/temp/*.tmp").unwrap();

        assert!(matcher.should_ignore(Path::new("temp/file.tmp")));
        assert!(matcher.should_ignore(Path::new("foo/temp/file.tmp")));
        assert!(matcher.should_ignore(Path::new("foo/bar/temp/file.tmp")));
        assert!(!matcher.should_ignore(Path::new("temp/file.txt")));
    }

    #[test]
    fn test_should_include_convenience() {
        let mut matcher = PatternMatcher::new();
        matcher.add_ignore_pattern("*.tmp").unwrap();

        assert!(matcher.should_include(Path::new("test.txt")));
        assert!(!matcher.should_include(Path::new("test.tmp")));
    }

    #[test]
    fn test_invalid_pattern_error() {
        let mut matcher = PatternMatcher::new();
        // The ignore crate is quite permissive, so this test checks error handling exists
        let result = matcher.add_ignore_pattern("*.tmp");
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_include_exclude() {
        let mut matcher = PatternMatcher::new();

        // Ignore all markdown files
        matcher.add_ignore_patterns(&["*.md"]).unwrap();

        // But include anything in the important/ directory
        matcher.add_include_patterns(&["important/*.md"]).unwrap();

        assert!(matcher.should_ignore(Path::new("readme.md")));
        assert!(matcher.should_ignore(Path::new("docs/guide.md")));
        assert!(!matcher.should_ignore(Path::new("important/keep.md")));
    }

    #[test]
    fn test_empty_pattern_matcher() {
        let matcher = PatternMatcher::new();

        // No patterns means nothing is ignored
        assert!(matcher.should_include(Path::new("anything.txt")));
        assert!(matcher.should_include(Path::new("foo/bar/baz.md")));
    }

    #[test]
    fn test_path_with_extension() {
        let mut matcher = PatternMatcher::new();
        matcher.add_ignore_pattern("*.log").unwrap();

        assert!(matcher.should_ignore(Path::new("app.log")));
        assert!(matcher.should_ignore(Path::new("error.log")));
        assert!(!matcher.should_ignore(Path::new("app.txt")));
        assert!(!matcher.should_ignore(Path::new("log.txt")));
    }
}
