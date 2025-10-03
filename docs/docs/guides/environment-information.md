---
sidebar_position: 2
---

# Environment Information

FerricLink provides comprehensive runtime environment information similar to LangChain's `env.py` functionality. This is useful for debugging, monitoring, and adapting behavior based on the runtime environment.

## Basic Usage

### Get Runtime Environment

```rust
use ferriclink_core::{get_runtime_environment, RuntimeEnvironment};

// Get cached runtime environment (recommended for most use cases)
let env = get_runtime_environment();

// Get fresh runtime environment (useful when environment might have changed)
let fresh_env = get_fresh_runtime_environment();
```

### Environment Information

The `RuntimeEnvironment` struct provides detailed information about:

- **Library Information**: Version, name, and build details
- **Platform Information**: Operating system, architecture, and target
- **Runtime Information**: Rust version and compiler details
- **Environment Variables**: Relevant system and FerricLink-specific variables
- **Features**: Enabled compile-time features
- **Memory Information**: System and process memory usage (when available)

## Key Features

### 1. Platform Detection

```rust
let env = get_runtime_environment();

match env.os.as_str() {
    "Linux" => {
        println!("Running on Linux with system-specific optimizations");
    }
    "macOS" => {
        println!("Running on macOS with Apple-specific features");
    }
    "Windows" => {
        println!("Running on Windows with Windows-specific APIs");
    }
    _ => {
        println!("Unknown platform, using generic implementations");
    }
}
```

### 2. Feature Detection

```rust
let env = get_runtime_environment();

// Check if specific features are enabled
if env.has_feature("http") {
    println!("HTTP functionality is available");
}

if env.has_feature("validation") {
    println!("Input validation is enabled");
}

// Check build type
if env.is_debug() {
    println!("Running in debug mode with enhanced logging");
} else {
    println!("Running in release mode with optimizations");
}
```

### 3. Environment Variables

```rust
let env = get_runtime_environment();

// Access environment variables
if let Some(rust_log) = env.get_env_var("RUST_LOG") {
    println!("Log level set to: {}", rust_log);
}

// Check for FerricLink-specific variables
if let Some(config_path) = env.get_env_var("FERRICLINK_CONFIG") {
    println!("Using config from: {}", config_path);
}
```

### 4. Memory Information

```rust
let env = get_runtime_environment();

if let Some(memory) = env.memory_info() {
    if let Some(total) = memory.total_memory {
        println!("Total system memory: {} MB", total / 1024 / 1024);
    }
    
    if let Some(available) = memory.available_memory {
        println!("Available memory: {} MB", available / 1024 / 1024);
    }
    
    if let Some(process) = memory.process_memory {
        println!("Process memory usage: {} MB", process / 1024 / 1024);
    }
}
```

## LangChain Compatibility

FerricLink's environment information is designed to be compatible with LangChain's format:

```rust
use ferriclink_core::get_runtime_environment;
use serde_json;

let env = get_runtime_environment();

// Create LangChain-compatible format
let langchain_format = serde_json::json!({
    "library_version": env.library_version,
    "library": env.library,
    "platform": env.platform,
    "runtime": env.runtime,
    "runtime_version": env.runtime_version,
});

println!("{}", serde_json::to_string_pretty(&langchain_format)?);
```

## Advanced Usage

### Environment-Specific Configuration

```rust
use ferriclink_core::get_runtime_environment;

fn configure_for_environment() -> Result<(), Box<dyn std::error::Error>> {
    let env = get_runtime_environment();
    
    // Configure based on platform
    match env.os.as_str() {
        "Linux" => {
            // Linux-specific configuration
            configure_linux_optimizations();
        }
        "macOS" => {
            // macOS-specific configuration
            configure_macos_features();
        }
        "Windows" => {
            // Windows-specific configuration
            configure_windows_apis();
        }
        _ => {
            // Generic configuration
            configure_generic();
        }
    }
    
    // Configure based on memory availability
    if let Some(memory) = env.memory_info() {
        if let Some(total) = memory.total_memory {
            if total < 2 * 1024 * 1024 * 1024 { // Less than 2GB
                configure_low_memory_mode();
            } else {
                configure_standard_mode();
            }
        }
    }
    
    Ok(())
}
```

### Serialization and Logging

```rust
use ferriclink_core::get_runtime_environment;
use serde_json;

let env = get_runtime_environment();

// Serialize for logging or transmission
let serialized = serde_json::to_string_pretty(env)?;
println!("Environment info: {}", serialized);

// Get a summary for quick reference
println!("{}", env.summary());
```

### Caching and Performance

The `get_runtime_environment()` function returns a cached instance, making it efficient for repeated use:

```rust
// First call - creates and caches the environment
let env1 = get_runtime_environment();

// Subsequent calls - returns the cached instance (very fast)
let env2 = get_runtime_environment();

// Both references point to the same data
assert!(std::ptr::eq(env1, env2));
```

## Error Handling

The environment functions are designed to be robust and handle missing information gracefully:

```rust
let env = get_runtime_environment();

// These will never panic, even if information is unavailable
println!("Rust version: {}", env.runtime_version); // "unknown" if not available
println!("Target: {}", env.target); // "unknown" if not available

// Memory information may not be available on all platforms
if let Some(memory) = env.memory_info() {
    // Use memory information
} else {
    // Handle case where memory info is not available
}
```

## Best Practices

1. **Use cached environment**: Prefer `get_runtime_environment()` over `get_fresh_runtime_environment()` unless you know the environment has changed.

2. **Handle missing information**: Always check for `None` values when accessing optional information like memory details.

3. **Platform-specific code**: Use environment information to enable platform-specific optimizations and features.

4. **Logging and debugging**: Include environment information in logs to help with debugging and support.

5. **Feature detection**: Use feature detection to conditionally enable functionality based on compile-time features.

## Related Documentation

- [Error Handling](/docs/guides/error-handling)
- [Configuration](/docs/guides/configuration)
- [API Reference](/api/latest/ferriclink_core/env)
