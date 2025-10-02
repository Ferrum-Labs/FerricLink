# FerricLink Development Commands

# Default recipe
default:
    @just --list

# Get current version
version:
    @grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'

# Show current version with formatting
current-version:
    @echo "Current version: $(just version)"

# Bump patch version (0.1.0 -> 0.1.1)
patch:
    @current=$$(just version) && \
    major=$$(echo $$current | cut -d. -f1) && \
    minor=$$(echo $$current | cut -d. -f2) && \
    patch=$$(echo $$current | cut -d. -f3) && \
    new_version="$$major.$$minor.$$((patch + 1))" && \
    sed -i "s/^version = \".*\"/version = \"$$new_version\"/" Cargo.toml && \
    echo "Bumped to version: $$new_version"

# Bump minor version (0.1.0 -> 0.2.0)
minor:
    @current=$$(just version) && \
    major=$$(echo $$current | cut -d. -f1) && \
    minor=$$(echo $$current | cut -d. -f2) && \
    new_version="$$major.$$((minor + 1)).0" && \
    sed -i "s/^version = \".*\"/version = \"$$new_version\"/" Cargo.toml && \
    echo "Bumped to version: $$new_version"

# Bump major version (0.1.0 -> 1.0.0)
major:
    @current=$$(just version) && \
    major=$$(echo $$current | cut -d. -f1) && \
    new_version="$$((major + 1)).0.0" && \
    sed -i "s/^version = \".*\"/version = \"$$new_version\"/" Cargo.toml && \
    echo "Bumped to version: $$new_version"

# Set specific version
set-version version:
    @if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then \
        echo "Error: Version must be in format X.Y.Z (e.g., 1.2.3)"; \
        exit 1; \
    fi
    @sed -i "s/^version = \".*\"/version = \"$version\"/" Cargo.toml
    @echo "Set version to: $version"

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run tests for specific package
test-package package:
    cargo test -p $package

# Check code without building
check:
    cargo check

# Check Rust version requirement
check-rust-version:
    @if [ -f scripts/check-rust.sh ]; then \
        bash scripts/check-rust.sh; \
    else \
        echo "Checking Rust version requirement..."; \
        required_version="1.85.0" && \
        current_version=$$(rustc --version | cut -d' ' -f2) && \
        echo "Required: $$required_version" && \
        echo "Current:  $$current_version" && \
        if [ "$$(printf '%s\n' "$$required_version" "$$current_version" | sort -V | head -n1)" = "$$required_version" ]; then \
            echo "✅ Rust version requirement satisfied"; \
        else \
            echo "❌ Rust version requirement not satisfied. Please upgrade to $$required_version or later"; \
            exit 1; \
        fi; \
    fi

# Check all packages
check-all:
    cargo check --workspace

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt -- --check

# Run clippy lints
lint:
    cargo clippy

# Run clippy with all warnings
lint-all:
    cargo clippy -- -W clippy::all

# Run all checks (format, lint, test)
ci:
    just check-rust-version
    just fmt-check
    just lint
    just test

# Clean build artifacts
clean:
    cargo clean

# Clean and rebuild
rebuild: clean build

# Clean and rebuild in release mode
rebuild-release: clean build-release

# Run examples
examples:
    cargo run --example

# Run basic usage example
example-basic:
    cargo run --example basic_usage

# List all examples
examples-list:
    @echo "Available examples:"
    @find examples -name "*.rs" | sed 's|examples/||' | sed 's|\.rs||' | sort

# Generate documentation
docs:
    cargo doc --open

# Generate documentation for all packages
docs-all:
    cargo doc --workspace --open

# Generate documentation without opening browser
docs-build:
    cargo doc --workspace

# Generate documentation with all features
docs-full:
    cargo doc --workspace --all-features --open

# Generate documentation and serve locally
docs-serve:
    cargo doc --workspace --no-deps
    @echo "Documentation generated in target/doc/"
    @echo "Open target/doc/ferriclink_core/index.html in your browser"

# Generate documentation for specific package
docs-package package:
    cargo doc -p $package --open

# Generate documentation using script
docs-script:
    bash scripts/docs.sh

# Generate documentation with custom options
docs-custom:
    @echo "Available options:"
    @echo "  just docs-script --help"
    @echo "  just docs-script --no-open"
    @echo "  just docs-script --serve"
    @echo "  just docs-script --package ferriclink-core"
    @echo "  just docs-script --features all"

# Clean documentation
docs-clean:
    rm -rf target/doc
    echo "Documentation cleaned"

# Run benchmarks
bench:
    cargo bench

# Check for outdated dependencies
outdated:
    cargo outdated

# Update dependencies
update:
    cargo update

# Update dependencies and check for security vulnerabilities
update-audit:
    cargo update
    cargo audit

# Install development dependencies
install-dev:
    cargo install cargo-audit cargo-outdated cargo-criterion

# Show project info
info:
    @echo "FerricLink Project Information"
    @echo "=============================="
    @echo "Version: $(just version)"
    @echo "Rust Edition: 2024"
    @echo "Min Rust Version: 1.85.0"
    @echo "License: Apache-2.0"
    @echo "Repository: https://github.com/ferrum-labs/ferriclink"
    @echo ""
    @echo "Available packages:"
    @find crates -name "Cargo.toml" -exec basename {} \; | sed 's/Cargo.toml/-core/' | sort

# Show help
help:
    @just --list

# Git helpers
git-tag-version:
    @version=$$(just version) && \
    git tag -a "v$$version" -m "Release version $$version" && \
    echo "Tagged version v$$version"

# Create a new release
release patch:
    @just patch
    @just git-tag-version
    @echo "Created patch release: $(just version)"

release minor:
    @just minor
    @just git-tag-version
    @echo "Created minor release: $(just version)"

release major:
    @just major
    @just git-tag-version
    @echo "Created major release: $(just version)"

# Publish to crates.io (dry run)
publish-dry-run:
    cargo publish --dry-run

# Publish to crates.io
publish:
    cargo publish

# Show workspace members
workspace-members:
    @cargo metadata --format-version 1 | jq -r '.workspace_members[]' | sed 's/.*ferriclink-//' | sort

# Show dependency tree
deps:
    cargo tree

# Show dependency tree for specific package
deps-package package:
    cargo tree -p $package

# Run cargo expand (requires cargo-expand)
expand package:
    cargo expand -p $package

# Run cargo expand for all packages
expand-all:
    @for pkg in $$(just workspace-members); do \
        echo "Expanding ferriclink-$$pkg..."; \
        cargo expand -p ferriclink-$$pkg; \
    done

# Development server (if applicable)
dev:
    @echo "Starting development server..."
    @echo "Note: Add your development server command here"

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Watch for changes and run check
watch-check:
    cargo watch -x check

# Watch for changes and run clippy
watch-lint:
    cargo watch -x clippy

# Show help with examples
help-examples:
    @echo "FerricLink Development Examples"
    @echo "==============================="
    @echo ""
    @echo "Requirements:"
    @echo "  Rust 1.85.0+              # Minimum Rust version"
    @echo "  just check-rust-version   # Check Rust version requirement"
    @echo ""
    @echo "Version Management:"
    @echo "  just version              # Show current version"
    @echo "  just patch                # Bump patch (0.1.0 -> 0.1.1)"
    @echo "  just minor                # Bump minor (0.1.0 -> 0.2.0)"
    @echo "  just major                # Bump major (0.1.0 -> 1.0.0)"
    @echo "  just set-version 1.2.3    # Set specific version"
    @echo ""
    @echo "Development:"
    @echo "  just build                # Build project"
    @echo "  just test                 # Run tests"
    @echo "  just check                # Check code"
    @echo "  just ci                   # Run all checks"
    @echo "  just watch-test           # Watch and test"
    @echo ""
    @echo "Documentation:"
    @echo "  just docs                 # Generate docs"
    @echo "  just docs-all             # Generate all docs"
    @echo ""
    @echo "Release:"
    @echo "  just release patch        # Create patch release"
    @echo "  just release minor        # Create minor release"
    @echo "  just release major        # Create major release"
