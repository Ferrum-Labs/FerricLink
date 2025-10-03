//! # Environment Information Example
//!
//! This example demonstrates how to use the environment information system
//! in FerricLink, similar to LangChain's env.py functionality.

use ferriclink_core::{RuntimeEnvironment, get_runtime_environment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FerricLink Environment Information Example ===\n");

    // Example 1: Get cached runtime environment
    println!("1. Cached Runtime Environment:");
    let env = get_runtime_environment();
    println!("Summary: {}", env.summary());
    println!("Library: {} {}", env.library, env.library_version);
    println!("Runtime: {} {}", env.runtime, env.runtime_version);
    println!("Platform: {}", env.platform);
    println!("Architecture: {}", env.architecture);
    println!("OS: {}", env.os);
    println!("Compiler: {}", env.compiler);
    println!("Target: {}", env.target);
    println!();

    // Example 2: Get fresh runtime environment
    println!("2. Fresh Runtime Environment:");
    let fresh_env = get_runtime_environment();
    println!("Summary: {}", fresh_env.summary());
    println!();

    // Example 3: Check features
    println!("3. Enabled Features:");
    for feature in &env.features {
        println!("  - {feature}");
    }
    println!();

    // Example 4: Check specific features
    println!("4. Feature Detection:");
    println!("  - Debug mode: {}", env.is_debug());
    println!("  - Release mode: {}", env.is_release());
    println!("  - Has 'http' feature: {}", env.has_feature("http"));
    println!(
        "  - Has 'validation' feature: {}",
        env.has_feature("validation")
    );
    println!();

    // Example 5: Environment variables
    println!("5. Relevant Environment Variables:");
    for (key, value) in &env.env_vars {
        // Truncate long values for display
        let display_value = if value.len() > 50 {
            format!("{}...", &value[..47])
        } else {
            value.clone()
        };
        println!("  - {key}: {display_value}");
    }
    println!();

    // Example 6: Memory information
    println!("6. Memory Information:");
    if let Some(memory) = env.memory_info() {
        if let Some(total) = memory.total_memory {
            println!("  - Total Memory: {} MB", total / 1024 / 1024);
        }
        if let Some(available) = memory.available_memory {
            println!("  - Available Memory: {} MB", available / 1024 / 1024);
        }
        if let Some(process) = memory.process_memory {
            println!("  - Process Memory: {} MB", process / 1024 / 1024);
        }
    } else {
        println!("  - Memory information not available on this platform");
    }
    println!();

    // Example 7: Serialization
    println!("7. Serialization:");
    let serialized = serde_json::to_string_pretty(env)?;
    println!("Serialized environment (first 500 chars):");
    println!("{}", &serialized[..500.min(serialized.len())]);
    if serialized.len() > 500 {
        println!("... (truncated)");
    }
    println!();

    // Example 8: Comparison with LangChain format
    println!("8. LangChain-Compatible Format:");
    let langchain_format = serde_json::json!({
        "library_version": env.library_version,
        "library": env.library,
        "platform": env.platform,
        "runtime": env.runtime,
        "runtime_version": env.runtime_version,
    });
    println!("{}", serde_json::to_string_pretty(&langchain_format)?);
    println!();

    // Example 9: Environment-specific behavior
    println!("9. Environment-Specific Behavior:");
    demonstrate_environment_specific_behavior(env);
    println!();

    println!("=== Example Complete ===");
    Ok(())
}

/// Demonstrate environment-specific behavior
fn demonstrate_environment_specific_behavior(env: &RuntimeEnvironment) {
    println!("  - Running on {} architecture", env.architecture);

    match env.os.as_str() {
        "Linux" => {
            println!("  - Linux detected: Using system-specific optimizations");
        }
        "macOS" => {
            println!("  - macOS detected: Using Apple-specific features");
        }
        "Windows" => {
            println!("  - Windows detected: Using Windows-specific APIs");
        }
        _ => {
            println!("  - Unknown OS: Using generic implementations");
        }
    }

    if env.is_debug() {
        println!("  - Debug build: Enhanced logging and error reporting enabled");
    } else {
        println!("  - Release build: Optimized for performance");
    }

    if env.has_feature("http") {
        println!("  - HTTP feature enabled: Network operations available");
    }

    if env.has_feature("validation") {
        println!("  - Validation feature enabled: Input validation available");
    }
}

/// Example of using environment information for configuration
#[allow(dead_code)]
fn configure_based_on_environment(
    env: &RuntimeEnvironment,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Configuring based on environment...");

    // Set log level based on build type
    if env.is_debug() {
        unsafe {
            std::env::set_var("RUST_LOG", "debug");
        }
        println!("  - Debug logging enabled");
    } else {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
        println!("  - Info logging enabled");
    }

    // Configure based on platform
    match env.os.as_str() {
        "Linux" => {
            println!("  - Linux-specific configuration applied");
        }
        "macOS" => {
            println!("  - macOS-specific configuration applied");
        }
        "Windows" => {
            println!("  - Windows-specific configuration applied");
        }
        _ => {
            println!("  - Generic configuration applied");
        }
    }

    // Set memory limits based on available memory
    if let Some(memory) = env.memory_info() {
        if let Some(total) = memory.total_memory {
            if total < 2 * 1024 * 1024 * 1024 {
                // Less than 2GB
                println!("  - Low memory system detected, using conservative settings");
            } else {
                println!("  - Sufficient memory available, using standard settings");
            }
        }
    }

    Ok(())
}
