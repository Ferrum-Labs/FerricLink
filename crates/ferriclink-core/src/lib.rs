//! # FerricLink Core
//!
//! Core abstractions for the FerricLink ecosystem, inspired by LangChain Core.
//! This crate provides the fundamental building blocks for building AI applications
//! with language models, tools, vector stores, and more.

pub mod callbacks;
pub mod documents;
pub mod embeddings;
pub mod errors;
pub mod language_models;
pub mod messages;
pub mod retrievers;
pub mod runnables;
pub mod serializable;
pub mod tools;
pub mod utils;
pub mod vectorstores;

// Re-exports for convenience
pub use errors::{FerricLinkError, Result};
pub use serializable::Serializable;

/// Version of the FerricLink Core crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the FerricLink Core crate
///
/// This function should be called early in your application to set up
/// logging and other global configurations.
pub fn init() -> Result<()> {
    tracing_subscriber::fmt::init();
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
        assert!(init().is_ok());
    }
}
