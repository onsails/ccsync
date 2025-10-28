//! Gitignore-style pattern matching using the ignore crate

use std::path::Path;

use anyhow::Context;
use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::error::Result;

/// Pattern matcher for file inclusion/exclusion
pub struct PatternMatcher {
    gitignore: Option<Gitignore>,
}

impl PatternMatcher {
    /// Create a new pattern matcher
    #[must_use]
    pub const fn new() -> Self {
        Self { gitignore: None }
    }

    /// Build pattern matcher from ignore and include patterns
    ///
    /// # Errors
    ///
    /// Returns an error if patterns are invalid.
    pub fn with_patterns(ignore_patterns: &[String], include_patterns: &[String]) -> Result<Self> {
        let mut builder = GitignoreBuilder::new("");

        // Add ignore patterns
        for pattern in ignore_patterns {
            builder
                .add_line(None, pattern)
                .with_context(|| format!("Invalid ignore pattern: '{pattern}'"))?;
        }

        // Add include patterns (negated ignores)
        for pattern in include_patterns {
            builder
                .add_line(None, &format!("!{pattern}"))
                .with_context(|| format!("Invalid include pattern: '{pattern}'"))?;
        }

        let gitignore = builder.build()?;

        Ok(Self {
            gitignore: Some(gitignore),
        })
    }

    /// Check if a path should be included based on patterns
    #[must_use]
    pub fn should_include(&self, path: &Path, is_dir: bool) -> bool {
        self.gitignore
            .as_ref()
            .is_none_or(|gi| !gi.matched(path, is_dir).is_ignore())
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
    fn test_no_patterns() {
        let matcher = PatternMatcher::new();
        assert!(matcher.should_include(&PathBuf::from("any/file.txt"), false));
    }

    #[test]
    fn test_ignore_pattern() {
        let matcher = PatternMatcher::with_patterns(
            &["*.tmp".to_string()],
            &[],
        )
        .unwrap();

        assert!(!matcher.should_include(&PathBuf::from("file.tmp"), false));
        assert!(matcher.should_include(&PathBuf::from("file.txt"), false));
    }

    #[test]
    fn test_include_overrides_ignore() {
        let matcher = PatternMatcher::with_patterns(
            &["*.tmp".to_string()],
            &["important.tmp".to_string()],
        )
        .unwrap();

        assert!(!matcher.should_include(&PathBuf::from("file.tmp"), false));
        assert!(matcher.should_include(&PathBuf::from("important.tmp"), false));
    }

    #[test]
    fn test_directory_patterns() {
        let matcher = PatternMatcher::with_patterns(
            &["node_modules/".to_string()],
            &[],
        )
        .unwrap();

        assert!(!matcher.should_include(&PathBuf::from("node_modules"), true));
        assert!(matcher.should_include(&PathBuf::from("src"), true));
    }
}
