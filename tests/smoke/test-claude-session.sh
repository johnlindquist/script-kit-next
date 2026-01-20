#!/bin/bash
# Test Claude Code CLI session persistence
# This tests whether the --session-id flag maintains conversation context

set -e

SESSION_ID=$(uuidgen)
echo "=== Testing Claude Code CLI Session Persistence ==="
echo "Session ID: $SESSION_ID"
echo ""

# First message
echo ">>> Message 1: Telling Claude my favorite color is blue"
RESPONSE1=$(claude \
    --setting-sources "" \
    --settings '{"disableAllHooks": true, "permissions": {"allow": ["WebSearch", "WebFetch", "Read"]}}' \
    --tools "WebSearch, WebFetch, Read" \
    --no-chrome \
    --disable-slash-commands \
    --print \
    --verbose \
    --output-format stream-json \
    --session-id "$SESSION_ID" \
    --system-prompt "You are a helpful AI assistant" \
    --model "haiku" \
    "My favorite color is blue. Just acknowledge this briefly." 2>&1 | grep -o '"text":"[^"]*"' | sed 's/"text":"//g' | sed 's/"//g' | tr -d '\n')

echo "Response 1: $RESPONSE1"
echo ""

# Second message - ask about the previous conversation
echo ">>> Message 2: Asking what my favorite color is"
RESPONSE2=$(claude \
    --setting-sources "" \
    --settings '{"disableAllHooks": true, "permissions": {"allow": ["WebSearch", "WebFetch", "Read"]}}' \
    --tools "WebSearch, WebFetch, Read" \
    --no-chrome \
    --disable-slash-commands \
    --print \
    --verbose \
    --output-format stream-json \
    --session-id "$SESSION_ID" \
    --system-prompt "You are a helpful AI assistant" \
    --model "haiku" \
    "What is my favorite color?" 2>&1 | grep -o '"text":"[^"]*"' | sed 's/"text":"//g' | sed 's/"//g' | tr -d '\n')

echo "Response 2: $RESPONSE2"
echo ""

# Check if the second response mentions blue
if echo "$RESPONSE2" | grep -qi "blue"; then
    echo "✅ SUCCESS: Claude remembered the conversation (mentioned 'blue')"
    exit 0
else
    echo "❌ FAILURE: Claude did NOT remember the conversation"
    echo "Expected mention of 'blue' in response"
    exit 1
fi
