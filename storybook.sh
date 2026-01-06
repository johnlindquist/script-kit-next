#!/bin/bash

# Storybook runner script for script-kit-gpui
# Uses cargo-watch to auto-rebuild on Rust file changes
# Clears screen between rebuilds for clean output

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
STORY=""
HOT_RELOAD=true  # Default to watch mode
LIST_STORIES=false

usage() {
    echo -e "${BLUE}Script Kit GPUI Storybook${NC}"
    echo ""
    echo "Usage: ./storybook.sh [OPTIONS] [STORY_NAME]"
    echo ""
    echo "Options:"
    echo "  -1, --once      Single run (no hot reload)"
    echo "  -l, --list      List available stories"
    echo "  -h, --help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./storybook.sh                    # Run with hot reload (default)"
    echo "  ./storybook.sh header-variations  # Run specific story with hot reload"
    echo "  ./storybook.sh -1                 # Single run, no watching"
    echo "  ./storybook.sh -1 button          # Run button story once"
    echo "  ./storybook.sh -l                 # List all available stories"
    echo ""
    echo "Available stories:"
    echo "  button, toast, form-fields, list-item, scrollbar,"
    echo "  design-tokens, header-variations"
    echo ""
}

while [[ $# -gt 0 ]]; do
    case $1 in
        -1|--once)
            HOT_RELOAD=false
            shift
            ;;
        -l|--list)
            LIST_STORIES=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            echo -e "${RED}Unknown option: $1${NC}"
            usage
            exit 1
            ;;
        *)
            STORY="$1"
            shift
            ;;
    esac
done

# Build the storybook args
STORYBOOK_ARGS=""
if [[ -n "$STORY" ]]; then
    STORYBOOK_ARGS="--story $STORY"
fi

# List stories mode
if [[ "$LIST_STORIES" == true ]]; then
    echo -e "${BLUE}Available stories:${NC}"
    echo ""
    echo "  button            - Button component variations"
    echo "  toast             - Toast notification styles"
    echo "  form-fields       - Form input components"
    echo "  list-item         - List item designs"
    echo "  scrollbar         - Scrollbar styles"
    echo "  design-tokens     - Design system tokens"
    echo "  header-variations - Prompt header variations"
    echo ""
    exit 0
fi

# Check if cargo-watch is installed (only needed for hot reload)
if [[ "$HOT_RELOAD" == true ]]; then
    if ! command -v cargo-watch &> /dev/null; then
        echo -e "${RED}❌ cargo-watch is not installed${NC}"
        echo ""
        echo "Install it with:"
        echo "  cargo install cargo-watch"
        echo ""
        echo "Or run without hot reload:"
        echo "  ./storybook.sh $STORY"
        exit 1
    fi
fi

# Hot reload mode
if [[ "$HOT_RELOAD" == true ]]; then
    echo -e "${GREEN}🚀 Starting storybook with hot reload...${NC}"
    echo -e "   ${YELLOW}Watching for changes to .rs files in src/stories/ and src/storybook/${NC}"
    if [[ -n "$STORY" ]]; then
        echo -e "   Story: ${BLUE}$STORY${NC}"
    fi
    echo "   Press Ctrl+C to stop"
    echo ""
    
    # Watch only storybook-related files for faster rebuilds
    # -w: watch specific directories
    # -c: clear screen between runs
    # -x: execute command
    cargo watch \
        -w src/stories \
        -w src/storybook \
        -w src/bin/storybook.rs \
        -w src/theme \
        -w src/components \
        -c \
        -x "run --bin storybook -- $STORYBOOK_ARGS"
else
    # Single run mode
    echo -e "${GREEN}🎨 Building and running storybook...${NC}"
    if [[ -n "$STORY" ]]; then
        echo -e "   Story: ${BLUE}$STORY${NC}"
    fi
    echo ""
    
    cargo run --bin storybook -- $STORYBOOK_ARGS
fi
