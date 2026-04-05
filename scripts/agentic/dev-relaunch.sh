#!/usr/bin/env bash
# scripts/agentic/dev-relaunch.sh — Stop stale session and start a fresh one after a successful build.
# Called by dev.sh after `cargo build --quiet` succeeds.
#
# Usage: dev-relaunch.sh [SESSION_NAME]
# Output: session JSON on stdout, DEV_SESSION_RELAUNCH status lines on stderr.

set -euo pipefail

SESSION_NAME="${1:-${SCRIPT_KIT_DEV_SESSION_NAME:-dev-watch}}"
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SESSION_SCRIPT="${PROJECT_ROOT}/scripts/agentic/session.sh"

echo "DEV_SESSION_RELAUNCH session=${SESSION_NAME} phase=stop-old" >&2
bash "${SESSION_SCRIPT}" stop "${SESSION_NAME}" >/dev/null 2>&1 || true

echo "DEV_SESSION_RELAUNCH session=${SESSION_NAME} phase=start-new" >&2
RESULT="$(bash "${SESSION_SCRIPT}" start "${SESSION_NAME}")"
printf '%s\n' "${RESULT}"

RESULT_JSON="${RESULT}" python3 - <<'PY'
import json
import os
import sys
try:
    data = json.loads(os.environ["RESULT_JSON"])
except Exception as exc:
    print(f"DEV_SESSION_RELAUNCH phase=parse-json status=error error={exc}", file=sys.stderr)
    raise SystemExit(0)
print(
    "DEV_SESSION_RELAUNCH "
    f"session={data.get('session')} "
    f"status={data.get('status')} "
    f"ready={data.get('ready')} "
    f"readyMarker={data.get('readyMarker')} "
    f"readyWaitMs={data.get('readyWaitMs')}",
    file=sys.stderr,
)
PY
