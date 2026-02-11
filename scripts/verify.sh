#!/usr/bin/env bash

set -u
set -o pipefail

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

run_step "fmt" cargo fmt --check
run_step "check" cargo check
run_step "clippy" cargo clippy --all-targets -- -D warnings
run_step "test" cargo test

printf "\nAll verification steps PASSED.\n"
