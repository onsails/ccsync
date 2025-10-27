mod cli;
mod commands;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Verbose mode enabled");
        println!("Dry run: {}", cli.dry_run);
        println!("Non-interactive: {}", cli.non_interactive);
    }

    match &cli.command {
        Commands::ToLocal { types, conflict } => {
            commands::ToLocal::execute(types, conflict, cli.verbose, cli.dry_run)
                .context("Failed to execute to-local command")?;
        }
        Commands::ToGlobal { types, conflict } => {
            commands::ToGlobal::execute(types, conflict, cli.verbose, cli.dry_run)
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
            commands::Config::execute(cli.verbose)
                .context("Failed to execute config command")?;
        }
    }

    Ok(())
}
