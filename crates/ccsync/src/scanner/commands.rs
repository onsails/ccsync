//! Recursive directory scanning for commands/
//!
//! Commands can use subdirectories for organization. Subdirectories are used
//! for organization and appear in the command description, but they do not
//! affect the command name itself.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::Result;

/// Scan the commands/ directory recursively for `.md` files
///
/// # Errors
///
/// Returns an error if directory traversal fails due to permission issues
/// or I/O errors.
pub fn scan(base: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(base).follow_links(false) {
        // We handle symlinks separately
        let entry = entry?; // Propagate errors instead of silently ignoring
        let path = entry.path();

        if entry.file_type().is_file() && path.extension().is_some_and(|ext| ext == "md") {
            files.push(path.to_path_buf());
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
    fn test_commands_recursive_structure() {
        let tmp = TempDir::new().unwrap();
        let commands_dir = tmp.path().join("commands");
        fs::create_dir(&commands_dir).unwrap();

        // Create commands at root level
        fs::write(commands_dir.join("optimize.md"), "optimize").unwrap();

        // Create nested commands
        let frontend = commands_dir.join("frontend");
        fs::create_dir(&frontend).unwrap();
        fs::write(frontend.join("component.md"), "component").unwrap();

        let backend = commands_dir.join("backend");
        fs::create_dir(&backend).unwrap();
        let api = backend.join("api");
        fs::create_dir(&api).unwrap();
        fs::write(api.join("endpoint.md"), "endpoint").unwrap();

        // Non-md file (should be ignored)
        fs::write(commands_dir.join("ignore.txt"), "ignore").unwrap();

        let files = scan(&commands_dir).unwrap();

        assert_eq!(files.len(), 3);
        assert!(files
            .iter()
            .any(|p| p.file_name().unwrap() == "optimize.md"));
        assert!(files
            .iter()
            .any(|p| p.ends_with("frontend/component.md")));
        assert!(files
            .iter()
            .any(|p| p.ends_with("backend/api/endpoint.md")));
    }

    #[test]
    fn test_commands_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let commands_dir = tmp.path().join("commands");
        fs::create_dir(&commands_dir).unwrap();

        let files = scan(&commands_dir).unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_commands_mixed_content() {
        let tmp = TempDir::new().unwrap();
        let commands_dir = tmp.path().join("commands");
        fs::create_dir(&commands_dir).unwrap();

        // MD files
        fs::write(commands_dir.join("command1.md"), "cmd1").unwrap();

        // Non-MD files (should be ignored)
        fs::write(commands_dir.join("readme.txt"), "readme").unwrap();
        fs::write(commands_dir.join("config.json"), "{}").unwrap();

        // Subdirectory with mixed files
        let subdir = commands_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("command2.md"), "cmd2").unwrap();
        fs::write(subdir.join("script.sh"), "#!/bin/bash").unwrap();

        let files = scan(&commands_dir).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files
            .iter()
            .all(|p| p.extension().unwrap() == "md"));
    }
}
