//! Example demonstrating FerricLink's global configuration functionality.
//!
//! This example shows how to use global settings similar to LangChain's globals.py,
//! but with Rust's type safety and thread safety guarantees.

use ferriclink_core::{
    InMemoryCache, TtlCache, clear_llm_cache, disable_debug, disable_verbose, enable_debug,
    enable_verbose, get_debug, get_verbose, globals_summary, has_llm_cache, init_globals, is_debug,
    is_verbose, reset_globals, set_debug, set_llm_cache, set_verbose, toggle_debug, toggle_verbose,
};

/// Demonstrate basic global configuration usage
fn basic_globals_example() {
    println!("=== Basic Global Configuration ===\n");

    // Initialize globals (usually done in main())
    init_globals().unwrap();

    // Check initial state
    println!("Initial state:");
    println!("  Verbose: {}", get_verbose());
    println!("  Debug: {}", get_debug());
    println!("  Has LLM cache: {}", has_llm_cache());
    println!();

    // Set verbose mode
    set_verbose(true);
    println!("After setting verbose to true:");
    println!("  Verbose: {}", get_verbose());
    println!("  is_verbose(): {}", is_verbose());
    println!();

    // Set debug mode
    set_debug(true);
    println!("After setting debug to true:");
    println!("  Debug: {}", get_debug());
    println!("  is_debug(): {}", is_debug());
    println!();

    // Toggle settings
    toggle_verbose();
    toggle_debug();
    println!("After toggling both:");
    println!("  Verbose: {}", get_verbose());
    println!("  Debug: {}", get_debug());
    println!();
}

/// Demonstrate LLM cache configuration
fn llm_cache_example() {
    println!("=== LLM Cache Configuration ===\n");

    // Check initial cache state
    println!("Initial cache state:");
    println!("  Has LLM cache: {}", has_llm_cache());
    println!();

    // Set an in-memory cache
    let cache = Box::new(InMemoryCache::new());
    set_llm_cache(Some(cache)).unwrap();

    println!("After setting in-memory cache:");
    println!("  Has LLM cache: {}", has_llm_cache());
    println!("  Cache type: InMemoryCache");
    println!();

    // Set a TTL cache
    let ttl_cache = Box::new(TtlCache::new(
        std::time::Duration::from_secs(3600), // 1 hour TTL
        Some(100),                            // Max size
    ));
    set_llm_cache(Some(ttl_cache)).unwrap();

    println!("After setting TTL cache:");
    println!("  Has LLM cache: {}", has_llm_cache());
    println!("  Cache type: TtlCache");
    println!();

    // Clear the cache
    clear_llm_cache().unwrap();

    println!("After clearing cache:");
    println!("  Has LLM cache: {}", has_llm_cache());
    println!();
}

/// Demonstrate convenience functions
fn convenience_functions_example() {
    println!("=== Convenience Functions ===\n");

    // Enable/disable functions
    enable_verbose();
    enable_debug();
    println!("After enable_verbose() and enable_debug():");
    println!("  Verbose: {}", get_verbose());
    println!("  Debug: {}", get_debug());
    println!();

    disable_verbose();
    disable_debug();
    println!("After disable_verbose() and disable_debug():");
    println!("  Verbose: {}", get_verbose());
    println!("  Debug: {}", get_debug());
    println!();

    // Toggle functions
    toggle_verbose();
    toggle_debug();
    println!("After toggle_verbose() and toggle_debug():");
    println!("  Verbose: {}", get_verbose());
    println!("  Debug: {}", get_debug());
    println!();
}

/// Demonstrate global configuration summary
fn configuration_summary_example() {
    println!("=== Configuration Summary ===\n");

    // Set some configuration
    set_verbose(true);
    set_debug(false);
    let cache = Box::new(InMemoryCache::new());
    set_llm_cache(Some(cache)).unwrap();

    // Get summary
    let summary = globals_summary();
    println!("Current global configuration:");
    println!("  {summary}");
    println!();
}

/// Demonstrate reset functionality
fn reset_example() {
    println!("=== Reset Functionality ===\n");

    // Set some configuration
    set_verbose(true);
    set_debug(true);
    let cache = Box::new(InMemoryCache::new());
    set_llm_cache(Some(cache)).unwrap();

    println!("Before reset:");
    println!("  Verbose: {}", get_verbose());
    println!("  Debug: {}", get_debug());
    println!("  Has LLM cache: {}", has_llm_cache());
    println!();

    // Reset to defaults
    reset_globals().unwrap();

    println!("After reset_globals():");
    println!("  Verbose: {}", get_verbose());
    println!("  Debug: {}", get_debug());
    println!("  Has LLM cache: {}", has_llm_cache());
    println!();
}

/// Demonstrate thread safety
fn thread_safety_example() {
    println!("=== Thread Safety Demo ===\n");

    use std::thread;

    // Set initial state
    set_verbose(false);
    set_debug(false);

    println!("Initial state:");
    println!("  Verbose: {}", get_verbose());
    println!("  Debug: {}", get_debug());
    println!();

    // Spawn multiple threads that modify global state
    let handles: Vec<_> = (0..5)
        .map(|i| {
            thread::spawn(move || {
                if i % 2 == 0 {
                    set_verbose(true);
                    set_debug(true);
                } else {
                    set_verbose(false);
                    set_debug(false);
                }

                // Small delay to simulate work
                std::thread::sleep(std::time::Duration::from_millis(10));

                (get_verbose(), get_debug())
            })
        })
        .collect();

    // Wait for all threads to complete
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    println!("Thread results:");
    for (i, (verbose, debug)) in results.iter().enumerate() {
        println!("  Thread {i}: verbose={verbose}, debug={debug}");
    }

    println!("\nFinal state:");
    println!("  Verbose: {}", get_verbose());
    println!("  Debug: {}", get_debug());
    println!();
}

/// Demonstrate error handling
fn error_handling_example() {
    println!("=== Error Handling ===\n");

    // Test cache operations
    let cache = Box::new(InMemoryCache::new());

    match set_llm_cache(Some(cache)) {
        Ok(_) => println!("✅ Successfully set LLM cache"),
        Err(e) => println!("❌ Error setting LLM cache: {e}"),
    }

    if has_llm_cache() {
        println!("✅ LLM cache is set");
    } else {
        println!("ℹ️  No LLM cache set");
    }

    match clear_llm_cache() {
        Ok(_) => println!("✅ Successfully cleared LLM cache"),
        Err(e) => println!("❌ Error clearing LLM cache: {e}"),
    }

    println!();
}

/// Demonstrate integration with other FerricLink components
fn integration_example() {
    println!("=== Integration Example ===\n");

    // Set up global configuration
    enable_verbose();
    enable_debug();

    let cache = Box::new(TtlCache::new(
        std::time::Duration::from_secs(1800), // 30 minutes
        Some(200),                            // Max size
    ));
    set_llm_cache(Some(cache)).unwrap();

    println!("Global configuration set up:");
    println!("  {}", globals_summary());
    println!();

    // Simulate using the global settings in application logic
    if is_verbose() {
        println!("🔍 Verbose mode enabled - showing detailed logs");
    }

    if is_debug() {
        println!("🐛 Debug mode enabled - showing debug information");
    }

    if has_llm_cache() {
        println!("💾 LLM cache enabled - responses will be cached");
    }

    println!();
}

/// Demonstrate configuration persistence
fn configuration_persistence_example() {
    println!("=== Configuration Persistence ===\n");

    // Set up configuration
    set_verbose(true);
    set_debug(false);
    let cache = Box::new(InMemoryCache::new());
    set_llm_cache(Some(cache)).unwrap();

    println!("Configuration set:");
    println!("  {}", globals_summary());
    println!();

    // Simulate application restart (globals persist within the same process)
    println!("After 'application restart' (same process):");
    println!("  {}", globals_summary());
    println!();

    // Reset and show difference
    reset_globals().unwrap();
    println!("After reset_globals():");
    println!("  {}", globals_summary());
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 FerricLink Global Configuration Examples\n");

    // Run all examples
    basic_globals_example();
    llm_cache_example();
    convenience_functions_example();
    configuration_summary_example();
    reset_example();
    thread_safety_example();
    error_handling_example();
    integration_example();
    configuration_persistence_example();

    println!("✅ All global configuration examples completed successfully!");
    println!("\n💡 Key Benefits of FerricLink Globals:");
    println!("  • Thread-safe global state management");
    println!("  • Type-safe configuration with compile-time guarantees");
    println!("  • LangChain-compatible API");
    println!("  • Easy integration with other FerricLink components");
    println!("  • Convenient helper functions for common operations");
    println!("  • Comprehensive error handling");

    Ok(())
}
