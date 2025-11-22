//! Interactive prompting for sync operations

use anyhow::{bail, Context, Result};
use ccsync_core::comparison::{DiffGenerator, DirectoryComparator, FileComparator};
use ccsync_core::sync::SyncAction;
use dialoguer::console::Term;

/// User's choice for a sync action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserChoice {
    /// Approve this action
    Yes,
    /// Skip this action
    No,
    /// Approve this and all remaining actions
    All,
    /// Skip this and all remaining actions
    None,
    /// Show diff and re-prompt
    Diff,
    /// Quit immediately
    Quit,
}

/// Session state tracking for "all" or "none" choices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionDecision {
    /// Ask for each action
    AskEach,
    /// Auto-approve all remaining
    ApproveAll,
    /// Auto-skip all remaining
    SkipAll,
}

/// Interactive prompter for sync operations
pub struct InteractivePrompter {
    session_state: SessionDecision,
}

impl InteractivePrompter {
    /// Create a new interactive prompter
    #[must_use]
    pub const fn new() -> Self {
        Self {
            session_state: SessionDecision::AskEach,
        }
    }

    /// Prompt user for approval of a sync action
    ///
    /// Returns true to proceed with the action, false to skip it.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - User selects "quit"
    /// - Terminal interaction fails
    pub fn prompt(&mut self, action: &SyncAction) -> Result<bool> {
        // Check session state first
        match self.session_state {
            SessionDecision::ApproveAll => return Ok(true),
            SessionDecision::SkipAll => return Ok(false),
            SessionDecision::AskEach => {
                // Continue to prompt
            }
        }

        // Show what action will be performed
        let description = Self::describe_action(action);
        println!("\n{description}");

        // Prompt with options
        loop {
            let choice = Self::show_prompt()?;

            match choice {
                UserChoice::Yes => return Ok(true),
                UserChoice::No => return Ok(false),
                UserChoice::All => {
                    self.session_state = SessionDecision::ApproveAll;
                    return Ok(true);
                }
                UserChoice::None => {
                    self.session_state = SessionDecision::SkipAll;
                    return Ok(false);
                }
                UserChoice::Diff => {
                    Self::show_diff(action);
                    // Loop back to re-prompt
                }
                UserChoice::Quit => {
                    bail!("User aborted sync operation");
                }
            }
        }
    }

    /// Show the selection prompt
    fn show_prompt() -> Result<UserChoice> {
        let term = Term::stderr();

        print!("Proceed? [y/n/a/s/d/q] (yes/no/all/skip-all/diff/quit): ");
        std::io::Write::flush(&mut std::io::stdout()).context("Failed to flush stdout")?;

        loop {
            let key = term
                .read_char()
                .context("Failed to read user input")?;

            // Echo the character
            println!("{key}");

            match key {
                'y' | 'Y' => return Ok(UserChoice::Yes),
                'n' | 'N' => return Ok(UserChoice::No),
                'a' | 'A' => return Ok(UserChoice::All),
                's' | 'S' => return Ok(UserChoice::None),
                'd' | 'D' => return Ok(UserChoice::Diff),
                'q' | 'Q' => return Ok(UserChoice::Quit),
                '\n' | '\r' => {
                    // Enter key - default to no
                    println!("(defaulted to 'no')");
                    return Ok(UserChoice::No);
                }
                _ => {
                    println!("Invalid key. Press y/n/a/s/d/q");
                    print!("Proceed? [y/n/a/s/d/q]: ");
                    std::io::Write::flush(&mut std::io::stdout())
                        .context("Failed to flush stdout")?;
                }
            }
        }
    }

    /// Describe the action in user-friendly terms
    fn describe_action(action: &SyncAction) -> String {
        match action {
            SyncAction::Create { source, dest } => {
                format!(
                    "📄 Create new file:\n  Source: {}\n  Dest:   {}",
                    source.display(),
                    dest.display()
                )
            }
            SyncAction::CreateDirectory { source, dest } => {
                format!(
                    "📁 Create new directory:\n  Source: {}\n  Dest:   {}",
                    source.display(),
                    dest.display()
                )
            }
            SyncAction::Skip { path, reason } => {
                format!("⊘ Skip file ({}):\n  → {}", reason, path.display())
            }
            SyncAction::Conflict {
                source,
                dest,
                strategy,
                source_newer,
            } => {
                let newer_indicator = if *source_newer {
                    "source newer"
                } else {
                    "dest newer"
                };
                format!(
                    "⚠️  Conflict detected ({}):\n  Source: {}\n  Dest:   {}\n  Strategy: {:?}",
                    newer_indicator,
                    source.display(),
                    dest.display(),
                    strategy
                )
            }
            SyncAction::DirectoryConflict {
                source,
                dest,
                strategy,
                source_newer,
            } => {
                let newer_indicator = if *source_newer {
                    "source newer"
                } else {
                    "dest newer"
                };
                format!(
                    "⚠️  Directory conflict detected ({}):\n  Source: {}\n  Dest:   {}\n  Strategy: {:?}",
                    newer_indicator,
                    source.display(),
                    dest.display(),
                    strategy
                )
            }
        }
    }

    /// Show a diff for the action
    fn show_diff(action: &SyncAction) {
        match action {
            SyncAction::Create { source, dest } => {
                // Show new file content as additions
                println!("\n--- New file ---");
                println!("+++ {}", dest.display());

                match std::fs::read_to_string(source) {
                    Ok(content) => {
                        println!();
                        for line in content.lines() {
                            println!("\x1b[32m+{line}\x1b[0m");
                        }
                    }
                    Err(e) => {
                        eprintln!("\nWarning: Failed to read file: {e}");
                        eprintln!("Source: {}", source.display());
                    }
                }
            }
            SyncAction::CreateDirectory { source, dest } => {
                println!("\n--- New directory ---");
                println!("+++ {} (from {})", dest.display(), source.display());
            }
            SyncAction::Skip { .. } => {
                println!("\n--- No diff (file will be skipped) ---");
            }
            SyncAction::Conflict { source, dest, .. } => {
                // Generate and display diff
                match FileComparator::generate_diff(source, dest) {
                    Ok(diff) => {
                        println!("\n{diff}");
                    }
                    Err(e) => {
                        eprintln!("\nWarning: Failed to generate diff: {e}");
                        eprintln!("Source: {}", source.display());
                        eprintln!("Dest:   {}", dest.display());
                        eprintln!("You can inspect these files manually.");
                    }
                }
            }
            SyncAction::DirectoryConflict { source, dest, .. } => {
                // Compare directories to get detailed diff
                match DirectoryComparator::compare(source, dest) {
                    Ok(comparison) => {
                        // Extract skill name from source path
                        let skill_name = source
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        match DiffGenerator::generate_directory_summary(
                            &comparison,
                            source,
                            dest,
                            skill_name,
                        ) {
                            Ok(summary) => {
                                println!("{summary}");
                            }
                            Err(e) => {
                                eprintln!("\nWarning: Failed to generate directory summary: {e}");
                                println!("\n--- Directory conflict ---");
                                println!("+++ {}", dest.display());
                                println!("--- {}", source.display());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("\nWarning: Failed to compare directories: {e}");
                        println!("\n--- Directory conflict ---");
                        println!("+++ {}", dest.display());
                        println!("--- {}", source.display());
                    }
                }
            }
        }
    }
}

impl Default for InteractivePrompter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_decision_states() {
        assert_eq!(SessionDecision::AskEach, SessionDecision::AskEach);
        assert_ne!(SessionDecision::AskEach, SessionDecision::ApproveAll);
    }

    #[test]
    fn test_user_choice_variants() {
        assert_eq!(UserChoice::Yes, UserChoice::Yes);
        assert_ne!(UserChoice::Yes, UserChoice::No);
    }

    #[test]
    fn test_prompter_creation() {
        let _prompter = InteractivePrompter::new();
        let _default_prompter = InteractivePrompter::default();
    }
}
