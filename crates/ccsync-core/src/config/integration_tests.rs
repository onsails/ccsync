//! Integration tests for configuration module

use std::fs;
use tempfile::TempDir;

use super::types::{FileType, SyncDirection, SyncRule};
use super::{Config, ConfigManager};

#[test]
fn test_full_config_workflow() {
    let tmp = TempDir::new().unwrap();
    let config_file = tmp.path().join("config.toml");

    fs::write(
        &config_file,
        r#"
ignore = ["*.tmp", "*.log"]
include = ["important.tmp"]
follow_symlinks = false
preserve_symlinks = false
dry_run = false
non_interactive = false

[[rules]]
patterns = ["*.md"]
direction = "to-local"
file_type = "text"
include = true
"#,
    )
    .unwrap();

    let _manager = ConfigManager::new();
    let config = ConfigManager::load(Some(&config_file)).unwrap();

    assert_eq!(config.ignore.len(), 2);
    assert_eq!(config.include.len(), 1);
    assert!(config.follow_symlinks != Some(true));
    assert_eq!(config.rules.len(), 1);
    assert_eq!(config.rules[0].direction, Some(SyncDirection::ToLocal));
}

#[test]
fn test_invalid_config_validation() {
    let tmp = TempDir::new().unwrap();
    let config_file = tmp.path().join("config.toml");

    fs::write(
        &config_file,
        r#"
follow_symlinks = true
preserve_symlinks = true
"#,
    )
    .unwrap();

    let _manager = ConfigManager::new();
    let result = ConfigManager::load(Some(&config_file));

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("follow_symlinks and preserve_symlinks")
    );
}

#[test]
fn test_config_with_rules() {
    let config = Config {
        ignore: vec!["*.tmp".to_string()],
        include: vec![],
        follow_symlinks: Some(false),
        preserve_symlinks: Some(false),
        dry_run: Some(false),
        non_interactive: Some(false),
        conflict_strategy: None,
        rules: vec![
            SyncRule {
                patterns: vec!["agents/*.md".to_string()],
                direction: Some(SyncDirection::ToLocal),
                file_type: Some(FileType::Text),
                include: true,
            },
            SyncRule {
                patterns: vec!["*.bin".to_string()],
                direction: None,
                file_type: Some(FileType::Binary),
                include: false,
            },
        ],
    };

    assert_eq!(config.rules.len(), 2);
    assert_eq!(config.rules[0].patterns[0], "agents/*.md");
    assert_eq!(config.rules[1].file_type, Some(FileType::Binary));
}
