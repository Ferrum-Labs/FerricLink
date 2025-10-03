---
sidebar_position: 4
---

# Model Rate Limit Error

**Error Code:** `MODEL_RATE_LIMIT`

This error occurs when you exceed the rate limits imposed by the model provider.

## Common Causes

1. **Too Many Requests**: Exceeding requests per minute/hour limits
2. **Token Limits**: Exceeding token usage limits
3. **Concurrent Requests**: Too many simultaneous requests
4. **Account Limits**: Exceeding your account's usage limits

## Solutions

### 1. Implement Exponential Backoff

```rust
use ferriclink_core::FerricLinkError;
use tokio::time::{sleep, Duration};

async fn retry_with_backoff<F, T>(mut operation: F) -> Result<T, FerricLinkError>
where
    F: FnMut() -> Result<T, FerricLinkError>,
{
    let mut delay = Duration::from_secs(1);
    let max_retries = 5;
    
    for attempt in 0..max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                if e.error_code() == Some(ErrorCode::ModelRateLimit) {
                    if attempt < max_retries - 1 {
                        println!("Rate limited, waiting {:?} before retry {}", delay, attempt + 1);
                        sleep(delay).await;
                        delay *= 2; // Exponential backoff
                        continue;
                    }
                }
                return Err(e);
            }
        }
    }
    
    Err(FerricLinkError::model_rate_limit("Max retries exceeded"))
}
```

### 2. Rate Limiting with Tokens

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

struct RateLimiter {
    semaphore: Arc<Semaphore>,
    max_requests_per_minute: usize,
}

impl RateLimiter {
    fn new(max_requests_per_minute: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_requests_per_minute)),
            max_requests_per_minute,
        }
    }
    
    async fn acquire(&self) -> Result<(), FerricLinkError> {
        self.semaphore.acquire().await
            .map_err(|_| FerricLinkError::model_rate_limit("Failed to acquire rate limit permit"))?
            .forget();
        Ok(())
    }
}
```

### 3. Monitor Usage

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct UsageMonitor {
    requests: HashMap<String, Vec<Instant>>,
    window: Duration,
}

impl UsageMonitor {
    fn new(window: Duration) -> Self {
        Self {
            requests: HashMap::new(),
            window,
        }
    }
    
    fn can_make_request(&mut self, key: &str, limit: usize) -> bool {
        let now = Instant::now();
        let requests = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        requests.retain(|&time| now.duration_since(time) < self.window);
        
        if requests.len() < limit {
            requests.push(now);
            true
        } else {
            false
        }
    }
}
```

## Prevention

- Implement proper rate limiting
- Use exponential backoff for retries
- Monitor your usage patterns
- Consider upgrading your plan
- Batch requests when possible
- Cache responses when appropriate

## Related Documentation

- [Rate Limiting](/docs/guides/rate-limiting)
- [Error Handling](/docs/guides/error-handling)
- [Best Practices](/docs/guides/best-practices)
