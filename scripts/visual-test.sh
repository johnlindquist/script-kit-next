#!/bin/bash
# Visual Test Runner for Script Kit GPUI
# Launches app with test script, captures screenshot, terminates
# Usage: ./scripts/visual-test.sh <test-script.ts> [wait-seconds]

set -e

SCRIPT_PATH="$1"
WAIT_SECS="${2:-2}"  # Default 2 seconds to let window render
SCREENSHOT_DIR="$(dirname "$0")/../.test-screenshots"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
TEST_NAME=$(basename "$SCRIPT_PATH" .ts 2>/dev/null || echo "unknown")
SCREENSHOT_FILE="${SCREENSHOT_DIR}/${TEST_NAME}-${TIMESTAMP}.png"
LOG_FILE="${SCREENSHOT_DIR}/${TEST_NAME}-${TIMESTAMP}.log"

if [ -z "$SCRIPT_PATH" ]; then
    echo "Usage: $0 <test-script.ts> [wait-seconds]"
    echo ""
    echo "Examples:"
    echo "  $0 tests/smoke/test-editor-height.ts"
    echo "  $0 tests/smoke/test-term-height.ts 3"
    exit 1
fi

# Ensure screenshot directory exists
mkdir -p "$SCREENSHOT_DIR"

echo "=== Visual Test Runner ===" | tee "$LOG_FILE"
echo "Script: $SCRIPT_PATH" | tee -a "$LOG_FILE"
echo "Wait: ${WAIT_SECS}s" | tee -a "$LOG_FILE"
echo "Screenshot: $SCREENSHOT_FILE" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Get absolute path
cd "$(dirname "$0")/.."
PROJECT_DIR=$(pwd)
FULL_SCRIPT_PATH="$PROJECT_DIR/$SCRIPT_PATH"

if [ ! -f "$FULL_SCRIPT_PATH" ]; then
    echo "ERROR: Script not found: $FULL_SCRIPT_PATH" | tee -a "$LOG_FILE"
    exit 1
fi

# Build first
echo "Building..." | tee -a "$LOG_FILE"
cargo build 2>&1 | grep -v "^warning:" | tail -5 | tee -a "$LOG_FILE"

# Start app in background with the test script
echo "" | tee -a "$LOG_FILE"
echo "Launching app with test script..." | tee -a "$LOG_FILE"
echo "{\"type\": \"run\", \"path\": \"$FULL_SCRIPT_PATH\"}" | ./target/debug/script-kit-gpui 2>&1 >> "$LOG_FILE" &
APP_PID=$!
echo "App PID: $APP_PID" | tee -a "$LOG_FILE"

# Wait for window to render
echo "Waiting ${WAIT_SECS}s for render..." | tee -a "$LOG_FILE"
sleep "$WAIT_SECS"

# Take screenshot - try window-specific first, fall back to full screen
echo "Capturing screenshot..." | tee -a "$LOG_FILE"

# Try to capture the specific window by finding script-kit-gpui process
# AppleScript returns window ID for the process (not always reliable for CLI apps)
# Try both the debug executable name and the app bundle approach
WINDOW_ID=""

# Method 1: Try to find by process name containing "script-kit"
WINDOW_ID=$(osascript -e '
tell application "System Events"
    set targetProc to first process whose name contains "script-kit"
    if exists (window 1 of targetProc) then
        return id of window 1 of targetProc
    end if
end tell
' 2>/dev/null || echo "")

# Method 2: If that fails, try via CGWindowListCopyWindowInfo
if [ -z "$WINDOW_ID" ] || [ "$WINDOW_ID" = "" ]; then
    # Get window ID using swift script that queries CGWindowListCopyWindowInfo
    WINDOW_ID=$(swift -e '
import Cocoa
let windowList = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[String: Any]] ?? []
for window in windowList {
    if let ownerName = window[kCGWindowOwnerName as String] as? String,
       ownerName.contains("script-kit") {
        if let windowID = window[kCGWindowNumber as String] as? Int {
            print(windowID)
            break
        }
    }
}
' 2>/dev/null || echo "")
fi

if [ -n "$WINDOW_ID" ] && [ "$WINDOW_ID" != "" ]; then
    echo "Found window ID: $WINDOW_ID" | tee -a "$LOG_FILE"
    screencapture -l"$WINDOW_ID" -x -o "$SCREENSHOT_FILE" 2>>"$LOG_FILE" || screencapture -m "$SCREENSHOT_FILE"
else
    echo "Window ID not found, capturing main display..." | tee -a "$LOG_FILE"
    screencapture -m "$SCREENSHOT_FILE"
fi

# Kill the app gracefully
echo "Terminating app..." | tee -a "$LOG_FILE"
kill "$APP_PID" 2>/dev/null || true

# Wait a moment for cleanup
sleep 0.5

# Force kill if still running
if kill -0 "$APP_PID" 2>/dev/null; then
    echo "Force killing app..." | tee -a "$LOG_FILE"
    kill -9 "$APP_PID" 2>/dev/null || true
fi

echo "" | tee -a "$LOG_FILE"
echo "=== Test Complete ===" | tee -a "$LOG_FILE"

if [ -f "$SCREENSHOT_FILE" ]; then
    echo "Screenshot: $SCREENSHOT_FILE" | tee -a "$LOG_FILE"
    echo "Log: $LOG_FILE" | tee -a "$LOG_FILE"
    
    # Output just the filename for programmatic parsing
    echo ""
    echo "SCREENSHOT_PATH=$SCREENSHOT_FILE"
else
    echo "ERROR: Screenshot capture failed" | tee -a "$LOG_FILE"
    exit 1
fi
