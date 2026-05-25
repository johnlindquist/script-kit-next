#!/usr/bin/env bash
set -euo pipefail

SKIP_BUNDLE=0
if [[ "${1:-}" == "--skip-bundle" ]]; then
  SKIP_BUNDLE=1
elif [[ $# -gt 0 ]]; then
  echo "usage: bash scripts/verify.sh [--skip-bundle]" >&2
  exit 64
fi

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

run_step "fmt"       cargo fmt --check
run_step "check"     cargo check --locked
run_step "clippy"    cargo clippy --locked --lib -- -D warnings
run_step "test-compile" cargo test --no-run --locked
run_step "sdk-types" bun run scripts/check-sdk-types.ts
run_step "sdk-tests" bun run scripts/test-runner.ts --parallel

if [[ "$SKIP_BUNDLE" -eq 0 ]]; then
  run_step "pi-sidecar"    bash scripts/prepare-pi-sidecar.sh
  run_step "bundle"        cargo bundle --locked --release --bin script-kit-gpui
  run_step "bundle-sidecar" bash scripts/install-pi-sidecar-into-bundle.sh
  run_step "bundle-verify" bash scripts/verify-macos-bundle.sh
fi

printf "\n[verify] COMPLETE skip_bundle=%s\n" "$SKIP_BUNDLE"
