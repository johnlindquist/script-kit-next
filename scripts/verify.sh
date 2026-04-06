#!/usr/bin/env bash
set -euo pipefail

run_step() {
  local name="$1"
  shift

  printf "\n[%s] RUN  %s\n" "$name" "$*"
  if "$@"; then
    printf "[%s] PASS\n" "$name"
  else
    local exit_code=$?
    printf "[%s] FAIL (exit %s)\n" "$name" "$exit_code" >&2
    exit "$exit_code"
  fi
}

run_step "fmt"       cargo fmt --check
run_step "check"     cargo check
run_step "clippy"    cargo clippy --lib -- -D warnings
run_step "nextest"   cargo nextest run --lib
run_step "sdk-types" bun run scripts/check-sdk-types.ts
run_step "sdk-tests" bun run scripts/test-runner.ts --parallel
run_step "bundle"    cargo bundle --release --bin script-kit-gpui

printf "\nAll verification steps PASSED.\n"
