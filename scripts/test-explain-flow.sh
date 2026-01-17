#!/bin/bash
# Automated test for the "Explain This" flow (hide -> chat)
# This script runs the test and checks logs for success/failure

set -e

cd /Users/johnlindquist/dev/script-kit-gpui

echo "=== Building app ==="
cargo build 2>&1 | tail -3

echo ""
echo "=== Copying SDK ==="
cp scripts/kit-sdk.ts ~/.scriptkit/sdk/kit-sdk.ts

echo ""
echo "=== Clearing old logs ==="
# Keep last 100 lines for context, clear the rest
tail -100 ~/.scriptkit/logs/script-kit-gpui.jsonl > /tmp/old-logs.jsonl 2>/dev/null || true
cp /tmp/old-logs.jsonl ~/.scriptkit/logs/script-kit-gpui.jsonl 2>/dev/null || true

echo ""
echo "=== Running test (15 second timeout) ==="
TEST_PATH="$(pwd)/tests/smoke/test-explain-flow.ts"

# Run with timeout, capture all output
timeout 15 bash -c "echo '{\"type\":\"run\",\"path\":\"$TEST_PATH\"}' | RUST_LOG=info ./target/debug/script-kit-gpui 2>&1" > /tmp/test-output.txt || true

echo ""
echo "=== Checking for key events in output ==="

# Check for the critical sequence
echo "Looking for: HideWindow, ShowChat, NEEDS_RESET, Force-killing..."
echo ""

grep -E "(HideWindow|ShowChat|NEEDS_RESET|Force-killing|chat\(\) completed|handle_initial_response)" /tmp/test-output.txt | head -30 || echo "(no matches in stdout)"

echo ""
echo "=== Checking logs file ==="
echo ""

# Get logs from this run (last 200 lines should be enough)
tail -200 ~/.scriptkit/logs/script-kit-gpui.jsonl | grep -E "(HideWindow|ShowChat|NEEDS_RESET|Force-killing|initial_response|Built-in AI)" | head -30 || echo "(no matches in log file)"

echo ""
echo "=== VERDICT ==="

# Check for success indicators
if tail -200 ~/.scriptkit/logs/script-kit-gpui.jsonl | grep -q "Built-in AI initial response complete"; then
    echo "SUCCESS: AI responded to initial messages"
elif tail -200 ~/.scriptkit/logs/script-kit-gpui.jsonl | grep -q "Force-killing script process"; then
    echo "FAILURE: Script was force-killed (NEEDS_RESET bug)"
elif grep -q "chat() completed" /tmp/test-output.txt; then
    echo "SUCCESS: Script completed normally"
else
    echo "UNKNOWN: Check logs manually"
fi

echo ""
echo "=== Full relevant log entries ==="
tail -200 ~/.scriptkit/logs/script-kit-gpui.jsonl | jq -r 'select(.message | test("NEEDS_RESET|HideWindow|ShowChat|initial_response|Force-killing|Built-in AI"; "i")) | "\(.timestamp) \(.message)"' 2>/dev/null | tail -20 || tail -50 /tmp/test-output.txt
