#!/usr/bin/env bash
# epoch-check.sh — drop-in preflight snippet for the tick's Step 1.
#
# Exits 0 if still within buffer; prints "STOP:<reason>" and exits 1 otherwise.
#
# Usage (inside the tick prompt's STEP 1):
#   bash looper/scripts/epoch-check.sh <BUFFER_EPOCH>
#
# The tick prompt embeds this check inline; this script exists for manual
# invocation (e.g., "is the buffer cutoff past yet?") outside a tick.

set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <buffer-epoch>" >&2
  exit 2
fi

BUFFER_EPOCH="$1"
NOW_EPOCH=$(date -u +%s)

if [ "$NOW_EPOCH" -ge "$BUFFER_EPOCH" ]; then
  DELTA=$(( NOW_EPOCH - BUFFER_EPOCH ))
  echo "STOP:now=${NOW_EPOCH} buffer=${BUFFER_EPOCH} delta=+${DELTA}s"
  exit 1
fi

REMAINING=$(( BUFFER_EPOCH - NOW_EPOCH ))
echo "OK:remaining=${REMAINING}s"
exit 0
