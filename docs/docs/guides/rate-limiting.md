---
sidebar_position: 3
---

# Rate Limiting Guide

FerricLink provides comprehensive rate limiting functionality similar to LangChain's `rate_limiters.py`, with Rust-specific optimizations and additional features.

## Overview

Rate limiting is essential for managing API calls to language models and other services that have usage restrictions. FerricLink's rate limiting system uses a token bucket algorithm that allows for both steady-state rate limiting and burst capacity.

## Basic Usage

### InMemoryRateLimiter

The `InMemoryRateLimiter` is the core rate limiting implementation, based on a token bucket algorithm:

```rust
use ferriclink_core::{InMemoryRateLimiter, BaseRateLimiter};

// Create a rate limiter: 1 request per second, max burst of 2
let rate_limiter = InMemoryRateLimiter::new(1.0, 0.1, 2.0);

// Acquire a token (blocking)
let acquired = rate_limiter.acquire(true)?;
if acquired {
    // Make your API call
    println!("Request allowed");
} else {
    println!("Request denied");
}

// Acquire a token (non-blocking)
let acquired = rate_limiter.acquire(false)?;
```

### Async Usage

```rust
use ferriclink_core::{InMemoryRateLimiter, BaseRateLimiter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rate_limiter = InMemoryRateLimiter::new(2.0, 0.1, 5.0);

    // Async acquire (blocking)
    let acquired = rate_limiter.aacquire(true).await?;
    
    // Async acquire (non-blocking)
    let acquired = rate_limiter.aacquire(false).await?;
    
    Ok(())
}
```

## Advanced Rate Limiting

### AdvancedRateLimiter with Retry Logic

The `AdvancedRateLimiter` adds retry logic and exponential backoff:

```rust
use ferriclink_core::{AdvancedRateLimiter, RateLimiterConfig};
use std::time::Duration;

let config = RateLimiterConfig {
    use_exponential_backoff: true,
    max_backoff_duration: Duration::from_secs(10),
    initial_backoff_duration: Duration::from_millis(100),
    max_retries: 5,
    log_events: true,
};

let rate_limiter = AdvancedRateLimiter::new(1.0, 0.1, 2.0, config);

// This will automatically retry with exponential backoff
let acquired = rate_limiter.aacquire(true).await?;
```

### Configuration Options

```rust
use ferriclink_core::RateLimiterConfig;
use std::time::Duration;

let config = RateLimiterConfig {
    // Enable exponential backoff on failures
    use_exponential_backoff: true,
    
    // Maximum backoff duration
    max_backoff_duration: Duration::from_secs(60),
    
    // Initial backoff duration
    initial_backoff_duration: Duration::from_millis(100),
    
    // Maximum number of retries
    max_retries: 10,
    
    // Enable logging of rate limiting events
    log_events: true,
};
```

## Rate Limiting Strategies

### Conservative Rate Limiting

For production environments with strict rate limits:

```rust
let conservative = InMemoryRateLimiter::new(
    1.0,   // 1 request per second
    0.1,   // Check every 100ms
    2.0    // Max burst of 2 requests
);
```

### Aggressive Rate Limiting

For testing or development with higher limits:

```rust
let aggressive = InMemoryRateLimiter::new(
    10.0,  // 10 requests per second
    0.01,  // Check every 10ms
    5.0    // Max burst of 5 requests
);
```

### Adaptive Rate Limiting

For dynamic environments with retry logic:

```rust
let config = RateLimiterConfig {
    use_exponential_backoff: true,
    max_backoff_duration: Duration::from_secs(5),
    initial_backoff_duration: Duration::from_millis(50),
    max_retries: 10,
    log_events: true,
};

let adaptive = AdvancedRateLimiter::new(2.0, 0.05, 3.0, config);
```

## LangChain Compatibility

FerricLink's rate limiters are designed to be compatible with LangChain's interface:

```rust
use ferriclink_core::{InMemoryRateLimiter, BaseRateLimiter};

// LangChain-compatible usage
let rate_limiter = InMemoryRateLimiter::new(0.1, 0.1, 1.0); // 1 request every 10 seconds

// Simulate LangChain model calls
for i in 1..=3 {
    println!("Call {}: Acquiring rate limit token...", i);
    
    let acquire_start = std::time::Instant::now();
    let acquired = rate_limiter.aacquire(true).await?;
    let acquire_duration = acquire_start.elapsed();
    
    if acquired {
        println!("Call {}: Token acquired (waited {:?})", i, acquire_duration);
        
        // Simulate the actual model call
        println!("Call {}: Making model request...", i);
        tokio::time::sleep(Duration::from_millis(100)).await;
        println!("Call {}: Model request completed", i);
    } else {
        println!("Call {}: Failed to acquire token", i);
    }
}
```

## Serialization and Configuration

### Saving Rate Limiter Configuration

```rust
use ferriclink_core::{InMemoryRateLimiter, InMemoryRateLimiterConfig};

let rate_limiter = InMemoryRateLimiter::new(2.0, 0.1, 5.0);

// Convert to serializable config
let config = rate_limiter.to_config();

// Serialize to JSON
let json = serde_json::to_string_pretty(&config)?;
println!("Rate limiter config: {}", json);

// Save to file
std::fs::write("rate_limiter_config.json", json)?;
```

### Loading Rate Limiter Configuration

```rust
use ferriclink_core::{InMemoryRateLimiter, InMemoryRateLimiterConfig};

// Load from file
let json = std::fs::read_to_string("rate_limiter_config.json")?;
let config: InMemoryRateLimiterConfig = serde_json::from_str(&json)?;

// Create rate limiter from config
let rate_limiter = InMemoryRateLimiter::from_config(config);
```

## Monitoring and Debugging

### Check Available Tokens

```rust
let rate_limiter = InMemoryRateLimiter::new(1.0, 0.1, 2.0);

// Check current token count
let tokens = rate_limiter.available_tokens().await;
println!("Available tokens: {:.2}", tokens);

// Check rate limiter properties
println!("Requests per second: {}", rate_limiter.requests_per_second());
println!("Max bucket size: {}", rate_limiter.max_bucket_size());
println!("Check interval: {}s", rate_limiter.check_every_n_seconds());
```

### Performance Monitoring

```rust
use std::time::Instant;

let rate_limiter = InMemoryRateLimiter::new(100.0, 0.001, 10.0);
let mut successful = 0;
let mut failed = 0;

let start = Instant::now();

for i in 1..=100 {
    let acquired = rate_limiter.aacquire(false).await?;
    if acquired {
        successful += 1;
    } else {
        failed += 1;
    }
    
    if i % 10 == 0 {
        println!("Progress: {}/100 ({} successful, {} failed)", i, successful, failed);
    }
}

let duration = start.elapsed();
let requests_per_second = 100.0 / duration.as_secs_f64();

println!("Results:");
println!("  - Successful: {}", successful);
println!("  - Failed: {}", failed);
println!("  - Total time: {:?}", duration);
println!("  - Effective rate: {:.2} requests/second", requests_per_second);
```

## Error Handling

### Rate Limit Errors

```rust
use ferriclink_core::{FerricLinkError, ErrorCode};

match rate_limiter.aacquire(false).await {
    Ok(true) => {
        // Request allowed
        make_api_call().await?;
    }
    Ok(false) => {
        // Request denied, try again later
        println!("Rate limited, retrying later...");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(e) => {
        // Handle error
        if let Some(code) = e.error_code() {
            match code {
                ErrorCode::ModelRateLimit => {
                    println!("Model rate limit exceeded");
                }
                _ => {
                    println!("Other error: {}", e);
                }
            }
        }
    }
}
```

### Retry with Exponential Backoff

```rust
use ferriclink_core::AdvancedRateLimiter;

let config = RateLimiterConfig {
    use_exponential_backoff: true,
    max_backoff_duration: Duration::from_secs(60),
    initial_backoff_duration: Duration::from_millis(100),
    max_retries: 5,
    log_events: true,
};

let rate_limiter = AdvancedRateLimiter::new(1.0, 0.1, 2.0, config);

// This will automatically handle retries with exponential backoff
let acquired = rate_limiter.aacquire(true).await?;
```

## Best Practices

### 1. Choose Appropriate Rates

- **Conservative**: Use lower rates for production APIs with strict limits
- **Aggressive**: Use higher rates for testing or APIs with generous limits
- **Adaptive**: Use retry logic for unreliable or variable-rate APIs

### 2. Monitor Performance

- Track successful vs failed requests
- Monitor effective request rates
- Adjust parameters based on actual usage patterns

### 3. Handle Errors Gracefully

- Implement proper error handling for rate limit failures
- Use exponential backoff for retries
- Log rate limiting events for debugging

### 4. Use Configuration Files

- Save rate limiter configurations to files
- Load configurations at runtime
- Allow easy adjustment without code changes

### 5. Test Thoroughly

- Test with different rate limiting scenarios
- Verify behavior under load
- Ensure proper token accumulation and consumption

## Related Documentation

- [Error Handling](/docs/guides/error-handling)
- [Environment Information](/docs/guides/environment-information)
- [API Reference](/api/latest/ferriclink_core/rate_limiters)
