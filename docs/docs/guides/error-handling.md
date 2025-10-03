---
sidebar_position: 1
---

# Error Handling Guide

FerricLink provides a comprehensive error handling system that helps you build robust AI applications. This guide covers best practices and patterns for handling errors effectively.

## Error Types

FerricLink uses structured error types that provide detailed information about what went wrong and how to fix it.

### Basic Error Handling

```rust
use ferriclink_core::{FerricLinkError, Result};

fn process_data(data: &str) -> Result&lt;String&gt; {
    if data.is_empty() {
        return Err(FerricLinkError::validation("Data cannot be empty"));
    }
    
    // Process the data
    Ok(data.to_uppercase())
}
```

### Error Code Checking

```rust
use ferriclink_core::{FerricLinkError, ErrorCode};

fn handle_error(error: FerricLinkError) {
    match error.error_code() {
        Some(ErrorCode::ModelRateLimit) => {
            println!("Rate limited - retry later");
        }
        Some(ErrorCode::OutputParsingFailure) => {
            println!("Parsing failed - check output format");
        }
        Some(ErrorCode::ModelAuthentication) => {
            println!("Authentication failed - check API key");
        }
        _ => {
            println!("Other error: {}", error);
        }
    }
}
```

## LLM Feedback Patterns

FerricLink supports sending feedback to language models when parsing fails, allowing for automatic retries.

### Basic LLM Feedback

```rust
use ferriclink_core::{FerricLinkError, OutputParserException};

async fn parse_with_retry(output: &str) -> Result<serde_json::Value, FerricLinkError> {
    match serde_json::from_str::<serde_json::Value>(output) {
        Ok(parsed) => Ok(parsed),
        Err(e) => {
            // Create parser exception with LLM feedback
            let parser_err = OutputParserException::with_llm_context(
                format!("Invalid JSON: {}", e),
                Some("Please provide valid JSON output".to_string()),
                Some(output.to_string()),
                true, // Send back to LLM
            );
            Err(parser_err.into())
        }
    }
}
```

### Retry with Feedback

```rust
async fn retry_with_feedback(error: FerricLinkError) -> Result<String, FerricLinkError> {
    if error.should_send_to_llm() {
        if let Some((observation, llm_output)) = error.llm_context() {
            let retry_prompt = format!(
                "Previous output was invalid: {}\nObservation: {}\nPlease try again.",
                llm_output.unwrap_or(""),
                observation.unwrap_or("")
            );
            
            // Send retry request to LLM
            return call_llm_with_prompt(&retry_prompt).await;
        }
    }
    
    Err(error)
}
```

## Error Recovery Patterns

### Exponential Backoff

```rust
use ferriclink_core::FerricLinkError;
use tokio::time::{sleep, Duration};

async fn retry_with_backoff<F, T>(
    mut operation: F,
    max_retries: usize,
) -> Result<T, FerricLinkError>
where
    F: FnMut() -> Result<T, FerricLinkError>,
{
    let mut delay = Duration::from_secs(1);
    
    for attempt in 0..max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt < max_retries - 1 {
                    println!("Attempt {} failed: {}, retrying in {:?}", 
                            attempt + 1, e, delay);
                    sleep(delay).await;
                    delay *= 2; // Exponential backoff
                } else {
                    return Err(e);
                }
            }
        }
    }
    
    Err(FerricLinkError::runtime("Max retries exceeded"))
}
```

### Circuit Breaker Pattern

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

struct CircuitBreaker {
    is_open: Arc&lt;AtomicBool&gt;,
    last_failure: Arc&lt;Mutex&lt;Option&lt;Instant&gt;&gt;&gt;,
    timeout: Duration,
}

impl CircuitBreaker {
    fn new(timeout: Duration) -> Self {
        Self {
            is_open: Arc::new(AtomicBool::new(false)),
            last_failure: Arc::new(Mutex::new(None)),
            timeout,
        }
    }
    
    async fn call<F, T>(&self, operation: F) -> Result<T, FerricLinkError>
    where
        F: FnOnce() -> Result<T, FerricLinkError>,
    {
        if self.is_open.load(Ordering::Relaxed) {
            if let Some(last_failure) = *self.last_failure.lock().await {
                if last_failure.elapsed() < self.timeout {
                    return Err(FerricLinkError::runtime("Circuit breaker is open"));
                } else {
                    // Reset circuit breaker
                    self.is_open.store(false, Ordering::Relaxed);
                }
            }
        }
        
        match operation() {
            Ok(result) => {
                // Reset on success
                self.is_open.store(false, Ordering::Relaxed);
                Ok(result)
            }
            Err(e) => {
                // Open circuit breaker on failure
                self.is_open.store(true, Ordering::Relaxed);
                *self.last_failure.lock().await = Some(Instant::now());
                Err(e)
            }
        }
    }
}
```

## Logging and Monitoring

### Structured Logging

```rust
use ferriclink_core::FerricLinkError;
use tracing::{error, warn, info};

fn log_error(error: &FerricLinkError) {
    match error.error_code() {
        Some(ErrorCode::ModelRateLimit) => {
            warn!("Rate limit exceeded: {}", error);
        }
        Some(ErrorCode::ModelAuthentication) => {
            error!("Authentication failed: {}", error);
        }
        Some(ErrorCode::OutputParsingFailure) => {
            info!("Parsing failed, will retry: {}", error);
        }
        _ => {
            error!("Unexpected error: {}", error);
        }
    }
}
```

### Error Metrics

```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct ErrorMetrics {
    total_errors: AtomicU64,
    rate_limit_errors: AtomicU64,
    parsing_errors: AtomicU64,
}

impl ErrorMetrics {
    fn record_error(&self, error: &FerricLinkError) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
        
        match error.error_code() {
            Some(ErrorCode::ModelRateLimit) => {
                self.rate_limit_errors.fetch_add(1, Ordering::Relaxed);
            }
            Some(ErrorCode::OutputParsingFailure) => {
                self.parsing_errors.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }
}
```

## Best Practices

1. **Always handle errors explicitly** - Don't ignore `Result` types
2. **Use specific error types** - Choose the most appropriate error type
3. **Provide context** - Include relevant information in error messages
4. **Implement retries** - For transient errors like rate limits
5. **Log errors appropriately** - Use appropriate log levels
6. **Monitor error rates** - Track error patterns and trends
7. **Test error scenarios** - Include error cases in your tests

## Related Documentation

- [Troubleshooting](/docs/troubleshooting)
- [API Reference](/api/latest/ferriclink_core/errors)
- [Examples](/docs/examples)
