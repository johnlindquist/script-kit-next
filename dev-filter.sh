#!/bin/bash

# Dev runner with log filtering for script-kit-gpui
# Usage: ./dev-filter.sh <filter_pattern>
#
# Examples:
#   ./dev-filter.sh VISIBILITY     # Window visibility logs
#   ./dev-filter.sh CHAT           # Chat prompt logs
#   ./dev-filter.sh "CHAT|VISIBILITY"  # Multiple patterns
#   ./dev-filter.sh ShowChat       # Specific message type
#   ./dev-filter.sh getSelectedText  # Selected text flow
#
# Log categories (from AGENTS.md):
#   P=POSITION A=APP U=UI S=STDIN H=HOTKEY V=VISIBILITY E=EXEC
#   K=KEY F=FOCUS T=THEME C=CACHE R=PERF W=WINDOW_MGR X=ERROR
#   M=MOUSE_HOVER L=SCROLL_STATE Q=SCROLL_PERF D=DESIGN B=SCRIPT N=CONFIG Z=RESIZE

# Check for required argument
if [ -z "$1" ]; then
    echo "❌ Missing required filter pattern"
    echo ""
    echo "Usage: ./dev-filter.sh <filter_pattern>"
    echo ""
    echo "Examples:"
    echo "  ./dev-filter.sh VISIBILITY        # Window visibility logs"
    echo "  ./dev-filter.sh CHAT              # Chat prompt logs"
    echo "  ./dev-filter.sh 'CHAT|VISIBILITY' # Multiple patterns (use quotes)"
    echo "  ./dev-filter.sh ShowChat          # Specific message type"
    echo "  ./dev-filter.sh getSelectedText   # Selected text flow"
    echo "  ./dev-filter.sh 'hide|show'       # Window show/hide events"
    echo ""
    echo "Log categories:"
    echo "  VISIBILITY, CHAT, EXEC, UI, HOTKEY, KEY, FOCUS, STDIN"
    echo "  THEME, CACHE, PERF, WINDOW_MGR, ERROR, CONFIG, RESIZE"
    echo ""
    exit 1
fi

FILTER_PATTERN="$1"

echo "🔍 Starting dev runner with log filter: $FILTER_PATTERN"
echo "   Environment: SCRIPT_KIT_AI_LOG=1 (compact AI logs)"
echo "   Quit the app (Cmd+Q) to exit this script"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Build first
echo "📦 Building..."
cargo build 2>&1 | grep -v "^warning:" | grep -v "Compiling" | grep -v "Finished" || true
echo ""
echo "🚀 App starting... (quit app with Cmd+Q to exit)"
echo ""

# Run with AI log mode and filter output
# --line-buffered ensures grep outputs immediately as lines arrive
# The script will exit when the app exits (pipe closes)
SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
    grep --line-buffered -iE "$FILTER_PATTERN"

EXIT_CODE=$?
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ App exited (code: $EXIT_CODE)"
