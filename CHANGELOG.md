# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive documentation website with Docusaurus
- GitHub Actions workflows for CI/CD and website deployment
- Release-please configuration for automated versioning
- Rust toolchain configuration for consistent builds
- **Structured Query Language (SQL)**: Internal representation for building composable query expressions
- **Exception System**: Comprehensive error handling with LangChain-compatible error codes and troubleshooting URLs
- **Environment Information**: Runtime environment details including OS, architecture, Rust version, and features
- **Rate Limiting**: Token bucket algorithm for controlling request rates with retry logic and exponential backoff
- **Caching System**: In-memory and TTL caches for LLM responses with LRU eviction and performance monitoring
- **Example Selectors**: Few-shot learning with length-based, semantic similarity, and MMR example selection
- **Global Configuration**: Application-wide settings for verbose, debug, and LLM cache management

### Changed
- Updated project metadata and dependencies
- Improved README documentation
- Enhanced workflow configurations
- Optimized release-please configuration for workspace
- Enhanced error handling with comprehensive error codes and troubleshooting URLs
- Improved documentation with detailed guides for all new features

### Fixed
- Configure docs.rs build compatibility
- Add conditional compilation for tracing in docs.rs environment
- Fix feature check in init() function
- Correct release-please configuration for proper compatibility
- Remove duplicate nested changelog files
- Fix workflow to update correct changelog files

### Deprecated
- N/A

### Removed
- Duplicate nested changelog files
- Unnecessary nested directory structure

### Security
- N/A

## [0.1.3] - 2025-10-03

### Fixed
- Configure docs.rs build compatibility
- Add conditional compilation for tracing in docs.rs environment
- Fix feature check in init() function
- Optimize release-please configuration for workspace
- Correct release-please configuration for proper compatibility

### Changed
- Updated release-please configuration for better workspace management
- Improved documentation build process
- Enhanced CI/CD workflow configuration

## [0.1.2] - 2025-10-02

### Added
- Comprehensive documentation website with Docusaurus
- GitHub Actions workflows for CI/CD and website deployment
- Release-please configuration for automated versioning
- Rust toolchain configuration for consistent builds

### Changed
- Updated project metadata and dependencies
- Improved README documentation
- Enhanced workflow configurations

## [0.1.1] - 2025-10-01

### Added
- Initial project setup and structure
- Basic crate configuration
- Core module implementations

## [0.1.0] - 2025-10-02

### Added
- Initial release of FerricLink Core
- Basic project structure and workspace configuration
- Core trait definitions and implementations
- Mock implementations for testing
- Documentation and examples
