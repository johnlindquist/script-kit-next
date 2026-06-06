#!/usr/bin/env bash
set -euo pipefail

SKIP_BUNDLE=0
ONLY=""
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

usage() {
  echo "usage: bash scripts/verify.sh [--skip-bundle] [--only <phase>]" >&2
  echo "phases: fmt check clippy test-compile sdk-types sdk-tests pi-sidecar bundle bundle-sidecar bundle-verify" >&2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-bundle)
      SKIP_BUNDLE=1
      shift
      ;;
    --only)
      if [[ $# -lt 2 || -z "${2:-}" ]]; then
        usage
        exit 64
      fi
      ONLY="$2"
      shift 2
      ;;
    --only=*)
      ONLY="${1#--only=}"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      exit 64
      ;;
  esac
done

CARGO_CMD="${SCRIPT_KIT_CARGO:-cargo}"

sanitize_id() {
  printf '%s' "$1" | tr -c 'a-zA-Z0-9._-' '-'
}

bundle_target_dir() {
  if [[ -n "${SCRIPT_KIT_BUNDLE_TARGET_DIR:-}" ]]; then
    printf '%s\n' "${SCRIPT_KIT_BUNDLE_TARGET_DIR}"
    return
  fi

  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    printf '%s\n' "${CARGO_TARGET_DIR}"
    return
  fi

  if [[ "${CARGO_CMD}" == *"agent-cargo.sh"* ]]; then
    local target_mode="${SCRIPT_KIT_AGENT_TARGET_MODE:-pool}"
    case "${target_mode}" in
      pool)
        local pool
        pool="$(sanitize_id "${SCRIPT_KIT_CARGO_TARGET_POOL:-agent-debug}")"
        printf '%s\n' "${REPO_ROOT}/target-agent/pools/${pool}"
        return
        ;;
      exclusive)
        local agent_id
        agent_id="$(sanitize_id "${SCRIPT_KIT_AGENT_ID:-${USER:-agent}-${PPID:-$$}}")"
        printf '%s\n' "${REPO_ROOT}/target-agent/agents/${agent_id}"
        return
        ;;
    esac
  fi

  printf '%s\n' "${REPO_ROOT}/target"
}

BUNDLE_APP_PATH="${SCRIPT_KIT_BUNDLE_APP_PATH:-$(bundle_target_dir)/release/bundle/osx/Script Kit.app}"

run_step() {
  local name="$1"
  shift

  printf "\n[verify] RUN  %s :: %s\n" "$name" "$*"
  if "$@"; then
    printf "[verify] PASS %s\n" "$name"
  else
    local exit_code=$?
    printf "[verify] FAIL %s (exit %s)\n" "$name" "$exit_code" >&2
    exit "$exit_code"
  fi
}

run_step_quiet() {
  local name="$1"
  shift

  printf "\n[verify] RUN  %s :: %s\n" "$name" "$*"
  if "$@" >/dev/null; then
    printf "[verify] PASS %s\n" "$name"
  else
    local exit_code=$?
    printf "[verify] FAIL %s (exit %s)\n" "$name" "$exit_code" >&2
    exit "$exit_code"
  fi
}

run_phase() {
  local phase="$1"

  case "$phase" in
    fmt)
      run_step "fmt" "$CARGO_CMD" fmt --check
      ;;
    check)
      run_step "check" "$CARGO_CMD" check --locked
      ;;
    clippy)
      run_step "clippy" "$CARGO_CMD" clippy --locked --lib --no-deps -- -D warnings
      ;;
    test-compile)
      run_step "test-compile" "$CARGO_CMD" test --no-run --locked
      ;;
    sdk-types)
      run_step "sdk-types" bun run scripts/check-sdk-types.ts
      ;;
    sdk-tests)
      run_step "sdk-tests" bun run scripts/test-runner.ts --parallel
      ;;
    pi-sidecar)
      run_step "pi-sidecar" bash scripts/prepare-pi-sidecar.sh
      ;;
    bundle)
      run_step_quiet "bundle-lock" "$CARGO_CMD" metadata --locked --format-version=1 --no-deps
      run_step "bundle" "$CARGO_CMD" bundle --release --bin script-kit-gpui
      ;;
    bundle-sidecar)
      run_step "bundle-sidecar" bash scripts/install-pi-sidecar-into-bundle.sh "${BUNDLE_APP_PATH}"
      ;;
    bundle-verify)
      run_step "bundle-verify" bash scripts/verify-macos-bundle.sh "${BUNDLE_APP_PATH}"
      ;;
    *)
      echo "unknown verify phase: $phase" >&2
      usage
      exit 64
      ;;
  esac
}

if [[ -n "$ONLY" ]]; then
  case "$ONLY" in
    fmt|check|clippy|test-compile|sdk-types|sdk-tests)
      run_phase "$ONLY"
      printf "\n[verify] COMPLETE skip_bundle=%s only=%s\n" "$SKIP_BUNDLE" "$ONLY"
      exit 0
      ;;
    pi-sidecar|bundle|bundle-sidecar|bundle-verify)
      if [[ "$SKIP_BUNDLE" -eq 1 ]]; then
        echo "verify phase '$ONLY' is disabled by --skip-bundle" >&2
        exit 64
      fi
      run_phase "$ONLY"
      printf "\n[verify] COMPLETE skip_bundle=%s only=%s\n" "$SKIP_BUNDLE" "$ONLY"
      exit 0
      ;;
    *)
      echo "unknown verify phase: $ONLY" >&2
      usage
      exit 64
      ;;
  esac
fi

run_phase "fmt"
run_phase "check"
run_phase "clippy"
run_phase "test-compile"
run_phase "sdk-types"
run_phase "sdk-tests"

if [[ "$SKIP_BUNDLE" -eq 0 ]]; then
  run_phase "pi-sidecar"
  run_phase "bundle"
  run_phase "bundle-sidecar"
  run_phase "bundle-verify"
fi

printf "\n[verify] COMPLETE skip_bundle=%s\n" "$SKIP_BUNDLE"
