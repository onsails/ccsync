//! Interactive prompting for sync operations

use anyhow::{bail, Context, Result};
use ccsync::comparison::FileComparator;
use ccsync::sync::SyncAction;
use dialoguer::Select;

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
        let items = ["yes", "no", "all", "none", "diff", "quit"];

        let selection = Select::new()
            .with_prompt("Proceed?")
            .items(&items)
            .default(0)
            .interact()
            .context("Failed to show prompt")?;

        Ok(match selection {
            0 => UserChoice::Yes,
            1 => UserChoice::No,
            2 => UserChoice::All,
            3 => UserChoice::None,
            4 => UserChoice::Diff,
            5 => UserChoice::Quit,
            _ => unreachable!(),
        })
    }

    /// Describe the action in user-friendly terms
    fn describe_action(action: &SyncAction) -> String {
        match action {
            SyncAction::Create { source: _, dest } => {
                format!(
                    "📄 Create new file:\n  → {}",
                    dest.display()
                )
            }
            SyncAction::Skip { path, reason } => {
                format!(
                    "⊘ Skip file ({}):\n  → {}",
                    reason,
                    path.display()
                )
            }
            SyncAction::Conflict {
                source,
                dest,
                strategy,
                source_newer,
            } => {
                let newer_indicator = if *source_newer { "source newer" } else { "dest newer" };
                format!(
                    "⚠️  Conflict detected ({}):\n  Source: {}\n  Dest:   {}\n  Strategy: {:?}",
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
            SyncAction::Create { source: _, dest } => {
                println!("\n--- New file (no diff available) ---");
                println!("Dest:   {}", dest.display());
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
                        eprintln!("Warning: Failed to generate diff: {e}");
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
