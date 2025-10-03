# FerricLink Core

[![Crates.io](https://img.shields.io/crates/v/ferriclink-core.svg)](https://crates.io/crates/ferriclink-core)
[![Documentation](https://docs.rs/ferriclink-core/badge.svg)](https://docs.rs/ferriclink-core)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](../../LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85.0%2B-orange.svg)](https://www.rust-lang.org)

Core abstractions for the FerricLink ecosystem, inspired by LangChain Core. This crate provides the fundamental building blocks for building AI applications with language models, tools, vector stores, and more.

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

Add FerricLink Core to your `Cargo.toml`:

```toml
[dependencies]
ferriclink-core = { version = "0.1", features = ["all"] }
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

## Core Modules

### Messages (`messages`)
Conversation handling with different message types:
- `HumanMessage` - User input
- `AIMessage` - AI responses  
- `SystemMessage` - System instructions
- `ToolMessage` - Tool outputs
- `AnyMessage` - Union type for all messages

### Language Models (`language_models`)
Abstractions for language models:
- `BaseLanguageModel` - Core language model trait
- `BaseLLM` - Text generation models
- `BaseChatModel` - Chat/conversation models
- `GenerationConfig` - Configuration for text generation

### Runnables (`runnables`)
Composable execution system:
- `Runnable<Input, Output>` - Core runnable trait
- `RunnableConfig` - Configuration for runs
- `RunnableSequence` - Chain multiple runnables
- `RunnableParallel` - Run multiple runnables in parallel

### Vector Stores (`vectorstores`)
Embedding storage and search:
- `VectorStore` - Core vector store trait
- `InMemoryVectorStore` - In-memory implementation
- `VectorSearchResult` - Search results with similarity scores

### Tools (`tools`)
Function calling system:
- `BaseTool` - Core tool trait
- `Tool` - Executable tools
- `ToolCall` - Tool invocation
- `ToolResult` - Tool execution results
- `ToolCollection` - Manage multiple tools

### Callbacks (`callbacks`)
Monitoring and tracing:
- `CallbackHandler` - Event handling trait
- `ConsoleCallbackHandler` - Console output
- `MemoryCallbackHandler` - In-memory storage
- `CallbackManager` - Manage multiple handlers

### Documents (`documents`)
Text processing:
- `Document` - Text with metadata
- `DocumentCollection` - Multiple documents
- `ToDocument` - Convert to documents
- `FromDocument` - Convert from documents

### Embeddings (`embeddings`)
Text embedding abstractions:
- `Embeddings` - Core embedding trait
- `Embedding` - Vector representation
- `MockEmbeddings` - Testing implementation

### Retrievers (`retrievers`)
Document retrieval:
- `BaseRetriever` - Core retriever trait
- `VectorStoreRetriever` - Vector-based retrieval
- `MultiRetriever` - Combine multiple retrievers

## Development

This crate is part of the FerricLink workspace. See the [main README](../../README.md) for development instructions.

### Prerequisites

- Rust 1.85.0+
- Tokio async runtime

### Building

```bash
# Build the crate
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Check code
cargo check

# Run clippy
cargo clippy
```

## API Stability

This crate is currently in version 0.1.0 and is under active development. The API may change between minor versions until we reach 1.0.0.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](../../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [main README](../../README.md) for contribution guidelines.

## Acknowledgments

- Inspired by [LangChain](https://github.com/langchain-ai/langchain)
- Built with [Tokio](https://tokio.rs/) for async runtime
- Uses [Serde](https://serde.rs/) for serialization
