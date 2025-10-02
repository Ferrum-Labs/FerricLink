# Contributing to FerricLink

Thank you for your interest in contributing to FerricLink! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Release Process](#release-process)

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

### Prerequisites

- Rust 1.85.0 or later
- Git
- A GitHub account

### Development Setup

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/ferriclink.git
   cd ferriclink
   ```

3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/ferrumlabs/ferriclink.git
   ```

4. **Install dependencies**:
   ```bash
   cargo build
   ```

5. **Run tests** to ensure everything works:
   ```bash
   cargo test
   ```

## Development Setup

### Using Just (Recommended)

We use [Just](https://github.com/casey/just) for task automation. Install it first:

```bash
# On macOS
brew install just

# On Linux
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash

# On Windows
scoop install just
```

Then you can use the available commands:

```bash
just --list  # List all available commands
just build   # Build the project
just test    # Run tests
just fmt     # Format code
just lint    # Run clippy
just docs    # Generate documentation
```

### Manual Setup

If you prefer not to use Just, you can run the commands manually:

```bash
# Build
cargo build

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy

# Documentation
cargo doc --open
```

## Making Changes

### Branch Strategy

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following our coding standards

3. **Test your changes** thoroughly

4. **Commit your changes** with clear, descriptive messages

### Coding Standards

- **Rust Style**: Follow standard Rust formatting (`cargo fmt`)
- **Documentation**: Add docstrings to all public APIs
- **Tests**: Write tests for new functionality
- **Error Handling**: Use proper error types and handling
- **Performance**: Consider performance implications of changes

### Commit Message Format

We follow conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test changes
- `chore`: Maintenance tasks

Examples:
```
feat(core): add new message type
fix(callbacks): resolve memory leak in handler
docs(readme): update installation instructions
```

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With features
cargo test --features all

# Integration tests
cargo test --test integration_tests
```

### Test Coverage

We aim for high test coverage. Run coverage analysis:

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html
```

### Writing Tests

- **Unit Tests**: Test individual functions and methods
- **Integration Tests**: Test component interactions
- **Property Tests**: Use `proptest` for complex data structures
- **Benchmarks**: Add benchmarks for performance-critical code

## Documentation

### Code Documentation

- Use `///` for public API documentation
- Include examples in docstrings
- Document all public types, functions, and methods
- Use `# Examples` sections for complex APIs

### README Updates

- Update relevant README files
- Include code examples
- Update installation instructions if needed

### Changelog

- Add entries to `CHANGELOG.md` for user-facing changes
- Follow the [Keep a Changelog](https://keepachangelog.com/) format

## Submitting Changes

### Pull Request Process

1. **Push your branch** to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create a Pull Request** on GitHub:
   - Use the provided template
   - Link any related issues
   - Provide a clear description

3. **Wait for review** and address feedback

4. **Squash commits** if requested

### Review Process

- All PRs require review from maintainers
- CI must pass before merging
- Address all review comments
- Keep PRs focused and reasonably sized

## Release Process

### Version Bumping

We use semantic versioning (semver):

- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features, backward compatible
- **Patch** (0.0.1): Bug fixes, backward compatible

### Release Workflow

1. **Update version** in `Cargo.toml`
2. **Update changelog** with new version
3. **Create release PR** for review
4. **Merge and tag** the release
5. **Publish to crates.io** (automated)

## Getting Help

- **GitHub Issues**: For bug reports and feature requests
- **Discussions**: For questions and general discussion
- **Email**: [maintainers@ferrumlabs.com](mailto:maintainers@ferrumlabs.com)

## Recognition

Contributors will be recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- Project documentation

Thank you for contributing to FerricLink! 🦀
