#!/usr/bin/env bash
# scripts/agentic/prune-cargo-targets.sh — Safely trim target/ and target-agent/.
#
# Goals:
#   - Never delete the whole target/ (cargo clean forces a cold rebuild with
#     no progress output).
#   - Use cargo-sweep to drop artifacts not touched recently. Dry-run first.
#   - Drop stale per-agent target-agent/<id>/ dirs that haven't been used in a
#     while.
#
# Usage:
#   scripts/agentic/prune-cargo-targets.sh                # dry-run, no changes
#   scripts/agentic/prune-cargo-targets.sh --apply        # actually prune
#   PRUNE_TIME_DAYS=14 PRUNE_AGENT_DAYS=7 scripts/agentic/prune-cargo-targets.sh --apply
#
# Env:
#   PRUNE_TIME_DAYS    — cargo sweep --time threshold (default: 14)
#   PRUNE_AGENT_DAYS   — find -mtime threshold for target-agent/<id>/ (default: 7)
#   PRUNE_INCREMENTAL_DAYS — find -mtime threshold for target/debug/incremental/* (default: 14)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

APPLY=0
if [ "${1:-}" = "--apply" ]; then
    APPLY=1
fi

PRUNE_TIME_DAYS="${PRUNE_TIME_DAYS:-14}"
PRUNE_AGENT_DAYS="${PRUNE_AGENT_DAYS:-7}"
PRUNE_INCREMENTAL_DAYS="${PRUNE_INCREMENTAL_DAYS:-14}"

log() { echo "[prune] $*" >&2; }

if [ "$APPLY" = "1" ]; then
    log "mode=APPLY — will actually delete"
else
    log "mode=DRY-RUN — no changes; pass --apply to prune"
fi

log "before sizes:"
du -sh target target/debug target/debug/incremental target-agent 2>/dev/null || true

# 1. cargo-sweep on target/
if ! command -v cargo-sweep >/dev/null 2>&1; then
    log "cargo-sweep not installed. Install with: cargo install cargo-sweep"
else
    if [ -d target ]; then
        log "cargo sweep --dry-run --time ${PRUNE_TIME_DAYS} (target/)"
        cargo sweep --dry-run --time "$PRUNE_TIME_DAYS" || true
        if [ "$APPLY" = "1" ]; then
            log "cargo sweep --time ${PRUNE_TIME_DAYS} (target/)"
            cargo sweep --time "$PRUNE_TIME_DAYS" || true
        fi
        log "cargo sweep --dry-run --installed (target/)"
        cargo sweep --dry-run --installed || true
        if [ "$APPLY" = "1" ]; then
            log "cargo sweep --installed (target/)"
            cargo sweep --installed || true
        fi
    fi
fi

# 2. Stale incremental dirs under target/debug/incremental
if [ -d target/debug/incremental ]; then
    log "stale incremental dirs (-mtime +${PRUNE_INCREMENTAL_DAYS}):"
    find target/debug/incremental -mindepth 1 -maxdepth 1 -type d -mtime +"$PRUNE_INCREMENTAL_DAYS" -print || true
    if [ "$APPLY" = "1" ]; then
        find target/debug/incremental -mindepth 1 -maxdepth 1 -type d -mtime +"$PRUNE_INCREMENTAL_DAYS" -exec rm -rf {} + || true
    fi
fi

# 3. Stale per-agent target-agent/<id>/ dirs
if [ -d target-agent ]; then
    log "stale target-agent/<id>/ dirs (-mtime +${PRUNE_AGENT_DAYS}):"
    find target-agent -mindepth 1 -maxdepth 1 -type d -mtime +"$PRUNE_AGENT_DAYS" -print || true
    if [ "$APPLY" = "1" ]; then
        find target-agent -mindepth 1 -maxdepth 1 -type d -mtime +"$PRUNE_AGENT_DAYS" -exec rm -rf {} + || true
    fi
fi

log "after sizes:"
du -sh target target/debug target/debug/incremental target-agent 2>/dev/null || true

if [ "$APPLY" != "1" ]; then
    log "Dry-run complete. Re-run with --apply to prune."
fi
