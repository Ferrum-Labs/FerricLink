---
sidebar_position: 4
---

# Caching Guide

FerricLink provides comprehensive caching functionality similar to LangChain's `caches.py`, with Rust-specific optimizations and additional features.

## Overview

Caching is essential for optimizing LLM applications by:

- **Reducing API costs** by avoiding duplicate requests
- **Improving response times** for repeated queries  
- **Providing resilience** against API failures
- **Enabling offline development** and testing

## Basic Usage

### InMemoryCache

The `InMemoryCache` is the core caching implementation, storing cached values in memory:

```rust
use ferriclink_core::{InMemoryCache, BaseCache};

// Create a cache with no size limit
let cache = InMemoryCache::new();

// Create a cache with size limit (LRU eviction)
let cache = InMemoryCache::with_max_size(Some(100));
```

### Basic Operations

```rust
use ferriclink_core::{InMemoryCache, BaseCache};

let cache = InMemoryCache::new();
let prompt = "What's the weather?";
let llm_string = "gpt-4o-mini:temperature=0.7";

// Look up cached result
if let Some(cached_result) = cache.lookup(prompt, llm_string)? {
    println!("Cache hit: {}", cached_result[0].text);
} else {
    // Make expensive LLM call
    let generations = expensive_llm_call(prompt).await;
    cache.update(prompt, llm_string, generations)?;
}
```

### Async Operations

```rust
use ferriclink_core::{InMemoryCache, BaseCache};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache = InMemoryCache::new();
    
    // Async lookup
    if let Some(cached_result) = cache.alookup(prompt, llm_string).await? {
        println!("Cache hit: {}", cached_result[0].text);
    } else {
        // Make expensive LLM call
        let generations = expensive_llm_call(prompt).await;
        cache.aupdate(prompt, llm_string, generations).await?;
    }
    
    Ok(())
}
```

## Advanced Caching

### TTL Cache

The `TtlCache` adds time-to-live functionality:

```rust
use ferriclink_core::{TtlCache, BaseCache};
use std::time::Duration;

// Create a TTL cache with 1 hour expiration
let cache = TtlCache::new(Duration::from_secs(3600), None);

// Entries automatically expire after 1 hour
cache.aupdate(prompt, llm_string, generations).await?;
```

### Cache Statistics

Monitor cache performance with built-in statistics:

```rust
use ferriclink_core::{InMemoryCache, BaseCache};

let cache = InMemoryCache::new();

// ... use cache ...

let stats = cache.stats().await;
println!("Hit Rate: {:.1}%", stats.hit_rate());
println!("Total Requests: {}", stats.total_requests());
println!("Cache Size: {}", stats.current_size);
```

## Integration with Language Models

### Basic Integration Pattern

```rust
use ferriclink_core::{InMemoryCache, BaseCache, language_models::Generation};

struct CachedLLM {
    cache: InMemoryCache,
    // ... other fields
}

impl CachedLLM {
    async fn generate_with_cache(
        &self,
        prompt: &str,
        llm_string: &str,
    ) -> Result&lt;Vec&lt;Generation&gt;&gt; {
        // Check cache first
        if let Some(cached) = self.cache.alookup(prompt, llm_string).await? {
            return Ok(cached);
        }
        
        // Make LLM call
        let generations = self.make_llm_call(prompt).await?;
        
        // Cache the result
        self.cache.aupdate(prompt, llm_string, generations.clone()).await?;
        
        Ok(generations)
    }
}
```

### Advanced Integration with Error Handling

```rust
use ferriclink_core::{InMemoryCache, BaseCache, errors::Result};

struct RobustCachedLLM {
    cache: InMemoryCache,
    fallback_cache: Option&lt;InMemoryCache&gt;,
}

impl RobustCachedLLM {
    async fn generate_with_fallback(
        &self,
        prompt: &str,
        llm_string: &str,
    ) -> Result&lt;Vec&lt;Generation&gt;&gt; {
        // Try primary cache
        if let Some(cached) = self.cache.alookup(prompt, llm_string).await? {
            return Ok(cached);
        }
        
        // Try fallback cache if available
        if let Some(fallback) = &self.fallback_cache {
            if let Some(cached) = fallback.alookup(prompt, llm_string).await? {
                // Update primary cache
                self.cache.aupdate(prompt, llm_string, cached.clone()).await?;
                return Ok(cached);
            }
        }
        
        // Make LLM call
        let generations = self.make_llm_call(prompt).await?;
        
        // Update both caches
        self.cache.aupdate(prompt, llm_string, generations.clone()).await?;
        if let Some(fallback) = &self.fallback_cache {
            let _ = fallback.aupdate(prompt, llm_string, generations.clone()).await;
        }
        
        Ok(generations)
    }
}
```

## Performance Optimization

### Cache Key Strategy

The cache uses `(prompt, llm_string)` as the key. Optimize by:

```rust
// Good: Include all relevant parameters in llm_string
let llm_string = format!("gpt-4o-mini:temperature={}:max_tokens={}", 
                        temperature, max_tokens);

// Bad: Missing parameters can cause cache misses
let llm_string = "gpt-4o-mini";
```

### Memory Management

```rust
// Monitor cache size
let stats = cache.stats().await;
if stats.current_size > 1000 {
    println!("Cache is getting large: {} entries", stats.current_size);
}

// Clear cache when needed
cache.clear().await?;
```

### Batch Operations

```rust
// Process multiple prompts efficiently
async fn process_prompts_batch(
    cache: &InMemoryCache,
    prompts: Vec<&str>,
    llm_string: &str,
) -> Result&lt;Vec&lt;Vec&lt;Generation&gt;&gt;&gt; {
    let mut results = Vec::new();
    
    for prompt in prompts {
        if let Some(cached) = cache.alookup(prompt, llm_string).await? {
            results.push(cached);
        } else {
            let generations = expensive_llm_call(prompt).await;
            cache.aupdate(prompt, llm_string, generations.clone()).await?;
            results.push(generations);
        }
    }
    
    Ok(results)
}
```

## Best Practices

### 1. Choose Appropriate Cache Size

```rust
// For development: small cache
let dev_cache = InMemoryCache::with_max_size(Some(100));

// For production: larger cache
let prod_cache = InMemoryCache::with_max_size(Some(10000));
```

### 2. Use TTL for Time-Sensitive Data

```rust
// Short TTL for real-time data
let weather_cache = TtlCache::new(Duration::from_secs(300), None); // 5 minutes

// Longer TTL for stable data
let knowledge_cache = TtlCache::new(Duration::from_secs(3600), None); // 1 hour
```

### 3. Monitor Cache Performance

```rust
async fn monitor_cache_performance(cache: &InMemoryCache) {
    let stats = cache.stats().await;
    
    if stats.hit_rate() < 50.0 {
        println!("Warning: Low cache hit rate: {:.1}%", stats.hit_rate());
    }
    
    if stats.current_size > 10000 {
        println!("Warning: Cache size is large: {}", stats.current_size);
    }
}
```

### 4. Handle Cache Failures Gracefully

```rust
async fn safe_cache_lookup(
    cache: &InMemoryCache,
    prompt: &str,
    llm_string: &str,
) -> Result&lt;Option&lt;Vec&lt;Generation&gt;&gt;&gt; {
    match cache.alookup(prompt, llm_string).await {
        Ok(result) => Ok(result),
        Err(e) => {
            eprintln!("Cache lookup failed: {}", e);
            Ok(None) // Continue without cache
        }
    }
}
```

## Comparison with LangChain

| Feature | LangChain Python | FerricLink Rust |
|---------|------------------|-----------------|
| **Base Interface** | `BaseCache` | `BaseCache` |
| **In-Memory Cache** | `InMemoryCache` | `InMemoryCache` |
| **Size Limits** | ✅ | ✅ (with LRU) |
| **TTL Support** | ❌ | ✅ (`TtlCache`) |
| **Statistics** | ❌ | ✅ (`CacheStats`) |
| **Thread Safety** | ✅ | ✅ (Arc&lt;RwLock&gt;) |
| **Async Support** | ✅ | ✅ |
| **Memory Efficiency** | Medium | High |
| **Performance** | Medium | High |

## Troubleshooting

### Common Issues

1. **Low Hit Rate**: Check if cache keys are consistent
2. **Memory Usage**: Monitor cache size and use appropriate limits
3. **Stale Data**: Use TTL cache for time-sensitive data
4. **Thread Safety**: Ensure proper async/await usage

### Debug Cache Behavior

```rust
async fn debug_cache(cache: &InMemoryCache, prompt: &str, llm_string: &str) {
    println!("Cache key: '{}' + '{}'", prompt, llm_string);
    
    let stats_before = cache.stats().await;
    let result = cache.alookup(prompt, llm_string).await.unwrap();
    let stats_after = cache.stats().await;
    
    println!("Before: hits={}, misses={}", stats_before.hits, stats_before.misses);
    println!("After: hits={}, misses={}", stats_after.hits, stats_after.misses);
    println!("Result: {:?}", result.is_some());
}
```

## Examples

See the [cache usage example](../../examples/cache_usage) for a complete working demonstration of all caching features.
