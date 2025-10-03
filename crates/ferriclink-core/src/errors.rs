//! Error types for FerricLink Core
//!
//! This module provides comprehensive error handling for the FerricLink ecosystem,
//! inspired by LangChain's exception system with Rust-specific improvements.

use std::fmt;
use thiserror::Error;

/// Result type alias for FerricLink operations
pub type Result<T> = std::result::Result<T, FerricLinkError>;

/// Error codes for structured error handling
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ErrorCode {
    /// Invalid prompt input provided
    InvalidPromptInput,
    /// Invalid tool results received
    InvalidToolResults,
    /// Message coercion failed
    MessageCoercionFailure,
    /// Model authentication failed
    ModelAuthentication,
    /// Model not found
    ModelNotFound,
    /// Model rate limit exceeded
    ModelRateLimit,
    /// Output parsing failed
    OutputParsingFailure,
    /// Serialization/deserialization error
    SerializationError,
    /// IO operation failed
    IoError,
    /// HTTP request failed
    HttpError,
    /// Validation failed
    ValidationError,
    /// Configuration error
    ConfigurationError,
    /// Runtime error
    RuntimeError,
    /// Feature not implemented
    NotImplemented,
    /// Generic error
    GenericError,
}

impl ErrorCode {
    /// Get the string representation of the error code
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::InvalidPromptInput => "INVALID_PROMPT_INPUT",
            ErrorCode::InvalidToolResults => "INVALID_TOOL_RESULTS",
            ErrorCode::MessageCoercionFailure => "MESSAGE_COERCION_FAILURE",
            ErrorCode::ModelAuthentication => "MODEL_AUTHENTICATION",
            ErrorCode::ModelNotFound => "MODEL_NOT_FOUND",
            ErrorCode::ModelRateLimit => "MODEL_RATE_LIMIT",
            ErrorCode::OutputParsingFailure => "OUTPUT_PARSING_FAILURE",
            ErrorCode::SerializationError => "SERIALIZATION_ERROR",
            ErrorCode::IoError => "IO_ERROR",
            ErrorCode::HttpError => "HTTP_ERROR",
            ErrorCode::ValidationError => "VALIDATION_ERROR",
            ErrorCode::ConfigurationError => "CONFIGURATION_ERROR",
            ErrorCode::RuntimeError => "RUNTIME_ERROR",
            ErrorCode::NotImplemented => "NOT_IMPLEMENTED",
            ErrorCode::GenericError => "GENERIC_ERROR",
        }
    }

    /// Get the troubleshooting URL for this error code
    pub fn troubleshooting_url(&self) -> String {
        format!(
            "https://ferrum-labs.github.io/FerricLink/docs/troubleshooting/errors/{}",
            self.as_str().to_lowercase()
        )
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Main error type for FerricLink Core
#[derive(Error, Debug)]
pub enum FerricLinkError {
    /// General FerricLink exception
    #[error("FerricLink error: {0}")]
    General(String),

    /// Tracer-related errors
    #[error("Tracer error: {0}")]
    Tracer(#[from] TracerException),

    /// Output parser errors with special handling
    #[error("Output parser error: {0}")]
    OutputParser(#[from] OutputParserException),

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

/// Tracer exception for tracing-related errors
#[derive(Error, Debug)]
#[error("Tracer error: {message}")]
pub struct TracerException {
    /// Error message
    pub message: String,
    /// Error code
    pub error_code: ErrorCode,
}

impl TracerException {
    /// Create a new tracer exception
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            error_code: ErrorCode::RuntimeError,
        }
    }

    /// Create a new tracer exception with specific error code
    pub fn with_code(message: impl Into<String>, error_code: ErrorCode) -> Self {
        Self {
            message: message.into(),
            error_code,
        }
    }
}

/// Output parser exception with special handling for LLM feedback
#[derive(Error, Debug)]
#[error("Output parser error: {message}")]
pub struct OutputParserException {
    /// Error message
    pub message: String,
    /// Error code
    pub error_code: ErrorCode,
    /// Observation that can be sent to the model
    pub observation: Option<String>,
    /// LLM output that caused the error
    pub llm_output: Option<String>,
    /// Whether to send context back to the LLM
    pub send_to_llm: bool,
}

impl OutputParserException {
    /// Create a new output parser exception
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            error_code: ErrorCode::OutputParsingFailure,
            observation: None,
            llm_output: None,
            send_to_llm: false,
        }
    }

    /// Create a new output parser exception with error code
    pub fn with_code(message: impl Into<String>, error_code: ErrorCode) -> Self {
        Self {
            message: message.into(),
            error_code,
            observation: None,
            llm_output: None,
            send_to_llm: false,
        }
    }

    /// Create a new output parser exception with LLM feedback context
    pub fn with_llm_context(
        message: impl Into<String>,
        observation: Option<String>,
        llm_output: Option<String>,
        send_to_llm: bool,
    ) -> Self {
        if send_to_llm && (observation.is_none() || llm_output.is_none()) {
            panic!("Arguments 'observation' & 'llm_output' are required if 'send_to_llm' is True");
        }

        Self {
            message: message.into(),
            error_code: ErrorCode::OutputParsingFailure,
            observation,
            llm_output,
            send_to_llm,
        }
    }

    /// Get the observation for LLM feedback
    pub fn observation(&self) -> Option<&str> {
        self.observation.as_deref()
    }

    /// Get the LLM output that caused the error
    pub fn llm_output(&self) -> Option<&str> {
        self.llm_output.as_deref()
    }

    /// Check if this error should be sent back to the LLM
    pub fn should_send_to_llm(&self) -> bool {
        self.send_to_llm
    }
}

impl FerricLinkError {
    /// Create a new general FerricLink error
    pub fn general(msg: impl Into<String>) -> Self {
        Self::General(msg.into())
    }

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

    /// Create a new error with error code and troubleshooting link
    pub fn with_code(msg: impl Into<String>, error_code: ErrorCode) -> Self {
        let message = create_error_message(msg, error_code);
        Self::General(message)
    }

    /// Create a new error with specific error code (for testing and internal use)
    pub fn with_error_code(msg: impl Into<String>, error_code: ErrorCode) -> Self {
        match error_code {
            ErrorCode::InvalidPromptInput => Self::invalid_prompt_input(msg),
            ErrorCode::InvalidToolResults => Self::invalid_tool_results(msg),
            ErrorCode::MessageCoercionFailure => Self::message_coercion_failure(msg),
            ErrorCode::ModelAuthentication => Self::model_authentication(msg),
            ErrorCode::ModelNotFound => Self::model_not_found(msg),
            ErrorCode::ModelRateLimit => Self::model_rate_limit(msg),
            ErrorCode::OutputParsingFailure => Self::output_parsing_failure(msg),
            ErrorCode::SerializationError => Self::validation(msg),
            ErrorCode::IoError => Self::runtime(msg),
            ErrorCode::HttpError => Self::runtime(msg),
            ErrorCode::ValidationError => Self::validation(msg),
            ErrorCode::ConfigurationError => Self::configuration(msg),
            ErrorCode::RuntimeError => Self::runtime(msg),
            ErrorCode::NotImplemented => Self::not_implemented(msg),
            ErrorCode::GenericError => Self::generic(msg),
        }
    }

    /// Get the error code if available
    pub fn error_code(&self) -> Option<ErrorCode> {
        match self {
            FerricLinkError::Tracer(e) => Some(e.error_code.clone()),
            FerricLinkError::OutputParser(e) => Some(e.error_code.clone()),
            FerricLinkError::Serialization(_) => Some(ErrorCode::SerializationError),
            FerricLinkError::Io(_) => Some(ErrorCode::IoError),
            #[cfg(feature = "http")]
            FerricLinkError::Http(_) => Some(ErrorCode::HttpError),
            FerricLinkError::Validation(_) => Some(ErrorCode::ValidationError),
            FerricLinkError::Configuration(_) => Some(ErrorCode::ConfigurationError),
            FerricLinkError::Runtime(_) => Some(ErrorCode::RuntimeError),
            FerricLinkError::NotImplemented(_) => Some(ErrorCode::NotImplemented),
            FerricLinkError::General(msg) => {
                // Try to determine error code from message content
                if msg.contains("INVALID_PROMPT_INPUT") {
                    Some(ErrorCode::InvalidPromptInput)
                } else if msg.contains("INVALID_TOOL_RESULTS") {
                    Some(ErrorCode::InvalidToolResults)
                } else if msg.contains("MESSAGE_COERCION_FAILURE") {
                    Some(ErrorCode::MessageCoercionFailure)
                } else if msg.contains("MODEL_AUTHENTICATION") {
                    Some(ErrorCode::ModelAuthentication)
                } else if msg.contains("MODEL_NOT_FOUND") {
                    Some(ErrorCode::ModelNotFound)
                } else if msg.contains("MODEL_RATE_LIMIT") {
                    Some(ErrorCode::ModelRateLimit)
                } else if msg.contains("OUTPUT_PARSING_FAILURE") {
                    Some(ErrorCode::OutputParsingFailure)
                } else {
                    Some(ErrorCode::GenericError)
                }
            }
            FerricLinkError::Generic(_) => Some(ErrorCode::GenericError),
        }
    }

    /// Check if this is an output parser error that should be sent to LLM
    pub fn should_send_to_llm(&self) -> bool {
        match self {
            FerricLinkError::OutputParser(e) => e.should_send_to_llm(),
            _ => false,
        }
    }

    /// Get LLM context if this is an output parser error
    pub fn llm_context(&self) -> Option<(Option<&str>, Option<&str>)> {
        match self {
            FerricLinkError::OutputParser(e) => Some((e.observation(), e.llm_output())),
            _ => None,
        }
    }
}

/// Create an error message with troubleshooting link
///
/// This function creates a comprehensive error message that includes
/// a link to the troubleshooting guide, similar to LangChain's approach.
///
/// # Arguments
///
/// * `message` - The base error message
/// * `error_code` - The error code for categorization
///
/// # Returns
///
/// A formatted error message with troubleshooting information
pub fn create_error_message(message: impl Into<String>, error_code: ErrorCode) -> String {
    let message = message.into();
    let troubleshooting_url = error_code.troubleshooting_url();
    let error_code_str = error_code.as_str();
    
    format!(
        "{message}\nError Code: {error_code_str}\nFor troubleshooting, visit: {troubleshooting_url}"
    )
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

/// Convenience functions for creating specific error types
impl FerricLinkError {
    /// Create an invalid prompt input error
    pub fn invalid_prompt_input(msg: impl Into<String>) -> Self {
        Self::General(create_error_message(msg, ErrorCode::InvalidPromptInput))
    }

    /// Create an invalid tool results error
    pub fn invalid_tool_results(msg: impl Into<String>) -> Self {
        Self::General(create_error_message(msg, ErrorCode::InvalidToolResults))
    }

    /// Create a message coercion failure error
    pub fn message_coercion_failure(msg: impl Into<String>) -> Self {
        Self::General(create_error_message(msg, ErrorCode::MessageCoercionFailure))
    }

    /// Create a model authentication error
    pub fn model_authentication(msg: impl Into<String>) -> Self {
        Self::General(create_error_message(msg, ErrorCode::ModelAuthentication))
    }

    /// Create a model not found error
    pub fn model_not_found(msg: impl Into<String>) -> Self {
        Self::General(create_error_message(msg, ErrorCode::ModelNotFound))
    }

    /// Create a model rate limit error
    pub fn model_rate_limit(msg: impl Into<String>) -> Self {
        Self::General(create_error_message(msg, ErrorCode::ModelRateLimit))
    }

    /// Create an output parsing failure error
    pub fn output_parsing_failure(msg: impl Into<String>) -> Self {
        Self::General(create_error_message(msg, ErrorCode::OutputParsingFailure))
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

    #[test]
    fn test_error_codes() {
        let code = ErrorCode::InvalidPromptInput;
        assert_eq!(code.as_str(), "INVALID_PROMPT_INPUT");
        assert!(code.troubleshooting_url().contains("ferrum-labs.github.io"));
    }

    #[test]
    fn test_tracer_exception() {
        let tracer_err = TracerException::new("test tracer error");
        assert_eq!(tracer_err.error_code, ErrorCode::RuntimeError);
        assert_eq!(tracer_err.message, "test tracer error");

        let tracer_err_with_code = TracerException::with_code("test", ErrorCode::ModelNotFound);
        assert_eq!(tracer_err_with_code.error_code, ErrorCode::ModelNotFound);
    }

    #[test]
    fn test_output_parser_exception() {
        let parser_err = OutputParserException::new("test parser error");
        assert_eq!(parser_err.error_code, ErrorCode::OutputParsingFailure);
        assert!(!parser_err.should_send_to_llm());

        let parser_err_with_context = OutputParserException::with_llm_context(
            "test error",
            Some("observation".to_string()),
            Some("llm output".to_string()),
            true,
        );
        assert!(parser_err_with_context.should_send_to_llm());
        assert_eq!(parser_err_with_context.observation(), Some("observation"));
        assert_eq!(parser_err_with_context.llm_output(), Some("llm output"));
    }

    #[test]
    fn test_error_with_code() {
        let err = FerricLinkError::with_error_code("test error", ErrorCode::ModelAuthentication);
        assert!(err.to_string().contains("test error"));
        assert_eq!(err.error_code(), Some(ErrorCode::ModelAuthentication));
    }

    #[test]
    fn test_convenience_error_functions() {
        let invalid_prompt = FerricLinkError::invalid_prompt_input("bad prompt");
        assert_eq!(invalid_prompt.error_code(), Some(ErrorCode::InvalidPromptInput));

        let model_auth = FerricLinkError::model_authentication("auth failed");
        assert_eq!(model_auth.error_code(), Some(ErrorCode::ModelAuthentication));

        let rate_limit = FerricLinkError::model_rate_limit("too many requests");
        assert_eq!(rate_limit.error_code(), Some(ErrorCode::ModelRateLimit));
    }

    #[test]
    fn test_llm_context() {
        let parser_err = OutputParserException::with_llm_context(
            "test",
            Some("obs".to_string()),
            Some("output".to_string()),
            true,
        );
        let ferric_err = FerricLinkError::OutputParser(parser_err);
        
        assert!(ferric_err.should_send_to_llm());
        let context = ferric_err.llm_context();
        assert_eq!(context, Some((Some("obs"), Some("output"))));
    }

    #[test]
    fn test_create_error_message() {
        let message = create_error_message("test error", ErrorCode::OutputParsingFailure);
        assert!(message.contains("test error"));
        assert!(message.contains("troubleshooting"));
        assert!(message.contains("ferrum-labs.github.io"));
    }

    #[test]
    fn test_into_ferriclink_error() {
        let err: FerricLinkError = "test error".into_ferriclink_error();
        assert!(matches!(err, FerricLinkError::Generic(_)));
        assert!(err.to_string().contains("test error"));
    }
}
