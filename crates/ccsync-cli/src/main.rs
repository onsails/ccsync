mod cli;
mod commands;
mod interactive;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    // Set up Ctrl+C handler for graceful interruption
    ctrlc::set_handler(|| {
        eprintln!("\n\nInterrupted by user (Ctrl+C)");
        std::process::exit(130); // Standard exit code for SIGINT
    })
    .context("Failed to set Ctrl+C handler")?;

    let cli = Cli::parse();

    if cli.verbose {
        println!("Verbose mode enabled");
        println!("Dry run: {}", cli.dry_run);
        println!("Yes all: {}", cli.yes_all);
    }

    match &cli.command {
        Commands::ToLocal { types, conflict } => {
            commands::ToLocal::execute(types, conflict, cli.verbose, cli.dry_run, cli.yes_all)
                .context("Failed to execute to-local command")?;
        }
        Commands::ToGlobal { types, conflict } => {
            commands::ToGlobal::execute(types, conflict, cli.verbose, cli.dry_run, cli.yes_all)
                .context("Failed to execute to-global command")?;
        }
        Commands::Status { types } => {
            commands::Status::execute(types, cli.verbose)
                .context("Failed to execute status command")?;
        }
        Commands::Diff { types } => {
            commands::Diff::execute(types, cli.verbose)
                .context("Failed to execute diff command")?;
        }
        Commands::Config => {
            commands::Config::execute(cli.verbose).context("Failed to execute config command")?;
        }
    }

    Ok(())
}
