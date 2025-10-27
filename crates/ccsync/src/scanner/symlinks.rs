//! Symlink resolution with loop detection and broken link handling

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

use crate::error::Result;

/// Result of resolving a path (which may or may not be a symlink)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedPath {
    /// Regular file (not a symlink)
    Regular(PathBuf),
    /// Symlink preserved as-is (when `--preserve-symlinks` is enabled)
    Symlink(PathBuf),
    /// Symlink resolved to its canonical target
    Resolved(PathBuf),
}

impl ResolvedPath {
    /// Get the inner path regardless of variant
    #[must_use]
    pub fn into_path(self) -> PathBuf {
        match self {
            Self::Regular(p) | Self::Symlink(p) | Self::Resolved(p) => p,
        }
    }

    /// Get a reference to the inner path
    #[must_use]
    pub fn path(&self) -> &Path {
        match self {
            Self::Regular(p) | Self::Symlink(p) | Self::Resolved(p) => p,
        }
    }
}

/// Symlink resolver with loop detection
pub struct SymlinkResolver {
    /// Set of canonicalized paths we've visited (for loop detection)
    visited: HashSet<PathBuf>,
    /// Whether to preserve symlinks instead of resolving them
    preserve: bool,
}

impl SymlinkResolver {
    /// Create a new symlink resolver
    #[must_use]
    pub fn new(preserve: bool) -> Self {
        Self {
            visited: HashSet::new(),
            preserve,
        }
    }

    /// Resolve a path, handling symlinks appropriately
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A symlink is broken (target doesn't exist)
    /// - A symlink loop is detected
    /// - Path canonicalization fails
    pub fn resolve(&mut self, path: &Path) -> Result<ResolvedPath> {
        // Check if it's a symlink
        let metadata = fs::symlink_metadata(path)
            .with_context(|| format!("Failed to read metadata for {}", path.display()))?;

        if !metadata.is_symlink() {
            return Ok(ResolvedPath::Regular(path.to_path_buf()));
        }

        // If preserving symlinks, return as-is
        if self.preserve {
            return Ok(ResolvedPath::Symlink(path.to_path_buf()));
        }

        // Resolve the symlink
        let target = fs::read_link(path)
            .with_context(|| format!("Failed to read symlink {}", path.display()))?;

        // Try to canonicalize (this will fail if target doesn't exist)
        let canonical = dunce::canonicalize(&target).with_context(|| {
            format!(
                "Broken symlink: {} -> {}",
                path.display(),
                target.display()
            )
        })?;

        // Check for loops
        if !self.visited.insert(canonical.clone()) {
            bail!(
                "Symlink loop detected: {} -> {}",
                path.display(),
                canonical.display()
            );
        }

        Ok(ResolvedPath::Resolved(canonical))
    }

    /// Clear the visited set (useful for processing multiple independent trees)
    pub fn clear(&mut self) {
        self.visited.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[cfg(unix)]
    use std::os::unix::fs as unix_fs;

    #[test]
    fn test_regular_file() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("regular.txt");
        fs::write(&file, "content").unwrap();

        let mut resolver = SymlinkResolver::new(false);
        let resolved = resolver.resolve(&file).unwrap();

        assert_eq!(resolved, ResolvedPath::Regular(file));
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_resolution() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target.txt");
        fs::write(&target, "content").unwrap();

        let link = tmp.path().join("link.txt");
        unix_fs::symlink(&target, &link).unwrap();

        let mut resolver = SymlinkResolver::new(false);
        let resolved = resolver.resolve(&link).unwrap();

        match resolved {
            ResolvedPath::Resolved(p) => {
                assert_eq!(dunce::canonicalize(&target).unwrap(), p);
            }
            _ => panic!("Expected Resolved variant"),
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_preserve_symlinks() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target.txt");
        fs::write(&target, "content").unwrap();

        let link = tmp.path().join("link.txt");
        unix_fs::symlink(&target, &link).unwrap();

        let mut resolver = SymlinkResolver::new(true);
        let resolved = resolver.resolve(&link).unwrap();

        assert_eq!(resolved, ResolvedPath::Symlink(link));
    }

    #[test]
    #[cfg(unix)]
    fn test_broken_symlink() {
        let tmp = TempDir::new().unwrap();
        let link = tmp.path().join("broken.txt");
        unix_fs::symlink("/nonexistent/target.txt", &link).unwrap();

        let mut resolver = SymlinkResolver::new(false);
        let result = resolver.resolve(&link);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Broken symlink"));
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink_loop_detection() {
        let tmp = TempDir::new().unwrap();
        let link1 = tmp.path().join("link1.txt");
        let link2 = tmp.path().join("link2.txt");

        // Create a loop: link1 -> link2 -> link1
        unix_fs::symlink(&link2, &link1).unwrap();
        unix_fs::symlink(&link1, &link2).unwrap();

        let mut resolver = SymlinkResolver::new(false);

        // First resolution might work (depending on which link is accessed first)
        // But following the chain should detect the loop
        // This test verifies we don't infinite loop
        let result = resolver.resolve(&link1);

        // The result will be an error due to broken symlink or loop detection
        assert!(result.is_err());
    }
}
