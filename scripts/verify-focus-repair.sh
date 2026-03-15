#!/usr/bin/env bash
set -euo pipefail

LOG_FILE="${TMPDIR:-/tmp}/focus-repair.log"

if cargo test --test focus_repair_regressions --color never 2>&1 | tee "$LOG_FILE"; then
  printf '{"suite":"focus_repair_regressions","status":"ok","log":"%s"}\n' "$LOG_FILE"
else
  status=$?
  printf '{"suite":"focus_repair_regressions","status":"failed","exit_code":%d,"log":"%s"}\n' "$status" "$LOG_FILE"
  exit "$status"
fi
