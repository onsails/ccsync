//! One-level subdirectory scanning for skills/
//!
//! Skills are organized as `skills/skill-name/SKILL.md` where each skill
//! has its own subdirectory containing a required `SKILL.md` file plus
//! optional supporting files.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;

/// Scan the skills/ directory for skill directories (one level deep)
///
/// Returns paths to skill directories that contain a `SKILL.md` file.
/// The entire directory will be synced, including all supporting files.
///
/// # Errors
///
/// Returns an error if the directory cannot be read or if there are
/// permission issues.
pub fn scan(base: &Path) -> Result<Vec<PathBuf>> {
    let mut directories = Vec::new();

    for entry in fs::read_dir(base)? {
        let entry = entry?;
        let path = entry.path();

        // Only process directories
        if path.is_dir() {
            let skill_md = path.join("SKILL.md");
            if skill_md.exists() && skill_md.is_file() {
                // Return the directory path, not the SKILL.md file
                directories.push(path);
            }
        }
    }

    Ok(directories)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_skills_one_level_structure() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir(&skills_dir).unwrap();

        // Create skill directories with SKILL.md
        let skill1 = skills_dir.join("skill-1");
        fs::create_dir(&skill1).unwrap();
        fs::write(skill1.join("SKILL.md"), "skill 1").unwrap();
        fs::write(skill1.join("helper.py"), "helper").unwrap();

        let skill2 = skills_dir.join("skill-2");
        fs::create_dir(&skill2).unwrap();
        fs::write(skill2.join("SKILL.md"), "skill 2").unwrap();

        // Skill without SKILL.md (should be ignored)
        let skill3 = skills_dir.join("skill-3");
        fs::create_dir(&skill3).unwrap();
        fs::write(skill3.join("README.md"), "readme").unwrap();

        // File directly in skills/ (should be ignored)
        fs::write(skills_dir.join("direct.md"), "ignore").unwrap();

        let directories = scan(&skills_dir).unwrap();

        assert_eq!(directories.len(), 2);
        assert!(directories.iter().any(|p| p.ends_with("skill-1")));
        assert!(directories.iter().any(|p| p.ends_with("skill-2")));
    }

    #[test]
    fn test_skills_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir(&skills_dir).unwrap();

        let directories = scan(&skills_dir).unwrap();
        assert_eq!(directories.len(), 0);
    }
}
