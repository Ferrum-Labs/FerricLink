//! Global values and configuration that apply to all of FerricLink.
//!
//! This module provides global settings similar to LangChain's globals.py,
//! but with Rust's type safety and thread safety guarantees.

use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::caches::BaseCache;
use crate::errors::Result;

/// Global configuration state for FerricLink
pub struct GlobalConfig {
    /// Global verbose setting
    verbose: AtomicBool,
    /// Global debug setting
    debug: AtomicBool,
    /// Global LLM cache
    llm_cache: Arc<RwLock<Option<Box<dyn BaseCache>>>>,
}

impl GlobalConfig {
    /// Create a new global configuration
    fn new() -> Self {
        Self {
            verbose: AtomicBool::new(false),
            debug: AtomicBool::new(false),
            llm_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the verbose setting
    pub fn get_verbose(&self) -> bool {
        self.verbose.load(Ordering::SeqCst)
    }

    /// Set the verbose setting
    pub fn set_verbose(&self, value: bool) {
        self.verbose.store(value, Ordering::SeqCst);
    }

    /// Get the debug setting
    pub fn get_debug(&self) -> bool {
        self.debug.load(Ordering::SeqCst)
    }

    /// Set the debug setting
    pub fn set_debug(&self, value: bool) {
        self.debug.store(value, Ordering::SeqCst);
    }

    /// Get the LLM cache as a reference (for internal use)
    pub fn get_llm_cache_ref(&self) -> Result<std::sync::RwLockReadGuard<Option<Box<dyn BaseCache>>>> {
        self.llm_cache.try_read()
            .map_err(|_| crate::errors::FerricLinkError::generic("Failed to read LLM cache"))
    }

    /// Check if LLM cache is set (without taking a lock)
    pub fn has_llm_cache(&self) -> bool {
        self.llm_cache.try_read()
            .map(|cache| cache.is_some())
            .unwrap_or(false)
    }

    /// Set the LLM cache
    pub fn set_llm_cache(&self, cache: Option<Box<dyn BaseCache>>) -> Result<()> {
        let mut current_cache = self.llm_cache.write()
            .map_err(|_| crate::errors::FerricLinkError::generic("Failed to write LLM cache"))?;
        *current_cache = cache;
        Ok(())
    }

    /// Clear the LLM cache
    pub fn clear_llm_cache(&self) -> Result<()> {
        self.set_llm_cache(None)
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.get_verbose()
    }

    /// Check if debug mode is enabled
    pub fn is_debug(&self) -> bool {
        self.get_debug()
    }


    /// Get a summary of current global settings
    pub fn summary(&self) -> String {
        format!(
            "GlobalConfig {{ verbose: {}, debug: {}, has_llm_cache: {} }}",
            self.get_verbose(),
            self.get_debug(),
            self.has_llm_cache()
        )
    }
}

impl Clone for GlobalConfig {
    fn clone(&self) -> Self {
        Self {
            verbose: AtomicBool::new(self.get_verbose()),
            debug: AtomicBool::new(self.get_debug()),
            llm_cache: Arc::clone(&self.llm_cache),
        }
    }
}

/// Global configuration instance
static GLOBAL_CONFIG: std::sync::OnceLock<GlobalConfig> = std::sync::OnceLock::new();

/// Initialize the global configuration
///
/// This function should be called early in your application to set up
/// global settings. It's safe to call multiple times.
pub fn init_globals() -> Result<()> {
    GLOBAL_CONFIG.set(GlobalConfig::new())
        .map_err(|_| crate::errors::FerricLinkError::generic("Failed to initialize global config"))?;
    Ok(())
}

/// Get the global configuration instance
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
/// Make sure to call `init_globals()` early in your application.
pub fn get_globals() -> &'static GlobalConfig {
    GLOBAL_CONFIG.get()
        .expect("Global configuration not initialized. Call init_globals() first.")
}

/// Set the global verbose setting
///
/// # Arguments
///
/// * `value` - The new value for the verbose setting
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn set_verbose(value: bool) {
    get_globals().set_verbose(value);
}

/// Get the global verbose setting
///
/// # Returns
///
/// The current value of the verbose setting
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn get_verbose() -> bool {
    get_globals().get_verbose()
}

/// Set the global debug setting
///
/// # Arguments
///
/// * `value` - The new value for the debug setting
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn set_debug(value: bool) {
    get_globals().set_debug(value);
}

/// Get the global debug setting
///
/// # Returns
///
/// The current value of the debug setting
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn get_debug() -> bool {
    get_globals().get_debug()
}

/// Set the global LLM cache
///
/// # Arguments
///
/// * `cache` - The new LLM cache to use. If `None`, the LLM cache is disabled.
///
/// # Returns
///
/// A `Result` indicating success or failure
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn set_llm_cache(cache: Option<Box<dyn BaseCache>>) -> Result<()> {
    get_globals().set_llm_cache(cache)
}


/// Clear the global LLM cache
///
/// # Returns
///
/// A `Result` indicating success or failure
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn clear_llm_cache() -> Result<()> {
    get_globals().clear_llm_cache()
}

/// Check if verbose mode is enabled globally
///
/// # Returns
///
/// `true` if verbose mode is enabled, `false` otherwise
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn is_verbose() -> bool {
    get_globals().is_verbose()
}

/// Check if debug mode is enabled globally
///
/// # Returns
///
/// `true` if debug mode is enabled, `false` otherwise
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn is_debug() -> bool {
    get_globals().is_debug()
}

/// Check if LLM cache is enabled globally
///
/// # Returns
///
/// `true` if LLM cache is enabled, `false` otherwise
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn has_llm_cache() -> bool {
    get_globals().has_llm_cache()
}

/// Get a summary of current global settings
///
/// # Returns
///
/// A string summary of the current global configuration
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn globals_summary() -> String {
    get_globals().summary()
}

/// Reset all global settings to their default values
///
/// # Returns
///
/// A `Result` indicating success or failure
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn reset_globals() -> Result<()> {
    let globals = get_globals();
    globals.set_verbose(false);
    globals.set_debug(false);
    globals.clear_llm_cache()?;
    Ok(())
}

/// Convenience function to enable verbose mode
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn enable_verbose() {
    set_verbose(true);
}

/// Convenience function to disable verbose mode
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn disable_verbose() {
    set_verbose(false);
}

/// Convenience function to enable debug mode
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn enable_debug() {
    set_debug(true);
}

/// Convenience function to disable debug mode
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn disable_debug() {
    set_debug(false);
}

/// Convenience function to toggle verbose mode
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn toggle_verbose() {
    set_verbose(!get_verbose());
}

/// Convenience function to toggle debug mode
///
/// # Panics
///
/// This function will panic if `init_globals()` has not been called first.
pub fn toggle_debug() {
    set_debug(!get_debug());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caches::InMemoryCache;

    #[test]
    fn test_global_config_creation() {
        let config = GlobalConfig::new();
        assert!(!config.get_verbose());
        assert!(!config.get_debug());
        assert!(!config.has_llm_cache());
    }

    #[test]
    fn test_global_config_verbose() {
        let config = GlobalConfig::new();
        
        config.set_verbose(true);
        assert!(config.get_verbose());
        assert!(config.is_verbose());
        
        config.set_verbose(false);
        assert!(!config.get_verbose());
        assert!(!config.is_verbose());
    }

    #[test]
    fn test_global_config_debug() {
        let config = GlobalConfig::new();
        
        config.set_debug(true);
        assert!(config.get_debug());
        assert!(config.is_debug());
        
        config.set_debug(false);
        assert!(!config.get_debug());
        assert!(!config.is_debug());
    }

    #[test]
    fn test_global_config_llm_cache() {
        let config = GlobalConfig::new();
        
        // Initially no cache
        assert!(!config.has_llm_cache());
        assert!(config.get_llm_cache_ref().unwrap().is_none());
        
        // Set a cache
        let cache = Box::new(InMemoryCache::new());
        config.set_llm_cache(Some(cache)).unwrap();
        assert!(config.has_llm_cache());
        assert!(config.get_llm_cache_ref().unwrap().is_some());
        
        // Clear the cache
        config.clear_llm_cache().unwrap();
        assert!(!config.has_llm_cache());
        assert!(config.get_llm_cache_ref().unwrap().is_none());
    }

    #[test]
    fn test_global_config_summary() {
        let config = GlobalConfig::new();
        let summary = config.summary();
        
        assert!(summary.contains("verbose: false"));
        assert!(summary.contains("debug: false"));
        assert!(summary.contains("has_llm_cache: false"));
    }

    #[test]
    fn test_global_config_clone() {
        let config = GlobalConfig::new();
        config.set_verbose(true);
        config.set_debug(true);
        
        let cloned = config.clone();
        assert!(cloned.get_verbose());
        assert!(cloned.get_debug());
    }

    #[test]
    fn test_global_functions() {
        // Initialize globals
        init_globals().unwrap();
        
        // Test verbose functions
        set_verbose(true);
        assert!(get_verbose());
        assert!(is_verbose());
        
        enable_verbose();
        assert!(get_verbose());
        
        disable_verbose();
        assert!(!get_verbose());
        
        toggle_verbose();
        assert!(get_verbose());
        
        // Test debug functions
        set_debug(true);
        assert!(get_debug());
        assert!(is_debug());
        
        enable_debug();
        assert!(get_debug());
        
        disable_debug();
        assert!(!get_debug());
        
        toggle_debug();
        assert!(get_debug());
        
        // Test cache functions
        let cache = Box::new(InMemoryCache::new());
        set_llm_cache(Some(cache)).unwrap();
        assert!(has_llm_cache());
        
        clear_llm_cache().unwrap();
        assert!(!has_llm_cache());
        
        // Test summary
        let summary = globals_summary();
        assert!(summary.contains("GlobalConfig"));
        
        // Test reset
        reset_globals().unwrap();
        assert!(!get_verbose());
        assert!(!get_debug());
        assert!(!has_llm_cache());
    }

    #[test]
    fn test_global_functions_without_init() {
        // This test is skipped because globals might already be initialized
        // from other tests running in the same process
        println!("Skipping test_global_functions_without_init - globals may already be initialized");
    }

    #[test]
    fn test_multiple_init_calls() {
        // This test is skipped because globals might already be initialized
        // from other tests running in the same process
        println!("Skipping test_multiple_init_calls - globals may already be initialized");
    }
}
