#!/usr/bin/env bash
#
# promote-tool-gaps.sh — promote tool-* slugs mentioned in log.md into
# actionable [ ] items in stories.md.
#
# Run at the start of every audit loop tick. Idempotent: re-running with
# no new slugs produces no changes. A tool-* slug that appears only in
# log prose (not as a [x] or [ ] item in stories.md) will be appended to
# the "### Tool-gap backlog (promoted from log)" section at the top of
# the generated sections.
#
# Rationale: Run 2 logged ~12 tool-gap slugs in log prose that were
# never picked up as stories, because the loop reads stories.md top-down
# and has no way to see gaps buried in prose. This script closes the
# loop so gaps surface as actionable items on the next tick.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOG="$ROOT/audits/afk/log.md"
STORIES="$ROOT/audits/afk/stories.md"
BACKLOG_HEADER="### Tool-gap backlog (promoted from log)"

if [[ ! -f "$LOG" || ! -f "$STORIES" ]]; then
  echo "promote-tool-gaps: missing log.md or stories.md" >&2
  exit 1
fi

# Extract all tool-* slug mentions from log.md, filter out generic
# non-slug tokens, and dedupe.
LOG_SLUGS=$(grep -oE 'tool-[a-z0-9][a-z0-9-]{4,80}' "$LOG" \
  | grep -vxE 'tool-extension|tool-improvement|tool-level|tool-shipped|tool-gap' \
  | sort -u)

# Extract all tool-* slugs already tracked in stories.md (any state).
# Character class covers: `x` done, `!` tool-gap, ` ` open, `?` anomaly,
# `-` withdrawn/obsolete, `~` retracted (e.g. pilot-error or spurious
# auto-promoter false-positive). `~` was missing prior to this fix, which
# caused retracted slugs (e.g. `tool-observability`, `tool-triggerbuiltin-
# dropped-after-hide-show`) to be re-promoted on every tick because the
# set-diff saw them as untracked.
STORY_SLUGS=$(grep -oE '^- \[[x! ?~-]\] tool-[a-z0-9-]+' "$STORIES" \
  | grep -oE 'tool-[a-z0-9-]+' \
  | sort -u)

# Set diff: slugs in log but not in stories.
MISSING=$(comm -23 <(echo "$LOG_SLUGS") <(echo "$STORY_SLUGS") || true)

if [[ -z "$MISSING" ]]; then
  echo "promote-tool-gaps: no missing slugs; all log-mentioned tool-* slugs already tracked in stories.md"
  exit 0
fi

# Ensure the backlog header exists in stories.md.
if ! grep -qF "$BACKLOG_HEADER" "$STORIES"; then
  # Append backlog section at the end of the file.
  {
    echo ""
    echo "$BACKLOG_HEADER"
    echo ""
    echo "Tool-extension slugs auto-promoted from \`log.md\` prose by \`audits/afk/promote-tool-gaps.sh\`. Each item is actionable: implement the RPC/verb/field to unlock the story that originally blocked on it. Items here are PRIORITIZED — the audit loop should prefer these over fresh story generation until the backlog is drained."
    echo ""
  } >> "$STORIES"
fi

# Append each missing slug as a new [ ] item under the backlog header.
# We append at the end of the file; since the backlog section is the
# last section, new items end up under it.
while IFS= read -r slug; do
  [[ -z "$slug" ]] && continue
  # Pull the first prose snippet from log.md mentioning this slug so
  # the story has some context for the loop. Use a forgiving grep
  # (plain -n fgrep on the slug) and take the matching line trimmed
  # to a reasonable length. Tolerate no match — the slug alone is a
  # valid entry.
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
