use std::path::PathBuf;

use anyhow::Context;
use ccsync_core::comparison::ConflictStrategy;
use ccsync_core::config::{Config, SyncDirection};
use ccsync_core::sync::{SyncEngine, SyncReporter};

use crate::cli::{ConfigType, ConflictMode};
use crate::commands::SyncOptions;
use crate::interactive::InteractivePrompter;

pub struct ToLocal;

impl ToLocal {
    pub fn execute(
        types: &[ConfigType],
        conflict: &ConflictMode,
        options: &SyncOptions,
    ) -> anyhow::Result<()> {
        if options.verbose {
            println!("Executing to-local command");
            println!("Types: {types:?}");
            println!("Conflict mode: {conflict:?}");
            println!("Dry run: {}", options.dry_run);
        }

        // Determine paths
        let global_path = Self::get_global_path()?;
        let local_path = Self::get_local_path()?;

        if options.verbose {
            println!("Global path: {}", global_path.display());
            println!("Local path: {}", local_path.display());
        }

        // Load configuration from files
        let mut config = options.load_config()?;

        // Merge CLI flags into loaded config (CLI takes precedence)
        Self::merge_cli_flags(&mut config, types, conflict, options.dry_run);

        // Initialize sync engine
        let engine = SyncEngine::new(config, SyncDirection::ToLocal)
            .context("Failed to initialize sync engine")?;

        // Execute sync with optional interactive approval
        let result = if options.yes_all || options.dry_run {
            // Non-interactive: auto-approve all or just preview
            engine
                .sync(&global_path, &local_path)
                .context("Sync operation failed")?
        } else {
            // Interactive mode: prompt for each action
            let mut prompter = InteractivePrompter::new();
            match engine.sync_with_approver(
                &global_path,
                &local_path,
                Some(Box::new(move |action| prompter.prompt(action))),
            ) {
                Ok(result) => result,
                Err(e) => {
                    // Check if this is a user abort (not a real error)
                    let err_msg = e.to_string();
                    if err_msg.contains("User aborted") {
                        eprintln!("\nSync cancelled by user.");
                        std::process::exit(0); // Clean exit, not an error
                    } else {
                        return Err(e).context("Sync operation failed");
                    }
                }
            }
        };

        // Display results
        let summary = SyncReporter::generate_summary(&result);
        println!("{summary}");

        Ok(())
    }

    fn get_global_path() -> anyhow::Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Failed to determine home directory")?;
        Ok(PathBuf::from(home).join(".claude"))
    }

    fn get_local_path() -> anyhow::Result<PathBuf> {
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;
        Ok(current_dir.join(".claude"))
    }

    fn merge_cli_flags(
        config: &mut Config,
        types: &[ConfigType],
        conflict: &ConflictMode,
        dry_run: bool,
    ) {
        // CLI flags override config file settings

        // Set dry run flag (override config)
        if dry_run {
            config.dry_run = Some(true);
        }

        // Set conflict strategy (override config)
        config.conflict_strategy = Some(Self::convert_conflict_mode(conflict));

        // Handle type filters - ADD to config patterns (additive, not replace)
        if !types.is_empty() {
            let cli_patterns = Self::build_type_patterns(types);
            config.include.extend(cli_patterns);
        }
    }

    const fn convert_conflict_mode(mode: &ConflictMode) -> ConflictStrategy {
        match mode {
            ConflictMode::Fail => ConflictStrategy::Fail,
            ConflictMode::Overwrite => ConflictStrategy::Overwrite,
            ConflictMode::Skip => ConflictStrategy::Skip,
            ConflictMode::Newer => ConflictStrategy::Newer,
        }
    }

    fn build_type_patterns(types: &[ConfigType]) -> Vec<String> {
        let mut patterns = Vec::new();

        for config_type in types {
            match config_type {
                ConfigType::Agents => patterns.push("agents/**".to_string()),
                ConfigType::Skills => patterns.push("skills/**".to_string()),
                ConfigType::Commands => patterns.push("commands/**".to_string()),
                ConfigType::All => {
                    patterns.push("**".to_string());
                    break;
                }
            }
        }

        patterns
    }
}
