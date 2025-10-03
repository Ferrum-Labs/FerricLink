---
sidebar_position: 1
---

# Invalid Prompt Input Error

**Error Code:** `INVALID_PROMPT_INPUT`

This error occurs when the prompt provided to a language model contains invalid characters, formatting, or structure.

## Common Causes

1. **Invalid Characters**: The prompt contains characters that the model doesn't support
2. **Encoding Issues**: The prompt has incorrect character encoding
3. **Format Violations**: The prompt doesn't follow the expected format for the model
4. **Length Limits**: The prompt exceeds the model's maximum input length

## Solutions

### 1. Check Character Encoding

```rust
use ferriclink_core::FerricLinkError;

// Ensure proper UTF-8 encoding
let prompt = "Your prompt here".to_string();
// Check for invalid characters
if !prompt.is_char_boundary_valid() {
    return Err(FerricLinkError::invalid_prompt_input("Invalid character encoding"));
}
```

### 2. Validate Prompt Format

```rust
fn validate_prompt(prompt: &str) -> Result<(), FerricLinkError> {
    // Check for required format elements
    if !prompt.contains("System:") && !prompt.contains("Human:") {
        return Err(FerricLinkError::invalid_prompt_input(
            "Prompt must contain System: or Human: markers"
        ));
    }
    
    // Check length limits
    if prompt.len() > 10000 {
        return Err(FerricLinkError::invalid_prompt_input(
            "Prompt exceeds maximum length of 10,000 characters"
        ));
    }
    
    Ok(())
}
```

### 3. Sanitize Input

```rust
fn sanitize_prompt(prompt: &str) -> String {
    prompt
        .chars()
        .filter(|c| c.is_ascii() || c.is_alphanumeric())
        .collect()
}
```

## Prevention

- Always validate prompts before sending to models
- Use proper encoding (UTF-8)
- Check model-specific requirements
- Implement length validation
- Sanitize user inputs

## Related Documentation

- [Message Types](/docs/concepts/messages)
- [Language Models](/docs/concepts/language-models)
- [Error Handling](/docs/guides/error-handling)
