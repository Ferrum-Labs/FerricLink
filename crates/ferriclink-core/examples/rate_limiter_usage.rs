//! # Rate Limiter Usage Example
//!
//! This example demonstrates how to use the rate limiting system
//! in FerricLink, similar to LangChain's rate_limiters.py functionality.

use ferriclink_core::{
    BaseRateLimiter, InMemoryRateLimiter, AdvancedRateLimiter, RateLimiterConfig,
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FerricLink Rate Limiter Example ===\n");

    // Example 1: Basic InMemoryRateLimiter
    println!("1. Basic InMemoryRateLimiter:");
    basic_rate_limiter_example().await?;
    println!();

    // Example 2: Advanced Rate Limiter with Retry Logic
    println!("2. Advanced Rate Limiter with Retry Logic:");
    advanced_rate_limiter_example().await?;
    println!();

    // Example 3: Burst Rate Limiting
    println!("3. Burst Rate Limiting:");
    burst_rate_limiter_example().await?;
    println!();

    // Example 4: Rate Limiter Configuration
    println!("4. Rate Limiter Configuration:");
    rate_limiter_config_example().await?;
    println!();

    // Example 5: Performance Testing
    println!("5. Performance Testing:");
    performance_test_example().await?;
    println!();

    // Example 6: LangChain-Compatible Usage
    println!("6. LangChain-Compatible Usage:");
    langchain_compatible_example().await?;
    println!();

    println!("=== Example Complete ===");
    Ok(())
}

/// Basic rate limiter example
async fn basic_rate_limiter_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a rate limiter that allows 1 request per second
    let rate_limiter = InMemoryRateLimiter::new(1.0, 0.1, 2.0);

    println!("  - Rate limiter: 1 request/second, max burst: 2");
    println!("  - Making 3 requests...");

    let start = Instant::now();

    for i in 1..=3 {
        let request_start = Instant::now();
        
        // Try to acquire a token (blocking)
        let acquired = rate_limiter.aacquire(true).await?;
        
        let request_duration = request_start.elapsed();
        
        if acquired {
            println!("    Request {}: SUCCESS (waited {:?})", i, request_duration);
        } else {
            println!("    Request {}: FAILED (waited {:?})", i, request_duration);
        }
    }

    let total_duration = start.elapsed();
    println!("  - Total time: {:?}", total_duration);
    println!("  - Available tokens: {:.2}", rate_limiter.available_tokens().await);

    Ok(())
}

/// Advanced rate limiter example with retry logic
async fn advanced_rate_limiter_example() -> Result<(), Box<dyn std::error::Error>> {
    let config = RateLimiterConfig {
        use_exponential_backoff: true,
        max_backoff_duration: Duration::from_secs(2),
        initial_backoff_duration: Duration::from_millis(50),
        max_retries: 5,
        log_events: true,
    };

    let rate_limiter = AdvancedRateLimiter::new(0.5, 0.05, 1.0, config);

    println!("  - Advanced rate limiter: 0.5 requests/second with retry logic");
    println!("  - Making 3 requests...");

    for i in 1..=3 {
        let start = Instant::now();
        
        match rate_limiter.aacquire(true).await {
            Ok(true) => {
                println!("    Request {}: SUCCESS (took {:?})", i, start.elapsed());
            }
            Ok(false) => {
                println!("    Request {}: FAILED (took {:?})", i, start.elapsed());
            }
            Err(e) => {
                println!("    Request {}: ERROR - {}", i, e);
            }
        }
    }

    Ok(())
}

/// Burst rate limiting example
async fn burst_rate_limiter_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a rate limiter that allows bursts up to 5 requests
    let rate_limiter = InMemoryRateLimiter::new(10.0, 0.01, 5.0);

    println!("  - Rate limiter: 10 requests/second, max burst: 5");
    println!("  - Testing burst capacity...");

    let start = Instant::now();
    let mut successful_requests = 0;

    // Try to make 7 requests quickly (should only succeed for first 5)
    for i in 1..=7 {
        let acquired = rate_limiter.aacquire(false).await?;
        if acquired {
            successful_requests += 1;
            println!("    Request {}: SUCCESS (burst)", i);
        } else {
            println!("    Request {}: FAILED (rate limited)", i);
        }
    }

    let duration = start.elapsed();
    println!("  - Successful requests: {}/7", successful_requests);
    println!("  - Time taken: {:?}", duration);
    println!("  - Available tokens: {:.2}", rate_limiter.available_tokens().await);

    // Wait a bit and try again
    println!("  - Waiting 1 second and trying again...");
    sleep(Duration::from_secs(1)).await;

    let acquired = rate_limiter.aacquire(false).await?;
    if acquired {
        println!("    Request after wait: SUCCESS");
    } else {
        println!("    Request after wait: FAILED");
    }

    Ok(())
}

/// Rate limiter configuration example
async fn rate_limiter_config_example() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = RateLimiterConfig::default();
    config.log_events = true;
    config.max_retries = 3;
    config.initial_backoff_duration = Duration::from_millis(100);

    let mut rate_limiter = AdvancedRateLimiter::new(0.2, 0.1, 1.0, config);

    println!("  - Configuration:");
    println!("    - Max retries: {}", rate_limiter.config().max_retries);
    println!("    - Initial backoff: {:?}", rate_limiter.config().initial_backoff_duration);
    println!("    - Max backoff: {:?}", rate_limiter.config().max_backoff_duration);
    println!("    - Exponential backoff: {}", rate_limiter.config().use_exponential_backoff);

    // Update configuration
    let new_config = RateLimiterConfig {
        use_exponential_backoff: false,
        max_backoff_duration: Duration::from_secs(1),
        initial_backoff_duration: Duration::from_millis(200),
        max_retries: 2,
        log_events: false,
    };

    rate_limiter.update_config(new_config);

    println!("  - Updated configuration:");
    println!("    - Max retries: {}", rate_limiter.config().max_retries);
    println!("    - Initial backoff: {:?}", rate_limiter.config().initial_backoff_duration);
    println!("    - Exponential backoff: {}", rate_limiter.config().use_exponential_backoff);

    Ok(())
}

/// Performance testing example
async fn performance_test_example() -> Result<(), Box<dyn std::error::Error>> {
    let rate_limiter = InMemoryRateLimiter::new(100.0, 0.001, 10.0);

    println!("  - Performance test: 100 requests/second, 10 burst capacity");
    println!("  - Making 50 requests...");

    let start = Instant::now();
    let mut successful = 0;
    let mut failed = 0;

    for i in 1..=50 {
        let acquired = rate_limiter.aacquire(false).await?;
        if acquired {
            successful += 1;
        } else {
            failed += 1;
        }

        // Print progress every 10 requests
        if i % 10 == 0 {
            println!("    Progress: {}/50 ({} successful, {} failed)", i, successful, failed);
        }
    }

    let duration = start.elapsed();
    let requests_per_second = 50.0 / duration.as_secs_f64();

    println!("  - Results:");
    println!("    - Successful: {}", successful);
    println!("    - Failed: {}", failed);
    println!("    - Total time: {:?}", duration);
    println!("    - Effective rate: {:.2} requests/second", requests_per_second);

    Ok(())
}

/// LangChain-compatible usage example
async fn langchain_compatible_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("  - LangChain-compatible rate limiter usage:");
    
    // This mimics how LangChain uses rate limiters
    let rate_limiter = InMemoryRateLimiter::new(0.1, 0.1, 1.0); // 1 request every 10 seconds

    println!("  - Simulating LangChain model calls with rate limiting...");

    for i in 1..=3 {
        println!("    Call {}: Acquiring rate limit token...", i);
        
        let acquire_start = Instant::now();
        let acquired = rate_limiter.aacquire(true).await?;
        let acquire_duration = acquire_start.elapsed();
        
        if acquired {
            println!("    Call {}: Token acquired (waited {:?})", i, acquire_duration);
            
            // Simulate the actual model call
            println!("    Call {}: Making model request...", i);
            sleep(Duration::from_millis(100)).await; // Simulate API call
            println!("    Call {}: Model request completed", i);
        } else {
            println!("    Call {}: Failed to acquire token", i);
        }
    }

    Ok(())
}

/// Example of using rate limiters with different strategies
async fn rate_limiter_strategies_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("  - Different rate limiting strategies:");

    // Conservative rate limiter for production
    let conservative = InMemoryRateLimiter::new(1.0, 0.1, 2.0);
    println!("    Conservative: 1 req/sec, burst 2");

    // Aggressive rate limiter for testing
    let aggressive = InMemoryRateLimiter::new(10.0, 0.01, 5.0);
    println!("    Aggressive: 10 req/sec, burst 5");

    // Adaptive rate limiter with retry logic
    let adaptive_config = RateLimiterConfig {
        use_exponential_backoff: true,
        max_backoff_duration: Duration::from_secs(5),
        initial_backoff_duration: Duration::from_millis(50),
        max_retries: 10,
        log_events: true,
    };
    let adaptive = AdvancedRateLimiter::new(2.0, 0.05, 3.0, adaptive_config);
    println!("    Adaptive: 2 req/sec, burst 3, with retry logic");

    Ok(())
}
