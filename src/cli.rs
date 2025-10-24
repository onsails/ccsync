//! CLI argument parsing and command dispatch module.
//!
//! This module handles all command-line interface parsing using the clap crate.
//! It defines the main commands (to-local, to-global, status, diff, config) and
//! their associated flags and options.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// ccsync - Bidirectional sync tool for Claude Code configuration files
#[derive(Parser, Debug)]
#[command(name = "ccsync")]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// The command to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Override global configuration directory path
    #[arg(long, global = true, value_name = "PATH")]
    pub global_path: Option<PathBuf>,

    /// Override local configuration directory path
    #[arg(long, global = true, value_name = "PATH")]
    pub local_path: Option<PathBuf>,

    /// Path to custom configuration file
    #[arg(long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Disable loading configuration files
    #[arg(long, global = true)]
    pub no_config: bool,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Copy configuration files from global to local directory
    ToLocal {
        /// Filter by configuration type
        #[arg(short = 't', long, value_enum)]
        r#type: Vec<ConfigType>,

        /// Run without making any changes
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Skip interactive prompts
        #[arg(long)]
        non_interactive: bool,

        /// Automatically approve all changes
        #[arg(short = 'y', long)]
        yes_all: bool,

        /// Conflict resolution strategy
        #[arg(long, value_enum, default_value = "fail")]
        conflict: ConflictStrategy,

        /// Preserve symlinks instead of following them
        #[arg(long)]
        preserve_symlinks: bool,
    },

    /// Copy configuration files from local to global directory
    ToGlobal {
        /// Filter by configuration type
        #[arg(short = 't', long, value_enum)]
        r#type: Vec<ConfigType>,

        /// Run without making any changes
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Skip interactive prompts
        #[arg(long)]
        non_interactive: bool,

        /// Automatically approve all changes
        #[arg(short = 'y', long)]
        yes_all: bool,

        /// Conflict resolution strategy
        #[arg(long, value_enum, default_value = "fail")]
        conflict: ConflictStrategy,

        /// Preserve symlinks instead of following them
        #[arg(long)]
        preserve_symlinks: bool,
    },

    /// Show current synchronization status
    Status {
        /// Filter by configuration type
        #[arg(short = 't', long, value_enum)]
        r#type: Vec<ConfigType>,

        /// Show detailed status information
        #[arg(short = 'l', long)]
        long: bool,
    },

    /// Show differences between local and global configurations
    Diff {
        /// Filter by configuration type
        #[arg(short = 't', long, value_enum)]
        r#type: Vec<ConfigType>,

        /// Show unified diff format
        #[arg(short = 'u', long)]
        unified: bool,
    },

    /// Show or validate configuration
    Config {
        /// Show merged configuration from all sources
        #[arg(long)]
        show: bool,

        /// Validate configuration file syntax
        #[arg(long)]
        validate: bool,

        /// Path to configuration file to validate
        #[arg(long, value_name = "FILE", requires = "validate")]
        file: Option<PathBuf>,
    },
}

/// Configuration file types
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum ConfigType {
    /// Slash commands
    Commands,
    /// Skills
    Skills,
    /// Subagents
    Subagents,
    /// All configuration types
    All,
}

/// Conflict resolution strategies
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Fail on conflicts (default)
    Fail,
    /// Overwrite destination with source
    Overwrite,
    /// Skip conflicting files
    Skip,
    /// Use newer file based on modification time
    Newer,
}

impl Cli {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_verify() {
        // Verify that the CLI is correctly configured
        Cli::command().debug_assert();
    }

    #[test]
    fn test_to_local_basic() {
        let args = vec!["ccsync", "to-local"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::ToLocal {
                dry_run,
                non_interactive,
                yes_all,
                ..
            } => {
                assert!(!dry_run);
                assert!(!non_interactive);
                assert!(!yes_all);
            }
            _ => panic!("Expected ToLocal command"),
        }
    }

    #[test]
    fn test_to_local_with_flags() {
        let args = vec![
            "ccsync",
            "to-local",
            "--dry-run",
            "--yes-all",
            "-t",
            "commands",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::ToLocal {
                dry_run,
                yes_all,
                r#type,
                ..
            } => {
                assert!(dry_run);
                assert!(yes_all);
                assert_eq!(r#type.len(), 1);
                assert_eq!(r#type[0], ConfigType::Commands);
            }
            _ => panic!("Expected ToLocal command"),
        }
    }

    #[test]
    fn test_to_global_with_conflict() {
        let args = vec!["ccsync", "to-global", "--conflict", "newer"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::ToGlobal { conflict, .. } => {
                assert_eq!(conflict, ConflictStrategy::Newer);
            }
            _ => panic!("Expected ToGlobal command"),
        }
    }

    #[test]
    fn test_status_command() {
        let args = vec!["ccsync", "status", "-l"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Status { long, .. } => {
                assert!(long);
            }
            _ => panic!("Expected Status command"),
        }
    }

    #[test]
    fn test_diff_command() {
        let args = vec!["ccsync", "diff", "-u", "-t", "skills", "-t", "commands"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Diff { unified, r#type } => {
                assert!(unified);
                assert_eq!(r#type.len(), 2);
            }
            _ => panic!("Expected Diff command"),
        }
    }

    #[test]
    fn test_config_command() {
        let args = vec!["ccsync", "config", "--show"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Config { show, validate, .. } => {
                assert!(show);
                assert!(!validate);
            }
            _ => panic!("Expected Config command"),
        }
    }

    #[test]
    fn test_global_flags() {
        let args = vec!["ccsync", "--verbose", "--no-config", "status"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.verbose);
        assert!(cli.no_config);
        assert!(matches!(cli.command, Commands::Status { .. }));
    }

    #[test]
    fn test_path_overrides() {
        let args = vec![
            "ccsync",
            "--global-path",
            "/custom/global",
            "--local-path",
            "/custom/local",
            "status",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.global_path, Some(PathBuf::from("/custom/global")));
        assert_eq!(cli.local_path, Some(PathBuf::from("/custom/local")));
    }

    #[test]
    fn test_config_file_override() {
        let args = vec!["ccsync", "--config", "/path/to/config.yaml", "status"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.config, Some(PathBuf::from("/path/to/config.yaml")));
    }

    #[test]
    fn test_multiple_types() {
        let args = vec![
            "ccsync",
            "to-local",
            "-t",
            "commands",
            "-t",
            "skills",
            "-t",
            "subagents",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::ToLocal { r#type, .. } => {
                assert_eq!(r#type.len(), 3);
                assert!(r#type.contains(&ConfigType::Commands));
                assert!(r#type.contains(&ConfigType::Skills));
                assert!(r#type.contains(&ConfigType::Subagents));
            }
            _ => panic!("Expected ToLocal command"),
        }
    }

    #[test]
    fn test_invalid_command() {
        let args = vec!["ccsync", "invalid-command"];
        let result = Cli::try_parse_from(args);

        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_command() {
        let args = vec!["ccsync"];
        let result = Cli::try_parse_from(args);

        // Should fail because a command is required
        assert!(result.is_err());
    }

    #[test]
    fn test_conflict_strategy_values() {
        let strategies = vec![
            ("fail", ConflictStrategy::Fail),
            ("overwrite", ConflictStrategy::Overwrite),
            ("skip", ConflictStrategy::Skip),
            ("newer", ConflictStrategy::Newer),
        ];

        for (name, expected) in strategies {
            let args = vec!["ccsync", "to-local", "--conflict", name];
            let cli = Cli::try_parse_from(args).unwrap();

            match cli.command {
                Commands::ToLocal { conflict, .. } => {
                    assert_eq!(conflict, expected);
                }
                _ => panic!("Expected ToLocal command"),
            }
        }
    }

    #[test]
    fn test_preserve_symlinks() {
        let args = vec!["ccsync", "to-local", "--preserve-symlinks"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::ToLocal {
                preserve_symlinks, ..
            } => {
                assert!(preserve_symlinks);
            }
            _ => panic!("Expected ToLocal command"),
        }
    }
}
