mod cli;
mod commands;

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
            commands::ToLocal::execute(types, conflict, cli.verbose, cli.dry_run)?;
        }
        Commands::ToGlobal { types, conflict } => {
            commands::ToGlobal::execute(types, conflict, cli.verbose, cli.dry_run)?;
        }
        Commands::Status { types } => {
            commands::Status::execute(types, cli.verbose)?;
        }
        Commands::Diff { types } => {
            commands::Diff::execute(types, cli.verbose)?;
        }
        Commands::Config => {
            commands::Config::execute(cli.verbose)?;
        }
    }

    Ok(())
}
