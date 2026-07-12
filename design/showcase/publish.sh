#!/usr/bin/env bash
# Stage the showcase for here.now publishing (site files only — no reference
# captures, briefs, or tooling) and publish. With no --slug argument the
# here-now publish script mints a BRAND NEW site.
set -euo pipefail
cd "$(dirname "$0")"

STAGE=".stage"
rm -rf "$STAGE"
mkdir -p "$STAGE/shared"
cp index.html "$STAGE/"
cp shared/tokens.css shared/components.css shared/fit.js "$STAGE/shared/"
rsync -a --include='*/' --include='index.html' --include='*.css' --include='*.js' --include='*.svg' --exclude='*' shots/ "$STAGE/shots/"

exec ~/.agents/skills/here-now/scripts/publish.sh "$STAGE" --client claude-code \
  --title "Script Kit — pixel-perfect HTML tour" \
  --description "Every frame is a live HTML/CSS recreation of the real app, pixel-matched against macOS captures." \
  "$@"
