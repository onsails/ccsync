use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Claude Configuration Synchronization Tool
///
/// Sync agents, skills, and commands between global (~/.claude) and project-specific (.claude) directories
#[derive(Parser, Debug)]
#[command(name = "ccsync")]
#[command(about, long_about = None, version)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Accept all items in interactive mode without prompting
    #[arg(long, global = true)]
    pub yes_all: bool,

    /// Preview changes without executing (dry-run)
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Override global path (default: ~/.claude)
    #[arg(long, global = true, value_name = "PATH")]
    pub global_path: Option<PathBuf>,

    /// Override local path (default: ./.claude)
    #[arg(long, global = true, value_name = "PATH")]
    pub local_path: Option<PathBuf>,

    /// Use specific config file
    #[arg(long, global = true, value_name = "PATH", conflicts_with = "no_config")]
    pub config: Option<PathBuf>,

    /// Ignore all config files
    #[arg(long, global = true, conflicts_with = "config")]
    pub no_config: bool,

    /// Preserve symlinks instead of following them
    #[arg(long, global = true)]
    pub preserve_symlinks: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Sync from global (~/.claude) to local (./.claude)
    ToLocal {
        /// Filter by configuration type(s)
        #[arg(short = 't', long = "type", value_enum)]
        types: Vec<ConfigType>,

        /// Conflict resolution strategy
        #[arg(long, value_enum, default_value = "fail")]
        conflict: ConflictMode,
    },

    /// Sync from local (./.claude) to global (~/.claude)
    ToGlobal {
        /// Filter by configuration type(s)
        #[arg(short = 't', long = "type", value_enum)]
        types: Vec<ConfigType>,

        /// Conflict resolution strategy
        #[arg(long, value_enum, default_value = "fail")]
        conflict: ConflictMode,
    },

    /// Show sync status without making changes
    Status {
        /// Filter by configuration type(s)
        #[arg(short = 't', long = "type", value_enum)]
        types: Vec<ConfigType>,
    },

    /// Display detailed differences between configurations
    Diff {
        /// Filter by configuration type(s)
        #[arg(short = 't', long = "type", value_enum)]
        types: Vec<ConfigType>,
    },

    /// Show active configuration and debug settings
    Config,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ConfigType {
    /// Agent configurations
    Agents,
    /// Skill configurations
    Skills,
    /// Command configurations
    Commands,
    /// All configuration types
    All,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ConflictMode {
    /// Exit on conflicts (default)
    Fail,
    /// Overwrite with warning
    Overwrite,
    /// Skip conflicting files
    Skip,
    /// Keep newer file
    Newer,
}
