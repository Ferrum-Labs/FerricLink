//! # Exception Usage Example
//!
//! This example demonstrates how to use the comprehensive exception system
//! in FerricLink, inspired by LangChain's error handling patterns.

use ferriclink_core::{
    ErrorCode, FerricLinkError, IntoFerricLinkError, OutputParserException, Result,
    TracerException, create_error_message,
};

fn main() -> Result<()> {
    println!("=== FerricLink Exception System Example ===\n");

    // Example 1: Basic error creation
    println!("1. Basic Error Creation:");
    let validation_err = FerricLinkError::validation("Invalid input provided");
    println!("Validation Error: {validation_err}");
    println!("Error Code: {:?}\n", validation_err.error_code());

    // Example 2: LangChain-compatible error types
    println!("2. LangChain-Compatible Error Types:");
    let invalid_prompt =
        FerricLinkError::invalid_prompt_input("Prompt contains invalid characters");
    println!("Invalid Prompt: {invalid_prompt}");
    println!("Error Code: {:?}\n", invalid_prompt.error_code());

    let model_auth = FerricLinkError::model_authentication("API key is invalid");
    println!("Model Auth Error: {model_auth}");
    println!("Error Code: {:?}\n", model_auth.error_code());

    let rate_limit = FerricLinkError::model_rate_limit("Rate limit exceeded");
    println!("Rate Limit Error: {rate_limit}");
    println!("Error Code: {:?}\n", rate_limit.error_code());

    // Example 3: Tracer exceptions
    println!("3. Tracer Exceptions:");
    let tracer_err = TracerException::new("Failed to initialize tracer");
    println!("Tracer Error: {tracer_err}");
    println!("Error Code: {:?}\n", tracer_err.error_code);

    let tracer_with_code = TracerException::with_code("Model not found", ErrorCode::ModelNotFound);
    println!("Tracer with Code: {tracer_with_code}");
    println!("Error Code: {:?}\n", tracer_with_code.error_code);

    // Example 4: Output parser exceptions with LLM feedback
    println!("4. Output Parser Exceptions:");
    let parser_err = OutputParserException::new("Failed to parse JSON output");
    println!("Parser Error: {parser_err}");
    println!("Should send to LLM: {}\n", parser_err.should_send_to_llm());

    let parser_with_context = OutputParserException::with_llm_context(
        "Invalid JSON structure",
        Some("The output should be valid JSON".to_string()),
        Some(r#"{"invalid": json}"#.to_string()),
        true,
    );
    println!("Parser with Context: {parser_with_context}");
    println!(
        "Should send to LLM: {}",
        parser_with_context.should_send_to_llm()
    );
    println!("Observation: {:?}", parser_with_context.observation());
    println!("LLM Output: {:?}\n", parser_with_context.llm_output());

    // Example 5: Error codes and troubleshooting
    println!("5. Error Codes and Troubleshooting:");
    let error_codes = [
        ErrorCode::InvalidPromptInput,
        ErrorCode::ModelAuthentication,
        ErrorCode::OutputParsingFailure,
        ErrorCode::ModelRateLimit,
    ];

    for code in &error_codes {
        println!("Error Code: {} ({})", code.as_str(), code);
        println!("Troubleshooting URL: {}", code.troubleshooting_url());
    }
    println!();

    // Example 6: Error message creation
    println!("6. Error Message Creation:");
    let custom_message = create_error_message(
        "Custom error occurred during processing",
        ErrorCode::RuntimeError,
    );
    println!("Custom Error Message:\n{custom_message}");
    println!();

    // Example 7: Error conversion
    println!("7. Error Conversion:");
    let string_error: FerricLinkError = "Something went wrong".into_ferriclink_error();
    println!("Converted Error: {string_error}");
    println!("Error Code: {:?}\n", string_error.error_code());

    // Example 8: Error handling in functions
    println!("8. Error Handling in Functions:");
    match simulate_llm_call() {
        Ok(result) => println!("LLM Call succeeded: {result}"),
        Err(e) => {
            println!("LLM Call failed: {e}");
            if e.should_send_to_llm() {
                if let Some((observation, llm_output)) = e.llm_context() {
                    println!("Should retry with context:");
                    println!("  Observation: {observation:?}");
                    println!("  LLM Output: {llm_output:?}");
                }
            }
        }
    }

    println!("\n=== Example Complete ===");
    Ok(())
}

/// Simulate an LLM call that might fail
fn simulate_llm_call() -> Result<String> {
    // Simulate a parsing error that should be sent back to the LLM
    let parser_err = OutputParserException::with_llm_context(
        "The model output was not in the expected format",
        Some("Please ensure the output is valid JSON".to_string()),
        Some(r#"{"result": incomplete"#.to_string()),
        true,
    );
    Err(parser_err.into())
}

/// Demonstrate error handling patterns
#[allow(dead_code)]
fn handle_errors() -> Result<()> {
    // Pattern 1: Specific error handling
    match FerricLinkError::model_authentication("Invalid API key") {
        FerricLinkError::General(msg) if msg.contains("MODEL_AUTHENTICATION") => {
            println!("Handling authentication error: {msg}");
        }
        _ => {}
    }

    // Pattern 2: Error code checking
    let err = FerricLinkError::output_parsing_failure("Parse failed");
    if let Some(code) = err.error_code() {
        match code {
            ErrorCode::OutputParsingFailure => {
                println!("Handling output parsing failure");
            }
            ErrorCode::ModelRateLimit => {
                println!("Handling rate limit - should retry later");
            }
            _ => {
                println!("Handling other error type: {code:?}");
            }
        }
    }

    // Pattern 3: LLM feedback handling
    let parser_err = OutputParserException::with_llm_context(
        "Invalid format",
        Some("Use JSON format".to_string()),
        Some("invalid output".to_string()),
        true,
    );
    let ferric_err: FerricLinkError = parser_err.into();

    if ferric_err.should_send_to_llm() {
        if let Some((observation, llm_output)) = ferric_err.llm_context() {
            println!("Sending feedback to LLM:");
            println!("  Observation: {observation:?}");
            println!("  Previous output: {llm_output:?}");
        }
    }

    Ok(())
}
