//! Configuration loading and merging logic.
//!
//! This module handles loading configuration files from multiple locations
//! with proper precedence and merging logic.

use super::config::{Config, ConfigError, DirectionConfig};
use std::env;
use std::path::PathBuf;

/// Configuration loader that handles multiple config file locations
/// and merges them with correct precedence.
///
/// Precedence order (highest to lowest):
/// 1. CLI flag (--config)
/// 2. .ccsync.local.yaml (project directory, gitignored)
/// 3. .ccsync.yaml (project directory, version controlled)
/// 4. $XDG_CONFIG_HOME/ccsync/config.yaml or ~/.config/ccsync/config.yaml
pub struct ConfigLoader {
    /// Project directory for local config files
    project_dir: Option<PathBuf>,
    /// Custom config file path from CLI
    custom_config: Option<PathBuf>,
}

impl ConfigLoader {
    /// Create a new configuration loader
    ///
    /// # Arguments
    ///
    /// * `project_dir` - Optional project directory for local configs
    /// * `custom_config` - Optional custom config file path from CLI
    pub fn new(project_dir: Option<PathBuf>, custom_config: Option<PathBuf>) -> Self {
        Self {
            project_dir,
            custom_config,
        }
    }

    /// Load and merge all configuration files with proper precedence
    ///
    /// Returns the merged configuration or an error if any config is invalid
    pub fn load_merged(&self) -> Result<Config, ConfigError> {
        let mut merged = Config::default();

        // Load in reverse precedence order (lowest to highest)
        // so higher precedence configs override lower ones

        // 1. Global config (lowest precedence)
        if let Some(global_config) = self.find_global_config() {
            if global_config.exists() {
                let config = Config::from_file(&global_config)?;
                merged = merged.merge(config);
            }
        }

        // 2. Project .ccsync.yaml
        if let Some(project_dir) = &self.project_dir {
            let project_config = project_dir.join(".ccsync.yaml");
            if project_config.exists() {
                let config = Config::from_file(&project_config)?;
                merged = merged.merge(config);
            }
        }

        // 3. Project .ccsync.local.yaml
        if let Some(project_dir) = &self.project_dir {
            let local_config = project_dir.join(".ccsync.local.yaml");
            if local_config.exists() {
                let config = Config::from_file(&local_config)?;
                merged = merged.merge(config);
            }
        }

        // 4. Custom config from CLI (highest precedence)
        if let Some(custom_path) = &self.custom_config {
            let config = Config::from_file(custom_path)?;
            merged = merged.merge(config);
        }

        Ok(merged)
    }

    /// Find the global configuration file path
    ///
    /// Uses XDG Base Directory specification for cross-platform config location.
    /// Returns path regardless of whether file exists (caller handles non-existent files).
    ///
    /// Checks in order:
    /// 1. $XDG_CONFIG_HOME/ccsync/config.yaml
    /// 2. ~/.config/ccsync/config.yaml (via dirs::config_dir)
    fn find_global_config(&self) -> Option<PathBuf> {
        // Try XDG_CONFIG_HOME first
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg_config).join("ccsync").join("config.yaml"));
        }

        // Fall back to standard config directory (uses dirs::config_dir which is secure)
        dirs::config_dir().map(|config_dir| config_dir.join("ccsync").join("config.yaml"))
    }
}

impl Config {
    /// Merge another configuration into this one
    ///
    /// The `other` configuration takes precedence for all non-None values.
    /// For DirectionConfig, merges are performed at the field level.
    ///
    /// # Arguments
    ///
    /// * `other` - Configuration to merge (higher precedence)
    ///
    /// # Returns
    ///
    /// New merged configuration
    pub fn merge(self, other: Config) -> Config {
        Config {
            to_local: Self::merge_direction_config(self.to_local, other.to_local),
            to_global: Self::merge_direction_config(self.to_global, other.to_global),
            conflict_strategy: other.conflict_strategy.or(self.conflict_strategy),
            non_interactive: other.non_interactive.or(self.non_interactive),
            preserve_symlinks: other.preserve_symlinks.or(self.preserve_symlinks),
        }
    }

    /// Merge two DirectionConfig options
    ///
    /// Performs field-level merging for nested structures.
    /// For Vec fields (ignore, include, types), the higher precedence value replaces the lower.
    fn merge_direction_config(
        lower: Option<DirectionConfig>,
        higher: Option<DirectionConfig>,
    ) -> Option<DirectionConfig> {
        match (lower, higher) {
            (None, None) => None,
            (Some(config), None) => Some(config),
            (None, Some(config)) => Some(config),
            (Some(lower_config), Some(higher_config)) => {
                // Higher precedence fields override lower precedence
                let merged = DirectionConfig {
                    ignore: higher_config.ignore.or(lower_config.ignore),
                    include: higher_config.include.or(lower_config.include),
                    types: higher_config.types.or(lower_config.types),
                };
                Some(merged)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::{ConfigType, ConflictStrategy};

    #[test]
    fn test_merge_empty_configs() {
        let config1 = Config::default();
        let config2 = Config::default();
        let merged = config1.merge(config2);

        assert!(merged.to_local.is_none());
        assert!(merged.to_global.is_none());
        assert!(merged.conflict_strategy.is_none());
    }

    #[test]
    fn test_merge_boolean_override() {
        let config1 = Config {
            non_interactive: Some(false),
            ..Default::default()
        };
        let config2 = Config {
            non_interactive: Some(true),
            ..Default::default()
        };
        let merged = config1.merge(config2);

        assert_eq!(merged.non_interactive, Some(true));
    }

    #[test]
    fn test_merge_conflict_strategy() {
        let config1 = Config {
            conflict_strategy: Some(ConflictStrategy::Fail),
            ..Default::default()
        };
        let config2 = Config {
            conflict_strategy: Some(ConflictStrategy::Skip),
            ..Default::default()
        };
        let merged = config1.merge(config2);

        assert_eq!(merged.conflict_strategy, Some(ConflictStrategy::Skip));
    }

    #[test]
    fn test_merge_preserves_lower_when_higher_none() {
        let config1 = Config {
            non_interactive: Some(true),
            conflict_strategy: Some(ConflictStrategy::Fail),
            ..Default::default()
        };
        let config2 = Config::default();
        let merged = config1.merge(config2);

        assert_eq!(merged.non_interactive, Some(true));
        assert_eq!(merged.conflict_strategy, Some(ConflictStrategy::Fail));
    }

    #[test]
    fn test_merge_direction_config_ignore_patterns() {
        let config1 = Config {
            to_local: Some(DirectionConfig {
                ignore: Some(vec!["*.tmp".to_string()]),
                types: Some(vec![ConfigType::Commands]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let config2 = Config {
            to_local: Some(DirectionConfig {
                ignore: Some(vec!["*.log".to_string(), "*.bak".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let merged = config1.merge(config2);
        let to_local = merged.to_local.unwrap();

        // Higher precedence ignore patterns should replace lower
        assert_eq!(to_local.ignore.as_ref().unwrap().len(), 2);
        assert!(
            to_local
                .ignore
                .as_ref()
                .unwrap()
                .contains(&"*.log".to_string())
        );
        assert!(
            to_local
                .ignore
                .as_ref()
                .unwrap()
                .contains(&"*.bak".to_string())
        );

        // Types from lower config should be preserved
        assert_eq!(to_local.types.as_ref().unwrap().len(), 1);
        assert_eq!(to_local.types.as_ref().unwrap()[0], ConfigType::Commands);
    }

    #[test]
    fn test_merge_direction_config_types_override() {
        let config1 = Config {
            to_local: Some(DirectionConfig {
                types: Some(vec![ConfigType::Commands]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let config2 = Config {
            to_local: Some(DirectionConfig {
                types: Some(vec![ConfigType::Skills, ConfigType::Subagents]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let merged = config1.merge(config2);
        let types = merged.to_local.unwrap().types.unwrap();

        // Higher precedence types should replace lower
        assert_eq!(types.len(), 2);
        assert!(types.contains(&ConfigType::Skills));
        assert!(types.contains(&ConfigType::Subagents));
    }

    #[test]
    fn test_merge_partial_direction_config() {
        let config1 = Config {
            to_local: Some(DirectionConfig {
                ignore: Some(vec!["*.tmp".to_string()]),
                types: Some(vec![ConfigType::Commands]),
                include: None,
            }),
            ..Default::default()
        };

        let config2 = Config {
            to_local: Some(DirectionConfig {
                include: Some(vec!["important-*.md".to_string()]),
                ignore: None,
                types: None,
            }),
            ..Default::default()
        };

        let merged = config1.merge(config2);
        let to_local = merged.to_local.unwrap();

        // Lower config fields should be preserved when higher is None
        assert_eq!(to_local.ignore.as_ref().unwrap()[0], "*.tmp");
        assert_eq!(to_local.types.as_ref().unwrap()[0], ConfigType::Commands);
        // Higher config fields should be added
        assert_eq!(to_local.include.as_ref().unwrap()[0], "important-*.md");
    }

    #[test]
    fn test_config_loader_new() {
        let loader = ConfigLoader::new(Some(PathBuf::from("/test")), None);
        assert_eq!(loader.project_dir, Some(PathBuf::from("/test")));
        assert!(loader.custom_config.is_none());
    }

    // Integration Tests

    #[test]
    fn test_load_merged_no_configs() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let loader = ConfigLoader::new(Some(temp_dir.path().to_path_buf()), None);

        let merged = loader.load_merged().unwrap();
        assert!(merged.to_local.is_none());
        assert!(merged.to_global.is_none());
    }

    #[test]
    fn test_load_merged_project_config_only() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".ccsync.yaml");

        let yaml = r#"
to_local:
  ignore:
    - "*.tmp"
  types:
    - commands
conflict_strategy: skip
"#;
        fs::write(&config_path, yaml).unwrap();

        let loader = ConfigLoader::new(Some(temp_dir.path().to_path_buf()), None);
        let merged = loader.load_merged().unwrap();

        assert!(merged.to_local.is_some());
        assert_eq!(merged.conflict_strategy, Some(ConflictStrategy::Skip));
    }

    #[test]
    fn test_load_merged_local_overrides_project() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Project config
        let project_config = temp_dir.path().join(".ccsync.yaml");
        fs::write(
            &project_config,
            r#"
conflict_strategy: fail
non_interactive: false
"#,
        )
        .unwrap();

        // Local config (higher precedence)
        let local_config = temp_dir.path().join(".ccsync.local.yaml");
        fs::write(
            &local_config,
            r#"
conflict_strategy: skip
"#,
        )
        .unwrap();

        let loader = ConfigLoader::new(Some(temp_dir.path().to_path_buf()), None);
        let merged = loader.load_merged().unwrap();

        // Local config should override conflict_strategy
        assert_eq!(merged.conflict_strategy, Some(ConflictStrategy::Skip));
        // Project config non_interactive should be preserved
        assert_eq!(merged.non_interactive, Some(false));
    }

    #[test]
    fn test_load_merged_custom_config_highest_precedence() {
        use std::fs;
        use tempfile::{NamedTempFile, TempDir};

        let temp_dir = TempDir::new().unwrap();

        // Project config
        let project_config = temp_dir.path().join(".ccsync.yaml");
        fs::write(
            &project_config,
            r#"
conflict_strategy: fail
non_interactive: false
"#,
        )
        .unwrap();

        // Custom config (highest precedence)
        let custom_config = NamedTempFile::new().unwrap();
        fs::write(
            custom_config.path(),
            r#"
conflict_strategy: newer
non_interactive: true
"#,
        )
        .unwrap();

        let loader = ConfigLoader::new(
            Some(temp_dir.path().to_path_buf()),
            Some(custom_config.path().to_path_buf()),
        );
        let merged = loader.load_merged().unwrap();

        // Custom config should override everything
        assert_eq!(merged.conflict_strategy, Some(ConflictStrategy::Newer));
        assert_eq!(merged.non_interactive, Some(true));
    }

    #[test]
    fn test_load_merged_direction_config_precedence() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Lower precedence: project config
        let project_config = temp_dir.path().join(".ccsync.yaml");
        fs::write(
            &project_config,
            r#"
to_local:
  ignore:
    - "*.tmp"
  types:
    - commands
"#,
        )
        .unwrap();

        // Higher precedence: local config
        let local_config = temp_dir.path().join(".ccsync.local.yaml");
        fs::write(
            &local_config,
            r#"
to_local:
  ignore:
    - "*.log"
    - "*.bak"
"#,
        )
        .unwrap();

        let loader = ConfigLoader::new(Some(temp_dir.path().to_path_buf()), None);
        let merged = loader.load_merged().unwrap();

        let to_local = merged.to_local.unwrap();
        // Local config ignore should override
        assert_eq!(to_local.ignore.as_ref().unwrap().len(), 2);
        assert!(
            to_local
                .ignore
                .as_ref()
                .unwrap()
                .contains(&"*.log".to_string())
        );
        // Types from project config should be preserved (local didn't override)
        assert_eq!(to_local.types.as_ref().unwrap().len(), 1);
        assert_eq!(to_local.types.as_ref().unwrap()[0], ConfigType::Commands);
    }

    #[test]
    fn test_load_merged_validates_final_config() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".ccsync.yaml");

        // Config with invalid content (empty types array)
        let yaml = r#"
to_local:
  types: []
"#;
        fs::write(&config_path, yaml).unwrap();

        let loader = ConfigLoader::new(Some(temp_dir.path().to_path_buf()), None);
        let result = loader.load_merged();

        // Should fail validation
        assert!(result.is_err());
        match result {
            Err(ConfigError::InvalidConfig { .. }) => (),
            _ => panic!("Expected InvalidConfig error from validation"),
        }
    }

    #[test]
    fn test_load_merged_three_configs() {
        use std::fs;
        use tempfile::{NamedTempFile, TempDir};

        let temp_dir = TempDir::new().unwrap();

        // Project config (lowest)
        let project_config = temp_dir.path().join(".ccsync.yaml");
        fs::write(
            &project_config,
            r#"
to_local:
  types:
    - commands
conflict_strategy: fail
non_interactive: false
preserve_symlinks: false
"#,
        )
        .unwrap();

        // Local config (middle)
        let local_config = temp_dir.path().join(".ccsync.local.yaml");
        fs::write(
            &local_config,
            r#"
conflict_strategy: skip
non_interactive: true
"#,
        )
        .unwrap();

        // Custom config (highest)
        let custom_config = NamedTempFile::new().unwrap();
        fs::write(
            custom_config.path(),
            r#"
preserve_symlinks: true
"#,
        )
        .unwrap();

        let loader = ConfigLoader::new(
            Some(temp_dir.path().to_path_buf()),
            Some(custom_config.path().to_path_buf()),
        );
        let merged = loader.load_merged().unwrap();

        // Custom overrides preserve_symlinks
        assert_eq!(merged.preserve_symlinks, Some(true));
        // Local overrides conflict_strategy and non_interactive
        assert_eq!(merged.conflict_strategy, Some(ConflictStrategy::Skip));
        assert_eq!(merged.non_interactive, Some(true));
        // Project config types are preserved
        assert!(merged.to_local.is_some());
        let to_local = merged.to_local.unwrap();
        assert_eq!(to_local.types.as_ref().unwrap()[0], ConfigType::Commands);
    }
}
