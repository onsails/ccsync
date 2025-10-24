//! Platform-specific utilities and cross-platform abstractions.
//!
//! This module provides platform-specific functionality and ensures
//! consistent behavior across Linux, macOS, and Windows.

use std::path::PathBuf;

/// Get the platform name for debugging and logging.
pub fn platform_name() -> &'static str {
    #[cfg(target_os = "linux")]
    return "Linux";
    
    #[cfg(target_os = "macos")]
    return "macOS";
    
    #[cfg(target_os = "windows")]
    return "Windows";
    
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    return "Unknown";
}

/// Get the global configuration directory path for Claude.
/// 
/// - Linux/macOS: `~/.claude`
/// - Windows: `%USERPROFILE%\.claude`
pub fn global_config_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|p| p.join(".claude"))
    }
    
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .map(|p| p.join(".claude"))
    }
}

/// Get the local configuration directory path (relative to current directory).
pub fn local_config_dir() -> PathBuf {
    PathBuf::from(".claude")
}

/// Check if the current platform is supported.
pub fn is_supported_platform() -> bool {
    cfg!(any(target_os = "linux", target_os = "macos", target_os = "windows"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_name() {
        let name = platform_name();
        assert!(!name.is_empty());
        assert!(["Linux", "macOS", "Windows", "Unknown"].contains(&name));
    }

    #[test]
    fn test_supported_platform() {
        assert!(is_supported_platform());
    }

    #[test]
    fn test_local_config_dir() {
        let dir = local_config_dir();
        assert_eq!(dir, PathBuf::from(".claude"));
    }

    #[test]
    fn test_global_config_dir() {
        let dir = global_config_dir();
        // Should return Some on all supported platforms
        assert!(dir.is_some());
    }
}
