# FerricLink

[![Crates.io](https://img.shields.io/crates/v/ferriclink-core.svg)](https://crates.io/crates/ferriclink-core)
[![Documentation](https://docs.rs/ferriclink-core/badge.svg)](https://docs.rs/ferriclink-core)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

A Rust library for building AI applications, inspired by LangChain. FerricLink provides the fundamental building blocks for creating composable AI workflows with language models, tools, vector stores, and more.

## Crates

- **[ferriclink-core](crates/ferriclink-core/)** - Core abstractions and traits
- More crates coming soon...

## Features

- **Runnable System**: Composable units of work that can be chained together
- **Message Types**: Human, AI, System, and Tool messages for conversation handling
- **Language Models**: Abstractions for LLMs and chat models
- **Vector Stores**: Embedding storage and similarity search
- **Tools**: Function calling and tool integration
- **Callbacks**: Monitoring and tracing system
- **Documents**: Text processing and retrieval
- **Async-First**: Built on Tokio for high-performance async operations

## Quick Start

Add FerricLink to your `Cargo.toml`:

```toml
[dependencies]
ferriclink-core = "0.1.0"
```

## Example

```rust
use ferriclink_core::{
    messages::AnyMessage,
    language_models::{mock_chat_model, GenerationConfig},
    runnables::Runnable,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a mock chat model
    let chat_model = mock_chat_model("gpt-4o-mini");
    
    // Create a conversation
    let messages = vec![
        AnyMessage::human("Hello, how are you?"),
    ];
    
    // Generate a response
    let response = chat_model.generate_chat(
        messages,
        Some(GenerationConfig::new().with_temperature(0.7)),
        None,
    ).await?;
    
    println!("Response: {}", response.text());
    Ok(())
}
```

For more detailed examples and API documentation, see the [ferriclink-core README](crates/ferriclink-core/README.md).

## Development

This project uses [Just](https://github.com/casey/just) for task management and follows semantic versioning.

### Prerequisites

- Rust 1.85+
- Just (optional but recommended)

### Common Commands

```bash
# Show available commands
just

# Build the project
just build

# Run tests
just test

# Run all checks
just ci

# Version management
just version              # Show current version
just patch                # Bump patch version
just minor                # Bump minor version
just major                # Bump major version
just set-version 1.2.3    # Set specific version

# Documentation
just docs                 # Generate and open docs
just docs-all             # Generate docs for all packages

# Development
just watch-test           # Watch and run tests
just watch-check          # Watch and run check
```

### Semantic Versioning

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for backwards-compatible functionality additions
- **PATCH** version for backwards-compatible bug fixes

Version changes are managed through the `justfile`:

```bash
just patch    # 0.1.0 -> 0.1.1
just minor    # 0.1.0 -> 0.2.0
just major    # 0.1.0 -> 1.0.0
```

## Architecture

FerricLink is organized into several core modules:

- **`runnables`**: Core Runnable trait and execution system
- **`messages`**: Message types for conversation handling
- **`language_models`**: LLM and chat model abstractions
- **`vectorstores`**: Vector storage and similarity search
- **`tools`**: Tool system for function calling
- **`callbacks`**: Monitoring and tracing system
- **`documents`**: Text processing and document handling
- **`embeddings`**: Embedding generation abstractions
- **`retrievers`**: Document retrieval system

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`just test`)
5. Run checks (`just ci`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [LangChain](https://github.com/langchain-ai/langchain)
- Built with [Tokio](https://tokio.rs/) for async runtime
- Uses [Serde](https://serde.rs/) for serialization

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes.
