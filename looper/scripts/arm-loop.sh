#!/usr/bin/env bash
# arm-loop.sh — compute epochs and emit a ready-to-paste tick prompt.
#
# Usage:
#   bash looper/scripts/arm-loop.sh <budget-min> <audit-root> <session-name> <agentic-cmd>
#
# Example:
#   bash looper/scripts/arm-loop.sh 420 audits/afk default "bash scripts/agentic/session.sh"
#
# Writes nothing; prints a filled tick prompt to stdout. Caller copies it
# into CronCreate (or runs it as a supervised first tick).

set -euo pipefail

if [ "$#" -lt 4 ]; then
  echo "usage: $0 <budget-min> <audit-root> <session-name> <agentic-cmd>" >&2
  exit 2
fi

BUDGET_MIN="$1"
AUDIT_ROOT="$2"
SESSION_NAME="$3"
AGENTIC_CMD="$4"

# Resolve looper/ relative to this script.
LOOPER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TICK_TEMPLATE="$LOOPER_DIR/tick-prompt.txt"

if [ ! -f "$TICK_TEMPLATE" ]; then
  echo "error: tick-prompt.txt not found at $TICK_TEMPLATE" >&2
  exit 1
fi

START_ISO=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
START_EPOCH=$(date -u -jf "%Y-%m-%dT%H:%M:%SZ" "$START_ISO" +%s 2>/dev/null \
  || date -u -d "$START_ISO" +%s)

DEADLINE_EPOCH=$(( START_EPOCH + BUDGET_MIN * 60 ))
BUFFER_EPOCH=$(( DEADLINE_EPOCH - 20 * 60 ))

DEADLINE_ISO=$(date -u -r "$DEADLINE_EPOCH" +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null \
  || date -u -d "@$DEADLINE_EPOCH" +"%Y-%m-%dT%H:%M:%SZ")
BUFFER_ISO=$(date -u -r "$BUFFER_EPOCH" +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null \
  || date -u -d "@$BUFFER_EPOCH" +"%Y-%m-%dT%H:%M:%SZ")

# Determine next run number from log.md header count.
RUN_N=1
if [ -f "$AUDIT_ROOT/log.md" ]; then
  PRIOR=$(grep -cE '^## Run [0-9]+ — started ' "$AUDIT_ROOT/log.md" || true)
  RUN_N=$(( PRIOR + 1 ))
fi

cat <<HEADER
# Copy the block below into CronCreate.
#
# Before arming:
# 1. Append this header to $AUDIT_ROOT/log.md:
#
#    ## Run $RUN_N — started $START_ISO — budget $BUDGET_MIN min — deadline $DEADLINE_ISO — buffer-cutoff $BUFFER_ISO
#    - Cron id: <fill in after CronCreate returns>
#
# 2. Commit the log header as:  audit(scheduler): Run $RUN_N started — budget $BUDGET_MIN min
# 3. CronCreate with the prompt below; capture the returned id; edit log.md to fill "Cron id: <id>".
# 4. Run one supervised tick; then the cron takes over.
#
# Epoch round-trip verification (must match ISOs above):
#   deadline: $DEADLINE_EPOCH → $(date -u -r "$DEADLINE_EPOCH" +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -u -d "@$DEADLINE_EPOCH" +"%Y-%m-%dT%H:%M:%SZ")
#   buffer:   $BUFFER_EPOCH → $(date -u -r "$BUFFER_EPOCH" +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -u -d "@$BUFFER_EPOCH" +"%Y-%m-%dT%H:%M:%SZ")
#
# Suggested minute offsets (never :00 or :30):
#   10-min cadence: :07,:17,:27,:37,:47,:57
#   15-min cadence: :04,:19,:34,:49

--- BEGIN TICK PROMPT ---
HEADER

sed \
  -e "s|<RUN_N>|$RUN_N|g" \
  -e "s|<START_ISO>|$START_ISO|g" \
  -e "s|<BUDGET_MIN>|$BUDGET_MIN|g" \
  -e "s|<DEADLINE_ISO>|$DEADLINE_ISO|g" \
  -e "s|<DEADLINE_EPOCH>|$DEADLINE_EPOCH|g" \
  -e "s|<BUFFER_ISO>|$BUFFER_ISO|g" \
  -e "s|<BUFFER_EPOCH>|$BUFFER_EPOCH|g" \
  -e "s|<AUDIT_ROOT>|$AUDIT_ROOT|g" \
  -e "s|<SESSION_NAME>|$SESSION_NAME|g" \
  -e "s|<AGENTIC_CMD>|$AGENTIC_CMD|g" \
  "$TICK_TEMPLATE"

cat <<FOOTER
--- END TICK PROMPT ---
FOOTER
