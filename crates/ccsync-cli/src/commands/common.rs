//! Common types and utilities for command execution

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
}
