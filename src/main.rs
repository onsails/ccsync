//! ccsync - Bidirectional sync tool for Claude Code configuration files
//!
//! This tool synchronizes configuration files between global (~/.claude) and
//! project-local (./.claude) directories, supporting commands, skills, and subagents.

mod cli;
mod errors;
mod models;
mod platform;
mod services;

use anyhow::Result;

fn main() -> Result<()> {
    // Check platform compatibility
    if !platform::is_supported_platform() {
        eprintln!("Error: Unsupported platform");
        std::process::exit(1);
    }

    // Parse CLI arguments
    let cli = cli::Cli::parse_args();

    // Enable verbose logging if requested
    if cli.verbose {
        println!("Verbose mode enabled");
        println!("Platform: {}", platform::platform_name());
    }

    // Dispatch command
    match &cli.command {
        cli::Commands::ToLocal { .. } => {
            println!("Executing to-local command...");
            // TODO: Implement in Task 6
        }
        cli::Commands::ToGlobal { .. } => {
            println!("Executing to-global command...");
            // TODO: Implement in Task 6
        }
        cli::Commands::Status { .. } => {
            println!("Executing status command...");
            // TODO: Implement in Task 14
        }
        cli::Commands::Diff { .. } => {
            println!("Executing diff command...");
            // TODO: Implement in Task 15
        }
        cli::Commands::Config { show, validate, .. } => {
            if *show {
                println!("Showing configuration...");
            }
            if *validate {
                println!("Validating configuration...");
            }
            // TODO: Implement in Task 11
        }
    }

    Ok(())
}
