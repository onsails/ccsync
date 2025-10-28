//! Configuration file discovery from multiple locations

use std::path::{Path, PathBuf};

use crate::error::Result;


/// Configuration file locations in order of precedence
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFiles {
    /// Config from CLI flag (highest precedence)
    pub cli: Option<PathBuf>,
    /// Project-local config (.ccsync.local)
    pub local: Option<PathBuf>,
    /// Project config (.ccsync)
    pub project: Option<PathBuf>,
    /// Global XDG config
    pub global: Option<PathBuf>,
}

/// Config file discovery
pub struct ConfigDiscovery;

impl ConfigDiscovery {
    /// Create a new config discovery instance
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Discover all available configuration files
    ///
    /// Returns a `ConfigFiles` struct with paths to discovered configs.
    ///
    /// # Errors
    ///
    /// Returns an error if a CLI config path is specified but doesn't exist.
    pub fn discover(cli_path: Option<&Path>) -> Result<ConfigFiles> {
        let cli = if let Some(p) = cli_path {
            if !p.exists() {
                anyhow::bail!("Config file specified via CLI does not exist: {}", p.display());
            }
            Some(p.to_path_buf())
        } else {
            None
        };

        let local = Self::find_file(".ccsync.local");
        let project = Self::find_file(".ccsync");
        let global = Self::find_global_config();

        Ok(ConfigFiles {
            cli,
            local,
            project,
            global,
        })
    }

    /// Find a config file in the current directory or parent directories
    ///
    /// Note: Does not follow symlinks for security reasons
    fn find_file(name: &str) -> Option<PathBuf> {
        let mut current = std::env::current_dir().ok()?;

        loop {
            let candidate = current.join(name);

            // Use symlink_metadata to avoid following symlinks (security)
            if let Ok(metadata) = candidate.symlink_metadata()
                && metadata.is_file() {
                    return Some(candidate);
                }

            // Move to parent directory
            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Find global config in XDG config directory
    ///
    /// Note: Does not follow symlinks for security reasons
    fn find_global_config() -> Option<PathBuf> {
        let config_dir = dirs::config_dir()?;
        let global_config = config_dir.join("ccsync").join("config.toml");

        // Use symlink_metadata to avoid following symlinks (security)
        if let Ok(metadata) = global_config.symlink_metadata()
            && metadata.is_file() {
                return Some(global_config);
            }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_no_configs() {
        let _discovery = ConfigDiscovery::new();
        let files = ConfigDiscovery::discover(None).unwrap();

        assert!(files.cli.is_none());
        // local, project, and global may or may not exist depending on test environment
    }

    #[test]
    fn test_discover_cli_config() {
        let tmp = TempDir::new().unwrap();
        let cli_config = tmp.path().join("custom.toml");
        fs::write(&cli_config, "# config").unwrap();

        let _discovery = ConfigDiscovery::new();
        let files = ConfigDiscovery::discover(Some(&cli_config)).unwrap();

        assert_eq!(files.cli, Some(cli_config));
    }

    #[test]
    fn test_discover_cli_config_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let cli_config = tmp.path().join("nonexistent.toml");

        let _discovery = ConfigDiscovery::new();
        let result = ConfigDiscovery::discover(Some(&cli_config));

        // Nonexistent CLI config should fail loudly (fail-fast principle)
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    // Note: Tests for find_file() that search from current directory are omitted
    // to avoid test environment pollution from std::env::set_current_dir().
    // The find_file() function is tested implicitly through the discover() tests
    // which will find .ccsync files if present in the repository.
}
