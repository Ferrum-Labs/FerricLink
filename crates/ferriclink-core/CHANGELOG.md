# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Structured Query Language (SQL)**: Internal representation for building composable query expressions with visitor pattern support
- **Exception System**: Comprehensive error handling with LangChain-compatible error codes, troubleshooting URLs, and specialized exception types
- **Environment Information**: Runtime environment details including OS, architecture, Rust version, compiler info, and enabled features
- **Rate Limiting**: Token bucket algorithm for controlling request rates with retry logic, exponential backoff, and serializable configuration
- **Caching System**: In-memory and TTL caches for LLM responses with LRU eviction, performance monitoring, and thread-safe operations
- **Example Selectors**: Few-shot learning with length-based, semantic similarity, and MMR example selection for dynamic prompt construction
- **Global Configuration**: Application-wide settings for verbose, debug, and LLM cache management with thread safety and convenience functions
- Comprehensive documentation and usage examples for all new features
- Integration with existing FerricLink Core ecosystem

### Features
- **Structured Query**: Visitor pattern, SQL/MongoDB translation, builders for common operations
- **Exceptions**: Error codes, troubleshooting URLs, TracerException, OutputParserException with LLM feedback
- **Environment**: Runtime info, feature detection, memory statistics, platform-specific details
- **Rate Limiting**: Token bucket, burst capacity, retry logic, exponential backoff, serialization
- **Caching**: In-memory cache, TTL cache, LRU eviction, cache statistics, async/sync support
- **Example Selectors**: Length-based selection, semantic similarity, MMR, utility functions
- **Global Config**: Thread-safe globals, verbose/debug modes, LLM cache management, convenience functions

### Changed
- Enhanced error handling with comprehensive error codes and troubleshooting URLs
- Improved documentation with detailed guides for all new features
- Updated lib.rs exports to include all new functionality
- Enhanced init() function to initialize global configuration

### Fixed
- Remove duplicate nested changelog files
- Fix workflow to update correct changelog files
- Resolve all clippy warnings and test failures

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
