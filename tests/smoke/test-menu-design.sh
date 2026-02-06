#!/bin/bash
# Visual test: main menu list design polish
# Usage: ./tests/smoke/test-menu-design.sh
# Screenshots: test-screenshots/menu-design-*.png

set -e
cd "$(dirname "$0")/../.."

SCREENSHOT_DIR="test-screenshots"
mkdir -p "$SCREENSHOT_DIR"
rm -f "$SCREENSHOT_DIR"/menu-design-*.png

PIPE=$(mktemp -u)
mkfifo "$PIPE"

export SCRIPT_KIT_AI_LOG=1

./target/debug/script-kit-gpui < "$PIPE" > /tmp/sk-test-menu-design.log 2>&1 &
APP_PID=$!
exec 3>"$PIPE"

sleep 3  # App startup

echo '{"type":"show"}' >&3
sleep 1

# Step 1: Capture initial main menu (grouped view)
echo "[TEST] Step 1: Capturing initial main menu..."
echo '{"type":"captureWindow","title":"","path":"'"$(pwd)/$SCREENSHOT_DIR"'/menu-design-1-grouped.png"}' >&3
sleep 1

# Step 2: Navigate down a few items
echo "[TEST] Step 2: Navigating down..."
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.3
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.3
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.5

echo '{"type":"captureWindow","title":"","path":"'"$(pwd)/$SCREENSHOT_DIR"'/menu-design-2-navigated.png"}' >&3
sleep 1

# Step 3: Type a search filter
echo "[TEST] Step 3: Filtering with search..."
echo '{"type":"setFilter","text":"set"}' >&3
sleep 1

echo '{"type":"captureWindow","title":"","path":"'"$(pwd)/$SCREENSHOT_DIR"'/menu-design-3-filtered.png"}' >&3
sleep 1

# Cleanup
exec 3>&-
rm -f "$PIPE"
kill $APP_PID 2>/dev/null || true
wait $APP_PID 2>/dev/null || true

echo "[TEST] Screenshots saved to $SCREENSHOT_DIR/:"
ls -la "$SCREENSHOT_DIR"/menu-design-*.png 2>/dev/null || echo "  (none)"
