---
sidebar_position: 2
---

# Model Authentication Error

**Error Code:** `MODEL_AUTHENTICATION`

This error occurs when there are issues with authenticating to a language model provider.

## Common Causes

1. **Invalid API Key**: The provided API key is incorrect or expired
2. **Missing API Key**: No API key was provided for the model
3. **Incorrect Environment Variable**: The API key environment variable is not set correctly
4. **Provider Issues**: The model provider's authentication service is down

## Solutions

### 1. Check API Key

```rust
use ferriclink_core::FerricLinkError;
use std::env;

fn get_api_key() -> Result<String, FerricLinkError> {
    env::var("OPENAI_API_KEY")
        .map_err(|_| FerricLinkError::model_authentication(
            "OPENAI_API_KEY environment variable not set"
        ))
}
```

### 2. Validate API Key Format

```rust
fn validate_api_key(key: &str) -> Result<(), FerricLinkError> {
    if key.is_empty() {
        return Err(FerricLinkError::model_authentication("API key is empty"));
    }
    
    if key.len() < 20 {
        return Err(FerricLinkError::model_authentication(
            "API key appears to be too short"
        ));
    }
    
    Ok(())
}
```

### 3. Test Authentication

```rust
async fn test_authentication(api_key: &str) -> Result<(), FerricLinkError> {
    // Make a simple test request to verify the API key
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.openai.com/v1/models")
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|_| FerricLinkError::model_authentication(
            "Failed to connect to API"
        ))?;
    
    if !response.status().is_success() {
        return Err(FerricLinkError::model_authentication(
            "API key authentication failed"
        ));
    }
    
    Ok(())
}
```

## Environment Setup

### 1. Set Environment Variables

```bash
# For OpenAI
export OPENAI_API_KEY="your-api-key-here"

# For Anthropic
export ANTHROPIC_API_KEY="your-api-key-here"

# For other providers
export MODEL_API_KEY="your-api-key-here"
```

### 2. Use .env Files

```rust
use dotenv::dotenv;

fn load_env() -> Result<(), FerricLinkError> {
    dotenv().map_err(|_| FerricLinkError::model_authentication(
        "Failed to load .env file"
    ))?;
    Ok(())
}
```

## Prevention

- Always validate API keys before use
- Use environment variables for sensitive data
- Implement proper error handling for auth failures
- Test authentication in development
- Keep API keys secure and rotate them regularly

## Related Documentation

- [Configuration](/docs/guides/configuration)
- [Environment Variables](/docs/guides/environment-setup)
- [API Keys](/docs/guides/api-keys)
