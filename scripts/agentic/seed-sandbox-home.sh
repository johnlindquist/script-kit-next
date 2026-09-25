#!/usr/bin/env bash
# Seed a driver/session sandbox HOME with the auth state a live Agent Chat,
# brain, or Flows probe needs, in one call, before the first AI command.
#
# Copies via APFS clonefile (cp -c, ~0 bytes, instant):
#   ~/.scriptkit/goose/config -> <sandbox>/.scriptkit/goose/config
#                                (goose provider sign-ins and config.yaml)
#   ~/.codex/auth.json        -> <sandbox>/.codex/auth.json   (Flows / Codex)
#   ~/.codex/config.toml      -> <sandbox>/.codex/config.toml (when present)
# goose sessions and ~/.codex history are NOT copied — they can be large.
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

seed() {
  # seed <relative-path> — clone one path from the source home when present.
  local relative="$1"
  if [ ! -e "$SOURCE_HOME/$relative" ]; then
    skipped+=("$SOURCE_HOME/$relative (missing)")
  elif [ -e "$SANDBOX_HOME/$relative" ]; then
    skipped+=("$SOURCE_HOME/$relative (already present in sandbox)")
  else
    clone "$SOURCE_HOME/$relative" "$SANDBOX_HOME/$relative"
  fi
}

seed ".scriptkit/goose/config"
seed ".codex/auth.json"
seed ".codex/config.toml"

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
