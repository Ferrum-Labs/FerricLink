#!/usr/bin/env bash
# Check Rust version requirement for FerricLink

set -e

REQUIRED_VERSION="1.85.0"
CURRENT_VERSION=$(rustc --version | cut -d' ' -f2)

echo "FerricLink Rust Version Check"
echo "============================="
echo "Required: $REQUIRED_VERSION"
echo "Current:  $CURRENT_VERSION"

# Compare versions using sort -V (version sort)
if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$CURRENT_VERSION" | sort -V | head -n1)" = "$REQUIRED_VERSION" ]; then
    echo "✅ Rust version requirement satisfied"
    exit 0
else
    echo "❌ Rust version requirement not satisfied"
    echo "Please upgrade to Rust $REQUIRED_VERSION or later"
    echo ""
    echo "To upgrade Rust:"
    echo "  rustup update stable"
    echo "  rustup default stable"
    exit 1
fi
