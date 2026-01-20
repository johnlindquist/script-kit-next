#!/bin/bash
# Test: Claude Code CLI Session Persistence Fix
# 
# This test validates that --session-id creates a new session
# and --resume continues an existing session.
#
# Expected behavior:
# - First message with --session-id: creates session, Claude remembers info
# - Second message with --resume: loads session, Claude recalls info

set -e

SESSION_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
echo "=== Claude Code Session Persistence Test ==="
echo "Session ID: $SESSION_ID"
echo ""

# First message - establish context with --session-id
echo "=== Message 1: Establishing context (--session-id) ==="
RESULT1=$(timeout 45 claude --print \
  --verbose \
  --output-format stream-json \
  --setting-sources "" \
  --tools "WebSearch, WebFetch, Read" \
  --no-chrome \
  --disable-slash-commands \
  --session-id "$SESSION_ID" \
  --system-prompt "You are a helpful assistant. Be very brief. Just acknowledge what you're told." \
  "Remember this: my secret code is BANANA42" 2>&1 | grep -E '"type":"(result)"' | head -1)

echo "Response 1: $RESULT1"
echo ""

# Brief pause for session to be written
sleep 1

# Second message - recall with --resume (THIS IS THE FIX!)
echo "=== Message 2: Recalling context (--resume) ==="
RESULT2=$(timeout 45 claude --print \
  --verbose \
  --output-format stream-json \
  --setting-sources "" \
  --tools "WebSearch, WebFetch, Read" \
  --no-chrome \
  --disable-slash-commands \
  --resume "$SESSION_ID" \
  "What is my secret code?" 2>&1 | grep -E '"type":"(result)"' | head -1)

echo "Response 2: $RESULT2"
echo ""

# Check if BANANA42 appears in the second response
if echo "$RESULT2" | grep -qi "BANANA42"; then
    echo "=== SUCCESS: Session persistence working! Claude remembered BANANA42 ==="
    exit 0
else
    echo "=== FAILURE: Claude did not remember the secret code ==="
    echo "This suggests --resume is not loading the session correctly."
    exit 1
fi
