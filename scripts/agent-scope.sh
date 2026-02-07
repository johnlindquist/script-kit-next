#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

usage() {
  cat <<'USAGE'
Usage: bash scripts/agent-scope.sh <module-name>

Prints files related to a module for agent scoping.

Examples:
  bash scripts/agent-scope.sh actions
  bash scripts/agent-scope.sh prompts
  bash scripts/agent-scope.sh theme
USAGE
}

escape_regex() {
  printf '%s' "$1" | sed -E 's/[][(){}.+*?^$|\\/]/\\&/g'
}

if [[ $# -ne 1 ]]; then
  usage
  exit 1
fi

case "$1" in
  -h|--help)
    usage
    exit 0
    ;;
esac

MODULE_NAME="${1#./}"
MODULE_NAME="${MODULE_NAME%/}"
MODULE_NAME_LOWER="$(printf '%s' "$MODULE_NAME" | tr '[:upper:]' '[:lower:]')"
MODULE_REGEX="$(escape_regex "$MODULE_NAME_LOWER")"

RESULTS_FILE="$(mktemp)"
ALL_FILES="$(rg --files src tests scripts 2>/dev/null || true)"
trap 'rm -f "$RESULTS_FILE"' EXIT

if [[ -d "src/$MODULE_NAME_LOWER" ]]; then
  rg --files "src/$MODULE_NAME_LOWER" 2>/dev/null >> "$RESULTS_FILE" || true
fi

if [[ -f "src/$MODULE_NAME_LOWER.rs" ]]; then
  echo "src/$MODULE_NAME_LOWER.rs" >> "$RESULTS_FILE"
fi

if [[ -d "tests/$MODULE_NAME_LOWER" ]]; then
  rg --files "tests/$MODULE_NAME_LOWER" 2>/dev/null >> "$RESULTS_FILE" || true
fi

if [[ -f "tests/$MODULE_NAME_LOWER.rs" ]]; then
  echo "tests/$MODULE_NAME_LOWER.rs" >> "$RESULTS_FILE"
fi

if [[ -n "$ALL_FILES" ]]; then
  printf '%s\n' "$ALL_FILES" \
    | grep -Ei "(^|/)${MODULE_REGEX}(/|[._-])|(^|/)[^/]*${MODULE_REGEX}[^/]*$" \
    >> "$RESULTS_FILE" || true
fi

if [[ ! -s "$RESULTS_FILE" ]]; then
  echo "No files found for module: $MODULE_NAME" >&2
  exit 1
fi

sort -u "$RESULTS_FILE"
