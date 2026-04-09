#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SESSION_NAME="${1:-main-menu-smoke}"
LABEL="${2:-main-menu-smoke}"
SCREEN_DIR="${PROJECT_ROOT}/test-screenshots"
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
OUT_PATH="${SCREEN_DIR}/${LABEL}-${TIMESTAMP}.png"
SESSION_STOPPED=0

stop_session() {
  if [[ "$SESSION_STOPPED" -eq 0 ]]; then
    bash "${PROJECT_ROOT}/scripts/agentic/session.sh" stop "$SESSION_NAME" >/dev/null 2>&1 || true
    SESSION_STOPPED=1
  fi
}

trap stop_session EXIT

mkdir -p "$SCREEN_DIR"
cd "$PROJECT_ROOT"

BUILD_LOG="$(mktemp /tmp/script-kit-main-menu-smoke-build.XXXXXX.log)"
cargo build >"$BUILD_LOG" 2>&1
tail -5 "$BUILD_LOG"
rm -f "$BUILD_LOG"

SESSION_JSON="$(bash scripts/agentic/session.sh start "$SESSION_NAME" 2>/dev/null)"
if [[ "$SESSION_JSON" != *'"status":"ok"'* ]]; then
  printf '%s\n' "$SESSION_JSON" >&2
  exit 1
fi

bash scripts/agentic/session.sh send "$SESSION_NAME" '{"type":"show"}' >/dev/null
sleep 0.3

bun scripts/agentic/verify-shot.ts \
  --session "$SESSION_NAME" \
  --label "$LABEL" \
  --out "$OUT_PATH" \
  --skip-state

stop_session
bash scripts/agentic/session.sh status "$SESSION_NAME"
printf 'SMOKE_SCREENSHOT=%s\n' "$OUT_PATH"
