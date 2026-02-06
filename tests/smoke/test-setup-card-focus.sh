#!/bin/bash
# Visual test: AI Setup Card focus indicators and Tab navigation
# Tests that Tab cycles between buttons with visible focus rings
#
# Usage: ./tests/smoke/test-setup-card-focus.sh
# Screenshots saved to: test-screenshots/setup-focus-*.png

set -e
cd "$(dirname "$0")/../.."

SCREENSHOT_DIR="test-screenshots"
mkdir -p "$SCREENSHOT_DIR"

# Clean up old screenshots
rm -f "$SCREENSHOT_DIR"/setup-focus-*.png

echo "[TEST] Building app..."
cargo build 2>&1 | tail -3

echo "[TEST] Starting AI setup card focus test..."

# Create a pipe for sending commands
PIPE=$(mktemp -u)
mkfifo "$PIPE"

# Ensure no AI providers are configured so the setup card shows
unset VERCEL_AI_GATEWAY_API_KEY
unset ANTHROPIC_API_KEY
unset OPENAI_API_KEY
export SCRIPT_KIT_AI_LOG=1

# Start the app with stdin from pipe, capture output (suppress to file)
./target/debug/script-kit-gpui < "$PIPE" > /tmp/sk-test-stdout.log 2>&1 &
APP_PID=$!

# Open file descriptor 3 to keep the pipe open for multiple writes
exec 3>"$PIPE"

# Give the app time to start
sleep 3

# Show the window first (starts hidden)
echo "[TEST] Showing window..."
echo '{"type":"show"}' >&3
sleep 1

# Step 1: Set filter text (needed for Tab to trigger inline AI chat)
echo "[TEST] Step 1: Setting filter text..."
echo '{"type":"setFilter","text":"test query"}' >&3
sleep 1

# Step 2: Tab to open inline AI chat -> shows setup card (no providers configured)
echo "[TEST] Step 2: Tab to open AI setup card..."
echo '{"type":"simulateKey","key":"tab","modifiers":[]}' >&3
sleep 1.5

# Step 3: Capture initial state (focus on "Configure Vercel AI Gateway" button, index 0)
echo "[TEST] Step 3: Capturing initial focus state (button 0)..."
echo '{"type":"captureWindow","title":"","path":"'"$(pwd)/$SCREENSHOT_DIR"'/setup-focus-0-configure.png"}' >&3
sleep 1

# Step 4: Tab to move focus to "Connect to Claude Code" button
echo "[TEST] Step 4: Tab to Claude Code button..."
echo '{"type":"simulateKey","key":"tab","modifiers":[]}' >&3
sleep 0.5

# Step 5: Capture focus on Claude Code button (index 1)
echo "[TEST] Step 5: Capturing Claude Code focus state (button 1)..."
echo '{"type":"captureWindow","title":"","path":"'"$(pwd)/$SCREENSHOT_DIR"'/setup-focus-1-claude-code.png"}' >&3
sleep 1

# Step 6: Tab again to cycle back to Configure button
echo "[TEST] Step 6: Tab back to Configure button..."
echo '{"type":"simulateKey","key":"tab","modifiers":[]}' >&3
sleep 0.5

# Step 7: Capture focus back on Configure button
echo "[TEST] Step 7: Capturing focus cycled back (button 0)..."
echo '{"type":"captureWindow","title":"","path":"'"$(pwd)/$SCREENSHOT_DIR"'/setup-focus-2-cycled-back.png"}' >&3
sleep 1

# Step 8: Arrow down to Claude Code
echo "[TEST] Step 8: Arrow down to Claude Code..."
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.5

# Step 9: Capture arrow navigation state
echo "[TEST] Step 9: Capturing arrow-down state (button 1)..."
echo '{"type":"captureWindow","title":"","path":"'"$(pwd)/$SCREENSHOT_DIR"'/setup-focus-3-arrow-down.png"}' >&3
sleep 1

# Step 10: Shift+Tab back
echo "[TEST] Step 10: Shift+Tab back to Configure..."
echo '{"type":"simulateKey","key":"tab","modifiers":["shift"]}' >&3
sleep 0.5

# Step 11: Capture Shift+Tab state
echo "[TEST] Step 11: Capturing Shift+Tab state (button 0)..."
echo '{"type":"captureWindow","title":"","path":"'"$(pwd)/$SCREENSHOT_DIR"'/setup-focus-4-shift-tab.png"}' >&3
sleep 1

# Clean up
echo "[TEST] Cleaning up..."
exec 3>&-  # Close the file descriptor
rm -f "$PIPE"
kill $APP_PID 2>/dev/null || true
wait $APP_PID 2>/dev/null || true

echo ""
echo "[TEST] Screenshots saved to $SCREENSHOT_DIR/:"
ls -la "$SCREENSHOT_DIR"/setup-focus-*.png 2>/dev/null || echo "  (no screenshots found)"
echo ""
echo "[TEST] Test complete! Review the screenshots to verify:"
echo "  - setup-focus-0-configure.png: Initial state, Configure button should have focus ring"
echo "  - setup-focus-1-claude-code.png: Claude Code button should have focus ring"
echo "  - setup-focus-2-cycled-back.png: Configure button should have focus ring again"
echo "  - setup-focus-3-arrow-down.png: Claude Code button (via arrow key)"
echo "  - setup-focus-4-shift-tab.png: Configure button (via Shift+Tab)"
