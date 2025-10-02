//! Error types for FerricLink Core

use std::fmt;
use thiserror::Error;

/// Result type alias for FerricLink operations
pub type Result<T> = std::result::Result<T, FerricLinkError>;

/// Main error type for FerricLink Core
#[derive(Error, Debug)]
pub enum FerricLinkError {
    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP client errors
    #[cfg(feature = "http")]
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Runtime errors
    #[error("Runtime error: {0}")]
    Runtime(String),

    /// Not implemented errors
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Generic errors
    #[error("Error: {0}")]
    Generic(String),
}

impl FerricLinkError {
    /// Create a new validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a new configuration error
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a new runtime error
    pub fn runtime(msg: impl Into<String>) -> Self {
        Self::Runtime(msg.into())
    }

    /// Create a new not implemented error
    pub fn not_implemented(msg: impl Into<String>) -> Self {
        Self::NotImplemented(msg.into())
    }

    /// Create a new generic error
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }
}

/// Trait for converting errors to FerricLinkError
pub trait IntoFerricLinkError {
    /// Convert to FerricLinkError
    fn into_ferriclink_error(self) -> FerricLinkError;
}

impl<T> IntoFerricLinkError for T
where
    T: fmt::Display,
{
    fn into_ferriclink_error(self) -> FerricLinkError {
        FerricLinkError::Generic(self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let validation_err = FerricLinkError::validation("test validation error");
        assert!(matches!(validation_err, FerricLinkError::Validation(_)));

        let config_err = FerricLinkError::configuration("test config error");
        assert!(matches!(config_err, FerricLinkError::Configuration(_)));

        let runtime_err = FerricLinkError::runtime("test runtime error");
        assert!(matches!(runtime_err, FerricLinkError::Runtime(_)));
    }

    #[test]
    fn test_error_display() {
        let err = FerricLinkError::validation("test error");
        assert!(err.to_string().contains("test error"));
    }
}
