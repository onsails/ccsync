//! Platform-specific utilities and cross-platform abstractions.
//!
//! This module provides platform-specific functionality and ensures
//! consistent behavior across Linux, macOS, and Windows.

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
}
