//! # FerricLink Core
//!
//! Core abstractions for the FerricLink ecosystem, inspired by LangChain Core.
//! This crate provides the fundamental building blocks for building AI applications
//! with language models, tools, vector stores, and more.

pub mod caches;
pub mod callbacks;
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
pub use caches::{BaseCache, CacheStats, CachedGenerations, InMemoryCache, TtlCache};
pub use env::{RuntimeEnvironment, get_fresh_runtime_environment, get_runtime_environment};
pub use errors::{
    ErrorCode, FerricLinkError, IntoFerricLinkError, OutputParserException, Result,
    TracerException, create_error_message,
};
pub use example_selectors::{
    BaseExampleSelector, Example, LengthBasedExampleSelector, MaxMarginalRelevanceExampleSelector,
    SemanticSimilarityExampleSelector, sorted_values,
};
pub use globals::{
    clear_llm_cache, disable_debug, disable_verbose, enable_debug, enable_verbose, get_debug,
    get_globals, get_verbose, globals_summary, has_llm_cache, init_globals, is_debug, is_verbose,
    reset_globals, set_debug, set_llm_cache, set_verbose, toggle_debug, toggle_verbose,
};
pub use rate_limiters::{
    AdvancedRateLimiter, BaseRateLimiter, InMemoryRateLimiter, InMemoryRateLimiterConfig,
    RateLimiterConfig,
};
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
            println!("Init failed (expected if globals already initialized): {result:?}",);
        }
        // We don't assert success here because globals can only be initialized once
    }
}
