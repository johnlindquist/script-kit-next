#!/usr/bin/env bash
# Replace the named dev app, then require it to own the main global shortcut.
# --check-hotkey SESSION is read-only and used before an unchanged-build skip.
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SESSION_SCRIPT="${PROJECT_ROOT}/scripts/agentic/session.sh"
WAIT_MS=10000
if [[ "${1:-}" == "--check-hotkey" ]]; then
  SESSION_NAME="${2:-${SCRIPT_KIT_DEV_SESSION_NAME:-dev-watch}}"
  WAIT_MS=0
  RESULT="$(bash "$SESSION_SCRIPT" status "$SESSION_NAME")"
else
  SESSION_NAME="${1:-${SCRIPT_KIT_DEV_SESSION_NAME:-dev-watch}}"
  echo "DEV_SESSION_RELAUNCH session=${SESSION_NAME} phase=stop-old" >&2
  if ! bash "$SESSION_SCRIPT" stop "$SESSION_NAME" >/dev/null; then
    echo "DEV_SESSION_RELAUNCH refused: previous exact session teardown not confirmed" >&2
    exit 78
  fi
  echo "DEV_SESSION_RELAUNCH session=${SESSION_NAME} phase=start-new" >&2
  set +e
  RESULT="$(bash "$SESSION_SCRIPT" start "$SESSION_NAME")"
  START_STATUS=$?
  set -e
  if [[ "$START_STATUS" != 0 ]]; then
    printf '%s\n' "$RESULT"
    exit "$START_STATUS"
  fi
fi

RESULT_JSON="$RESULT" EXPECTED_SESSION="$SESSION_NAME" WAIT_MS="$WAIT_MS" python3 -B - <<'PY'
import json
import os
from pathlib import Path
import re
import sys
import time

try:
    data = json.loads(os.environ["RESULT_JSON"])
except (ValueError, TypeError):
    print(json.dumps({"status": "error", "ready": False, "error": {"code": "dev_session_invalid_receipt"}}))
    sys.exit(78)

def fail(code):
    data.update(status="error", ready=False, mainHotkeyRegistered=False, error={"code": code})
    print(json.dumps(data))
    print(f"DEV_SESSION_RELAUNCH refused: {code}; global shortcut delivery is not ready", file=sys.stderr)
    sys.exit(78)

pid = data.get("pid")
generation = data.get("sessionGeneration")
if (data.get("status") != "ok" or data.get("session") != os.environ["EXPECTED_SESSION"]
    or not (data.get("ready") is True or data.get("healthy") is True)
    or type(pid) is not int or pid <= 0 or not isinstance(generation, str) or not generation
    or not isinstance(data.get("log"), str)
    or (os.environ.get("SCRIPT_KIT_GPUI_BINARY") and data.get("binary") != os.environ["SCRIPT_KIT_GPUI_BINARY"])):
    fail("dev_session_not_ready")
log = Path(data["log"])
deadline = time.monotonic() + int(os.environ["WAIT_MS"]) / 1000
position = 0
while True:
    try:
        os.kill(pid, 0)
        if (log.parent / "generation").read_text().strip() != generation or (log.parent / "pid").read_text().strip() != str(pid):
            fail("dev_session_identity_changed")
        with log.open() as stream:
            stream.seek(position)
            while True:
                position = stream.tell()
                line = stream.readline()
                if not line or not line.endswith("\n"):
                    break
                if "BUILTIN_HOTKEY_REGISTER_FAILED action=Main launcher" in line:
                    fail("dev_main_hotkey_registration_failed")
                match = re.search(r"BUILTIN_HOTKEY_REGISTERED action=Main launcher display=(.*?) id=(\d+)", line)
                if match:
                    data.update(mainHotkeyRegistered=True, mainHotkeyDisplay=match.group(1), mainHotkeyId=int(match.group(2)))
                    print(json.dumps(data))
                    print(f"DEV_SESSION_RELAUNCH session={data['session']} mainHotkeyRegistered=true display={match.group(1)}", file=sys.stderr)
                    sys.exit(0)
    except OSError:
        fail("dev_session_owner_unavailable")
    if time.monotonic() >= deadline:
        fail("dev_main_hotkey_registration_unverified")
    time.sleep(0.05)
PY
