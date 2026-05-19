#!/usr/bin/env bash
# Run cargo from an AI agent (Claude Code, Codex, etc.) against a bounded
# agent-owned CARGO_TARGET_DIR so it does not contend on `target/.cargo-lock`
# with the always-on `./dev.sh` cargo-watch loop.
#
# Usage:
#   ./scripts/agentic/agent-cargo.sh test --lib context_picker
#   ./scripts/agentic/agent-cargo.sh check --lib
#   SCRIPT_KIT_CARGO_TARGET_POOL=agent-debug ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui
#   SCRIPT_KIT_AGENT_TARGET_MODE=exclusive SCRIPT_KIT_AGENT_ID=claude-a ./scripts/agentic/agent-cargo.sh check --lib
#
# Opt-in sccache:
#   SCRIPT_KIT_AGENT_USE_SCCACHE=1 ./scripts/agentic/agent-cargo.sh check --lib

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

sanitize_id() {
  printf '%s' "$1" | tr -c 'a-zA-Z0-9._-' '-'
}

agent_id="$(sanitize_id "${SCRIPT_KIT_AGENT_ID:-${USER:-agent}-${PPID:-$$}}")"
target_mode="${SCRIPT_KIT_AGENT_TARGET_MODE:-pool}"
pool="$(sanitize_id "${SCRIPT_KIT_CARGO_TARGET_POOL:-agent-debug}")"

case "$target_mode" in
  pool)
    target_dir="${REPO_ROOT}/target-agent/pools/${pool}"
    lock_name="pool-${pool}"
    ;;
  exclusive)
    target_dir="${REPO_ROOT}/target-agent/agents/${agent_id}"
    lock_name="agent-${agent_id}"
    ;;
  *)
    echo "AGENT_CARGO error: SCRIPT_KIT_AGENT_TARGET_MODE must be pool or exclusive; got ${target_mode}" >&2
    exit 2
    ;;
esac

lock_root="${REPO_ROOT}/target-agent/.locks"
lock_dir="${lock_root}/${lock_name}.lock"
mkdir -p "$target_dir" "$lock_root"

export CARGO_TARGET_DIR="$target_dir"

rustc_wrapper_state="none"
if [[ "${SCRIPT_KIT_AGENT_USE_SCCACHE:-0}" == "1" ]]; then
  if command -v sccache >/dev/null 2>&1; then
    export RUSTC_WRAPPER="sccache"
    rustc_wrapper_state="sccache"
  else
    echo "AGENT_CARGO warning: SCRIPT_KIT_AGENT_USE_SCCACHE=1 but sccache not on PATH; continuing without it" >&2
  fi
fi

acquire_lock() {
  local timeout="${SCRIPT_KIT_AGENT_LOCK_TIMEOUT_SEC:-600}"
  local start elapsed old_pid
  start="$(date +%s)"

  while ! mkdir "$lock_dir" 2>/dev/null; do
    if [[ -f "${lock_dir}/pid" ]]; then
      old_pid="$(cat "${lock_dir}/pid" 2>/dev/null || true)"
      if [[ -n "$old_pid" ]] && ! kill -0 "$old_pid" 2>/dev/null; then
        echo "AGENT_CARGO stale_lock pid=${old_pid} lock=${lock_dir}; removing" >&2
        rm -rf "$lock_dir"
        continue
      fi
    fi

    elapsed=$(( $(date +%s) - start ))
    if [[ "$elapsed" -ge "$timeout" ]]; then
      echo "AGENT_CARGO error: timed out waiting for ${lock_name} after ${timeout}s" >&2
      exit 70
    fi
    echo "AGENT_CARGO waiting mode=${target_mode} pool=${pool} elapsed=${elapsed}s lock=${lock_dir}" >&2
    sleep 5
  done

  {
    echo "$$" > "${lock_dir}/pid"
    printf '%s\n' "$agent_id" > "${lock_dir}/owner"
    printf '%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "${lock_dir}/started_at"
    printf '%q ' cargo "$@" > "${lock_dir}/command"
    printf '\n' >> "${lock_dir}/command"
  } 2>/dev/null || true
}

release_lock() {
  rm -rf "$lock_dir" 2>/dev/null || true
}

acquire_lock "$@"
trap release_lock EXIT INT TERM

echo "AGENT_CARGO mode=${target_mode} pool=${pool} target_dir=${CARGO_TARGET_DIR} lock=${lock_name} rustc_wrapper=${rustc_wrapper_state} cargo $*" >&2

set +e
cargo "$@"
status=$?
set -e
release_lock
exit "$status"
