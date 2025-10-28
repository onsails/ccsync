//! Sync orchestration - coordinates the sync workflow

use std::path::Path;

use anyhow::Context;

use super::actions::SyncActionResolver;
use super::executor::FileOperationExecutor;
use super::SyncResult;
use crate::comparison::{ConflictStrategy, FileComparator};
use crate::config::{Config, PatternMatcher, SyncDirection};
use crate::error::Result;
use crate::scanner::{FileFilter, Scanner};

/// Main sync engine
pub struct SyncEngine {
    config: Config,
    direction: SyncDirection,
    pattern_matcher: Option<PatternMatcher>,
}

impl SyncEngine {
    /// Create a new sync engine
    ///
    /// # Errors
    ///
    /// Returns an error if pattern compilation fails.
    pub fn new(config: Config, direction: SyncDirection) -> Result<Self> {
        // Compile pattern matcher once during construction
        let pattern_matcher = if !config.ignore.is_empty() || !config.include.is_empty() {
            Some(PatternMatcher::with_patterns(
                &config.ignore,
                &config.include,
            )?)
        } else {
            None
        };

        Ok(Self {
            config,
            direction,
            pattern_matcher,
        })
    }

    /// Execute the sync operation
    ///
    /// # Errors
    ///
    /// Returns an error if sync fails.
    pub fn sync(&self, source_root: &Path, dest_root: &Path) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        // Scan source directory
        let filter = FileFilter::new();
        let scanner = Scanner::new(filter, self.config.preserve_symlinks == Some(true));
        let scan_result = scanner.scan(source_root);

        // Process each scanned file
        let executor = FileOperationExecutor::new(self.config.dry_run == Some(true));
        let conflict_strategy = self.get_conflict_strategy();

        for file in &scan_result.files {
            // Apply pattern filter
            if let Some(ref matcher) = self.pattern_matcher
                && !matcher.should_include(&file.path, false) {
                    result.skipped += 1;
                    continue;
                }

            // Get relative path
            let rel_path = file
                .path
                .strip_prefix(source_root)
                .with_context(|| format!("Failed to strip prefix from {}", file.path.display()))?;

            let dest_path = dest_root.join(rel_path);

            // Compare files
            let comparison = FileComparator::compare(
                &file.path,
                &dest_path,
                conflict_strategy,
            )?;

            // Determine action
            let action = SyncActionResolver::resolve(
                file.path.clone(),
                dest_path,
                &comparison,
            );

            // Execute action
            if let Err(e) = executor.execute(&action, &mut result) {
                result.errors.push(e.to_string());
            }
        }

        // Log warnings from scanner
        for warning in &scan_result.warnings {
            eprintln!("Warning: {warning}");
        }

        Ok(result)
    }

    /// Get conflict strategy from config or use default
    fn get_conflict_strategy(&self) -> ConflictStrategy {
        self.config.conflict_strategy.unwrap_or(ConflictStrategy::Fail)
    }
}
