#!/usr/bin/env bash
# Smoke/stress helper for Parallel RPC v1.
# Usage:
#   scripts/agentic/verify-parallel-rpc.sh [SESSION_NAME]
#
# Requires a running session (scripts/agentic/session.sh start) and a built binary.

set -euo pipefail

SESSION_NAME="${1:-parallel-rpc-smoke}"
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SESSION_SH="${PROJECT_ROOT}/scripts/agentic/session.sh"

if ! "${SESSION_SH}" status "${SESSION_NAME}" | grep -q '"status":"ok"'; then
  echo "Starting session ${SESSION_NAME}..." >&2
  "${SESSION_SH}" start "${SESSION_NAME}" >/dev/null
fi

payload() {
  local id="$1"
  printf '{"type":"getState","requestId":"%s","summaryOnly":true}' "$id"
}

run_one() {
  local id="$1"
  "${SESSION_SH}" rpc "${SESSION_NAME}" "$(payload "$id")" --expect stateResult --timeout 8000
}

echo "Sequential baseline (5x getState)..." >&2
for index in 1 2 3 4 5; do
  run_one "seq-${index}" | grep -q '"status":"ok"' || {
    echo "Sequential RPC ${index} failed" >&2
    exit 1
  }
done

echo "Parallel burst (5x getState)..." >&2
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

for index in 1 2 3 4 5; do
  (
    run_one "par-${index}" > "${tmpdir}/par-${index}.json" 2>"${tmpdir}/par-${index}.err" || true
  ) &
done
wait

failures=0
for index in 1 2 3 4 5; do
  if ! grep -q '"status":"ok"' "${tmpdir}/par-${index}.json" 2>/dev/null; then
    echo "Parallel RPC ${index} failed:" >&2
    cat "${tmpdir}/par-${index}.json" "${tmpdir}/par-${index}.err" >&2 || true
    failures=$((failures + 1))
  fi
done

if [ "$failures" -gt 0 ]; then
  echo "${failures}/5 parallel RPCs failed" >&2
  exit 1
fi

echo "OK: 5 sequential + 5 parallel getState RPCs succeeded for session ${SESSION_NAME}" >&2
