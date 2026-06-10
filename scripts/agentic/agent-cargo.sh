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
# Parallel tasks that need a stable binary should NOT mint a new pool. Build in
# the shared pool and export an APFS clone of the binary (~0 bytes, instant):
#   SCRIPT_KIT_AGENT_ARTIFACT_NAME=<task> ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui
#   # -> target-agent/artifacts/<task>/script-kit-gpui
#
# Disk policy (enforced synchronously at lock acquisition, before cargo runs):
#   SCRIPT_KIT_AGENT_TARGET_BUDGET_GB  total budget for target-agent pools+agents (default 40)
#   SCRIPT_KIT_AGENT_MIN_FREE_GB       free-disk floor that triggers LRU pool eviction (default 25)
#   SCRIPT_KIT_AGENT_CRITICAL_FREE_GB  harder floor; below it the requested pool's
#                                      own incremental/ dir is pruned too (default 10)
# Eviction only removes unlocked pools/agent dirs, LRU first, never the one
# being requested. Deterministic and synchronous: it never races a live build.
#
# Cache-size policy:
#   CARGO_PROFILE_DEV_DEBUG defaults to line-tables-only (usable backtraces, far
#   smaller deps/incremental). CARGO_INCREMENTAL stays on only for the default
#   shared pool; ephemeral pools and exclusive dirs get CARGO_INCREMENTAL=0.
#   Both respect pre-set env overrides.
#
# sccache: SCRIPT_KIT_AGENT_USE_SCCACHE=auto (default) uses sccache when on
# PATH, 1 forces (warns if missing), 0 disables.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

sanitize_id() {
  printf '%s' "$1" | tr -c 'a-zA-Z0-9._-' '-'
}

agent_id="$(sanitize_id "${SCRIPT_KIT_AGENT_ID:-${USER:-agent}-${PPID:-$$}}")"
target_mode="${SCRIPT_KIT_AGENT_TARGET_MODE:-pool}"
pool="$(sanitize_id "${SCRIPT_KIT_CARGO_TARGET_POOL:-agent-debug}")"
default_pool="agent-debug"

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

# Slim debug info: agents read backtraces, they do not attach debuggers.
export CARGO_PROFILE_DEV_DEBUG="${CARGO_PROFILE_DEV_DEBUG:-line-tables-only}"

# Incremental compilation is worth its disk cost only in the long-lived shared
# pool; ephemeral pools/exclusive dirs rarely live long enough to amortize it.
if [[ -z "${CARGO_INCREMENTAL:-}" ]]; then
  if [[ "$target_mode" != "pool" || "$pool" != "$default_pool" ]]; then
    export CARGO_INCREMENTAL=0
  fi
fi

rustc_wrapper_state="none"
use_sccache="${SCRIPT_KIT_AGENT_USE_SCCACHE:-auto}"
if [[ "$use_sccache" == "1" || "$use_sccache" == "auto" ]]; then
  if command -v sccache >/dev/null 2>&1; then
    export RUSTC_WRAPPER="sccache"
    export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-10G}"
    rustc_wrapper_state="sccache"
  elif [[ "$use_sccache" == "1" ]]; then
    echo "AGENT_CARGO warning: SCRIPT_KIT_AGENT_USE_SCCACHE=1 but sccache not on PATH; continuing without it" >&2
  fi
fi

free_disk_kb() {
  df -k "$REPO_ROOT" | awk 'NR==2 {print $4}'
}

dir_kb() {
  du -sk "$1" 2>/dev/null | awk '{print $1}'
}

# A candidate dir is evictable if no live lock holds it.
candidate_locked() {
  local dir="$1" name lock pid
  name="$(basename "$dir")"
  case "$dir" in
    */pools/*) lock="${lock_root}/pool-${name}.lock" ;;
    *) lock="${lock_root}/agent-${name}.lock" ;;
  esac
  [[ -d "$lock" ]] || return 1
  pid="$(cat "${lock}/pid" 2>/dev/null || true)"
  if [[ -n "$pid" ]] && ! kill -0 "$pid" 2>/dev/null; then
    return 1
  fi
  return 0
}

# Print evictable candidate dirs (not ours, not locked), LRU first.
eviction_candidates() {
  local dir stamp
  for dir in "${REPO_ROOT}"/target-agent/pools/* "${REPO_ROOT}"/target-agent/agents/*; do
    [[ -d "$dir" ]] || continue
    [[ "$dir" == "$target_dir" ]] && continue
    candidate_locked "$dir" && continue
    if [[ -f "${dir}/.last_used" ]]; then
      stamp="$(stat -f '%m' "${dir}/.last_used" 2>/dev/null || echo 0)"
    else
      stamp="$(stat -f '%m' "$dir" 2>/dev/null || echo 0)"
    fi
    printf '%s\t%s\n' "$stamp" "$dir"
  done | sort -n | cut -f2
}

total_agent_target_kb() {
  local dir total=0 kb
  for dir in "${REPO_ROOT}"/target-agent/pools/* "${REPO_ROOT}"/target-agent/agents/*; do
    [[ -d "$dir" ]] || continue
    kb="$(dir_kb "$dir")"
    total=$(( total + ${kb:-0} ))
  done
  echo "$total"
}

enforce_disk_budget() {
  local budget_gb="${SCRIPT_KIT_AGENT_TARGET_BUDGET_GB:-40}"
  local min_free_gb="${SCRIPT_KIT_AGENT_MIN_FREE_GB:-25}"
  local critical_free_gb="${SCRIPT_KIT_AGENT_CRITICAL_FREE_GB:-10}"
  local budget_kb=$(( budget_gb * 1024 * 1024 ))
  local min_free_kb=$(( min_free_gb * 1024 * 1024 ))
  local critical_free_kb=$(( critical_free_gb * 1024 * 1024 ))
  local total_kb free_kb dir

  total_kb="$(total_agent_target_kb)"
  free_kb="$(free_disk_kb)"

  if (( total_kb <= budget_kb && free_kb >= min_free_kb )); then
    return 0
  fi

  echo "AGENT_CARGO disk_budget total=$((total_kb / 1024 / 1024))G/${budget_gb}G free=$((free_kb / 1024 / 1024))G/min${min_free_gb}G; evicting LRU pools" >&2

  while IFS= read -r dir; do
    (( total_kb <= budget_kb && free_kb >= min_free_kb )) && break
    echo "AGENT_CARGO evict dir=${dir} size=$(( $(dir_kb "$dir") / 1024 / 1024 ))G" >&2
    rm -rf "$dir"
    total_kb="$(total_agent_target_kb)"
    free_kb="$(free_disk_kb)"
  done < <(eviction_candidates)

  # Last resort: prune our own incremental cache (safe; next build is just slower).
  if (( free_kb < critical_free_kb )) && [[ -d "${target_dir}/debug/incremental" ]]; then
    echo "AGENT_CARGO evict_incremental dir=${target_dir}/debug/incremental size=$(( $(dir_kb "${target_dir}/debug/incremental") / 1024 / 1024 ))G" >&2
    rm -rf "${target_dir}/debug/incremental"
    free_kb="$(free_disk_kb)"
  fi

  if (( free_kb < min_free_kb )); then
    echo "AGENT_CARGO warning: free disk still $((free_kb / 1024 / 1024))G < ${min_free_gb}G after eviction; the system watcher may intervene" >&2
  fi
}

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

# After a successful `build --bin X`, clone the binary to a stable per-task
# path so parallel drivers never need a 26 GB pool of their own. APFS clones
# (cp -c) are instant and copy-on-write.
export_artifacts() {
  local artifact_name profile_dir="debug" bins=() i=0 argc=$#
  local args=("$@")
  artifact_name="$(sanitize_id "${SCRIPT_KIT_AGENT_ARTIFACT_NAME:-}")"
  [[ -n "$artifact_name" ]] || return 0
  [[ "${args[0]:-}" == "build" ]] || return 0

  while (( i < argc )); do
    case "${args[$i]}" in
      --bin)
        (( i + 1 < argc )) && bins+=("${args[$((i + 1))]}")
        ;;
      --release)
        profile_dir="release"
        ;;
      --profile)
        if (( i + 1 < argc )); then
          profile_dir="${args[$((i + 1))]}"
          [[ "$profile_dir" == "dev" ]] && profile_dir="debug"
        fi
        ;;
    esac
    i=$(( i + 1 ))
  done

  if (( ${#bins[@]} == 0 )); then
    echo "AGENT_CARGO warning: SCRIPT_KIT_AGENT_ARTIFACT_NAME=${artifact_name} set but no --bin in build args; nothing exported" >&2
    return 0
  fi

  local artifact_dir="${REPO_ROOT}/target-agent/artifacts/${artifact_name}"
  mkdir -p "$artifact_dir"
  local bin src dest tmp
  for bin in "${bins[@]}"; do
    src="${target_dir}/${profile_dir}/${bin}"
    if [[ ! -x "$src" ]]; then
      echo "AGENT_CARGO warning: built binary not found at ${src}; skipped export" >&2
      continue
    fi
    dest="${artifact_dir}/${bin}"
    tmp="${dest}.tmp.$$"
    if ! cp -c "$src" "$tmp" 2>/dev/null; then
      cp -p "$src" "$tmp"
    fi
    mv -f "$tmp" "$dest"
    echo "AGENT_CARGO artifact bin=${bin} path=${dest}" >&2
  done
}

acquire_lock "$@"
trap release_lock EXIT INT TERM

touch "${target_dir}/.last_used" 2>/dev/null || true
enforce_disk_budget

echo "AGENT_CARGO mode=${target_mode} pool=${pool} target_dir=${CARGO_TARGET_DIR} lock=${lock_name} rustc_wrapper=${rustc_wrapper_state} debug=${CARGO_PROFILE_DEV_DEBUG} incremental=${CARGO_INCREMENTAL:-default} cargo $*" >&2

set +e
cargo "$@"
status=$?
set -e

if [[ "$status" -eq 0 ]]; then
  export_artifacts "$@"
fi

release_lock
exit "$status"
