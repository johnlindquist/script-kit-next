#!/usr/bin/env bash
#
# Phase 0 Consistency Ratchet
# Prevents NEW violations of consistency rules in changed files.
# Run against the diff from main (or a specified base branch).
#
# Usage:
#   ./scripts/consistency-ratchet.sh          # check diff vs main
#   ./scripts/consistency-ratchet.sh origin/main  # explicit base
#
# Exit codes:
#   0 = no new violations
#   1 = new violations found

set -euo pipefail

BASE="${1:-main}"
FAIL=0

# Get list of added/modified Rust files in src/ (not deleted)
changed_files() {
  git diff --name-only --diff-filter=ACM "$BASE"...HEAD -- 'src/*.rs' 'src/**/*.rs' 2>/dev/null \
    || git diff --name-only --diff-filter=ACM HEAD -- 'src/*.rs' 'src/**/*.rs' 2>/dev/null \
    || true
}

# Get only the added lines (+ prefix) in changed src/ files
added_lines() {
  git diff "$BASE"...HEAD -- 'src/*.rs' 'src/**/*.rs' 2>/dev/null \
    | grep -F '+' | grep '^+' | grep -v '^+++' \
    || git diff HEAD -- 'src/*.rs' 'src/**/*.rs' 2>/dev/null \
    | grep -F '+' | grep '^+' | grep -v '^+++' \
    || true
}

echo "=== Consistency Ratchet (base: $BASE) ==="
echo ""

# --- Rule 1: No new part_*.rs files ---
new_part_files=$(changed_files | grep -E 'part_[0-9]+\.rs$' || true)
if [ -n "$new_part_files" ]; then
  echo "FAIL: New part_*.rs files detected (use semantic names instead):"
  echo "$new_part_files" | sed 's/^/  /'
  FAIL=1
else
  echo "PASS: No new part_*.rs files"
fi

# --- Rule 1b: No new include!("part_ usage ---
new_includes=$(added_lines | grep 'include!.*"part_' || true)
if [ -n "$new_includes" ]; then
  echo "FAIL: New include!(\"part_...\") usage detected:"
  echo "$new_includes" | sed 's/^/  /'
  FAIL=1
else
  echo "PASS: No new include!(\"part_\") usage"
fi

# --- Rule 2: No new numbered test directories ---
new_numbered_tests=$(changed_files | grep -E '_tests_[0-9]+/' || true)
if [ -n "$new_numbered_tests" ]; then
  echo "FAIL: New numbered test directories detected (use semantic names):"
  echo "$new_numbered_tests" | sed 's/^/  /'
  FAIL=1
else
  echo "PASS: No new numbered test directories"
fi

# --- Rule 3: No new log:: macro usage in src/ (non-test) ---
# Filter out lines that are in #[cfg(test)] blocks (heuristic: check for _test in filename)
new_log_macros=$(added_lines | grep 'log::info!\|log::warn!\|log::error!\|log::debug!\|log::trace!' | grep -v '#\[cfg(test)\]' | grep -v '// bridge' || true)
if [ -n "$new_log_macros" ]; then
  echo "FAIL: New log:: macro usage detected (use tracing:: instead):"
  echo "$new_log_macros" | head -10 | sed 's/^/  /'
  count=$(echo "$new_log_macros" | wc -l | tr -d ' ')
  if [ "$count" -gt 10 ]; then
    echo "  ... and $((count - 10)) more"
  fi
  FAIL=1
else
  echo "PASS: No new log:: macro usage"
fi

echo ""

if [ "$FAIL" -eq 1 ]; then
  echo "=== RATCHET FAILED ==="
  echo "Fix the violations above before committing."
  echo "See CLEANUP.md for the full consistency plan."
  exit 1
else
  echo "=== RATCHET PASSED ==="
  exit 0
fi
