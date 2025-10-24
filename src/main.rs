//! ccsync - Bidirectional sync tool for Claude Code configuration files
//!
//! This tool synchronizes configuration files between global (~/.claude) and
//! project-local (./.claude) directories, supporting commands, skills, and subagents.

mod cli;
mod errors;
mod models;
mod platform;
mod services;

use errors::Result;

fn main() -> Result<()> {
    // Check platform compatibility
    if !platform::is_supported_platform() {
        eprintln!("Error: Unsupported platform");
        std::process::exit(1);
    }

    // Parse CLI arguments
    let _cli = cli::Cli::parse();

    // TODO: Implement command dispatch in Task 2
    println!("ccsync v{}", env!("CARGO_PKG_VERSION"));
    println!("Platform: {}", platform::platform_name());
    println!("Bidirectional sync tool for Claude Code configuration files");

    Ok(())
}
