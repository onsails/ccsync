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
    /// Whether to preserve symlinks instead of resolving them
    preserve: bool,
}

impl SymlinkResolver {
    /// Create a new symlink resolver
    #[must_use]
    pub const fn new(preserve: bool) -> Self {
        Self { preserve }
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

        // Resolve the symlink with loop detection
        Self::resolve_symlink_chain(path)
    }

    /// Resolve a symlink chain, detecting loops
    fn resolve_symlink_chain(path: &Path) -> Result<ResolvedPath> {
        let mut visited = HashSet::new();
        let mut current = path.to_path_buf();

        // Follow the symlink chain
        loop {
            // Canonicalize the current path to detect loops
            let Ok(canonical) = dunce::canonicalize(&current) else {
                // If canonicalization fails, try to get more context
                let target = fs::read_link(&current).with_context(|| {
                    format!("Failed to read symlink {}", current.display())
                })?;

                // Handle relative symlinks
                let absolute_target = if target.is_relative() {
                    current
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(&target)
                } else {
                    target
                };

                bail!(
                    "Broken symlink: {} -> {}",
                    path.display(),
                    absolute_target.display()
                );
            };

            // Check for loops
            if !visited.insert(canonical.clone()) {
                bail!("Symlink loop detected: {}", path.display());
            }

            // Check if the canonical path is still a symlink
            let metadata = fs::symlink_metadata(&canonical)
                .with_context(|| format!("Failed to read metadata for {}", canonical.display()))?;

            if !metadata.is_symlink() {
                // We've reached the end of the chain
                return Ok(ResolvedPath::Resolved(canonical));
            }

            // Continue following the chain
            current = canonical;
        }
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

    #[test]
    #[cfg(unix)]
    fn test_multiple_symlinks_same_target() {
        let tmp = TempDir::new().unwrap();

        // Create a shared target
        let target = tmp.path().join("shared_target.txt");
        fs::write(&target, "shared content").unwrap();

        // Create two independent symlinks pointing to the same target
        let link1 = tmp.path().join("link1.txt");
        let link2 = tmp.path().join("link2.txt");
        unix_fs::symlink(&target, &link1).unwrap();
        unix_fs::symlink(&target, &link2).unwrap();

        let mut resolver = SymlinkResolver::new(false);

        // Resolve both symlinks - neither should fail
        let resolved1 = resolver.resolve(&link1).unwrap();
        let resolved2 = resolver.resolve(&link2).unwrap();

        // Both should resolve to the same target
        match (resolved1, resolved2) {
            (ResolvedPath::Resolved(p1), ResolvedPath::Resolved(p2)) => {
                assert_eq!(p1, p2);
                assert_eq!(p1, dunce::canonicalize(&target).unwrap());
            }
            _ => panic!("Expected both to be Resolved variants"),
        }
    }
}
