---
sidebar_position: 6
---

# Global Configuration Guide

FerricLink provides comprehensive global configuration functionality similar to LangChain's `globals.py`, but with Rust's type safety and thread safety guarantees.

## Overview

Global configuration allows you to manage application-wide settings that affect all FerricLink components:

- **Verbose Mode**: Control detailed logging and output
- **Debug Mode**: Enable debug information and diagnostics
- **LLM Cache**: Set a global cache for language model responses
- **Thread Safety**: All operations are thread-safe and concurrent

## Basic Usage

### Initialization

```rust
use ferriclink_core::{init_globals, set_verbose, set_debug, set_llm_cache};

// Initialize global configuration (usually done in main())
init_globals().unwrap();

// Set global settings
set_verbose(true);
set_debug(false);
```

### Verbose Mode

```rust
use ferriclink_core::{set_verbose, get_verbose, is_verbose, enable_verbose, disable_verbose, toggle_verbose};

// Set verbose mode
set_verbose(true);
println!("Verbose mode: {}", get_verbose());

// Check if verbose mode is enabled
if is_verbose() {
    println!("Detailed logging enabled");
}

// Convenience functions
enable_verbose();
disable_verbose();
toggle_verbose();
```

### Debug Mode

```rust
use ferriclink_core::{set_debug, get_debug, is_debug, enable_debug, disable_debug, toggle_debug};

// Set debug mode
set_debug(true);
println!("Debug mode: {}", get_debug());

// Check if debug mode is enabled
if is_debug() {
    println!("Debug information enabled");
}

// Convenience functions
enable_debug();
disable_debug();
toggle_debug();
```

### LLM Cache

```rust
use ferriclink_core::{set_llm_cache, clear_llm_cache, has_llm_cache, InMemoryCache, TtlCache};

// Set an in-memory cache
let cache = Box::new(InMemoryCache::new());
set_llm_cache(Some(cache)).unwrap();

// Set a TTL cache
let ttl_cache = Box::new(TtlCache::new(
    std::time::Duration::from_secs(3600), // 1 hour TTL
    Some(100), // Max size
));
set_llm_cache(Some(ttl_cache)).unwrap();

// Check if cache is enabled
if has_llm_cache() {
    println!("LLM responses will be cached");
}

// Clear the cache
clear_llm_cache().unwrap();
```

## Advanced Usage

### Configuration Summary

```rust
use ferriclink_core::globals_summary;

// Get a summary of current configuration
let summary = globals_summary();
println!("Current configuration: {}", summary);
// Output: GlobalConfig { verbose: true, debug: false, has_llm_cache: true }
```

### Reset Configuration

```rust
use ferriclink_core::reset_globals;

// Reset all settings to defaults
reset_globals().unwrap();
// verbose: false, debug: false, llm_cache: None
```

### Thread Safety

```rust
use std::thread;
use ferriclink_core::{set_verbose, get_verbose};

// Global settings are thread-safe
let handles: Vec<_> = (0..5).map(|i| {
    thread::spawn(move || {
        set_verbose(i % 2 == 0);
        get_verbose()
    })
}).collect();

// Wait for all threads
for handle in handles {
    let result = handle.join().unwrap();
    println!("Thread result: {}", result);
}
```

## Integration Patterns

### Application Initialization

```rust
use ferriclink_core::{init, init_globals, set_verbose, set_debug, set_llm_cache, InMemoryCache};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize FerricLink (includes globals)
    init()?;
    
    // Configure global settings
    set_verbose(true);
    set_debug(cfg!(debug_assertions));
    
    // Set up caching
    let cache = Box::new(InMemoryCache::new());
    set_llm_cache(Some(cache))?;
    
    // Your application logic here
    run_application()?;
    
    Ok(())
}
```

### Conditional Behavior

```rust
use ferriclink_core::{is_verbose, is_debug, has_llm_cache};

fn process_request(input: &str) -> Result&lt;String&gt; {
    if is_verbose() {
        println!("Processing request: {}", input);
    }
    
    if is_debug() {
        println!("Debug: Input length: {}", input.len());
    }
    
    // Process with or without caching
    if has_llm_cache() {
        // Use cached response if available
        process_with_cache(input)
    } else {
        // Process directly
        process_directly(input)
    }
}
```

### Environment-Based Configuration

```rust
use ferriclink_core::{set_verbose, set_debug, set_llm_cache, InMemoryCache, TtlCache};

fn configure_from_environment() -> Result<()> {
    // Set verbose mode from environment
    if std::env::var("FERRICLINK_VERBOSE").is_ok() {
        set_verbose(true);
    }
    
    // Set debug mode from environment
    if std::env::var("FERRICLINK_DEBUG").is_ok() {
        set_debug(true);
    }
    
    // Configure cache based on environment
    if let Ok(cache_size) = std::env::var("FERRICLINK_CACHE_SIZE") {
        let size: usize = cache_size.parse().unwrap_or(100);
        let cache = Box::new(InMemoryCache::new());
        set_llm_cache(Some(cache))?;
    }
    
    Ok(())
}
```

## Error Handling

### Safe Initialization

```rust
use ferriclink_core::{init_globals, get_verbose};

fn safe_initialize() -> Result<()> {
    // Initialize globals safely
    match init_globals() {
        Ok(_) => {
            println!("Global configuration initialized");
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to initialize globals: {}", e);
            Err(e)
        }
    }
}

fn safe_usage() -> Result<()> {
    // Check if globals are initialized before use
    if std::panic::catch_unwind(|| get_verbose()).is_err() {
        return Err("Global configuration not initialized".into());
    }
    
    // Safe to use globals
    println!("Verbose mode: {}", get_verbose());
    Ok(())
}
```

### Graceful Degradation

```rust
use ferriclink_core::{is_verbose, is_debug, has_llm_cache};

fn process_with_fallback(input: &str) -> String {
    // Use globals if available, otherwise use defaults
    let verbose = std::panic::catch_unwind(|| is_verbose()).unwrap_or(false);
    let debug = std::panic::catch_unwind(|| is_debug()).unwrap_or(false);
    let cached = std::panic::catch_unwind(|| has_llm_cache()).unwrap_or(false);
    
    if verbose {
        println!("Processing with verbose mode");
    }
    
    if debug {
        println!("Debug information available");
    }
    
    if cached {
        println!("Using cached responses");
    }
    
    // Process input
    format!("Processed: {}", input)
}
```

## Best Practices

### 1. Early Initialization

```rust
// Initialize globals as early as possible in your application
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize before any other FerricLink operations
    init_globals()?;
    
    // Configure based on environment or config
    configure_globals()?;
    
    // Run your application
    run_app()?;
    
    Ok(())
}
```

### 2. Environment-Based Configuration

```rust
fn configure_globals() -> Result<()> {
    // Use environment variables for configuration
    if std::env::var("RUST_LOG").unwrap_or_default().contains("debug") {
        set_debug(true);
    }
    
    if std::env::var("FERRICLINK_VERBOSE").is_ok() {
        set_verbose(true);
    }
    
    // Configure cache based on environment
    if let Ok(ttl_seconds) = std::env::var("FERRICLINK_CACHE_TTL") {
        let ttl = std::time::Duration::from_secs(ttl_seconds.parse()?);
        let cache = Box::new(TtlCache::new(ttl, None));
        set_llm_cache(Some(cache))?;
    }
    
    Ok(())
}
```

### 3. Thread-Safe Operations

```rust
use std::sync::Arc;
use std::thread;

fn parallel_processing() -> Result<()> {
    let handles: Vec<_> = (0..4).map(|i| {
        thread::spawn(move || {
            // Each thread can safely access globals
            if is_verbose() {
                println!("Thread {} processing", i);
            }
            
            if is_debug() {
                println!("Thread {} debug info", i);
            }
            
            // Process data
            process_data(i)
        })
    }).collect();
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap()?;
    }
    
    Ok(())
}
```

### 4. Configuration Validation

```rust
fn validate_configuration() -> Result<()> {
    let summary = globals_summary();
    println!("Current configuration: {}", summary);
    
    // Validate settings
    if is_verbose() && !is_debug() {
        println!("Warning: Verbose mode enabled but debug mode disabled");
    }
    
    if has_llm_cache() {
        println!("LLM caching enabled - responses will be cached");
    } else {
        println!("LLM caching disabled - all requests will be processed");
    }
    
    Ok(())
}
```

## Comparison with LangChain

| Feature | LangChain Python | FerricLink Rust |
|---------|------------------|-----------------|
| **Global Verbose** | `set_verbose()` / `get_verbose()` | `set_verbose()` / `get_verbose()` |
| **Global Debug** | `set_debug()` / `get_debug()` | `set_debug()` / `get_debug()` |
| **Global Cache** | `set_llm_cache()` / `get_llm_cache()` | `set_llm_cache()` / `has_llm_cache()` |
| **Thread Safety** | GIL limitations | **True thread safety** |
| **Type Safety** | Runtime checks | **Compile-time guarantees** |
| **Performance** | Medium | **High** |
| **Memory Safety** | Runtime checks | **Compile-time guarantees** |
| **Convenience Functions** | Basic | **Rich set of helpers** |

## Troubleshooting

### Common Issues

1. **Not Initialized**: Call `init_globals()` before using global functions
2. **Thread Safety**: All operations are thread-safe by design
3. **Cache Management**: Use `has_llm_cache()` to check cache status
4. **Configuration Reset**: Use `reset_globals()` to restore defaults

### Debug Configuration

```rust
use ferriclink_core::{globals_summary, is_verbose, is_debug, has_llm_cache};

fn debug_globals() {
    println!("=== Global Configuration Debug ===");
    println!("Summary: {}", globals_summary());
    println!("Verbose: {}", is_verbose());
    println!("Debug: {}", is_debug());
    println!("Has LLM Cache: {}", has_llm_cache());
    println!("================================");
}
```

## Examples

See the [globals usage example](../../examples/globals_usage) for a complete working demonstration of all global configuration features.
