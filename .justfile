# Global FerricLink Commands
# This file contains project-wide commands

# Show project status
status:
    @echo "FerricLink Project Status"
    @echo "========================="
    @echo "Version: $(just version)"
    @echo "Last commit: $(git log -1 --format='%h %s' 2>/dev/null || echo 'No git repository')"
    @echo "Working directory: $(pwd)"
    @echo ""
    @echo "Quick commands:"
    @echo "  just build     - Build the project"
    @echo "  just test      - Run tests"
    @echo "  just check     - Check code"
    @echo "  just ci        - Run all checks"
    @echo "  just version   - Show current version"
    @echo "  just patch     - Bump patch version"
    @echo "  just minor     - Bump minor version"
    @echo "  just major     - Bump major version"

# Initialize git hooks (if not already done)
init-hooks:
    @if [ ! -d .git/hooks ]; then \
        echo "Not a git repository. Run 'git init' first."; \
        exit 1; \
    fi
    @echo "Setting up git hooks..."
    @echo '#!/bin/sh\njust ci' > .git/hooks/pre-commit
    @chmod +x .git/hooks/pre-commit
    @echo "Pre-commit hook installed"

# Show version history
version-history:
    @echo "Version History"
    @echo "==============="
    @git tag --sort=-version:refname 2>/dev/null || echo "No version tags found"

# Show changelog (if exists)
changelog:
    @if [ -f CHANGELOG.md ]; then \
        cat CHANGELOG.md; \
    else \
        echo "No CHANGELOG.md found"; \
    fi

# Create a new changelog entry
changelog-entry type message:
    @if [ ! -f CHANGELOG.md ]; then \
        echo "# Changelog\n\nAll notable changes to this project will be documented in this file.\n" > CHANGELOG.md; \
    fi
    @version=$$(just version) && \
    date=$$(date +%Y-%m-%d) && \
    sed -i "2i\\\n## [$$version] - $$date\n\n- $$type: $$message\n" CHANGELOG.md && \
    echo "Added changelog entry for version $$version"

# Show project metrics
metrics:
    @echo "FerricLink Project Metrics"
    @echo "=========================="
    @echo "Lines of code:"
    @find crates -name "*.rs" -exec wc -l {} + | tail -1
    @echo ""
    @echo "Number of files:"
    @find crates -name "*.rs" | wc -l
    @echo ""
    @echo "Dependencies:"
    @cargo tree --depth 1 | grep -c "├\|└" || echo "0"
    @echo ""
    @echo "Test coverage:"
    @echo "Run 'cargo tarpaulin' for test coverage (requires cargo-tarpaulin)"

# Security audit
audit:
    @if command -v cargo-audit >/dev/null 2>&1; then \
        cargo audit; \
    else \
        echo "cargo-audit not installed. Run 'cargo install cargo-audit'"; \
    fi

# License check
license-check:
    @echo "Checking licenses..."
    @if command -v cargo-license >/dev/null 2>&1; then \
        cargo license; \
    else \
        echo "cargo-license not installed. Run 'cargo install cargo-license'"; \
    fi

# Full project check
full-check: ci audit
    @echo "Full project check completed"

# Show help with examples
help-examples:
    @echo "FerricLink Development Examples"
    @echo "==============================="
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
