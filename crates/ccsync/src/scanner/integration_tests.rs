//! Integration tests for the scanner module

use std::fs;
use tempfile::TempDir;

use super::{FileFilter, Scanner};

#[test]
fn test_full_scan_all_directory_types() {
    let tmp = TempDir::new().unwrap();

    // Create agents/ directory (flat)
    let agents = tmp.path().join("agents");
    fs::create_dir(&agents).unwrap();
    fs::write(agents.join("agent1.md"), "agent 1").unwrap();
    fs::write(agents.join("agent2.md"), "agent 2").unwrap();
    fs::write(agents.join("ignore.txt"), "ignore").unwrap();

    // Create skills/ directory (one level)
    let skills = tmp.path().join("skills");
    fs::create_dir(&skills).unwrap();

    let skill1 = skills.join("skill-1");
    fs::create_dir(&skill1).unwrap();
    fs::write(skill1.join("SKILL.md"), "skill 1").unwrap();
    fs::write(skill1.join("helper.py"), "helper").unwrap();

    let skill2 = skills.join("skill-2");
    fs::create_dir(&skill2).unwrap();
    fs::write(skill2.join("SKILL.md"), "skill 2").unwrap();

    // Create commands/ directory (recursive)
    let commands = tmp.path().join("commands");
    fs::create_dir(&commands).unwrap();
    fs::write(commands.join("root-command.md"), "root").unwrap();

    let subdir = commands.join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("nested-command.md"), "nested").unwrap();

    // Run scanner
    let filter = FileFilter::new();
    let scanner = Scanner::new(filter, false);
    let result = scanner.scan(tmp.path());

    // Should find: 2 agents, 2 skills, 2 commands = 6 total
    assert_eq!(result.files.len(), 6);

    // Verify each type is found
    assert!(result.files.iter().any(|f| f.path.ends_with("agent1.md")));
    assert!(result.files.iter().any(|f| f.path.ends_with("agent2.md")));
    assert!(result.files.iter().any(|f| f.path.ends_with("skill-1/SKILL.md")));
    assert!(result.files.iter().any(|f| f.path.ends_with("skill-2/SKILL.md")));
    assert!(result.files.iter().any(|f| f.path.ends_with("root-command.md")));
    assert!(result
        .files
        .iter()
        .any(|f| f.path.ends_with("subdir/nested-command.md")));
}

#[test]
fn test_scan_missing_directories() {
    let tmp = TempDir::new().unwrap();

    // Only create agents/ directory
    let agents = tmp.path().join("agents");
    fs::create_dir(&agents).unwrap();
    fs::write(agents.join("agent.md"), "agent").unwrap();

    // Skills and commands directories don't exist

    let filter = FileFilter::new();
    let scanner = Scanner::new(filter, false);
    let result = scanner.scan(tmp.path());

    // Should only find the one agent file
    assert_eq!(result.files.len(), 1);
    assert!(result.files[0].path.ends_with("agent.md"));
}

#[test]
fn test_scan_empty_directories() {
    let tmp = TempDir::new().unwrap();

    // Create empty directories
    fs::create_dir(tmp.path().join("agents")).unwrap();
    fs::create_dir(tmp.path().join("skills")).unwrap();
    fs::create_dir(tmp.path().join("commands")).unwrap();

    let filter = FileFilter::new();
    let scanner = Scanner::new(filter, false);
    let result = scanner.scan(tmp.path());

    assert_eq!(result.files.len(), 0);
}

#[cfg(unix)]
#[test]
fn test_scan_with_symlinks() {
    use std::os::unix::fs as unix_fs;

    let tmp = TempDir::new().unwrap();

    // Create agents/ with a real file
    let agents = tmp.path().join("agents");
    fs::create_dir(&agents).unwrap();

    let target = agents.join("target.md");
    fs::write(&target, "target content").unwrap();

    // Create a symlink to the target
    let link = agents.join("link.md");
    unix_fs::symlink(&target, &link).unwrap();

    let filter = FileFilter::new();
    let scanner = Scanner::new(filter, false);
    let result = scanner.scan(tmp.path());

    // Should resolve both files
    assert_eq!(result.files.len(), 2);
}

#[cfg(unix)]
#[test]
fn test_scan_with_broken_symlink() {
    use std::os::unix::fs as unix_fs;

    let tmp = TempDir::new().unwrap();

    let agents = tmp.path().join("agents");
    fs::create_dir(&agents).unwrap();

    // Create a good file
    fs::write(agents.join("good.md"), "good").unwrap();

    // Create a broken symlink
    let broken = agents.join("broken.md");
    unix_fs::symlink("/nonexistent/file.md", &broken).unwrap();

    let filter = FileFilter::new();
    let scanner = Scanner::new(filter, false);
    let result = scanner.scan(tmp.path());

    // Should find only the good file (broken symlink should be skipped with warning)
    assert_eq!(result.files.len(), 1);
    assert!(result.files[0].path.ends_with("good.md"));
    // Note: Warnings are collected for broken symlinks during resolution
}

#[cfg(unix)]
#[test]
fn test_scan_preserve_symlinks() {
    use std::os::unix::fs as unix_fs;

    let tmp = TempDir::new().unwrap();

    let agents = tmp.path().join("agents");
    fs::create_dir(&agents).unwrap();

    let target = agents.join("target.md");
    fs::write(&target, "target").unwrap();

    let link = agents.join("link.md");
    unix_fs::symlink(&target, &link).unwrap();

    let filter = FileFilter::new();
    let scanner = Scanner::new(filter, true); // preserve_symlinks = true
    let result = scanner.scan(tmp.path());

    // Both should be found, link preserved as-is
    assert_eq!(result.files.len(), 2);
}

#[test]
fn test_scan_cross_platform_paths() {
    let tmp = TempDir::new().unwrap();

    // Test with various path separators and unicode
    let agents = tmp.path().join("agents");
    fs::create_dir(&agents).unwrap();

    // Regular ASCII
    fs::write(agents.join("simple.md"), "simple").unwrap();

    // Unicode characters
    fs::write(agents.join("unicode-中文.md"), "unicode").unwrap();

    let filter = FileFilter::new();
    let scanner = Scanner::new(filter, false);
    let result = scanner.scan(tmp.path());

    assert_eq!(result.files.len(), 2);
}
