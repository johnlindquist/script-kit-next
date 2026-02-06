#!/usr/bin/env bash
# Visual test: Verify main menu design polish changes
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
SCREENSHOT_DIR="$PROJECT_DIR/test-screenshots"
mkdir -p "$SCREENSHOT_DIR"

# Clean up old verification screenshots
rm -f "$SCREENSHOT_DIR"/menu-verify-*.png

# Unset AI keys
unset VERCEL_AI_GATEWAY_API_KEY 2>/dev/null || true
unset ANTHROPIC_API_KEY 2>/dev/null || true
unset OPENAI_API_KEY 2>/dev/null || true

export SCRIPT_KIT_AI_LOG=1

PIPE=$(mktemp -u)
mkfifo "$PIPE"

"$PROJECT_DIR/target/debug/script-kit-gpui" < "$PIPE" > /tmp/sk-test-verify-stdout.log 2>&1 &
APP_PID=$!

exec 3>"$PIPE"

sleep 3

# 1. Initial grouped menu
echo '{"type":"show"}' >&3
sleep 1.5

echo '{"type":"captureWindow","title":"","path":"'"$SCREENSHOT_DIR/menu-verify-01-initial.png"'"}' >&3
sleep 1

# 2. Selection state (navigate down 3 items)
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.3
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.3
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.5

echo '{"type":"captureWindow","title":"","path":"'"$SCREENSHOT_DIR/menu-verify-02-selected.png"'"}' >&3
sleep 1

# 3. Scrolled to section header boundary
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.2
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.2
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.2
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.2
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.5

echo '{"type":"captureWindow","title":"","path":"'"$SCREENSHOT_DIR/menu-verify-03-scrolled.png"'"}' >&3
sleep 1

# 4. Search "set"
echo '{"type":"setFilter","text":"set"}' >&3
sleep 1

echo '{"type":"captureWindow","title":"","path":"'"$SCREENSHOT_DIR/menu-verify-04-search.png"'"}' >&3
sleep 1

# 5. Search "open"
echo '{"type":"setFilter","text":""}' >&3
sleep 0.5
echo '{"type":"setFilter","text":"open"}' >&3
sleep 1

echo '{"type":"captureWindow","title":"","path":"'"$SCREENSHOT_DIR/menu-verify-05-search2.png"'"}' >&3
sleep 1

# Cleanup
exec 3>&-
rm -f "$PIPE"
sleep 0.5
kill $APP_PID 2>/dev/null || true
wait $APP_PID 2>/dev/null || true

echo "Verification screenshots saved to $SCREENSHOT_DIR/menu-verify-*.png"
ls -la "$SCREENSHOT_DIR"/menu-verify-*.png 2>/dev/null || echo "No screenshots found"
