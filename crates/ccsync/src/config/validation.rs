//! Configuration validation and error reporting

use super::types::Config;
use crate::error::Result;

/// Configuration validator
pub struct ConfigValidator;

impl ConfigValidator {
    /// Create a new config validator
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validate a configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid.
    pub fn validate( config: &Config) -> Result<()> {
        // Check for conflicting settings
        if config.follow_symlinks && config.preserve_symlinks {
            anyhow::bail!(
                "Conflicting configuration: both follow_symlinks and preserve_symlinks are enabled"
            );
        }

        // Validate patterns are not empty strings
        for pattern in &config.ignore {
            if pattern.trim().is_empty() {
                anyhow::bail!("Ignore pattern cannot be empty");
            }
        }

        for pattern in &config.include {
            if pattern.trim().is_empty() {
                anyhow::bail!("Include pattern cannot be empty");
            }
        }

        // Validate rules
        for (idx, rule) in config.rules.iter().enumerate() {
            if rule.patterns.is_empty() {
                anyhow::bail!("Rule #{} has no patterns", idx + 1);
            }

            for pattern in &rule.patterns {
                if pattern.trim().is_empty() {
                    anyhow::bail!("Rule #{} has empty pattern", idx + 1);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{FileType, SyncDirection, SyncRule};

    #[test]
    fn test_validate_empty_config() {
        let config = Config::default();
        let _validator = ConfigValidator::new();

        assert!(ConfigValidator::validate(&config).is_ok());
    }

    #[test]
    fn test_validate_conflicting_symlink_settings() {
        let mut config = Config::default();
        config.follow_symlinks = true;
        config.preserve_symlinks = true;

        let _validator = ConfigValidator::new();
        let result = ConfigValidator::validate(&config);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("follow_symlinks and preserve_symlinks"));
    }

    #[test]
    fn test_validate_empty_pattern() {
        let mut config = Config::default();
        config.ignore.push("   ".to_string());

        let _validator = ConfigValidator::new();
        let result = ConfigValidator::validate(&config);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_rule_with_no_patterns() {
        let mut config = Config::default();
        config.rules.push(SyncRule {
            patterns: vec![],
            direction: Some(SyncDirection::ToLocal),
            file_type: Some(FileType::Text),
            include: true,
        });

        let _validator = ConfigValidator::new();
        let result = ConfigValidator::validate(&config);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("has no patterns"));
    }

    #[test]
    fn test_validate_valid_config() {
        let mut config = Config::default();
        config.ignore.push("*.tmp".to_string());
        config.include.push("important.tmp".to_string());
        config.follow_symlinks = false;
        config.preserve_symlinks = false;

        let _validator = ConfigValidator::new();
        assert!(ConfigValidator::validate(&config).is_ok());
    }
}
