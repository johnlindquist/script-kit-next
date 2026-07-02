#!/usr/bin/env bash
# Seed a driver/session sandbox HOME with the auth state a live Agent Chat or
# brain probe needs, in one call. Replaces the order-sensitive manual dance of
# copying ~/.pi and ~/.codex into the sandbox after launch and before the
# first openAi/agent-chat command.
#
# Copies via APFS clonefile (cp -c, ~0 bytes, instant):
#   ~/.pi                -> <sandbox>/.pi          (Pi agent auth + state, ~300MB logical)
#   ~/.codex/auth.json   -> <sandbox>/.codex/auth.json
#   ~/.codex/config.toml -> <sandbox>/.codex/config.toml   (when present)
# ~/.codex is NOT copied wholesale — it can be tens of GB of session history.
#
# Usage:
#   bash scripts/agentic/seed-sandbox-home.sh <sandbox-home-dir>
#
# The Driver sandbox HOME lives at <driver.sessionDir>/home, or pass
# seedAgentAuth: true to Driver.launch to run this automatically.
# For brain-inbox probes, additionally launch the app with
# SCRIPT_KIT_TEST_BRAIN_DB_PATH=<path> to mock the inbox database.
set -euo pipefail

SANDBOX_HOME="${1:?usage: seed-sandbox-home.sh <sandbox-home-dir>}"
SOURCE_HOME="${SEED_SOURCE_HOME:-$HOME}"
mkdir -p "$SANDBOX_HOME"

seeded=()
skipped=()

clone() {
  # clone <src> <dest> — APFS clonefile with plain-copy fallback.
  local src="$1" dest="$2"
  mkdir -p "$(dirname "$dest")"
  if cp -Rc "$src" "$dest" 2>/dev/null || cp -R "$src" "$dest"; then
    seeded+=("$dest")
  else
    skipped+=("$src (copy failed)")
  fi
}

if [ -d "$SOURCE_HOME/.pi" ]; then
  if [ -e "$SANDBOX_HOME/.pi" ]; then
    skipped+=("$SOURCE_HOME/.pi (already present in sandbox)")
  else
    clone "$SOURCE_HOME/.pi" "$SANDBOX_HOME/.pi"
  fi
else
  skipped+=("$SOURCE_HOME/.pi (missing)")
fi

for file in auth.json config.toml; do
  if [ -f "$SOURCE_HOME/.codex/$file" ]; then
    if [ -e "$SANDBOX_HOME/.codex/$file" ]; then
      skipped+=("$SOURCE_HOME/.codex/$file (already present in sandbox)")
    else
      clone "$SOURCE_HOME/.codex/$file" "$SANDBOX_HOME/.codex/$file"
    fi
  else
    skipped+=("$SOURCE_HOME/.codex/$file (missing)")
  fi
done

json_array() {
  local out="" item
  for item in "$@"; do
    [ -z "$item" ] && continue
    [ -n "$out" ] && out+=","
    out+="\"${item//\"/\\\"}\""
  done
  printf '[%s]' "$out"
}

printf '{"schemaVersion":1,"tool":"seed-sandbox-home","sandboxHome":"%s","seeded":%s,"skipped":%s}\n' \
  "$SANDBOX_HOME" \
  "$(json_array "${seeded[@]:-}")" \
  "$(json_array "${skipped[@]:-}")"
