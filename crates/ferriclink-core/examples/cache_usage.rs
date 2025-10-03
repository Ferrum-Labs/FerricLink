//! Example demonstrating FerricLink's caching functionality.
//!
//! This example shows how to use the caching system to improve performance
//! and reduce costs when working with language models.

use ferriclink_core::{
    caches::{BaseCache, InMemoryCache, TtlCache},
    language_models::Generation,
};
use std::time::Duration;

/// Create a mock generation for testing
fn create_mock_generation(text: &str) -> Generation {
    Generation {
        text: text.to_string(),
        generation_info: std::collections::HashMap::new(),
    }
}

/// Simulate an expensive LLM call
async fn expensive_llm_call(prompt: &str, model: &str) -> Vec<Generation> {
    // Simulate network delay
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Simulate different responses based on prompt
    let response = match prompt {
        "Hello" => "Hi there! How can I help you today?",
        "What's the weather?" => "I don't have access to real-time weather data, but I can help you find weather information!",
        "Tell me a joke" => "Why don't scientists trust atoms? Because they make up everything!",
        _ => "I'm not sure how to respond to that. Could you try rephrasing your question?",
    };
    
    println!("🤖 LLM Call: '{}' -> '{}' (model: {})", prompt, response, model);
    
    vec![create_mock_generation(response)]
}

/// Demonstrate basic cache usage
async fn basic_cache_example() {
    println!("=== Basic Cache Example ===");
    
    let cache = InMemoryCache::new();
    let prompt = "Hello";
    let llm_string = "gpt-4o-mini:temperature=0.7";
    
    // First call - cache miss
    println!("First call (should be cache miss):");
    let start = std::time::Instant::now();
    let result = cache.alookup(prompt, llm_string).await.unwrap();
    let duration = start.elapsed();
    
    if result.is_none() {
        println!("Cache miss! Making expensive LLM call...");
        let generations = expensive_llm_call(prompt, "gpt-4o-mini").await;
        cache.aupdate(prompt, llm_string, generations).await.unwrap();
        println!("Response cached successfully");
    }
    
    println!("First call took: {:?}", duration);
    
    // Second call - cache hit
    println!("\nSecond call (should be cache hit):");
    let start = std::time::Instant::now();
    let result = cache.alookup(prompt, llm_string).await.unwrap();
    let duration = start.elapsed();
    
    if let Some(generations) = result {
        println!("Cache hit! Got response: {}", generations[0].text);
    }
    
    println!("Second call took: {:?}", duration);
    
    // Show cache statistics
    let stats = cache.stats().await;
    println!("\nCache Statistics:");
    println!("  Hits: {}", stats.hits);
    println!("  Misses: {}", stats.misses);
    println!("  Hit Rate: {:.1}%", stats.hit_rate());
    println!("  Current Size: {}", stats.current_size);
}

/// Demonstrate cache with size limits
async fn size_limited_cache_example() {
    println!("\n=== Size Limited Cache Example ===");
    
    let cache = InMemoryCache::with_max_size(Some(2));
    
    // Add entries up to the limit
    let prompts = ["Hello", "What's the weather?", "Tell me a joke"];
    let llm_string = "gpt-4o-mini:temperature=0.7";
    
    for prompt in &prompts {
        println!("Adding: '{}'", prompt);
        let generations = expensive_llm_call(prompt, "gpt-4o-mini").await;
        cache.aupdate(prompt, llm_string, generations).await.unwrap();
    }
    
    // Check what's in the cache
    println!("\nCache contents after adding 3 items (max size: 2):");
    for prompt in &prompts {
        let result = cache.alookup(prompt, llm_string).await.unwrap();
        if result.is_some() {
            println!("  '{}' -> Cached", prompt);
        } else {
            println!("  '{}' -> Evicted", prompt);
        }
    }
    
    let stats = cache.stats().await;
    println!("\nCache Statistics:");
    println!("  Current Size: {}", stats.current_size);
    println!("  Max Size Reached: {}", stats.max_size_reached);
}

/// Demonstrate TTL cache
async fn ttl_cache_example() {
    println!("\n=== TTL Cache Example ===");
    
    let cache = TtlCache::new(Duration::from_millis(500), None);
    let prompt = "Hello";
    let llm_string = "gpt-4o-mini:temperature=0.7";
    
    // Add entry
    println!("Adding entry with 500ms TTL...");
    let generations = expensive_llm_call(prompt, "gpt-4o-mini").await;
    cache.aupdate(prompt, llm_string, generations).await.unwrap();
    
    // Immediate lookup - should hit
    let result = cache.alookup(prompt, llm_string).await.unwrap();
    if result.is_some() {
        println!("Immediate lookup: Cache hit!");
    }
    
    // Wait for expiration
    println!("Waiting for TTL expiration (500ms)...");
    tokio::time::sleep(Duration::from_millis(600)).await;
    
    // Lookup after expiration - should miss
    let result = cache.alookup(prompt, llm_string).await.unwrap();
    if result.is_none() {
        println!("After expiration: Cache miss (entry expired)");
    }
    
    let stats = cache.stats().await;
    println!("\nTTL Cache Statistics:");
    println!("  Hits: {}", stats.hits);
    println!("  Misses: {}", stats.misses);
}

/// Demonstrate cache performance benefits
async fn performance_example() {
    println!("\n=== Performance Example ===");
    
    let cache = InMemoryCache::new();
    let prompt = "What's the weather?";
    let llm_string = "gpt-4o-mini:temperature=0.7";
    
    // Warm up the cache
    let generations = expensive_llm_call(prompt, "gpt-4o-mini").await;
    cache.aupdate(prompt, llm_string, generations).await.unwrap();
    
    // Benchmark cached vs uncached calls
    let iterations = 10;
    
    println!("Benchmarking {} iterations...", iterations);
    
    // Cached calls
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = cache.alookup(prompt, llm_string).await.unwrap();
    }
    let cached_duration = start.elapsed();
    
    // Simulate uncached calls (without cache)
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = expensive_llm_call(prompt, "gpt-4o-mini").await;
    }
    let uncached_duration = start.elapsed();
    
    println!("Results:");
    println!("  Cached calls: {:?} ({:.2}ms per call)", 
             cached_duration, cached_duration.as_millis() as f64 / iterations as f64);
    println!("  Uncached calls: {:?} ({:.2}ms per call)", 
             uncached_duration, uncached_duration.as_millis() as f64 / iterations as f64);
    
    let speedup = uncached_duration.as_millis() as f64 / cached_duration.as_millis() as f64;
    println!("  Speedup: {:.1}x faster with cache", speedup);
}

/// Demonstrate cache statistics and monitoring
async fn monitoring_example() {
    println!("\n=== Cache Monitoring Example ===");
    
    let cache = InMemoryCache::new();
    let llm_string = "gpt-4o-mini:temperature=0.7";
    
    // Simulate a mix of cache hits and misses
    let prompts = ["Hello", "Hello", "What's up?", "Hello", "How are you?", "Hello"];
    
    for prompt in &prompts {
        let result = cache.alookup(prompt, llm_string).await.unwrap();
        
        if result.is_none() {
            println!("Cache miss for '{}' - making LLM call", prompt);
            let generations = expensive_llm_call(prompt, "gpt-4o-mini").await;
            cache.aupdate(prompt, llm_string, generations).await.unwrap();
        } else {
            println!("Cache hit for '{}'", prompt);
        }
    }
    
    let stats = cache.stats().await;
    println!("\nFinal Cache Statistics:");
    println!("  Total Requests: {}", stats.total_requests());
    println!("  Cache Hits: {}", stats.hits);
    println!("  Cache Misses: {}", stats.misses);
    println!("  Hit Rate: {:.1}%", stats.hit_rate());
    println!("  Updates: {}", stats.updates);
    println!("  Current Size: {}", stats.current_size);
    println!("  Max Size Reached: {}", stats.max_size_reached);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 FerricLink Cache Usage Examples\n");
    
    // Run all examples
    basic_cache_example().await;
    size_limited_cache_example().await;
    ttl_cache_example().await;
    performance_example().await;
    monitoring_example().await;
    
    println!("\n✅ All cache examples completed successfully!");
    println!("\n💡 Key Benefits of Caching:");
    println!("  • Reduces API costs by avoiding duplicate requests");
    println!("  • Improves response times for repeated queries");
    println!("  • Provides resilience against API failures");
    println!("  • Enables offline development and testing");
    
    Ok(())
}
