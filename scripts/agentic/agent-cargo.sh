#!/usr/bin/env bash
# Run cargo from an AI agent (Claude Code, Codex, etc.) against a per-agent
# CARGO_TARGET_DIR so it does not contend on `target/.cargo-lock` with the
# always-on `./dev.sh` cargo-watch loop.
#
# Usage:
#   ./scripts/agentic/agent-cargo.sh test --lib context_picker
#   ./scripts/agentic/agent-cargo.sh check --lib
#   SCRIPT_KIT_AGENT_ID=claude-a ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui
#
# Opt-in sccache:
#   SCRIPT_KIT_AGENT_USE_SCCACHE=1 ./scripts/agentic/agent-cargo.sh check --lib

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

agent_id="${SCRIPT_KIT_AGENT_ID:-${USER:-agent}-${PPID:-$$}}"
# Sanitize: keep [a-zA-Z0-9._-], replace everything else with `-`.
agent_id="$(printf '%s' "$agent_id" | tr -c 'a-zA-Z0-9._-' '-')"

target_dir="${REPO_ROOT}/target-agent/${agent_id}"
mkdir -p "$target_dir"

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

echo "AGENT_CARGO target_dir=${CARGO_TARGET_DIR} rustc_wrapper=${rustc_wrapper_state} cargo $*" >&2

exec cargo "$@"
