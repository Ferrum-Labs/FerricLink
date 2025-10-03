//! Interface for rate limiters and an in-memory rate limiter.
//!
//! This module provides rate limiting functionality for FerricLink, similar to
//! LangChain's rate_limiters.py with Rust-specific optimizations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::errors::Result;
use crate::impl_serializable;

/// Base trait for all rate limiters.
///
/// Usage of the base limiter is through the acquire and aacquire methods depending
/// on whether running in a sync or async context.
///
/// Implementations are free to add a timeout parameter to their initialize method
/// to allow users to specify a timeout for acquiring the necessary tokens when
/// using a blocking call.
///
/// Current limitations:
///
/// - Rate limiting information is not surfaced in tracing or callbacks. This means
///   that the total time it takes to invoke a chat model will encompass both
///   the time spent waiting for tokens and the time spent making the request.
#[async_trait]
pub trait BaseRateLimiter: Send + Sync {
    /// Attempt to acquire the necessary tokens for the rate limiter.
    ///
    /// This method blocks until the required tokens are available if `blocking`
    /// is set to true.
    ///
    /// If `blocking` is set to false, the method will immediately return the result
    /// of the attempt to acquire the tokens.
    ///
    /// # Arguments
    ///
    /// * `blocking` - If true, the method will block until the tokens are available.
    ///   If false, the method will return immediately with the result of
    ///   the attempt. Defaults to true.
    ///
    /// # Returns
    ///
    /// True if the tokens were successfully acquired, false otherwise.
    fn acquire(&self, blocking: bool) -> Result<bool>;

    /// Attempt to acquire the necessary tokens for the rate limiter. Async version.
    ///
    /// This method blocks until the required tokens are available if `blocking`
    /// is set to true.
    ///
    /// If `blocking` is set to false, the method will return immediately with the result
    /// of the attempt to acquire the tokens.
    ///
    /// # Arguments
    ///
    /// * `blocking` - If true, the method will block until the tokens are available.
    ///   If false, the method will return immediately with the result of
    ///   the attempt. Defaults to true.
    ///
    /// # Returns
    ///
    /// True if the tokens were successfully acquired, false otherwise.
    async fn aacquire(&self, blocking: bool) -> Result<bool>;
}

/// An in-memory rate limiter based on a token bucket algorithm.
///
/// This is an in-memory rate limiter, so it cannot rate limit across
/// different processes.
///
/// The rate limiter only allows time-based rate limiting and does not
/// take into account any information about the input or the output, so it
/// cannot be used to rate limit based on the size of the request.
///
/// It is thread safe and can be used in either a sync or async context.
///
/// The in-memory rate limiter is based on a token bucket. The bucket is filled
/// with tokens at a given rate. Each request consumes a token. If there are
/// not enough tokens in the bucket, the request is blocked until there are
/// enough tokens.
///
/// These *tokens* have NOTHING to do with LLM tokens. They are just
/// a way to keep track of how many requests can be made at a given time.
///
/// Current limitations:
///
/// - The rate limiter is not designed to work across different processes. It is
///   an in-memory rate limiter, but it is thread safe.
/// - The rate limiter only supports time-based rate limiting. It does not take
///   into account the size of the request or any other factors.
///
/// # Example
///
/// ```rust
/// use ferriclink_core::rate_limiters::InMemoryRateLimiter;
/// use std::time::Duration;
///
/// let rate_limiter = InMemoryRateLimiter::new(
///     0.1,  // Can only make a request once every 10 seconds
///     0.1,  // Wake up every 100 ms to check whether allowed to make a request
///     10.0, // Controls the maximum burst size
/// );
///
/// // Use with a language model
/// // let model = ChatAnthropic::new()
/// //     .with_rate_limiter(rate_limiter);
/// ```
#[derive(Debug, Clone)]
pub struct InMemoryRateLimiter {
    /// Number of requests that we can make per second
    requests_per_second: f64,
    /// Number of tokens in the bucket
    available_tokens: Arc<Mutex<f64>>,
    /// Maximum number of tokens that can be in the bucket
    max_bucket_size: f64,
    /// The last time we tried to consume tokens
    last: Arc<Mutex<Option<Instant>>>,
    /// Check whether tokens are available every this many seconds
    check_every_n_seconds: f64,
}

/// Serializable version of InMemoryRateLimiter for configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InMemoryRateLimiterConfig {
    /// Number of requests that we can make per second
    pub requests_per_second: f64,
    /// Maximum number of tokens that can be in the bucket
    pub max_bucket_size: f64,
    /// Check whether tokens are available every this many seconds
    pub check_every_n_seconds: f64,
}

impl InMemoryRateLimiter {
    /// Create a new in-memory rate limiter based on a token bucket.
    ///
    /// These *tokens* have NOTHING to do with LLM tokens. They are just
    /// a way to keep track of how many requests can be made at a given time.
    ///
    /// This rate limiter is designed to work in a threaded environment.
    ///
    /// It works by filling up a bucket with tokens at a given rate. Each
    /// request consumes a given number of tokens. If there are not enough
    /// tokens in the bucket, the request is blocked until there are enough
    /// tokens.
    ///
    /// # Arguments
    ///
    /// * `requests_per_second` - The number of tokens to add per second to the bucket.
    ///   The tokens represent "credit" that can be used to make requests.
    /// * `check_every_n_seconds` - Check whether the tokens are available
    ///   every this many seconds. Can be a float to represent
    ///   fractions of a second.
    /// * `max_bucket_size` - The maximum number of tokens that can be in the bucket.
    ///   Must be at least 1. Used to prevent bursts of requests.
    ///
    /// # Panics
    ///
    /// Panics if `max_bucket_size` is less than 1.0.
    pub fn new(
        requests_per_second: f64,
        check_every_n_seconds: f64,
        max_bucket_size: f64,
    ) -> Self {
        assert!(
            max_bucket_size >= 1.0,
            "max_bucket_size must be at least 1.0"
        );
        assert!(
            requests_per_second > 0.0,
            "requests_per_second must be greater than 0.0"
        );
        assert!(
            check_every_n_seconds > 0.0,
            "check_every_n_seconds must be greater than 0.0"
        );

        Self {
            requests_per_second,
            available_tokens: Arc::new(Mutex::new(1.0)), // Start with 1 token to allow first request
            max_bucket_size,
            last: Arc::new(Mutex::new(None)),
            check_every_n_seconds,
        }
    }

    /// Try to consume a token.
    ///
    /// # Returns
    ///
    /// True means that the tokens were consumed, and the caller can proceed to
    /// make the request. False means that the tokens were not consumed, and
    /// the caller should try again later.
    async fn consume(&self) -> Result<bool> {
        let mut available_tokens = self.available_tokens.lock().await;
        let mut last = self.last.lock().await;

        let now = Instant::now();

        // Initialize on first call to avoid a burst
        if last.is_none() {
            *last = Some(now);
        }

        let elapsed = now.duration_since(last.unwrap()).as_secs_f64();

        if elapsed * self.requests_per_second >= 1.0 {
            *available_tokens += elapsed * self.requests_per_second;
            *last = Some(now);
        }

        // Make sure that we don't exceed the bucket size.
        // This is used to prevent bursts of requests.
        *available_tokens = (*available_tokens).min(self.max_bucket_size);

        // As long as we have at least one token, we can proceed.
        if *available_tokens >= 1.0 {
            *available_tokens -= 1.0;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get the current number of available tokens
    pub async fn available_tokens(&self) -> f64 {
        *self.available_tokens.lock().await
    }

    /// Get the maximum bucket size
    pub fn max_bucket_size(&self) -> f64 {
        self.max_bucket_size
    }

    /// Get the requests per second rate
    pub fn requests_per_second(&self) -> f64 {
        self.requests_per_second
    }

    /// Get the check interval in seconds
    pub fn check_every_n_seconds(&self) -> f64 {
        self.check_every_n_seconds
    }

    /// Convert to a serializable configuration
    pub fn to_config(&self) -> InMemoryRateLimiterConfig {
        InMemoryRateLimiterConfig {
            requests_per_second: self.requests_per_second,
            max_bucket_size: self.max_bucket_size,
            check_every_n_seconds: self.check_every_n_seconds,
        }
    }

    /// Create from a serializable configuration
    pub fn from_config(config: InMemoryRateLimiterConfig) -> Self {
        Self::new(
            config.requests_per_second,
            config.check_every_n_seconds,
            config.max_bucket_size,
        )
    }
}

impl_serializable!(InMemoryRateLimiterConfig, ["ferriclink", "rate_limiters", "in_memory_rate_limiter_config"]);

#[async_trait]
impl BaseRateLimiter for InMemoryRateLimiter {
    fn acquire(&self, blocking: bool) -> Result<bool> {
        // For sync context, we need to use a blocking approach
        if !blocking {
            // Use tokio::runtime::Handle::try_current() to run async code in sync context
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                return handle.block_on(self.consume());
            } else {
                // If we're not in an async context, create a new runtime
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| crate::errors::FerricLinkError::runtime(format!("Failed to create runtime: {}", e)))?;
                return rt.block_on(self.consume());
            }
        }

        // For blocking mode, we need to poll until we can acquire
        loop {
            let acquired = if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.block_on(self.consume())?
            } else {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| crate::errors::FerricLinkError::runtime(format!("Failed to create runtime: {}", e)))?;
                rt.block_on(self.consume())?
            };

            if acquired {
                return Ok(true);
            }

            // Sleep for the check interval
            std::thread::sleep(Duration::from_secs_f64(self.check_every_n_seconds));
        }
    }

    async fn aacquire(&self, blocking: bool) -> Result<bool> {
        if !blocking {
            return self.consume().await;
        }

        loop {
            if self.consume().await? {
                return Ok(true);
            }

            sleep(Duration::from_secs_f64(self.check_every_n_seconds)).await;
        }
    }
}

/// A more advanced rate limiter that supports different rate limiting strategies.
#[derive(Debug, Clone)]
pub struct AdvancedRateLimiter {
    /// The underlying rate limiter
    inner: InMemoryRateLimiter,
    /// Additional configuration
    config: RateLimiterConfig,
}

/// Configuration for the advanced rate limiter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiterConfig {
    /// Whether to use exponential backoff on rate limit errors
    pub use_exponential_backoff: bool,
    /// Maximum backoff duration
    pub max_backoff_duration: Duration,
    /// Initial backoff duration
    pub initial_backoff_duration: Duration,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Whether to log rate limiting events
    pub log_events: bool,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            use_exponential_backoff: true,
            max_backoff_duration: Duration::from_secs(60),
            initial_backoff_duration: Duration::from_millis(100),
            max_retries: 5,
            log_events: false,
        }
    }
}

impl AdvancedRateLimiter {
    /// Create a new advanced rate limiter
    pub fn new(
        requests_per_second: f64,
        check_every_n_seconds: f64,
        max_bucket_size: f64,
        config: RateLimiterConfig,
    ) -> Self {
        Self {
            inner: InMemoryRateLimiter::new(requests_per_second, check_every_n_seconds, max_bucket_size),
            config,
        }
    }

    /// Acquire with retry logic and exponential backoff
    pub async fn acquire_with_retry(&self, blocking: bool) -> Result<bool> {
        let mut backoff_duration = self.config.initial_backoff_duration;
        let mut retries = 0;

        loop {
            match self.inner.aacquire(blocking).await {
                Ok(true) => {
                    if self.config.log_events {
                        println!("Rate limiter: Token acquired successfully");
                    }
                    return Ok(true);
                }
                Ok(false) => {
                    if !blocking {
                        return Ok(false);
                    }

                    if retries >= self.config.max_retries {
                        return Err(crate::errors::FerricLinkError::model_rate_limit(
                            "Max retries exceeded for rate limiter"
                        ));
                    }

                    if self.config.log_events {
                        println!(
                            "Rate limiter: Token not available, retrying in {:?} (attempt {})",
                            backoff_duration, retries + 1
                        );
                    }

                    sleep(backoff_duration).await;

                    if self.config.use_exponential_backoff {
                        backoff_duration = backoff_duration
                            .mul_f64(2.0)
                            .min(self.config.max_backoff_duration);
                    }

                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &RateLimiterConfig {
        &self.config
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: RateLimiterConfig) {
        self.config = config;
    }
}

#[async_trait]
impl BaseRateLimiter for AdvancedRateLimiter {
    fn acquire(&self, blocking: bool) -> Result<bool> {
        self.inner.acquire(blocking)
    }

    async fn aacquire(&self, blocking: bool) -> Result<bool> {
        self.acquire_with_retry(blocking).await
    }
}

impl_serializable!(RateLimiterConfig, ["ferriclink", "rate_limiters", "rate_limiter_config"]);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_rate_limiter_basic() {
        let rate_limiter = InMemoryRateLimiter::new(1.0, 0.1, 2.0);

        // First request should succeed
        assert!(rate_limiter.aacquire(true).await.unwrap());

        // Second request should fail immediately (no blocking)
        assert!(!rate_limiter.aacquire(false).await.unwrap());

        // Wait a bit and try again
        sleep(Duration::from_millis(1100)).await;
        assert!(rate_limiter.aacquire(true).await.unwrap());
    }

    #[tokio::test]
    async fn test_in_memory_rate_limiter_burst() {
        let rate_limiter = InMemoryRateLimiter::new(10.0, 0.01, 5.0);

        // First call initializes the timer and should succeed
        assert!(rate_limiter.aacquire(false).await.unwrap());

        // Wait enough time to accumulate more tokens
        sleep(Duration::from_millis(200)).await;

        // Should be able to make more requests
        let mut successful = 1; // We already made one successful request
        for _ in 0..4 {
            if rate_limiter.aacquire(false).await.unwrap() {
                successful += 1;
            }
        }

        // Should have made at least 2 successful requests total
        assert!(successful >= 2);
    }

    #[tokio::test]
    async fn test_advanced_rate_limiter() {
        let config = RateLimiterConfig {
            use_exponential_backoff: true,
            max_backoff_duration: Duration::from_secs(1),
            initial_backoff_duration: Duration::from_millis(10),
            max_retries: 3,
            log_events: false,
        };

        let rate_limiter = AdvancedRateLimiter::new(1.0, 0.01, 1.0, config);

        // First request should succeed
        assert!(rate_limiter.aacquire(true).await.unwrap());

        // Second request should succeed after retry
        assert!(rate_limiter.aacquire(true).await.unwrap());
    }

    #[test]
    fn test_rate_limiter_creation() {
        let rate_limiter = InMemoryRateLimiter::new(2.0, 0.1, 5.0);
        assert_eq!(rate_limiter.requests_per_second(), 2.0);
        assert_eq!(rate_limiter.check_every_n_seconds(), 0.1);
        assert_eq!(rate_limiter.max_bucket_size(), 5.0);
    }

    #[test]
    #[should_panic(expected = "max_bucket_size must be at least 1.0")]
    fn test_invalid_max_bucket_size() {
        InMemoryRateLimiter::new(1.0, 0.1, 0.5);
    }

    #[test]
    #[should_panic(expected = "requests_per_second must be greater than 0.0")]
    fn test_invalid_requests_per_second() {
        InMemoryRateLimiter::new(0.0, 0.1, 1.0);
    }

    #[test]
    #[should_panic(expected = "check_every_n_seconds must be greater than 0.0")]
    fn test_invalid_check_interval() {
        InMemoryRateLimiter::new(1.0, 0.0, 1.0);
    }

    #[tokio::test]
    async fn test_available_tokens() {
        let rate_limiter = InMemoryRateLimiter::new(1.0, 0.1, 2.0);

        // Initially should have 1 token (to allow first request)
        assert_eq!(rate_limiter.available_tokens().await, 1.0);

        // First call should succeed and consume the token
        rate_limiter.aacquire(false).await.unwrap();
        assert_eq!(rate_limiter.available_tokens().await, 0.0);

        // Wait enough time to accumulate more tokens (1 second for 1 token at 1 req/sec)
        sleep(Duration::from_millis(1100)).await;
        
        // Try to acquire a token to trigger token accumulation
        let acquired = rate_limiter.aacquire(false).await.unwrap();
        assert!(acquired, "Should have acquired a token after waiting");
        
        // After acquiring the token, should have fewer tokens (but not necessarily 0 due to accumulation)
        let tokens_after = rate_limiter.available_tokens().await;
        assert!(tokens_after >= 0.0);
    }

    #[tokio::test]
    async fn test_serialization() {
        let rate_limiter = InMemoryRateLimiter::new(2.0, 0.1, 5.0);
        let config = rate_limiter.to_config();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized_config: InMemoryRateLimiterConfig = serde_json::from_str(&serialized).unwrap();
        let deserialized_rate_limiter = InMemoryRateLimiter::from_config(deserialized_config);

        assert_eq!(rate_limiter.requests_per_second(), deserialized_rate_limiter.requests_per_second());
        assert_eq!(rate_limiter.check_every_n_seconds(), deserialized_rate_limiter.check_every_n_seconds());
        assert_eq!(rate_limiter.max_bucket_size(), deserialized_rate_limiter.max_bucket_size());
    }
}
