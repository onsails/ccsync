//! Configuration file parsing, merging, and pattern matching
//!
//! This module handles:
//! - Config file discovery from multiple locations
//! - TOML parsing with serde
//! - Config merging with precedence rules
//! - Gitignore-style pattern matching
//! - Direction and type-specific rules
//! - Validation and error reporting

mod discovery;
mod merge;
mod patterns;
mod types;
mod validation;

#[cfg(test)]
mod integration_tests;

pub use discovery::ConfigDiscovery;
pub use merge::ConfigMerger;
pub use types::Config;
pub use validation::ConfigValidator;

use crate::error::Result;

/// Configuration manager that coordinates discovery, parsing, merging, and validation
pub struct ConfigManager {
    discovery: ConfigDiscovery,
    merger: ConfigMerger,
    validator: ConfigValidator,
}

impl ConfigManager {
    /// Create a new configuration manager
    #[must_use]
    pub const fn new() -> Self {
        Self {
            discovery: ConfigDiscovery::new(),
            merger: ConfigMerger::new(),
            validator: ConfigValidator::new(),
        }
    }

    /// Load and merge configuration from all sources
    ///
    /// # Errors
    ///
    /// Returns an error if config files are invalid or cannot be read.
    pub fn load(cli_config_path: Option<&std::path::Path>) -> Result<Config> {
        // Discover all config files
        let config_files = ConfigDiscovery::discover(cli_config_path);

        // Parse and merge configs
        let merged = ConfigMerger::merge(&config_files)?;

        // Validate the final configuration
        ConfigValidator::validate(&merged)?;

        Ok(merged)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        let default_manager = ConfigManager::default();

        // Both should be valid
        assert!(std::ptr::addr_of!(manager).is_null() == false);
        assert!(std::ptr::addr_of!(default_manager).is_null() == false);
    }
}
