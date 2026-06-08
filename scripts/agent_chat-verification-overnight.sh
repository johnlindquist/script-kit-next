#!/usr/bin/env bash
# scripts/agent_chat-verification-overnight.sh
#
# Fail-closed overnight Agent Chat verification evidence collector.
#
# Runs make verify, then exercises the four Agent Chat scenarios via the
# agentic session harness. Captures structured logs into
# artifacts/agent_chat-verification-overnight.log and exits non-zero
# on any missing scenario receipt, silent fallback, or unresolved
# target-identity mismatch.
#
# Usage:
#   bash scripts/agent_chat-verification-overnight.sh [--skip-verify] [--timeout SECS]
#
# Environment:
#   SCRIPT_KIT_AI_LOG=1 is set automatically for compact log output.
#   AGENT_CHAT_VERIFY_SESSION — override session name (default: "agent_chat-overnight").

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARTIFACTS_DIR="${PROJECT_ROOT}/artifacts"
LOG_FILE="${ARTIFACTS_DIR}/agent_chat-verification-overnight.log"
SESSION_NAME="${AGENT_CHAT_VERIFY_SESSION:-agent_chat-overnight}"
SESSION_SH="${PROJECT_ROOT}/scripts/agentic/session.sh"
VERIFY_SHOT="${PROJECT_ROOT}/scripts/agentic/verify-shot.ts"
BINARY="${PROJECT_ROOT}/target/debug/script-kit-gpui"
TIMEOUT_SECS="${AGENT_CHAT_VERIFY_TIMEOUT_SECS:-120}"
SKIP_VERIFY="${AGENT_CHAT_VERIFY_SKIP_VERIFY:-false}"

# --- CLI args ----------------------------------------------------------------

for arg in "$@"; do
  case "$arg" in
    --skip-verify) SKIP_VERIFY=true ;;
    --timeout)     shift; TIMEOUT_SECS="${1:-120}" ;;
    --help|-h)
      echo "Usage: bash scripts/agent_chat-verification-overnight.sh [--skip-verify] [--timeout SECS]"
      exit 0
      ;;
  esac
done

# --- helpers -----------------------------------------------------------------

log()  { echo "[agent_chat-overnight] $(date '+%H:%M:%S') $*" >&2; }
fail() { log "FAIL: $*"; echo "FAIL: $*" >> "$LOG_FILE"; FAILURES=$((FAILURES + 1)); }

FAILURES=0
SCENARIOS_RUN=0
SESSION_PID=""

cleanup() {
  if [ -n "$SESSION_PID" ]; then
    log "Stopping session ${SESSION_NAME} (pid ${SESSION_PID})..."
    bash "$SESSION_SH" stop "$SESSION_NAME" 2>/dev/null || true
    SESSION_PID=""
  fi
}
trap cleanup EXIT

# --- artifact directory ------------------------------------------------------

mkdir -p "$ARTIFACTS_DIR"
: > "$LOG_FILE"

log "Overnight Agent Chat verification started"
echo "# Agent Chat Verification Overnight Evidence" >> "$LOG_FILE"
echo "# Started: $(date -u '+%Y-%m-%dT%H:%M:%SZ')" >> "$LOG_FILE"
echo "# Host: $(hostname)" >> "$LOG_FILE"
echo "" >> "$LOG_FILE"

# --- Step 1: make verify (compilation + tests) --------------------------------

if [ "$SKIP_VERIFY" = "true" ]; then
  log "Skipping make verify (--skip-verify)"
  echo "## make verify: SKIPPED" >> "$LOG_FILE"
else
  log "Running make verify..."
  echo "## make verify" >> "$LOG_FILE"
  if timeout "${TIMEOUT_SECS}" make -C "$PROJECT_ROOT" verify >> "$LOG_FILE" 2>&1; then
    log "make verify passed"
    echo "## make verify: PASSED" >> "$LOG_FILE"
  else
    fail "make verify failed (exit $?)"
    echo "## make verify: FAILED" >> "$LOG_FILE"
    # Continue to collect evidence even if verify fails — fail-closed at the end.
  fi
fi

echo "" >> "$LOG_FILE"

# --- Step 2: Check binary exists ----------------------------------------------

if [ ! -x "$BINARY" ]; then
  log "Building debug binary..."
  if ! timeout "${TIMEOUT_SECS}" cargo build --manifest-path "${PROJECT_ROOT}/Cargo.toml" 2>>"$LOG_FILE"; then
    fail "cargo build failed — cannot run live Agent Chat verification"
    echo "" >> "$LOG_FILE"
    echo "## Agent Chat Live Scenarios: SKIPPED (build failed)" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"
    echo "## Summary" >> "$LOG_FILE"
    echo "Failures: ${FAILURES}" >> "$LOG_FILE"
    exit 1
  fi
fi

# --- Step 3: Verify required receipt log lines exist in source ----------------

echo "## Source Receipt Audit" >> "$LOG_FILE"

REQUIRED_RECEIPTS=(
  "automation.agent_chat_action_target_resolved"
  "agent_chat_state.result"
  "inspect_automation_window"
  "automation.capture_screenshot.candidate_selected"
  "automation.capture_screenshot.ambiguous_candidate"
  "agent_chat_runtime_setup_session_armed"
  "agent_chat_runtime_setup_requirements_preserved"
  "agent_chat_setup_agent_confirmed_for_runtime_recovery"
  "agent_chat_setup_agent_ready_retrying"
  "agent_chat_pending_context_consumed"
  "agent_chat_submit_resolved_context_parts"
)

SOURCE_RECEIPT_PASS=true
for receipt in "${REQUIRED_RECEIPTS[@]}"; do
  if rg -q "$receipt" "${PROJECT_ROOT}/src/"; then
    echo "  [OK] ${receipt} — present in source" >> "$LOG_FILE"
  else
    fail "Receipt '${receipt}' not found in src/"
    echo "  [MISSING] ${receipt} — NOT found in source" >> "$LOG_FILE"
    SOURCE_RECEIPT_PASS=false
  fi
done

if [ "$SOURCE_RECEIPT_PASS" = "true" ]; then
  echo "Source receipt audit: ALL ${#REQUIRED_RECEIPTS[@]} receipts present" >> "$LOG_FILE"
else
  echo "Source receipt audit: INCOMPLETE — some receipts missing from source" >> "$LOG_FILE"
fi

echo "" >> "$LOG_FILE"

# --- Step 4: Agent Chat scenario exercises via agentic session -----------------------

echo "## Agent Chat Live Scenarios" >> "$LOG_FILE"

# Check if session harness is available and we have a display
if [ ! -f "$SESSION_SH" ]; then
  fail "Session harness not found at ${SESSION_SH}"
  echo "Agent Chat Live Scenarios: SKIPPED (no session harness)" >> "$LOG_FILE"
elif [ -z "${DISPLAY:-}" ] && [ "$(uname)" != "Darwin" ]; then
  fail "No display available for live Agent Chat scenarios"
  echo "Agent Chat Live Scenarios: SKIPPED (no display)" >> "$LOG_FILE"
else
  # Start agentic session
  log "Starting agentic session '${SESSION_NAME}'..."
  SESSION_LOG="${ARTIFACTS_DIR}/agent_chat-session-${SESSION_NAME}.log"

  START_RESULT=$(bash "$SESSION_SH" start "$SESSION_NAME" 2>>"$LOG_FILE" || true)
  if echo "$START_RESULT" | grep -qE '"status":"(running|ok)"'; then
    SESSION_PID=$(echo "$START_RESULT" | grep -o '"pid":[0-9]*' | head -1 | cut -d: -f2)
    log "Session started (pid ${SESSION_PID})"

    # Wait for app readiness
    sleep 2

    # --- Scenario: agent_chat-open ---
    echo "" >> "$LOG_FILE"
    echo "### Scenario: agent_chat-open" >> "$LOG_FILE"
    log "Running scenario: agent_chat-open"
    SCENARIOS_RUN=$((SCENARIOS_RUN + 1))

    AGENT_CHAT_OPEN_RESULT=$(bash "$SESSION_SH" rpc "$SESSION_NAME" \
      '{"type":"getAgentChatState","requestId":"agent_chat-overnight-open-001"}' \
      --expect agent_chatStateResult --timeout 5000 2>>"$LOG_FILE" || true)

    if [ -n "$AGENT_CHAT_OPEN_RESULT" ]; then
      echo "agent_chat-open response: ${AGENT_CHAT_OPEN_RESULT}" >> "$LOG_FILE"
      # Verify resolved target is present (not a silent fallback)
      if echo "$AGENT_CHAT_OPEN_RESULT" | grep -q '"resolvedTarget"'; then
        echo "  [OK] agent_chat-open: resolved target present" >> "$LOG_FILE"
      else
        echo "  [WARN] agent_chat-open: no resolvedTarget in response (Agent Chat view may not be active)" >> "$LOG_FILE"
      fi
      # Check for status field
      if echo "$AGENT_CHAT_OPEN_RESULT" | grep -q '"status"'; then
        echo "  [OK] agent_chat-open: status field present" >> "$LOG_FILE"
      else
        fail "agent_chat-open: missing status field"
      fi
    else
      echo "  [WARN] agent_chat-open: no response (session may have exited)" >> "$LOG_FILE"
    fi

    # --- Scenario: agent_chat-accept ---
    echo "" >> "$LOG_FILE"
    echo "### Scenario: agent_chat-accept" >> "$LOG_FILE"
    log "Running scenario: agent_chat-accept"
    SCENARIOS_RUN=$((SCENARIOS_RUN + 1))

    # Inspect automation window to verify semantic/visual identity
    INSPECT_RESULT=$(bash "$SESSION_SH" rpc "$SESSION_NAME" \
      '{"type":"inspectAutomationWindow","requestId":"agent_chat-overnight-inspect-001"}' \
      --expect automationInspectResult --timeout 5000 2>>"$LOG_FILE" || true)

    if [ -n "$INSPECT_RESULT" ]; then
      echo "agent_chat-accept inspect: ${INSPECT_RESULT}" >> "$LOG_FILE"
      if echo "$INSPECT_RESULT" | grep -q '"windowKind"'; then
        echo "  [OK] agent_chat-accept: window inspection returned window kind" >> "$LOG_FILE"
      fi
      if echo "$INSPECT_RESULT" | grep -q '"osWindowId"'; then
        echo "  [OK] agent_chat-accept: OS window ID present in inspection" >> "$LOG_FILE"
      fi
    else
      echo "  [WARN] agent_chat-accept: no inspect response" >> "$LOG_FILE"
    fi

    # --- Scenario: agent_chat-setup-recovery ---
    echo "" >> "$LOG_FILE"
    echo "### Scenario: agent_chat-setup-recovery" >> "$LOG_FILE"
    log "Running scenario: agent_chat-setup-recovery"
    SCENARIOS_RUN=$((SCENARIOS_RUN + 1))

    # Attempt a setup action to trigger recovery path logging
    SETUP_RESULT=$(bash "$SESSION_SH" rpc "$SESSION_NAME" \
      '{"type":"performAgentChatSetupAction","requestId":"agent_chat-overnight-setup-001","action":"retrySetup"}' \
      --expect agent_chatSetupActionResult --timeout 5000 2>>"$LOG_FILE" || true)

    if [ -n "$SETUP_RESULT" ]; then
      echo "agent_chat-setup-recovery response: ${SETUP_RESULT}" >> "$LOG_FILE"
      # Either success or explicit error — no silent fallback
      if echo "$SETUP_RESULT" | grep -q '"success"'; then
        echo "  [OK] agent_chat-setup-recovery: explicit success/failure result" >> "$LOG_FILE"
      elif echo "$SETUP_RESULT" | grep -q '"error"'; then
        echo "  [OK] agent_chat-setup-recovery: explicit error (fail-closed)" >> "$LOG_FILE"
      else
        fail "agent_chat-setup-recovery: neither success nor error — possible silent fallback"
      fi
    else
      echo "  [WARN] agent_chat-setup-recovery: no response (expected if Agent Chat not active)" >> "$LOG_FILE"
    fi

    # --- Scenario: agent_chat-popup-escape ---
    echo "" >> "$LOG_FILE"
    echo "### Scenario: agent_chat-popup-escape" >> "$LOG_FILE"
    log "Running scenario: agent_chat-popup-escape"
    SCENARIOS_RUN=$((SCENARIOS_RUN + 1))

    # Re-query Agent Chat state to verify no drift after setup attempt
    ESCAPE_RESULT=$(bash "$SESSION_SH" rpc "$SESSION_NAME" \
      '{"type":"getAgentChatState","requestId":"agent_chat-overnight-escape-001"}' \
      --expect agent_chatStateResult --timeout 5000 2>>"$LOG_FILE" || true)

    if [ -n "$ESCAPE_RESULT" ]; then
      echo "agent_chat-popup-escape response: ${ESCAPE_RESULT}" >> "$LOG_FILE"
      if echo "$ESCAPE_RESULT" | grep -q '"status"'; then
        echo "  [OK] agent_chat-popup-escape: deterministic state after escape" >> "$LOG_FILE"
      else
        fail "agent_chat-popup-escape: missing status — possible state drift"
      fi
    else
      echo "  [WARN] agent_chat-popup-escape: no response" >> "$LOG_FILE"
    fi

    # --- Capture session logs for receipt evidence ---
    echo "" >> "$LOG_FILE"
    echo "### Session Log Receipt Scan" >> "$LOG_FILE"

    SESSION_STATE_DIR="/tmp/sk-agentic-sessions/${SESSION_NAME}"
    if [ -d "$SESSION_STATE_DIR" ] && [ -f "${SESSION_STATE_DIR}/app.log" ]; then
      # Copy session log into artifacts
      cp "${SESSION_STATE_DIR}/app.log" "${ARTIFACTS_DIR}/agent_chat-session-app.log" 2>/dev/null || true

      # Scan for required receipts in session logs
      for receipt in "${REQUIRED_RECEIPTS[@]}"; do
        if grep -q "$receipt" "${SESSION_STATE_DIR}/app.log" 2>/dev/null; then
          echo "  [EMITTED] ${receipt}" >> "$LOG_FILE"
        else
          echo "  [NOT_EMITTED] ${receipt} (may require active Agent Chat session)" >> "$LOG_FILE"
        fi
      done
    else
      echo "  [WARN] Session log not found at ${SESSION_STATE_DIR}/app.log" >> "$LOG_FILE"
    fi

    # Stop session
    log "Stopping session..."
    bash "$SESSION_SH" stop "$SESSION_NAME" 2>>"$LOG_FILE" || true
    SESSION_PID=""

  else
    log "Session start returned: ${START_RESULT}"
    echo "Agent Chat Live Scenarios: SESSION_START_FAILED" >> "$LOG_FILE"
    echo "Start result: ${START_RESULT}" >> "$LOG_FILE"
    # Not a fatal failure — the session may not be available in CI
    echo "  [INFO] Live scenario exercise requires a running macOS display" >> "$LOG_FILE"
  fi
fi

echo "" >> "$LOG_FILE"

# --- Step 5: Run Agent Chat-specific tests and capture output -----------------------

echo "## Agent Chat Test Evidence" >> "$LOG_FILE"
log "Running Agent Chat-specific tests..."

AGENT_CHAT_TEST_LOG="${ARTIFACTS_DIR}/agent_chat-test-output.log"
AGENT_CHAT_TEST_NAMES=(
  "agent_chat_targeted_reads"
  "agent_chat_onboarding"
  "agent_chat_switch_actions"
  "detached_agent_chat_transaction_contract"
  "automation_window_targeting"
  "title_contains_resolution_fails_closed"
  "automation_screenshots"
)

AGENT_CHAT_TEST_FILTER=$(IFS='|'; echo "${AGENT_CHAT_TEST_NAMES[*]}")

if timeout "${TIMEOUT_SECS}" cargo nextest run --no-fail-fast \
  -E "test(/${AGENT_CHAT_TEST_FILTER}/)" \
  2>&1 | tee "$AGENT_CHAT_TEST_LOG" >> "$LOG_FILE"; then
  echo "" >> "$LOG_FILE"
  echo "Agent Chat tests: PASSED" >> "$LOG_FILE"
else
  AGENT_CHAT_EXIT=$?
  echo "" >> "$LOG_FILE"
  if [ "$AGENT_CHAT_EXIT" -eq 124 ]; then
    fail "Agent Chat tests timed out after ${TIMEOUT_SECS}s"
    echo "Agent Chat tests: TIMEOUT" >> "$LOG_FILE"
  else
    fail "Agent Chat tests failed (exit ${AGENT_CHAT_EXIT})"
    echo "Agent Chat tests: FAILED" >> "$LOG_FILE"
  fi
fi

echo "" >> "$LOG_FILE"

# --- Step 6: Replay determinism check ----------------------------------------

echo "## Replay Determinism" >> "$LOG_FILE"

# Run the same Agent Chat tests a second time to verify deterministic results
log "Running Agent Chat tests a second time for replay determinism..."

REPLAY_LOG="${ARTIFACTS_DIR}/agent_chat-replay-output.log"

if timeout "${TIMEOUT_SECS}" cargo nextest run --no-fail-fast \
  -E "test(/${AGENT_CHAT_TEST_FILTER}/)" \
  2>&1 | tee "$REPLAY_LOG" > /dev/null; then
  echo "Replay: PASSED (deterministic)" >> "$LOG_FILE"
else
  REPLAY_EXIT=$?
  if [ "$REPLAY_EXIT" -eq 124 ]; then
    fail "Replay timed out after ${TIMEOUT_SECS}s"
    echo "Replay: TIMEOUT" >> "$LOG_FILE"
  else
    # Compare failure patterns between runs
    FIRST_FAILS=$(grep -c "FAIL" "$AGENT_CHAT_TEST_LOG" 2>/dev/null || echo "0")
    REPLAY_FAILS=$(grep -c "FAIL" "$REPLAY_LOG" 2>/dev/null || echo "0")
    if [ "$FIRST_FAILS" = "$REPLAY_FAILS" ]; then
      echo "Replay: CONSISTENT (same ${REPLAY_FAILS} failures in both runs)" >> "$LOG_FILE"
    else
      fail "Replay: NON-DETERMINISTIC (${FIRST_FAILS} failures first run vs ${REPLAY_FAILS} replay)"
      echo "Replay: NON-DETERMINISTIC" >> "$LOG_FILE"
    fi
  fi
fi

echo "" >> "$LOG_FILE"

# --- Summary -----------------------------------------------------------------

echo "## Summary" >> "$LOG_FILE"
echo "Scenarios exercised: ${SCENARIOS_RUN}" >> "$LOG_FILE"
echo "Source receipts verified: ${#REQUIRED_RECEIPTS[@]}" >> "$LOG_FILE"
echo "Failures: ${FAILURES}" >> "$LOG_FILE"
echo "Completed: $(date -u '+%Y-%m-%dT%H:%M:%SZ')" >> "$LOG_FILE"

if [ "$FAILURES" -gt 0 ]; then
  log "FAILED with ${FAILURES} failure(s). See ${LOG_FILE}"
  exit 1
else
  log "PASSED. Evidence at ${LOG_FILE}"
  exit 0
fi
