//! Common types and utilities for command execution

use ccsync_core::config::{Config, ConfigManager};

/// Execution options for sync commands
#[allow(clippy::struct_excessive_bools)]
pub struct SyncOptions<'a> {
    /// Enable verbose output
    pub verbose: bool,
    /// Preview changes without applying (dry-run)
    pub dry_run: bool,
    /// Auto-approve all operations without prompting
    pub yes_all: bool,
    /// Path to custom config file
    pub config_path: Option<&'a std::path::Path>,
    /// Skip loading all config files
    pub no_config: bool,
}

impl<'a> SyncOptions<'a> {
    /// Create new sync options
    #[must_use]
    #[allow(clippy::fn_params_excessive_bools)]
    pub const fn new(
        verbose: bool,
        dry_run: bool,
        yes_all: bool,
        config_path: Option<&'a std::path::Path>,
        no_config: bool,
    ) -> Self {
        Self {
            verbose,
            dry_run,
            yes_all,
            config_path,
            no_config,
        }
    }

    /// Load configuration from files or use defaults
    ///
    /// # Errors
    ///
    /// Returns an error if config file is explicitly specified but cannot be loaded.
    pub fn load_config(&self) -> anyhow::Result<Config> {
        if self.no_config {
            if self.verbose {
                println!("Skipping config file loading (--no-config)");
            }
            return Ok(Config::default());
        }

        match ConfigManager::load(self.config_path) {
            Ok(config) => Ok(config),
            Err(e) => {
                // If user explicitly specified a config file, fail hard
                if self.config_path.is_some() {
                    anyhow::bail!("Failed to load config file: {e}");
                }

                // Otherwise, warn and use defaults
                if self.verbose {
                    eprintln!("Warning: Failed to load config files: {e}");
                    eprintln!("Using default configuration");
                }
                Ok(Config::default())
            }
        }
    }
}
