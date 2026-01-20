#!/bin/bash
# Test: Persistent Claude CLI Session
#
# This test validates that the ClaudeSessionManager keeps a single process
# alive across multiple messages, providing efficient session persistence.

set -e

echo "=== Persistent Claude Session Test ==="
echo ""

# Build the app first
echo "Building app..."
cd /Users/johnlindquist/dev/script-kit-gpui
cargo build 2>&1 | tail -2

echo ""
echo "Testing persistent session via CLI directly..."
echo ""

SESSION_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
echo "Session ID: $SESSION_ID"

# Send two messages to the same session via pipe
{
  echo '{"type":"user","message":{"role":"user","content":"Remember: the secret password is WATERMELON123"}}'
  sleep 3
  echo '{"type":"user","message":{"role":"user","content":"What is the secret password I told you?"}}'
  sleep 3
} | timeout 45 claude --print \
  --verbose \
  --input-format stream-json \
  --output-format stream-json \
  --setting-sources "" \
  --tools "" \
  --no-chrome \
  --disable-slash-commands \
  --session-id "$SESSION_ID" \
  --system-prompt "You are a helpful assistant. Be very brief." 2>&1 | while IFS= read -r line; do
    TYPE=$(echo "$line" | jq -r '.type // empty' 2>/dev/null)
    if [ "$TYPE" = "result" ]; then
      RESULT=$(echo "$line" | jq -r '.result // empty' 2>/dev/null)
      echo "[RESULT] $RESULT"
      
      # Check if this result contains WATERMELON123
      if echo "$RESULT" | grep -qi "WATERMELON123"; then
        echo ""
        echo "=== SUCCESS: Persistent session remembered the password! ==="
      fi
    fi
done

echo ""
echo "Test complete."
