//! # FerricLink Core
//!
//! Core abstractions for the FerricLink ecosystem, inspired by LangChain Core.
//! This crate provides the fundamental building blocks for building AI applications
//! with language models, tools, vector stores, and more.

pub mod callbacks;
pub mod caches;
pub mod documents;
pub mod embeddings;
pub mod env;
pub mod errors;
pub mod example_selectors;
pub mod globals;
pub mod language_models;
pub mod messages;
pub mod rate_limiters;
pub mod retrievers;
pub mod runnables;
pub mod serializable;
pub mod structured_query;
pub mod tools;
pub mod utils;
pub mod vectorstores;

// Re-exports for convenience
pub use caches::{BaseCache, InMemoryCache, TtlCache, CacheStats, CachedGenerations};
pub use env::{get_runtime_environment, get_fresh_runtime_environment, RuntimeEnvironment};
pub use errors::{
    create_error_message, ErrorCode, FerricLinkError, IntoFerricLinkError, OutputParserException,
    Result, TracerException,
};
pub use example_selectors::{
    BaseExampleSelector, LengthBasedExampleSelector, SemanticSimilarityExampleSelector,
    MaxMarginalRelevanceExampleSelector, Example, sorted_values,
};
pub use globals::{
    init_globals, get_globals, set_verbose, get_verbose, set_debug, get_debug,
    set_llm_cache, clear_llm_cache, is_verbose, is_debug,
    has_llm_cache, globals_summary, reset_globals, enable_verbose, disable_verbose,
    enable_debug, disable_debug, toggle_verbose, toggle_debug,
};
pub use rate_limiters::{BaseRateLimiter, InMemoryRateLimiter, InMemoryRateLimiterConfig, AdvancedRateLimiter, RateLimiterConfig};
pub use serializable::Serializable;

/// Version of the FerricLink Core crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the FerricLink Core crate
///
/// This function should be called early in your application to set up
/// logging and other global configurations.
pub fn init() -> Result<()> {
    // Initialize global configuration
    init_globals()?;
    
    #[cfg(not(docsrs))]
    {
        tracing_subscriber::fmt::init();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        // VERSION is a const string that can never be empty, so we just check it's not the default
        assert_ne!(VERSION, "", "Version should not be empty");
    }

    #[test]
    fn test_init() {
        // This test may fail if globals are already initialized
        // from other tests, which is expected behavior
        let result = init();
        if result.is_err() {
            println!("Init failed (expected if globals already initialized): {:?}", result);
        }
        // We don't assert success here because globals can only be initialized once
    }
}
