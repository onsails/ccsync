//! Sync operation reporting and statistics

use std::fmt::Write;

use super::SyncResult;

/// Sync operation reporter
pub struct SyncReporter;

impl SyncReporter {
    /// Generate a summary report
    #[must_use]
    pub fn generate_summary(result: &SyncResult) -> String {
        let mut output = String::new();

        output.push_str("\n=== Sync Summary ===\n");
        let _ = writeln!(output, "Created:  {}", result.created);
        let _ = writeln!(output, "Updated:  {}", result.updated);
        let _ = writeln!(output, "Deleted:  {}", result.deleted);

        // Show skipped count with reasons breakdown
        if result.skipped > 0 && !result.skip_reasons.is_empty() {
            let _ = write!(output, "Skipped:  {}", result.skipped);
            let mut reasons: Vec<_> = result.skip_reasons.iter().collect();
            reasons.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
            for (reason, count) in reasons {
                let _ = write!(output, " ({reason}: {count})");
            }
            let _ = writeln!(output);
        } else {
            let _ = writeln!(output, "Skipped:  {}", result.skipped);
        }

        let _ = writeln!(output, "Conflicts: {}", result.conflicts);

        if !result.errors.is_empty() {
            let _ = writeln!(output, "\nErrors ({}):", result.errors.len());
            for error in &result.errors {
                let _ = writeln!(output, "  - {error}");
            }
        }

        let _ = writeln!(output, "\nTotal operations: {}", result.total_operations());

        if result.is_success() {
            output.push_str("Status: ✓ Success\n");
        } else {
            output.push_str("Status: ✗ Completed with errors\n");
        }

        output
    }
}
