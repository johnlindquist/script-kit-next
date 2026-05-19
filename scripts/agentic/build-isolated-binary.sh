#!/usr/bin/env bash
# Timeboxed agent-cargo build for isolated DevTools (never block silently).
#
# Usage:
#   build-isolated-binary.sh [TIMEOUT_SEC]
#
# stdout: final JSON envelope (when DEVTOOLS_SESSION_JSON=1 or --json)
# stderr: progress heartbeats
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=devtools-session-lib.sh
source "${SCRIPT_DIR}/devtools-session-lib.sh"

cd "$DEVTOOLS_SESSION_REPO_ROOT"

TIMEOUT_SEC="${1:-120}"
JSON_OUT="${DEVTOOLS_SESSION_JSON:-0}"
if [[ "${1:-}" == "--json" ]]; then
  JSON_OUT=1
  TIMEOUT_SEC="${2:-120}"
fi

export SCRIPT_KIT_AGENT_ID="${SCRIPT_KIT_AGENT_ID:-dt-agent-build}"
LOG="/tmp/sk-isolated-build-${SCRIPT_KIT_AGENT_ID}.log"
: > "$LOG"

emit_build_json() {
  local status="$1"
  local phase="$2"
  local code="${3:-}"
  local message="${4:-}"
  local elapsed="${5:-0}"
  if [[ "$JSON_OUT" -eq 1 ]]; then
    if [[ "$status" == "ok" ]]; then
      printf '{"schemaVersion":1,"tool":"build-isolated-binary","status":"ok","phase":"%s","agentId":"%s","targetDir":"target-agent/%s","promotedTo":"target/debug/script-kit-gpui","elapsedSec":%s,"log":"%s"}\n' \
        "$phase" "$SCRIPT_KIT_AGENT_ID" "$SCRIPT_KIT_AGENT_ID" "$elapsed" "$LOG"
    else
      printf '{"schemaVersion":1,"tool":"build-isolated-binary","status":"error","phase":"%s","agentId":"%s","elapsedSec":%s,"log":"%s","error":{"code":"%s","message":"%s"}}\n' \
        "$phase" "$SCRIPT_KIT_AGENT_ID" "$elapsed" "$LOG" "$code" "$(json_escape "$message")"
    fi
  fi
}

if detect_dev_sh; then
  echo "[build-isolated] fail: ./dev.sh running — will not promote over target/debug" >&2
  emit_build_json error build dev_sh_running_build_conflict "./dev.sh is running; stop dev.sh before promoting target/debug." "$SECONDS"
  exit 11
fi

echo "[build-isolated] agent_id=${SCRIPT_KIT_AGENT_ID} timeout=${TIMEOUT_SEC}s log=${LOG}" >&2

(
  ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui --message-format=short 2>&1 | tee -a "$LOG"
) &
build_pid=$!

start_epoch=$(date +%s)
while kill -0 "$build_pid" 2>/dev/null; do
  elapsed=$(( $(date +%s) - start_epoch ))
  if [[ "$elapsed" -ge "$TIMEOUT_SEC" ]]; then
    echo "[build-isolated] timeout after ${TIMEOUT_SEC}s — killing cargo (pid ${build_pid})" >&2
    kill "$build_pid" 2>/dev/null || true
    sleep 2
    kill -9 "$build_pid" 2>/dev/null || true
    tail -n 30 "$LOG" >&2 || true
    emit_build_json error build build_timeout "cargo build exceeded ${TIMEOUT_SEC}s" "$elapsed"
    exit 30
  fi
  if [[ -f "$LOG" ]]; then
    lines="$(wc -l < "$LOG" | tr -d '[:space:]')"
  else
    lines=0
  fi
  echo "[build-isolated] ${elapsed}s elapsed, log_lines=${lines}" >&2
  sleep 5
done

wait "$build_pid"
status=$?
elapsed=$(( $(date +%s) - start_epoch ))

if [[ "$status" -ne 0 ]]; then
  echo "[build-isolated] cargo failed (exit ${status})" >&2
  tail -n 40 "$LOG" >&2 || true
  emit_build_json error build build_failed "cargo build failed with exit ${status}" "$elapsed"
  exit 31
fi

SRC="${DEVTOOLS_SESSION_REPO_ROOT}/target-agent/${SCRIPT_KIT_AGENT_ID}/debug/script-kit-gpui"
if [[ ! -f "$SRC" ]]; then
  echo "[build-isolated] fail: built binary missing at ${SRC}" >&2
  emit_build_json error promote promote_failed "built binary missing at ${SRC}" "$elapsed"
  exit 32
fi

cp -f "$SRC" "$DEVTOOLS_SESSION_BINARY"
echo "[build-isolated] promoted → target/debug/script-kit-gpui" >&2
emit_build_json ok promote "" "" "$elapsed"
exit 0
