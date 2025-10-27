//! Configuration merging with precedence rules
//!
//! # Merging Semantics
//!
//! - **Arrays** (ignore, include, rules): Additive - all values from all configs are combined
//! - **Booleans**: OR semantics - if any config sets to `true`, result is `true`
//!
//! ## Boolean OR Semantics
//!
//! The boolean fields use OR semantics for safety. This means:
//! - A lower-precedence config that enables a feature cannot be disabled by higher-precedence configs
//! - Example: If global config has `follow_symlinks = true`, project config setting it to `false` won't disable it
//! - Rationale: Explicit enablement in any config file should be honored (explicit > implicit)
//!
//! If you need true override semantics, use `Option<bool>` in TOML with `Some(true)` vs `Some(false)` vs `None`.

use std::fs;
use std::path::Path;

use anyhow::Context;

use super::discovery::ConfigFiles;
use super::types::Config;
use crate::error::Result;

/// Configuration merger
pub struct ConfigMerger;

impl ConfigMerger {
    /// Create a new config merger
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Merge multiple config files with precedence rules
    ///
    /// Precedence order (highest to lowest):
    /// 1. CLI config
    /// 2. .ccsync.local
    /// 3. .ccsync
    /// 4. Global config
    ///
    /// # Errors
    ///
    /// Returns an error if config files cannot be read or parsed.
    pub fn merge( files: &ConfigFiles) -> Result<Config> {
        let mut merged = Config::default();

        // Load and merge in reverse precedence order (lowest to highest)
        if let Some(global) = &files.global {
            Self::merge_into(&mut merged, global)?;
        }

        if let Some(project) = &files.project {
            Self::merge_into(&mut merged, project)?;
        }

        if let Some(local) = &files.local {
            Self::merge_into(&mut merged, local)?;
        }

        if let Some(cli) = &files.cli {
            Self::merge_into(&mut merged, cli)?;
        }

        Ok(merged)
    }

    /// Load and merge a single config file into the existing config
    fn merge_into(base: &mut Config, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        // Merge: additive for arrays, OR semantics for booleans
        base.ignore.extend(config.ignore);
        base.include.extend(config.include);
        base.rules.extend(config.rules);

        // Boolean flags use OR semantics: if any config sets to true, it's true
        // This means lower-precedence configs can enable features that higher-precedence
        // configs cannot disable. This is intentional for safety (explicit > implicit).
        base.follow_symlinks |= config.follow_symlinks;
        base.preserve_symlinks |= config.preserve_symlinks;
        base.dry_run |= config.dry_run;
        base.non_interactive |= config.non_interactive;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_merge_empty_config() {
        let files = ConfigFiles {
            cli: None,
            local: None,
            project: None,
            global: None,
        };

        let _merger = ConfigMerger::new();
        let config = ConfigMerger::merge(&files).unwrap();

        assert!(config.ignore.is_empty());
        assert!(config.include.is_empty());
    }

    #[test]
    fn test_merge_single_config() {
        let tmp = TempDir::new().unwrap();
        let config_file = tmp.path().join("config.toml");
        fs::write(
            &config_file,
            r#"
ignore = ["*.tmp", "*.log"]
follow_symlinks = true
"#,
        )
        .unwrap();

        let files = ConfigFiles {
            cli: None,
            local: None,
            project: Some(config_file),
            global: None,
        };

        let _merger = ConfigMerger::new();
        let config = ConfigMerger::merge(&files).unwrap();

        assert_eq!(config.ignore.len(), 2);
        assert!(config.follow_symlinks);
    }

    #[test]
    fn test_merge_precedence() {
        let tmp = TempDir::new().unwrap();

        let global = tmp.path().join("global.toml");
        fs::write(&global, r#"ignore = ["*.tmp"]"#).unwrap();

        let project = tmp.path().join("project.toml");
        fs::write(&project, r#"ignore = ["*.log"]"#).unwrap();

        let files = ConfigFiles {
            cli: None,
            local: None,
            project: Some(project),
            global: Some(global),
        };

        let _merger = ConfigMerger::new();
        let config = ConfigMerger::merge(&files).unwrap();

        // Both patterns should be present (additive merging)
        assert_eq!(config.ignore.len(), 2);
        assert!(config.ignore.contains(&"*.tmp".to_string()));
        assert!(config.ignore.contains(&"*.log".to_string()));
    }

    #[test]
    fn test_merge_boolean_override() {
        let tmp = TempDir::new().unwrap();

        let global = tmp.path().join("global.toml");
        fs::write(&global, r#"follow_symlinks = false"#).unwrap();

        let project = tmp.path().join("project.toml");
        fs::write(&project, r#"follow_symlinks = true"#).unwrap();

        let files = ConfigFiles {
            cli: None,
            local: None,
            project: Some(project),
            global: Some(global),
        };

        let _merger = ConfigMerger::new();
        let config = ConfigMerger::merge(&files).unwrap();

        // Project config should override global
        assert!(config.follow_symlinks);
    }
}
