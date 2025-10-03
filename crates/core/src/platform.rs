//! Platform detection utilities for cross-platform plugin resolution

use std::env;

/// Information about the current platform for plugin resolution
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    /// Rust target triple (e.g., "x86_64-unknown-linux-gnu")
    pub target: &'static str,
    /// Dynamic library file extension (e.g., "so", "dylib", "dll")
    pub extension: &'static str,
}

impl PlatformInfo {
    /// Detect the current platform
    pub fn current() -> Self {
        Self::from_os_arch(env::consts::OS, env::consts::ARCH)
    }

    /// Create platform info from OS and architecture strings
    pub fn from_os_arch(os: &str, arch: &str) -> Self {
        match (os, arch) {
            ("linux", "x86_64") => Self {
                target: "x86_64-unknown-linux-gnu",
                extension: "so",
            },
            ("linux", "aarch64") => Self {
                target: "aarch64-unknown-linux-gnu",
                extension: "so",
            },
            ("macos", "x86_64") => Self {
                target: "x86_64-apple-darwin",
                extension: "dylib",
            },
            ("macos", "aarch64") => Self {
                target: "aarch64-apple-darwin",
                extension: "dylib",
            },
            ("windows", "x86_64") => Self {
                target: "x86_64-pc-windows-msvc",
                extension: "dll",
            },
            ("windows", "aarch64") => Self {
                target: "aarch64-pc-windows-msvc",
                extension: "dll",
            },
            _ => panic!(
                "Unsupported platform: {}-{}\nSupported platforms: linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64, windows-x86_64, windows-aarch64",
                os, arch
            ),
        }
    }

    /// Get the current platform's dynamic library extension
    pub fn current_extension() -> &'static str {
        if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = PlatformInfo::current();
        assert!(!platform.target.is_empty());
        assert!(!platform.extension.is_empty());
    }

    #[test]
    fn test_linux_x86_64() {
        let platform = PlatformInfo::from_os_arch("linux", "x86_64");
        assert_eq!(platform.target, "x86_64-unknown-linux-gnu");
        assert_eq!(platform.extension, "so");
    }

    #[test]
    fn test_macos_aarch64() {
        let platform = PlatformInfo::from_os_arch("macos", "aarch64");
        assert_eq!(platform.target, "aarch64-apple-darwin");
        assert_eq!(platform.extension, "dylib");
    }

    #[test]
    fn test_windows_x86_64() {
        let platform = PlatformInfo::from_os_arch("windows", "x86_64");
        assert_eq!(platform.target, "x86_64-pc-windows-msvc");
        assert_eq!(platform.extension, "dll");
    }

    #[test]
    #[should_panic(expected = "Unsupported platform")]
    fn test_unsupported_platform() {
        PlatformInfo::from_os_arch("unknown", "unknown");
    }
}
