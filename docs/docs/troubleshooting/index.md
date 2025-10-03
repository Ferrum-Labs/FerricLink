---
sidebar_position: 1
---

# Troubleshooting

This section provides solutions to common issues you might encounter when using FerricLink.

## Error Codes

FerricLink uses structured error codes to help you quickly identify and resolve issues. Each error includes a specific error code and a link to detailed troubleshooting information.

### Common Error Types

- **INVALID_PROMPT_INPUT** - Issues with prompt formatting or content
- **INVALID_TOOL_RESULTS** - Problems with tool execution results
- **MESSAGE_COERCION_FAILURE** - Message type conversion errors
- **MODEL_AUTHENTICATION** - API key or authentication issues
- **MODEL_NOT_FOUND** - Model name or configuration problems
- **MODEL_RATE_LIMIT** - Rate limiting from model providers
- **OUTPUT_PARSING_FAILURE** - Issues parsing model outputs

## Getting Help

If you can't find a solution to your problem:

1. Check the specific error page for your error code
2. Search the [GitHub Issues](https://github.com/Ferrum-Labs/FerricLink/issues)
3. Join our community discussions
4. Create a new issue with detailed information

## Error Handling Best Practices

### 1. Check Error Codes

```rust
use ferriclink_core::{FerricLinkError, ErrorCode};

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(e) => {
        if let Some(code) = e.error_code() {
            match code {
                ErrorCode::ModelRateLimit => {
                    println!("Rate limited - retry later");
                }
                ErrorCode::OutputParsingFailure => {
                    println!("Parsing failed - check output format");
                }
                _ => {
                    println!("Other error: {:?}", code);
                }
            }
        }
    }
}
```

### 2. Handle LLM Feedback

```rust
use ferriclink_core::{FerricLinkError, OutputParserException};

if let Err(e) = result {
    if e.should_send_to_llm() {
        if let Some((observation, llm_output)) = e.llm_context() {
            // Send feedback to LLM for retry
            println!("Observation: {:?}", observation);
            println!("Previous output: {:?}", llm_output);
        }
    }
}
```

### 3. Use Troubleshooting Links

All errors include troubleshooting URLs that provide specific guidance:

```rust
let err = FerricLinkError::model_authentication("Invalid API key");
println!("Error: {}", err);
// This will include a link to the troubleshooting page
```
