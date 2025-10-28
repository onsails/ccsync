//! Sync operation reporting and statistics

use super::SyncResult;

/// Sync operation reporter
pub struct SyncReporter;

impl SyncReporter {
    /// Create a new reporter
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Generate a summary report
    #[must_use]
    pub fn generate_summary(result: &SyncResult) -> String {
        let mut output = String::new();

        output.push_str("\n=== Sync Summary ===\n");
        output.push_str(&format!("Created:  {}\n", result.created));
        output.push_str(&format!("Updated:  {}\n", result.updated));
        output.push_str(&format!("Deleted:  {}\n", result.deleted));
        output.push_str(&format!("Skipped:  {}\n", result.skipped));
        output.push_str(&format!("Conflicts: {}\n", result.conflicts));

        if !result.errors.is_empty() {
            output.push_str(&format!("\nErrors ({}):\n", result.errors.len()));
            for error in &result.errors {
                output.push_str(&format!("  - {error}\n"));
            }
        }

        output.push_str(&format!(
            "\nTotal operations: {}\n",
            result.total_operations()
        ));

        if result.is_success() {
            output.push_str("Status: ✓ Success\n");
        } else {
            output.push_str("Status: ✗ Completed with errors\n");
        }

        output
    }
}
