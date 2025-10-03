//! Utilities for getting information about the runtime environment.
//!
//! This module provides functionality to gather and report information about
//! the FerricLink runtime environment, similar to LangChain's env.py.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::impl_serializable;

/// Information about the FerricLink runtime environment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeEnvironment {
    /// Version of the FerricLink Core library
    pub library_version: String,
    /// Name of the library
    pub library: String,
    /// Platform information
    pub platform: String,
    /// Runtime language
    pub runtime: String,
    /// Runtime version
    pub runtime_version: String,
    /// Architecture information
    pub architecture: String,
    /// Operating system information
    pub os: String,
    /// Compiler information
    pub compiler: String,
    /// Target triple
    pub target: String,
    /// Additional environment variables
    pub env_vars: HashMap<String, String>,
    /// Features enabled at compile time
    pub features: Vec<String>,
}

impl RuntimeEnvironment {
    /// Create a new runtime environment instance
    pub fn new() -> Self {
        Self {
            library_version: crate::VERSION.to_string(),
            library: "ferriclink-core".to_string(),
            platform: get_platform_info(),
            runtime: "rust".to_string(),
            runtime_version: get_rust_version(),
            architecture: get_architecture(),
            os: get_os_info(),
            compiler: get_compiler_info(),
            target: get_target_triple(),
            env_vars: get_relevant_env_vars(),
            features: get_enabled_features(),
        }
    }

    /// Get a summary of the runtime environment
    pub fn summary(&self) -> String {
        format!(
            "FerricLink {} on {} {} ({})",
            self.library_version,
            self.os,
            self.architecture,
            self.runtime_version
        )
    }

    /// Check if a specific feature is enabled
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.contains(&feature.to_string())
    }

    /// Get environment variable value
    pub fn get_env_var(&self, key: &str) -> Option<&String> {
        self.env_vars.get(key)
    }

    /// Check if running in debug mode
    pub fn is_debug(&self) -> bool {
        cfg!(debug_assertions)
    }

    /// Check if running in release mode
    pub fn is_release(&self) -> bool {
        !self.is_debug()
    }

    /// Get memory information (if available)
    pub fn memory_info(&self) -> Option<MemoryInfo> {
        get_memory_info()
    }
}

impl Default for RuntimeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl_serializable!(RuntimeEnvironment, ["ferriclink", "env", "runtime_environment"]);

/// Memory information about the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryInfo {
    /// Total system memory in bytes
    pub total_memory: Option<u64>,
    /// Available memory in bytes
    pub available_memory: Option<u64>,
    /// Memory used by the process in bytes
    pub process_memory: Option<u64>,
}

impl_serializable!(MemoryInfo, ["ferriclink", "env", "memory_info"]);

/// Global runtime environment instance (cached)
static RUNTIME_ENV: OnceLock<RuntimeEnvironment> = OnceLock::new();

/// Get information about the FerricLink runtime environment.
///
/// This function returns a cached instance of the runtime environment
/// information, similar to LangChain's `get_runtime_environment()`.
///
/// # Returns
///
/// A `RuntimeEnvironment` struct with information about the runtime environment.
pub fn get_runtime_environment() -> &'static RuntimeEnvironment {
    RUNTIME_ENV.get_or_init(RuntimeEnvironment::new)
}

/// Get a fresh runtime environment instance (not cached)
pub fn get_fresh_runtime_environment() -> RuntimeEnvironment {
    RuntimeEnvironment::new()
}

/// Get platform information
fn get_platform_info() -> String {
    format!("{} {}", get_os_info(), get_architecture())
}

/// Get operating system information
fn get_os_info() -> String {
    #[cfg(target_os = "linux")]
    {
        "Linux".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        "macOS".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "Windows".to_string()
    }
    #[cfg(target_os = "freebsd")]
    {
        "FreeBSD".to_string()
    }
    #[cfg(target_os = "openbsd")]
    {
        "OpenBSD".to_string()
    }
    #[cfg(target_os = "netbsd")]
    {
        "NetBSD".to_string()
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    )))]
    {
        "Unknown".to_string()
    }
}

/// Get architecture information
fn get_architecture() -> String {
    #[cfg(target_arch = "x86_64")]
    {
        "x86_64".to_string()
    }
    #[cfg(target_arch = "x86")]
    {
        "x86".to_string()
    }
    #[cfg(target_arch = "aarch64")]
    {
        "aarch64".to_string()
    }
    #[cfg(target_arch = "arm")]
    {
        "arm".to_string()
    }
    #[cfg(target_arch = "riscv64")]
    {
        "riscv64".to_string()
    }
    #[cfg(target_arch = "powerpc64")]
    {
        "powerpc64".to_string()
    }
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "x86",
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "riscv64",
        target_arch = "powerpc64"
    )))]
    {
        "unknown".to_string()
    }
}

/// Get Rust version information
fn get_rust_version() -> String {
    std::env::var("RUSTC_VERSION")
        .or_else(|_| std::env::var("RUSTC_VERSION_MAJOR").map(|v| format!("{}.0.0", v)))
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Get compiler information
fn get_compiler_info() -> String {
    let rustc_version = get_rust_version();
    if rustc_version == "unknown" {
        "rustc (version unknown)".to_string()
    } else {
        format!("rustc {}", rustc_version)
    }
}

/// Get target triple
fn get_target_triple() -> String {
    std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string())
}

/// Get relevant environment variables
fn get_relevant_env_vars() -> HashMap<String, String> {
    let mut env_vars = HashMap::new();
    
    // Common environment variables that might be relevant
    let relevant_vars = [
        "RUST_LOG",
        "RUST_BACKTRACE",
        "CARGO_PKG_NAME",
        "CARGO_PKG_VERSION",
        "PATH",
        "HOME",
        "USER",
        "SHELL",
        "TERM",
        "LANG",
        "LC_ALL",
        "TZ",
    ];
    
    for var in &relevant_vars {
        if let Ok(value) = std::env::var(var) {
            env_vars.insert(var.to_string(), value);
        }
    }
    
    // Add FerricLink-specific environment variables
    for (key, value) in std::env::vars() {
        if key.starts_with("FERRICLINK_") || key.starts_with("FERRIC_") {
            env_vars.insert(key, value);
        }
    }
    
    env_vars
}

/// Get enabled features at compile time
fn get_enabled_features() -> Vec<String> {
    let mut features = Vec::new();
    
    // Check for common features
    #[cfg(feature = "http")]
    features.push("http".to_string());
    
    #[cfg(feature = "validation")]
    features.push("validation".to_string());
    
    #[cfg(feature = "all")]
    features.push("all".to_string());
    
    // Add debug/release info
    if cfg!(debug_assertions) {
        features.push("debug".to_string());
    } else {
        features.push("release".to_string());
    }
    
    // Add target info
    if cfg!(target_pointer_width = "64") {
        features.push("64bit".to_string());
    } else if cfg!(target_pointer_width = "32") {
        features.push("32bit".to_string());
    }
    
    features
}

/// Get memory information (if available)
fn get_memory_info() -> Option<MemoryInfo> {
    #[cfg(target_os = "linux")]
    {
        get_linux_memory_info()
    }
    #[cfg(target_os = "macos")]
    {
        get_macos_memory_info()
    }
    #[cfg(target_os = "windows")]
    {
        get_windows_memory_info()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

#[cfg(target_os = "linux")]
fn get_linux_memory_info() -> Option<MemoryInfo> {
    use std::fs;
    
    // Read /proc/meminfo
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_memory = None;
    let mut available_memory = None;
    
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(value) = line.split_whitespace().nth(1) {
                total_memory = value.parse::<u64>().ok().map(|kb| kb * 1024);
            }
        } else if line.starts_with("MemAvailable:") {
            if let Some(value) = line.split_whitespace().nth(1) {
                available_memory = value.parse::<u64>().ok().map(|kb| kb * 1024);
            }
        }
    }
    
    // Get process memory usage
    let process_memory = get_process_memory_usage();
    
    Some(MemoryInfo {
        total_memory,
        available_memory,
        process_memory,
    })
}

#[cfg(target_os = "macos")]
fn get_macos_memory_info() -> Option<MemoryInfo> {
    // On macOS, we can use system_profiler or sysctl
    // This is a simplified implementation
    let process_memory = get_process_memory_usage();
    
    Some(MemoryInfo {
        total_memory: None,
        available_memory: None,
        process_memory,
    })
}

#[cfg(target_os = "windows")]
fn get_windows_memory_info() -> Option<MemoryInfo> {
    // On Windows, we would use Windows API calls
    // This is a simplified implementation
    let process_memory = get_process_memory_usage();
    
    Some(MemoryInfo {
        total_memory: None,
        available_memory: None,
        process_memory,
    })
}

/// Get process memory usage (simplified implementation)
fn get_process_memory_usage() -> Option<u64> {
    // This is a placeholder - in a real implementation, you would
    // use platform-specific APIs to get actual memory usage
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_environment_creation() {
        let env = RuntimeEnvironment::new();
        assert_eq!(env.library, "ferriclink-core");
        assert_eq!(env.runtime, "rust");
        assert!(!env.library_version.is_empty());
        assert!(!env.platform.is_empty());
    }

    #[test]
    fn test_runtime_environment_caching() {
        let env1 = get_runtime_environment();
        let env2 = get_runtime_environment();
        assert!(std::ptr::eq(env1, env2));
    }

    #[test]
    fn test_runtime_environment_summary() {
        let env = RuntimeEnvironment::new();
        let summary = env.summary();
        assert!(summary.contains("FerricLink"));
        assert!(summary.contains(&env.library_version));
    }

    #[test]
    fn test_feature_detection() {
        let env = RuntimeEnvironment::new();
        // These features should be present based on our Cargo.toml
        assert!(env.has_feature("debug") || env.has_feature("release"));
    }

    #[test]
    fn test_environment_variables() {
        let env = RuntimeEnvironment::new();
        // Should have some environment variables
        assert!(!env.env_vars.is_empty());
    }

    #[test]
    fn test_serialization() {
        let env = RuntimeEnvironment::new();
        let serialized = serde_json::to_string(&env).unwrap();
        let deserialized: RuntimeEnvironment = serde_json::from_str(&serialized).unwrap();
        assert_eq!(env, deserialized);
    }
}
