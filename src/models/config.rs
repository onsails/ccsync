//! Configuration file data structures and parsing logic.
//!
//! This module defines the configuration schema for ccsync, supporting
//! YAML format (.ccsync.yaml). Configuration files can specify ignore
//! patterns, type filters, and sync behavior.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Configuration error types
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Error reading configuration file
    #[error("Failed to read config file {path}: {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Error parsing YAML configuration
    #[error("Failed to parse YAML config file {path}: {source}")]
    YamlParseError {
        path: PathBuf,
        source: serde_yml::Error,
    },

    /// Invalid configuration structure
    #[error("Invalid configuration in {path}: {message}")]
    InvalidConfig { path: PathBuf, message: String },

    /// Configuration file too large
    #[error("Config file {path} exceeds maximum size of {max_size} bytes (actual: {actual_size} bytes)")]
    FileTooLarge {
        path: PathBuf,
        max_size: u64,
        actual_size: u64,
    },
}

/// Main configuration structure that can be loaded from YAML files.
///
/// Configuration files support multiple locations with precedence:
/// 1. CLI flags (highest)
/// 2. .ccsync.local.yaml (project directory, not version controlled)
/// 3. .ccsync.yaml (project directory, version controlled)
/// 4. $XDG_CONFIG_HOME/ccsync/config.yaml (or ~/.config/ccsync/config.yaml)
///
/// Example YAML format:
/// ```yaml
/// to_local:
///   ignore:
///     - "commands/personal-*.md"
///     - "agents/"
///   types:
///     - commands
///     - skills
///
/// to_global:
///   ignore:
///     - "commands/team-*.md"
///   types:
///     - all
///
/// conflict_strategy: skip
/// non_interactive: false
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Configuration for syncing from global to local (to-local command)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_local: Option<DirectionConfig>,

    /// Configuration for syncing from local to global (to-global command)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_global: Option<DirectionConfig>,

    /// Default conflict resolution strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_strategy: Option<ConflictStrategy>,

    /// Whether to run in non-interactive mode by default
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_interactive: Option<bool>,

    /// Whether to preserve symlinks instead of following them
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_symlinks: Option<bool>,
}

/// Configuration specific to a sync direction (to-local or to-global)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct DirectionConfig {
    /// Patterns to ignore during sync (gitignore syntax)
    /// Examples:
    /// - "commands/personal-*.md" - ignore specific files
    /// - "agents/" - ignore directory
    /// - "*.secret" - ignore all files with .secret extension
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,

    /// Patterns to explicitly include (overrides ignore patterns)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,

    /// Types to sync by default (commands, skills, subagents, all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<ConfigType>>,
}

/// Configuration type filter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ConfigType {
    /// Slash commands
    Commands,
    /// Skills
    Skills,
    /// Subagents
    Subagents,
    /// All configuration types
    All,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    /// Fail on conflicts (default)
    Fail,
    /// Overwrite destination with source
    Overwrite,
    /// Skip conflicting files
    Skip,
    /// Use newer file based on modification time
    Newer,
}

impl Config {
    /// Maximum allowed configuration file size (1 MB)
    const MAX_CONFIG_SIZE: u64 = 1024 * 1024;

    /// Parse configuration from a YAML file.
    ///
    /// This method includes security checks:
    /// - File size validation (max 1 MB)
    /// - Automatic validation of parsed configuration
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .yaml configuration file
    ///
    /// # Returns
    ///
    /// Parsed and validated configuration or an error if parsing/validation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ccsync::models::config::Config;
    /// use std::path::Path;
    ///
    /// let config = Config::from_file(Path::new(".ccsync.yaml"))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        // Check file size before reading to prevent OOM
        let metadata = std::fs::metadata(path).map_err(|e| ConfigError::ReadError {
            path: path.to_path_buf(),
            source: e,
        })?;

        let file_size = metadata.len();
        if file_size > Self::MAX_CONFIG_SIZE {
            return Err(ConfigError::FileTooLarge {
                path: path.to_path_buf(),
                max_size: Self::MAX_CONFIG_SIZE,
                actual_size: file_size,
            });
        }

        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadError {
            path: path.to_path_buf(),
            source: e,
        })?;

        let config = Self::from_yaml(&content, path)?;

        // Automatically validate after parsing
        config.validate(path)?;

        Ok(config)
    }

    /// Parse configuration from a YAML string with a given path for error reporting.
    ///
    /// # Arguments
    ///
    /// * `content` - The YAML configuration content as a string
    /// * `path` - Path used for error reporting
    pub fn from_yaml(content: &str, path: &Path) -> Result<Self, ConfigError> {
        serde_yml::from_str(content).map_err(|e| ConfigError::YamlParseError {
            path: path.to_path_buf(),
            source: e,
        })
    }

    /// Validate the configuration for logical consistency
    ///
    /// Checks include:
    /// - Type conflicts (e.g., 'all' with other types)
    /// - Empty type arrays
    /// - Duplicate types
    /// - Path traversal in patterns
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, or an error describing the issue
    pub fn validate(&self, path: &Path) -> Result<(), ConfigError> {
        // Validate to_local configuration
        if let Some(ref to_local) = self.to_local {
            Self::validate_direction_config(to_local, "to_local", path)?;
        }

        // Validate to_global configuration
        if let Some(ref to_global) = self.to_global {
            Self::validate_direction_config(to_global, "to_global", path)?;
        }

        Ok(())
    }

    /// Helper function to validate a DirectionConfig
    fn validate_direction_config(
        config: &DirectionConfig,
        name: &str,
        path: &Path,
    ) -> Result<(), ConfigError> {
        // Validate types
        if let Some(ref types) = config.types {
            // Check for empty types array
            if types.is_empty() {
                return Err(ConfigError::InvalidConfig {
                    path: path.to_path_buf(),
                    message: format!("{}.types cannot be empty", name),
                });
            }

            // Check for 'all' with other types
            if types.contains(&ConfigType::All) && types.len() > 1 {
                return Err(ConfigError::InvalidConfig {
                    path: path.to_path_buf(),
                    message: format!("{}.types cannot contain 'all' with other types", name),
                });
            }

            // Check for duplicate types
            let unique_types: HashSet<_> = types.iter().collect();
            if unique_types.len() != types.len() {
                return Err(ConfigError::InvalidConfig {
                    path: path.to_path_buf(),
                    message: format!("{}.types contains duplicate values", name),
                });
            }
        }

        // Validate ignore patterns
        if let Some(ref ignore) = config.ignore {
            for pattern in ignore {
                Self::validate_pattern(pattern, path, &format!("{}.ignore", name))?;
            }
        }

        // Validate include patterns
        if let Some(ref include) = config.include {
            for pattern in include {
                Self::validate_pattern(pattern, path, &format!("{}.include", name))?;
            }
        }

        Ok(())
    }

    /// Validate a single pattern for security issues
    fn validate_pattern(pattern: &str, path: &Path, field: &str) -> Result<(), ConfigError> {
        // Check for empty patterns
        if pattern.trim().is_empty() {
            return Err(ConfigError::InvalidConfig {
                path: path.to_path_buf(),
                message: format!("{} contains empty pattern", field),
            });
        }

        // Check for absolute paths (Unix and Windows)
        if pattern.starts_with('/') || pattern.chars().nth(1) == Some(':') {
            return Err(ConfigError::InvalidConfig {
                path: path.to_path_buf(),
                message: format!(
                    "{} contains absolute path '{}' which is not allowed",
                    field, pattern
                ),
            });
        }

        // Check for parent directory traversal
        if pattern.contains("..") {
            return Err(ConfigError::InvalidConfig {
                path: path.to_path_buf(),
                message: format!(
                    "{} contains path traversal '..' in pattern '{}' which is not allowed",
                    field, pattern
                ),
            });
        }

        Ok(())
    }

    /// Serialize configuration to YAML format
    pub fn to_yaml(&self) -> Result<String, serde_yml::Error> {
        serde_yml::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_config() {
        let yaml = "";
        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        assert!(config.to_local.is_none());
        assert!(config.to_global.is_none());
    }

    #[test]
    fn test_basic_yaml_parsing() {
        let yaml = r#"
to_local:
  ignore:
    - "commands/personal-*.md"
    - "agents/"
  types:
    - commands
    - skills

to_global:
  ignore:
    - "commands/team-*.md"
  types:
    - all

conflict_strategy: skip
non_interactive: false
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();

        // Check to_local
        let to_local = config.to_local.unwrap();
        assert_eq!(to_local.ignore.as_ref().unwrap().len(), 2);
        assert_eq!(
            to_local.ignore.as_ref().unwrap()[0],
            "commands/personal-*.md"
        );
        assert_eq!(to_local.types.as_ref().unwrap().len(), 2);
        assert_eq!(to_local.types.as_ref().unwrap()[0], ConfigType::Commands);

        // Check to_global
        let to_global = config.to_global.unwrap();
        assert_eq!(to_global.ignore.as_ref().unwrap().len(), 1);
        assert_eq!(to_global.types.as_ref().unwrap()[0], ConfigType::All);

        // Check global settings
        assert_eq!(config.conflict_strategy, Some(ConflictStrategy::Skip));
        assert_eq!(config.non_interactive, Some(false));
    }

    #[test]
    fn test_minimal_config() {
        let yaml = r#"
to_local:
  ignore:
    - "*.secret"
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let to_local = config.to_local.unwrap();
        assert_eq!(to_local.ignore.as_ref().unwrap()[0], "*.secret");
        assert!(to_local.types.is_none());
        assert!(config.to_global.is_none());
    }

    #[test]
    fn test_invalid_yaml() {
        let yaml = r#"
to_local:
  ignore:
    - "test"
  invalid_field: true
"#;

        let result = Config::from_yaml(yaml, Path::new("test.yaml"));
        assert!(result.is_err());
        match result {
            Err(ConfigError::YamlParseError { .. }) => (),
            _ => panic!("Expected YamlParseError"),
        }
    }

    #[test]
    fn test_malformed_yaml() {
        let yaml = r#"
to_local:
  ignore:
    - "test
    - missing quote
"#;

        let result = Config::from_yaml(yaml, Path::new("test.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_conflicting_types() {
        let config = Config {
            to_local: Some(DirectionConfig {
                types: Some(vec![ConfigType::All, ConfigType::Commands]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let result = config.validate(Path::new("test.yaml"));
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidConfig { message, .. }) => {
                assert!(message.contains("cannot contain 'all' with other types"));
            }
            _ => panic!("Expected InvalidConfig error"),
        }
    }

    #[test]
    fn test_roundtrip_yaml() {
        let config = Config {
            to_local: Some(DirectionConfig {
                ignore: Some(vec!["*.test".to_string()]),
                types: Some(vec![ConfigType::Commands]),
                ..Default::default()
            }),
            conflict_strategy: Some(ConflictStrategy::Skip),
            ..Default::default()
        };

        let yaml = config.to_yaml().unwrap();
        let parsed = Config::from_yaml(&yaml, Path::new("test.yaml")).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn test_include_patterns() {
        let yaml = r#"
to_local:
  include:
    - "commands/important-*.md"
  ignore:
    - "commands/*.md"
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let to_local = config.to_local.unwrap();
        assert_eq!(to_local.include.as_ref().unwrap().len(), 1);
        assert_eq!(to_local.ignore.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_all_conflict_strategies() {
        let strategies = vec!["fail", "overwrite", "skip", "newer"];

        for strategy in strategies {
            let yaml = format!("conflict_strategy: {}", strategy);
            let config = Config::from_yaml(&yaml, Path::new("test.yaml")).unwrap();
            assert!(config.conflict_strategy.is_some());
        }
    }

    #[test]
    fn test_all_config_types() {
        let yaml = r#"
to_local:
  types:
    - commands
    - skills
    - subagents
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let types = config.to_local.unwrap().types.unwrap();
        assert_eq!(types.len(), 3);
        assert!(types.contains(&ConfigType::Commands));
        assert!(types.contains(&ConfigType::Skills));
        assert!(types.contains(&ConfigType::Subagents));
    }

    #[test]
    fn test_preserve_symlinks() {
        let yaml = "preserve_symlinks: true";
        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        assert_eq!(config.preserve_symlinks, Some(true));
    }

    #[test]
    fn test_comments_in_yaml() {
        let yaml = r#"
# This is a comment
to_local:
  # Ignore personal commands
  ignore:
    - "commands/personal-*.md"  # inline comment
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let to_local = config.to_local.unwrap();
        assert_eq!(to_local.ignore.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_empty_arrays() {
        let yaml = r#"
to_local:
  ignore: []
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let to_local = config.to_local.unwrap();
        assert_eq!(to_local.ignore.as_ref().unwrap().len(), 0);
    }

    // Security Tests

    #[test]
    fn test_validate_empty_types_array() {
        let yaml = r#"
to_local:
  types: []
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let result = config.validate(Path::new("test.yaml"));
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidConfig { message, .. }) => {
                assert!(message.contains("cannot be empty"));
            }
            _ => panic!("Expected InvalidConfig error for empty types"),
        }
    }

    #[test]
    fn test_validate_duplicate_types() {
        let yaml = r#"
to_local:
  types:
    - commands
    - skills
    - commands
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let result = config.validate(Path::new("test.yaml"));
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidConfig { message, .. }) => {
                assert!(message.contains("duplicate"));
            }
            _ => panic!("Expected InvalidConfig error for duplicate types"),
        }
    }

    #[test]
    fn test_validate_path_traversal_in_ignore() {
        let yaml = r#"
to_local:
  ignore:
    - "../etc/passwd"
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let result = config.validate(Path::new("test.yaml"));
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidConfig { message, .. }) => {
                assert!(message.contains("path traversal"));
            }
            _ => panic!("Expected InvalidConfig error for path traversal"),
        }
    }

    #[test]
    fn test_validate_absolute_path_unix() {
        let yaml = r#"
to_local:
  ignore:
    - "/etc/passwd"
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let result = config.validate(Path::new("test.yaml"));
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidConfig { message, .. }) => {
                assert!(message.contains("absolute path"));
            }
            _ => panic!("Expected InvalidConfig error for absolute path"),
        }
    }

    #[test]
    fn test_validate_absolute_path_windows() {
        let yaml = r#"
to_local:
  ignore:
    - "C:/Windows/System32"
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let result = config.validate(Path::new("test.yaml"));
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidConfig { message, .. }) => {
                assert!(message.contains("absolute path"));
            }
            _ => panic!("Expected InvalidConfig error for Windows absolute path"),
        }
    }

    #[test]
    fn test_validate_empty_pattern() {
        let yaml = r#"
to_local:
  ignore:
    - "  "
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let result = config.validate(Path::new("test.yaml"));
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidConfig { message, .. }) => {
                assert!(message.contains("empty pattern"));
            }
            _ => panic!("Expected InvalidConfig error for empty pattern"),
        }
    }

    #[test]
    fn test_file_size_limit() {
        use tempfile::NamedTempFile;
        use std::io::Write;

        // Create a temporary file larger than MAX_CONFIG_SIZE
        let mut temp_file = NamedTempFile::new().unwrap();
        let large_content = "a".repeat((Config::MAX_CONFIG_SIZE + 1) as usize);
        temp_file.write_all(large_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let result = Config::from_file(temp_file.path());
        assert!(result.is_err());
        match result {
            Err(ConfigError::FileTooLarge { .. }) => (),
            _ => panic!("Expected FileTooLarge error"),
        }
    }

    #[test]
    fn test_from_file_auto_validates() {
        use tempfile::NamedTempFile;
        use std::io::Write;

        // Create a config with invalid content (empty types array)
        let mut temp_file = NamedTempFile::new().unwrap();
        let invalid_yaml = r#"
to_local:
  types: []
"#;
        temp_file.write_all(invalid_yaml.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // from_file should automatically validate and fail
        let result = Config::from_file(temp_file.path());
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidConfig { .. }) => (),
            _ => panic!("Expected InvalidConfig error from auto-validation"),
        }
    }

    #[test]
    fn test_from_file_success() {
        use tempfile::NamedTempFile;
        use std::io::Write;

        let mut temp_file = NamedTempFile::new().unwrap();
        let valid_yaml = r#"
to_local:
  ignore:
    - "*.tmp"
  types:
    - commands
"#;
        temp_file.write_all(valid_yaml.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let config = Config::from_file(temp_file.path()).unwrap();
        assert!(config.to_local.is_some());
    }

    #[test]
    fn test_validate_path_traversal_in_include() {
        let yaml = r#"
to_global:
  include:
    - "foo/../bar"
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        let result = config.validate(Path::new("test.yaml"));
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidConfig { message, .. }) => {
                assert!(message.contains("path traversal"));
            }
            _ => panic!("Expected path traversal error in include"),
        }
    }

    #[test]
    fn test_validate_valid_patterns() {
        let yaml = r#"
to_local:
  ignore:
    - "*.tmp"
    - "commands/test-*.md"
    - "agents/"
  types:
    - commands
    - skills
"#;

        let config = Config::from_yaml(yaml, Path::new("test.yaml")).unwrap();
        // Should not error
        config.validate(Path::new("test.yaml")).unwrap();
    }
}
