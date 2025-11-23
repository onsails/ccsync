//! Bidirectional synchronization engine
//!
//! This module implements the core sync logic for to-local and to-global operations.
//! Interactive prompts are NOT implemented here - they will be added in Task 4.
//! The sync engine uses ConflictStrategy from config/CLI flags directly.

mod actions;
mod executor;
mod orchestrator;
mod reporting;

// Public exports for CLI integration
pub use actions::SyncAction;
pub use orchestrator::{ApprovalCallback, SyncEngine};
pub use reporting::SyncReporter;

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
    /// Skip reasons with counts
    pub skip_reasons: std::collections::HashMap<String, usize>,
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

        // Should fail due to conflict with Fail strategy
        let result = engine.sync(source_dir.path(), dest_dir.path());
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("Sync failed"));
        assert!(err_msg.contains("Conflict"));
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

    #[test]
    fn test_sync_pattern_matching_with_relative_paths() {
        let (source_dir, dest_dir) = setup_test_dirs();

        // Create multiple files with git-* pattern in agents/
        create_test_file(source_dir.path(), "agents/git-commit.md", "git commit agent");
        create_test_file(source_dir.path(), "agents/git-helper.md", "git helper agent");
        create_test_file(source_dir.path(), "agents/other-agent.md", "other agent");
        // Create a skill (skills/test-skill/SKILL.md is the expected structure)
        create_test_file(source_dir.path(), "skills/test-skill/SKILL.md", "test skill");

        // Configure to ignore agents/git-* pattern (relative path)
        let mut config = Config::default();
        config.ignore = vec!["agents/git-*".to_string()];

        let engine = SyncEngine::new(config, SyncDirection::ToLocal).unwrap();
        let result = engine.sync(source_dir.path(), dest_dir.path()).unwrap();

        // Should create 2 files (other-agent.md and skills/test-skill/SKILL.md)
        // Should skip 2 files (git-commit.md and git-helper.md)
        assert_eq!(result.created, 2);
        assert_eq!(result.skipped, 2);
        assert!(result.is_success());

        // Verify git-* files were NOT created
        assert!(!dest_dir.path().join("agents/git-commit.md").exists());
        assert!(!dest_dir.path().join("agents/git-helper.md").exists());

        // Verify other files WERE created
        assert!(dest_dir.path().join("agents/other-agent.md").exists());
        assert!(dest_dir.path().join("skills/test-skill/SKILL.md").exists());
    }

    #[test]
    fn test_sync_skill_directory_create_new() {
        let (source_dir, dest_dir) = setup_test_dirs();

        // Create a skill directory with multiple files
        create_test_file(source_dir.path(), "skills/rust-dev/SKILL.md", "skill content");
        create_test_file(
            source_dir.path(),
            "skills/rust-dev/scripts/check.sh",
            "script content",
        );
        create_test_file(
            source_dir.path(),
            "skills/rust-dev/assets/logo.png",
            "image data",
        );

        let config = Config::default();
        let engine = SyncEngine::new(config, SyncDirection::ToLocal).unwrap();
        let result = engine.sync(source_dir.path(), dest_dir.path()).unwrap();

        // Directory should be created
        assert_eq!(result.created, 1);
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 0);
        assert!(result.is_success());

        // Verify entire directory structure was copied
        assert!(dest_dir.path().join("skills/rust-dev").exists());
        assert!(dest_dir.path().join("skills/rust-dev/SKILL.md").exists());
        assert!(dest_dir
            .path()
            .join("skills/rust-dev/scripts/check.sh")
            .exists());
        assert!(dest_dir
            .path()
            .join("skills/rust-dev/assets/logo.png")
            .exists());
    }

    #[test]
    fn test_sync_skill_directory_identical() {
        let (source_dir, dest_dir) = setup_test_dirs();

        // Create identical skill directories in both locations
        let skill_content = "identical skill";
        let script_content = "identical script";

        create_test_file(
            source_dir.path(),
            "skills/test-skill/SKILL.md",
            skill_content,
        );
        create_test_file(
            source_dir.path(),
            "skills/test-skill/helper.py",
            script_content,
        );

        create_test_file(dest_dir.path(), "skills/test-skill/SKILL.md", skill_content);
        create_test_file(dest_dir.path(), "skills/test-skill/helper.py", script_content);

        let config = Config::default();
        let engine = SyncEngine::new(config, SyncDirection::ToLocal).unwrap();
        let result = engine.sync(source_dir.path(), dest_dir.path()).unwrap();

        // Should skip identical directory
        assert_eq!(result.created, 0);
        assert_eq!(result.updated, 0);
        assert_eq!(result.skipped, 1);
        assert!(result.is_success());
    }

    #[test]
    fn test_sync_skill_directory_conflict_overwrite() {
        let (source_dir, dest_dir) = setup_test_dirs();

        // Create different versions of the same skill
        create_test_file(
            source_dir.path(),
            "skills/changed-skill/SKILL.md",
            "new content",
        );
        create_test_file(
            dest_dir.path(),
            "skills/changed-skill/SKILL.md",
            "old content",
        );

        let mut config = Config::default();
        config.conflict_strategy = Some(crate::comparison::ConflictStrategy::Overwrite);

        let engine = SyncEngine::new(config, SyncDirection::ToLocal).unwrap();
        let result = engine.sync(source_dir.path(), dest_dir.path()).unwrap();

        // Should update the directory
        assert_eq!(result.created, 0);
        assert_eq!(result.updated, 1);
        assert!(result.is_success());

        // Verify dest has source content
        let content = fs::read_to_string(dest_dir.path().join("skills/changed-skill/SKILL.md"))
            .unwrap();
        assert_eq!(content, "new content");
    }

    #[test]
    fn test_sync_skill_directory_conflict_with_interactive_approval() {
        let (source_dir, dest_dir) = setup_test_dirs();

        // Create different versions of the same skill
        create_test_file(
            source_dir.path(),
            "skills/approved-skill/SKILL.md",
            "new content",
        );
        create_test_file(
            dest_dir.path(),
            "skills/approved-skill/SKILL.md",
            "old content",
        );

        // Use ConflictStrategy::Fail (default for interactive mode)
        // but provide an approval callback that approves
        let mut config = Config::default();
        config.conflict_strategy = Some(crate::comparison::ConflictStrategy::Fail);

        let engine = SyncEngine::new(config, SyncDirection::ToLocal).unwrap();

        // Approval callback that approves everything
        let approver = Box::new(|_action: &SyncAction| Ok(true));

        let result = engine
            .sync_with_approver(source_dir.path(), dest_dir.path(), Some(approver))
            .unwrap();

        // BUG: Currently this test fails because even though user approved,
        // the executor sees ConflictStrategy::Fail and bails
        // Expected: Should update the directory since user approved
        assert_eq!(result.updated, 1, "User approved the conflict, should update");
        assert_eq!(result.created, 0);
        assert!(result.is_success());

        // Verify dest has source content
        let content = fs::read_to_string(dest_dir.path().join("skills/approved-skill/SKILL.md"))
            .unwrap();
        assert_eq!(content, "new content");
    }
}
