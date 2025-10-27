//! Flat directory scanning for agents/
//!
//! Agents are stored as flat `.md` files directly in the `agents/` directory.
//! No subdirectories are traversed.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;

/// Scan the agents/ directory for `.md` files (flat structure)
///
/// # Errors
///
/// Returns an error if the directory cannot be read or if there are
/// permission issues.
pub fn scan(base: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();

        // Only include files (not directories)
        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_agents_flat_structure() {
        let tmp = TempDir::new().unwrap();
        let agents_dir = tmp.path().join("agents");
        fs::create_dir(&agents_dir).unwrap();

        // Create test files
        fs::write(agents_dir.join("agent1.md"), "agent 1").unwrap();
        fs::write(agents_dir.join("agent2.md"), "agent 2").unwrap();
        fs::write(agents_dir.join("not-md.txt"), "ignore").unwrap();

        // Create subdirectory (should be ignored)
        fs::create_dir(agents_dir.join("subdir")).unwrap();
        fs::write(agents_dir.join("subdir").join("nested.md"), "ignore").unwrap();

        let files = scan(&agents_dir).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files
            .iter()
            .any(|p| p.file_name().unwrap() == "agent1.md"));
        assert!(files
            .iter()
            .any(|p| p.file_name().unwrap() == "agent2.md"));
    }

    #[test]
    fn test_agents_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let agents_dir = tmp.path().join("agents");
        fs::create_dir(&agents_dir).unwrap();

        let files = scan(&agents_dir).unwrap();
        assert_eq!(files.len(), 0);
    }
}
