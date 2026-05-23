#!/usr/bin/env bash
set -euo pipefail

APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/focus_badge" && pwd)"
VOLUME="${1:-}"

if [[ -z "$VOLUME" ]]; then
  for candidate in /Volumes/Badger2350 /Volumes/Tufty2350 /Volumes/Blinky; do
    if [[ -d "$candidate" ]]; then
      VOLUME="$candidate"
      break
    fi
  done
fi

if [[ -z "$VOLUME" || ! -d "$VOLUME" ]]; then
  echo "Badge volume not found."
  echo "Put the badge into disk mode, then run:"
  echo "  $0 /Volumes/Badger2350"
  exit 1
fi

mkdir -p "$VOLUME/apps"
rm -rf "$VOLUME/apps/focus_badge"
cp -R "$APP_DIR" "$VOLUME/apps/focus_badge"
rm -rf "$VOLUME/apps/focus_badge/__pycache__"

echo "Installed Focus Badge to $VOLUME/apps/focus_badge"
echo "Safely eject/unmount the badge, then launch Focus Badge from the menu."
