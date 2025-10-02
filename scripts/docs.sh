#!/usr/bin/env bash
# Documentation generation script for FerricLink

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
OPEN_BROWSER=true
SERVE_LOCAL=false
PACKAGE=""
FEATURES=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-open)
            OPEN_BROWSER=false
            shift
            ;;
        --serve)
            SERVE_LOCAL=true
            shift
            ;;
        --package)
            PACKAGE="$2"
            shift 2
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        --help|-h)
            echo "FerricLink Documentation Generator"
            echo "=================================="
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --no-open        Don't open browser after generation"
            echo "  --serve          Serve documentation locally"
            echo "  --package PKG    Generate docs for specific package"
            echo "  --features FEAT  Enable specific features"
            echo "  --help, -h       Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                           # Generate all docs and open browser"
            echo "  $0 --no-open                # Generate docs without opening browser"
            echo "  $0 --package ferriclink-core # Generate docs for core package only"
            echo "  $0 --serve                  # Generate and serve locally"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}FerricLink Documentation Generator${NC}"
echo "=================================="

# Build the cargo doc command
CMD="cargo doc"

if [ -n "$PACKAGE" ]; then
    CMD="$CMD -p $PACKAGE"
    echo -e "${YELLOW}Generating documentation for package: $PACKAGE${NC}"
else
    echo -e "${YELLOW}Generating documentation for all packages${NC}"
    CMD="$CMD --workspace"
fi

if [ -n "$FEATURES" ]; then
    CMD="$CMD --features $FEATURES"
    echo -e "${YELLOW}With features: $FEATURES${NC}"
fi

if [ "$SERVE_LOCAL" = true ]; then
    CMD="$CMD --no-deps"
    echo -e "${YELLOW}Generating without dependencies${NC}"
fi

if [ "$OPEN_BROWSER" = false ]; then
    echo -e "${YELLOW}Not opening browser${NC}"
else
    CMD="$CMD --open"
fi

echo -e "${BLUE}Running: $CMD${NC}"
echo ""

# Execute the command
if eval $CMD; then
    echo ""
    echo -e "${GREEN}✅ Documentation generated successfully!${NC}"
    
    if [ "$SERVE_LOCAL" = true ]; then
        echo ""
        echo -e "${BLUE}Documentation location:${NC}"
        echo "  target/doc/ferriclink_core/index.html"
        echo ""
        echo -e "${BLUE}To serve locally:${NC}"
        echo "  cd target/doc && python3 -m http.server 8000"
        echo "  Then open: http://localhost:8000/ferriclink_core/index.html"
    fi
else
    echo ""
    echo -e "${RED}❌ Documentation generation failed!${NC}"
    exit 1
fi
