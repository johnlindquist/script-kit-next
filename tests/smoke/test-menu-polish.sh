#!/usr/bin/env bash
# Visual test: Main menu design polish - captures multiple states for review
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
SCREENSHOT_DIR="$PROJECT_DIR/test-screenshots"
mkdir -p "$SCREENSHOT_DIR"

# Clean up old screenshots
rm -f "$SCREENSHOT_DIR"/menu-polish-*.png

# Unset AI keys to avoid triggering AI features
unset VERCEL_AI_GATEWAY_API_KEY 2>/dev/null || true
unset ANTHROPIC_API_KEY 2>/dev/null || true
unset OPENAI_API_KEY 2>/dev/null || true

export SCRIPT_KIT_AI_LOG=1

PIPE=$(mktemp -u)
mkfifo "$PIPE"

"$PROJECT_DIR/target/debug/script-kit-gpui" < "$PIPE" > /tmp/sk-test-menu-polish-stdout.log 2>&1 &
APP_PID=$!

exec 3>"$PIPE"

sleep 3

# 1. Show window - initial grouped menu state
echo '{"type":"show"}' >&3
sleep 1.5

echo '{"type":"captureWindow","title":"","path":"'"$SCREENSHOT_DIR/menu-polish-01-initial.png"'"}' >&3
sleep 1

# 2. Navigate down a few items to show selection state
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.3
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.3
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.5

echo '{"type":"captureWindow","title":"","path":"'"$SCREENSHOT_DIR/menu-polish-02-selected.png"'"}' >&3
sleep 1

# 3. Navigate further to show section headers
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.3
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.3
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.3
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.3
echo '{"type":"simulateKey","key":"down","modifiers":[]}' >&3
sleep 0.5

echo '{"type":"captureWindow","title":"","path":"'"$SCREENSHOT_DIR/menu-polish-03-scrolled.png"'"}' >&3
sleep 1

# 4. Search filtering
echo '{"type":"setFilter","text":"set"}' >&3
sleep 1

echo '{"type":"captureWindow","title":"","path":"'"$SCREENSHOT_DIR/menu-polish-04-search.png"'"}' >&3
sleep 1

# 5. Clear and try another search
echo '{"type":"setFilter","text":""}' >&3
sleep 0.5
echo '{"type":"setFilter","text":"open"}' >&3
sleep 1

echo '{"type":"captureWindow","title":"","path":"'"$SCREENSHOT_DIR/menu-polish-05-search2.png"'"}' >&3
sleep 1

# Cleanup
exec 3>&-
rm -f "$PIPE"
sleep 0.5
kill $APP_PID 2>/dev/null || true
wait $APP_PID 2>/dev/null || true

echo "Screenshots saved to $SCREENSHOT_DIR/menu-polish-*.png"
ls -la "$SCREENSHOT_DIR"/menu-polish-*.png 2>/dev/null || echo "No screenshots found"
