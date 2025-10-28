//! File filtering based on CLI arguments and configuration

use std::path::Path;

/// Pattern for matching file paths
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    /// Match files with a specific extension (e.g., "md")
    Extension(String),
    /// Match files with a specific name (e.g., "SKILL.md")
    FileName(String),
    /// Match files whose path contains a substring
    Contains(String),
}

impl Pattern {
    /// Check if this pattern matches the given path
    #[must_use]
    pub fn matches(&self, path: &Path) -> bool {
        match self {
            Self::Extension(ext) => path.extension().is_some_and(|e| e == ext.as_str()),
            Self::FileName(name) => path.file_name().is_some_and(|n| n == name.as_str()),
            Self::Contains(substring) => {
                // Check if the path contains the substring
                path.to_str().is_some_and(|s| s.contains(substring))
            }
        }
    }
}

/// File filter that combines CLI and config patterns
#[derive(Debug, Clone, Default)]
pub struct FileFilter {
    /// Patterns from CLI arguments (higher precedence)
    cli_patterns: Vec<Pattern>,
    /// Patterns from config file (lower precedence)
    config_patterns: Vec<Pattern>,
}

impl FileFilter {
    /// Create a new file filter
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add CLI patterns (higher precedence)
    #[must_use]
    pub fn with_cli_patterns(mut self, patterns: Vec<Pattern>) -> Self {
        self.cli_patterns = patterns;
        self
    }

    /// Add config patterns (lower precedence)
    #[must_use]
    pub fn with_config_patterns(mut self, patterns: Vec<Pattern>) -> Self {
        self.config_patterns = patterns;
        self
    }

    /// Check if a path should be included based on filters
    ///
    /// If CLI patterns are specified, they take precedence.
    /// If no patterns are specified, all files are included.
    #[must_use]
    pub fn should_include(&self, path: &Path) -> bool {
        // If no filters specified, include everything
        if self.cli_patterns.is_empty() && self.config_patterns.is_empty() {
            return true;
        }

        // CLI patterns take precedence if any exist
        if !self.cli_patterns.is_empty() {
            return self.cli_patterns.iter().any(|p| p.matches(path));
        }

        // Fall back to config patterns
        self.config_patterns.iter().any(|p| p.matches(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extension_pattern() {
        let pattern = Pattern::Extension("md".to_string());
        assert!(pattern.matches(&PathBuf::from("file.md")));
        assert!(!pattern.matches(&PathBuf::from("file.txt")));
    }

    #[test]
    fn test_filename_pattern() {
        let pattern = Pattern::FileName("SKILL.md".to_string());
        assert!(pattern.matches(&PathBuf::from("/path/to/SKILL.md")));
        assert!(!pattern.matches(&PathBuf::from("/path/to/OTHER.md")));
    }

    #[test]
    fn test_contains_pattern() {
        let pattern = Pattern::Contains("skill".to_string());
        assert!(pattern.matches(&PathBuf::from("/skills/my-skill/SKILL.md")));
        assert!(!pattern.matches(&PathBuf::from("/agents/agent.md")));
    }

    #[test]
    fn test_filter_no_patterns() {
        let filter = FileFilter::new();
        assert!(filter.should_include(&PathBuf::from("any-file.md")));
    }

    #[test]
    fn test_filter_cli_precedence() {
        let filter = FileFilter::new()
            .with_cli_patterns(vec![Pattern::Extension("md".to_string())])
            .with_config_patterns(vec![Pattern::Extension("txt".to_string())]);

        assert!(filter.should_include(&PathBuf::from("file.md")));
        assert!(!filter.should_include(&PathBuf::from("file.txt")));
    }

    #[test]
    fn test_filter_config_fallback() {
        let filter =
            FileFilter::new().with_config_patterns(vec![Pattern::Extension("md".to_string())]);

        assert!(filter.should_include(&PathBuf::from("file.md")));
        assert!(!filter.should_include(&PathBuf::from("file.txt")));
    }

    #[test]
    fn test_filter_multiple_patterns() {
        let filter = FileFilter::new().with_cli_patterns(vec![
            Pattern::Extension("md".to_string()),
            Pattern::FileName("README".to_string()),
        ]);

        assert!(filter.should_include(&PathBuf::from("file.md")));
        assert!(filter.should_include(&PathBuf::from("/path/README")));
        assert!(!filter.should_include(&PathBuf::from("file.txt")));
    }
}
