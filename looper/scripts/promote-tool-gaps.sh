#!/usr/bin/env bash
#
# promote-tool-gaps.sh — promote tool-* slugs mentioned in log.md into
# actionable [ ] items in stories.md.
#
# Run at the start of every audit-loop tick (Step 3). Idempotent: re-running
# with no new slugs produces no changes. A tool-* slug that appears only in
# log prose (not as a [x] / [ ] / [!] / [?] item in stories.md) will be
# appended to the "### Tool-gap backlog (promoted from log)" section at the
# end of stories.md.
#
# Rationale: prose-only tool-gap mentions in the log never become stories,
# because the loop reads stories.md top-down and has no way to see gaps
# buried in prose. This script closes the loop so gaps surface as actionable
# items on the next tick.
#
# Install: copy this file into audits/afk/ (or your <AUDIT_ROOT>).
# Usage:   bash audits/afk/promote-tool-gaps.sh
# Exit:    0 if no changes OR slugs promoted; 1 if log/stories missing.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG="$ROOT/audits/afk/log.md"
STORIES="$ROOT/audits/afk/stories.md"
BACKLOG_HEADER="### Tool-gap backlog (promoted from log)"

if [[ ! -f "$LOG" || ! -f "$STORIES" ]]; then
  echo "promote-tool-gaps: missing log.md or stories.md" >&2
  exit 1
fi

# Extract tool-* slug mentions from log.md. Filter out generic non-slug tokens
# that appear in prose but aren't real slugs.
LOG_SLUGS=$(grep -oE 'tool-[a-z0-9][a-z0-9-]{4,80}' "$LOG" \
  | grep -vxE 'tool-extension|tool-improvement|tool-level|tool-shipped|tool-gap|tool-gaps' \
  | sort -u)

# Extract tool-* slugs already tracked in stories.md in any state.
STORY_SLUGS=$(grep -oE '^- \[[x! ?-]\] tool-[a-z0-9-]+' "$STORIES" \
  | grep -oE 'tool-[a-z0-9-]+' \
  | sort -u)

# Set diff: slugs in log but not yet in stories.
MISSING=$(comm -23 <(echo "$LOG_SLUGS") <(echo "$STORY_SLUGS") || true)

if [[ -z "$MISSING" ]]; then
  echo "promote-tool-gaps: no missing slugs; all log-mentioned tool-* slugs already tracked in stories.md"
  exit 0
fi

# Ensure the backlog header exists.
if ! grep -qF "$BACKLOG_HEADER" "$STORIES"; then
  {
    echo ""
    echo "$BACKLOG_HEADER"
    echo ""
    echo "Tool-extension slugs auto-promoted from \`log.md\` prose by \`audits/afk/promote-tool-gaps.sh\`. Each item is actionable: implement the RPC/verb/field to unlock the story that originally blocked on it. The audit loop prefers these over fresh story generation until drained."
    echo ""
  } >> "$STORIES"
fi

# Append each missing slug as a new [ ] item with a prose snippet for context.
while IFS= read -r slug; do
  [[ -z "$slug" ]] && continue
  CONTEXT=$(grep -m 1 -F "$slug" "$LOG" 2>/dev/null || true)
  CONTEXT=$(printf '%s' "$CONTEXT" | head -c 400 | tr '\n' ' ' | sed 's/  */ /g' | sed 's/^[ .*-]*//')
  if [[ -z "$CONTEXT" ]]; then
    echo "- [ ] ${slug}: Auto-promoted from log.md (no inline context found)." >> "$STORIES"
  else
    echo "- [ ] ${slug}: Auto-promoted from log.md. Context: ${CONTEXT}" >> "$STORIES"
  fi
done <<< "$MISSING"

COUNT=$(echo "$MISSING" | grep -c . || true)
echo "promote-tool-gaps: promoted $COUNT slugs to stories.md backlog"
echo "$MISSING"
