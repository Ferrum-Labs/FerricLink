---
sidebar_position: 3
---

# Output Parsing Failure Error

**Error Code:** `OUTPUT_PARSING_FAILURE`

This error occurs when FerricLink cannot parse the output from a language model into the expected format.

## Common Causes

1. **Invalid JSON**: The model output is not valid JSON
2. **Missing Fields**: Required fields are missing from the parsed output
3. **Type Mismatches**: The parsed values don't match expected types
4. **Malformed Structure**: The output structure doesn't match the expected schema

## Solutions

### 1. Basic JSON Parsing

```rust
use ferriclink_core::{FerricLinkError, OutputParserException, ErrorCode};
use serde_json::Value;

fn parse_model_output(output: &str) -> Result<Value, FerricLinkError> {
    serde_json::from_str(output)
        .map_err(|e| {
            OutputParserException::with_llm_context(
                format!("Failed to parse JSON: {}", e),
                Some("Please ensure the output is valid JSON".to_string()),
                Some(output.to_string()),
                true, // Send back to LLM for retry
            ).into()
        })
}
```

### 2. Structured Output Parsing

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct ModelResponse {
    answer: String,
    confidence: f64,
    reasoning: Option<String>,
}

fn parse_structured_output(output: &str) -> Result<ModelResponse, FerricLinkError> {
    let parsed: ModelResponse = serde_json::from_str(output)
        .map_err(|e| {
            OutputParserException::with_llm_context(
                format!("Invalid response structure: {}", e),
                Some("Please provide a response with 'answer' and 'confidence' fields".to_string()),
                Some(output.to_string()),
                true,
            ).into()
        })?;
    
    // Validate required fields
    if parsed.answer.is_empty() {
        return Err(OutputParserException::with_llm_context(
            "Answer field cannot be empty",
            Some("Please provide a non-empty answer".to_string()),
            Some(output.to_string()),
            true,
        ).into());
    }
    
    Ok(parsed)
}
```

### 3. Handle LLM Feedback

```rust
use ferriclink_core::FerricLinkError;

async fn handle_parsing_error(error: FerricLinkError) -> Result<String, FerricLinkError> {
    if error.should_send_to_llm() {
        if let Some((observation, llm_output)) = error.llm_context() {
            // Create a retry prompt with feedback
            let retry_prompt = format!(
                "Previous output was invalid: {}\nObservation: {}\nPlease try again with valid JSON.",
                llm_output.unwrap_or(""),
                observation.unwrap_or("")
            );
            
            // Send retry request to LLM
            return retry_with_feedback(&retry_prompt).await;
        }
    }
    
    Err(error)
}
```

### 4. Robust Parsing with Fallbacks

```rust
fn robust_parse(output: &str) -> Result<Value, FerricLinkError> {
    // Try direct parsing first
    if let Ok(parsed) = serde_json::from_str::<Value>(output) {
        return Ok(parsed);
    }
    
    // Try to extract JSON from markdown code blocks
    if let Some(json_match) = extract_json_from_markdown(output) {
        return serde_json::from_str(&json_match)
            .map_err(|e| FerricLinkError::output_parsing_failure(
                format!("Failed to parse extracted JSON: {}", e)
            ));
    }
    
    // Try to fix common JSON issues
    let fixed = fix_common_json_issues(output);
    serde_json::from_str(&fixed)
        .map_err(|e| FerricLinkError::output_parsing_failure(
            format!("Failed to parse even after fixes: {}", e)
        ))
}

fn extract_json_from_markdown(text: &str) -> Option<String> {
    // Look for ```json ... ``` blocks
    let re = regex::Regex::new(r"```json\s*(.*?)\s*```").ok()?;
    let captures = re.captures(text)?;
    Some(captures.get(1)?.as_str().to_string())
}

fn fix_common_json_issues(json: &str) -> String {
    json
        .replace("'", "\"")  // Replace single quotes with double quotes
        .replace("True", "true")  // Fix Python boolean
        .replace("False", "false")
        .replace("None", "null")  // Fix Python null
}
```

## Prevention

- Use clear instructions for structured output
- Implement validation schemas
- Provide examples in prompts
- Use retry mechanisms with feedback
- Implement robust parsing with fallbacks

## Related Documentation

- [Output Parsing](/docs/guides/output-parsing)
- [Structured Outputs](/docs/guides/structured-outputs)
- [Error Handling](/docs/guides/error-handling)
