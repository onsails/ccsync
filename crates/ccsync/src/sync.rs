//! Bidirectional synchronization engine
//!
//! This module implements the core sync logic for to-local and to-global operations.
//! Interactive prompts are NOT implemented here - they will be added in Task 4.
//! The sync engine uses ConflictStrategy from config/CLI flags directly.

mod actions;
mod executor;
mod orchestrator;
mod reporting;

// Public exports for tests (will be made pub for CLI integration in future)
#[cfg(test)]
pub(crate) use orchestrator::SyncEngine;
#[cfg(test)]
pub(crate) use reporting::SyncReporter;

/// Synchronization result with statistics
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    /// Files created
    pub created: usize,
    /// Files updated
    pub updated: usize,
    /// Files deleted
    pub deleted: usize,
    /// Files skipped
    pub skipped: usize,
    /// Conflicts encountered
    pub conflicts: usize,
    /// Errors encountered
    pub errors: Vec<String>,
}

impl SyncResult {
    /// Total operations performed
    #[must_use]
    pub const fn total_operations(&self) -> usize {
        self.created + self.updated + self.deleted
    }

    /// Whether sync was successful (no errors)
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

#[cfg(test)]
mod integration_tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::*;
    use crate::comparison::ConflictStrategy;
    use crate::config::{Config, SyncDirection};

    fn setup_test_dirs() -> (TempDir, TempDir) {
        let source = TempDir::new().unwrap();
        let dest = TempDir::new().unwrap();
        (source, dest)
    }

    fn create_test_file(dir: &Path, rel_path: &str, content: &str) {
        let path = dir.join(rel_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn test_sync_create_new_files() {
        let (source_dir, dest_dir) = setup_test_dirs();

        // Create files in source
        create_test_file(source_dir.path(), "agents/test.md", "test agent");
        create_test_file(source_dir.path(), "skills/skill1/SKILL.md", "test skill");

        let config = Config::default();
        let engine = SyncEngine::new(config, SyncDirection::ToLocal).unwrap();

        let result = engine.sync(source_dir.path(), dest_dir.path()).unwrap();

        assert_eq!(result.created, 2);
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 0);
        assert!(result.is_success());

        // Verify files were created
        assert!(dest_dir.path().join("agents/test.md").exists());
        assert!(dest_dir.path().join("skills/skill1/SKILL.md").exists());
    }

    #[test]
    fn test_sync_skip_identical_files() {
        let (source_dir, dest_dir) = setup_test_dirs();

        // Create identical files in both
        let content = "identical content";
        create_test_file(source_dir.path(), "agents/test.md", content);
        create_test_file(dest_dir.path(), "agents/test.md", content);

        let config = Config::default();
        let engine = SyncEngine::new(config, SyncDirection::ToLocal).unwrap();

        let result = engine.sync(source_dir.path(), dest_dir.path()).unwrap();

        assert_eq!(result.created, 0);
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 1);
        assert!(result.is_success());
    }

    #[test]
    fn test_sync_with_ignore_patterns() {
        let (source_dir, dest_dir) = setup_test_dirs();

        create_test_file(source_dir.path(), "agents/include.md", "include");
        create_test_file(source_dir.path(), "agents/ignore.md", "ignore");

        let mut config = Config::default();
        config.ignore = vec!["**/ignore.md".to_string()];

        let engine = SyncEngine::new(config, SyncDirection::ToLocal).unwrap();
        let result = engine.sync(source_dir.path(), dest_dir.path()).unwrap();

        assert_eq!(result.created, 1);
        assert_eq!(result.skipped, 1);
        assert!(dest_dir.path().join("agents/include.md").exists());
        assert!(!dest_dir.path().join("agents/ignore.md").exists());
    }

    #[test]
    fn test_sync_conflict_fail_strategy() {
        let (source_dir, dest_dir) = setup_test_dirs();

        // Create different content in both
        create_test_file(source_dir.path(), "agents/test.md", "source content");
        create_test_file(dest_dir.path(), "agents/test.md", "dest content");

        let config = Config::default(); // Default is Fail
        let engine = SyncEngine::new(config, SyncDirection::ToLocal).unwrap();

        let result = engine.sync(source_dir.path(), dest_dir.path()).unwrap();

        // Should encounter conflict and record error
        assert_eq!(result.conflicts, 0);
        assert!(!result.errors.is_empty());
        assert!(!result.is_success());
    }

    #[test]
    fn test_sync_dry_run() {
        let (source_dir, dest_dir) = setup_test_dirs();

        create_test_file(source_dir.path(), "agents/test.md", "test content");

        let mut config = Config::default();
        config.dry_run = Some(true);

        let engine = SyncEngine::new(config, SyncDirection::ToLocal).unwrap();
        let result = engine.sync(source_dir.path(), dest_dir.path()).unwrap();

        // Should report as created
        assert_eq!(result.created, 1);
        assert!(result.is_success());

        // But file should NOT actually exist (dry run)
        assert!(!dest_dir.path().join("agents/test.md").exists());
    }

    #[test]
    fn test_sync_bidirectional() {
        let (dir1, dir2) = setup_test_dirs();

        // Create file in dir1
        create_test_file(dir1.path(), "agents/from1.md", "content 1");

        let config = Config::default();

        // Sync to dir2
        let engine = SyncEngine::new(config.clone(), SyncDirection::ToLocal).unwrap();
        let result = engine.sync(dir1.path(), dir2.path()).unwrap();
        assert_eq!(result.created, 1);

        // Create file in dir2
        create_test_file(dir2.path(), "agents/from2.md", "content 2");

        // Sync back to dir1
        let engine = SyncEngine::new(config, SyncDirection::ToGlobal).unwrap();
        let result = engine.sync(dir2.path(), dir1.path()).unwrap();
        assert_eq!(result.created, 1);

        // Both files should exist in both directories
        assert!(dir1.path().join("agents/from1.md").exists());
        assert!(dir1.path().join("agents/from2.md").exists());
        assert!(dir2.path().join("agents/from1.md").exists());
        assert!(dir2.path().join("agents/from2.md").exists());
    }

    #[test]
    fn test_sync_update_existing_files() {
        let (source_dir, dest_dir) = setup_test_dirs();

        // Create initial identical files
        create_test_file(source_dir.path(), "agents/test.md", "v1");
        create_test_file(dest_dir.path(), "agents/test.md", "v1");

        // Update source file
        create_test_file(source_dir.path(), "agents/test.md", "v2");

        let mut config = Config::default();
        config.conflict_strategy = Some(ConflictStrategy::Overwrite);

        let engine = SyncEngine::new(config, SyncDirection::ToLocal).unwrap();
        let result = engine.sync(source_dir.path(), dest_dir.path()).unwrap();

        // Should update the file
        assert_eq!(result.updated, 1);
        assert_eq!(result.created, 0);
        assert!(result.is_success());

        // Verify content was updated
        let content = fs::read_to_string(dest_dir.path().join("agents/test.md")).unwrap();
        assert_eq!(content, "v2");
    }

    #[test]
    fn test_sync_reporter() {
        let mut result = SyncResult::default();
        result.created = 5;
        result.updated = 3;
        result.skipped = 2;

        let summary = SyncReporter::generate_summary(&result);

        assert!(summary.contains("Created:  5"));
        assert!(summary.contains("Updated:  3"));
        assert!(summary.contains("Skipped:  2"));
        assert!(summary.contains("Total operations: 8"));
        assert!(summary.contains("✓ Success"));
    }

    #[test]
    fn test_sync_reporter_with_errors() {
        let mut result = SyncResult::default();
        result.created = 1;
        result.errors.push("Test error".to_string());

        let summary = SyncReporter::generate_summary(&result);

        assert!(summary.contains("Errors (1)"));
        assert!(summary.contains("Test error"));
        assert!(summary.contains("✗ Completed with errors"));
        assert!(!result.is_success());
    }
}
