//! Configuration types and structures

use serde::{Deserialize, Serialize};

/// Sync direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncDirection {
    /// Sync from global to local
    ToLocal,
    /// Sync from local to global
    ToGlobal,
}

/// File type for type-specific rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    /// Text files
    Text,
    /// Binary files
    Binary,
    /// Symlinks
    Symlink,
    /// Any file type
    Any,
}

/// Sync rule for direction and type-specific configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncRule {
    /// File patterns this rule applies to
    #[serde(default)]
    pub patterns: Vec<String>,

    /// Sync direction this rule applies to (optional, applies to all if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<SyncDirection>,

    /// File type this rule applies to (optional, applies to all if None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<FileType>,

    /// Whether to include (true) or exclude (false) matching files
    pub include: bool,
}

/// Main configuration structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    /// Patterns to ignore (exclude from sync)
    #[serde(default)]
    pub ignore: Vec<String>,

    /// Patterns to explicitly include (override ignores)
    #[serde(default)]
    pub include: Vec<String>,

    /// Follow symlinks
    #[serde(default)]
    pub follow_symlinks: bool,

    /// Preserve symlinks instead of resolving them
    #[serde(default)]
    pub preserve_symlinks: bool,

    /// Dry run mode (don't actually sync)
    #[serde(default)]
    pub dry_run: bool,

    /// Non-interactive mode (no prompts)
    #[serde(default)]
    pub non_interactive: bool,

    /// Advanced sync rules (direction and type-specific)
    #[serde(default)]
    pub rules: Vec<SyncRule>,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.ignore.is_empty());
        assert!(config.include.is_empty());
        assert!(!config.follow_symlinks);
        assert!(!config.preserve_symlinks);
    }

    #[test]
    fn test_sync_direction_serde() {
        let to_local = SyncDirection::ToLocal;
        let to_global = SyncDirection::ToGlobal;

        let to_local_str = serde_json::to_string(&to_local).unwrap();
        let to_global_str = serde_json::to_string(&to_global).unwrap();

        assert_eq!(to_local_str, r#""to-local""#);
        assert_eq!(to_global_str, r#""to-global""#);
    }

    #[test]
    fn test_file_type_serde() {
        let text = FileType::Text;
        let binary = FileType::Binary;

        let text_str = serde_json::to_string(&text).unwrap();
        let binary_str = serde_json::to_string(&binary).unwrap();

        assert_eq!(text_str, r#""text""#);
        assert_eq!(binary_str, r#""binary""#);
    }

    #[test]
    fn test_sync_rule() {
        let rule = SyncRule {
            patterns: vec!["*.md".to_string()],
            direction: Some(SyncDirection::ToLocal),
            file_type: Some(FileType::Text),
            include: true,
        };

        assert_eq!(rule.patterns.len(), 1);
        assert_eq!(rule.direction, Some(SyncDirection::ToLocal));
        assert!(rule.include);
    }
}
