# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of FerricLink Core
- Core abstractions for AI applications inspired by LangChain
- Comprehensive callback system for monitoring and tracing
- Document processing and management capabilities
- Language model abstractions (LLM and Chat models)
- Message system with support for various message types
- Runnable trait system for composable units of work
- Vector store abstractions for embeddings and similarity search
- Tool system for function calling and execution
- Retriever system for document retrieval
- Serialization framework for data persistence
- Utility functions for color output and text formatting

### Features
- **Callbacks**: Run tracking, console and memory handlers, callback management
- **Documents**: Text processing, metadata management, chunking and joining
- **Embeddings**: Vector operations, similarity calculations, mock implementations
- **Language Models**: Base traits for LLMs and chat models with mock implementations
- **Messages**: Human, AI, System, Tool, and AnyMessage types with content blocks
- **Runnables**: Lambda, async, sequence, and parallel runnable implementations
- **Vector Stores**: In-memory vector store with similarity search
- **Tools**: Function tools, tool collections, and runnable tools
- **Retrievers**: Vector store retrievers, multi-retrievers with combination methods
- **Serialization**: JSON serialization/deserialization with namespace support

## [0.1.3] - 2025-10-03

### Fixed
- Add conditional compilation for tracing in docs.rs environment
- Fix feature check in init() function for proper feature detection

### Changed
- Updated crate metadata and dependencies
- Improved crate README documentation

## [0.1.2] - 2025-10-02

### Added
- Enhanced crate documentation and examples
- Improved error handling and validation

### Changed
- Updated crate dependencies and metadata
- Enhanced crate README with better examples

## [0.1.1] - 2025-10-01

### Added
- Initial crate implementation
- Core module structure and basic functionality

## [0.1.0] - 2025-10-01

### Added
- Initial release of ferriclink-core crate
- Core abstractions and traits for AI applications
- Basic module implementations
