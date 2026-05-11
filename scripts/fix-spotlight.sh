#!/usr/bin/env bash
# fix-spotlight.sh — Diagnose and repair macOS Spotlight indexing for the
# user's home volume so that Script Kit's file search (mdfind) returns results.
#
# Usage:
#   ./scripts/fix-spotlight.sh                # diagnose + enable + reindex
#   ./scripts/fix-spotlight.sh --erase        # also wipe and rebuild the index (slower, more thorough)
#   ./scripts/fix-spotlight.sh --diagnose     # read-only: show state, never call sudo
#   ./scripts/fix-spotlight.sh --no-wait      # don't poll for reindex completion
#   ./scripts/fix-spotlight.sh --probe word   # change the sanity-check probe word (default: mp4)
#
# Requires: sudo (for mdutil -i / -E), unless --diagnose is passed.

set -uo pipefail

# ---------- args ----------
ERASE=0
DIAGNOSE=0
WAIT_FOR_REINDEX=1
PROBE="mp4"
while [ $# -gt 0 ]; do
    case "$1" in
        --erase)    ERASE=1 ;;
        --diagnose) DIAGNOSE=1 ;;
        --no-wait)  WAIT_FOR_REINDEX=0 ;;
        --probe)    shift; PROBE="${1:-mp4}" ;;
        -h|--help)
            sed -n '2,12p' "$0"; exit 0 ;;
        *) echo "Unknown arg: $1" >&2; exit 2 ;;
    esac
    shift
done

# ---------- logging ----------
LOG_DIR="${HOME}/.cache/script-kit-gpui/spotlight-repair"
mkdir -p "$LOG_DIR"
LOG_FILE="${LOG_DIR}/fix-spotlight-$(date +%Y%m%d-%H%M%S).log"

# colors only if tty
if [ -t 1 ]; then
    C_RESET=$'\033[0m'; C_DIM=$'\033[2m'; C_BOLD=$'\033[1m'
    C_RED=$'\033[31m'; C_GRN=$'\033[32m'; C_YEL=$'\033[33m'
    C_BLU=$'\033[34m'; C_CYA=$'\033[36m'
else
    C_RESET=""; C_DIM=""; C_BOLD=""
    C_RED=""; C_GRN=""; C_YEL=""; C_BLU=""; C_CYA=""
fi

ts()    { date '+%Y-%m-%d %H:%M:%S'; }
log()   { printf '%s %s%s%s\n' "$(ts)" "$C_DIM" "$*" "$C_RESET" | tee -a "$LOG_FILE"; }
info()  { printf '%s %s[INFO]%s %s\n'  "$(ts)" "$C_BLU"  "$C_RESET" "$*" | tee -a "$LOG_FILE"; }
ok()    { printf '%s %s[ OK ]%s %s\n'  "$(ts)" "$C_GRN"  "$C_RESET" "$*" | tee -a "$LOG_FILE"; }
warn()  { printf '%s %s[WARN]%s %s\n'  "$(ts)" "$C_YEL"  "$C_RESET" "$*" | tee -a "$LOG_FILE"; }
err()   { printf '%s %s[FAIL]%s %s\n'  "$(ts)" "$C_RED"  "$C_RESET" "$*" | tee -a "$LOG_FILE"; }
hr()    { printf '%s%s%s\n' "$C_DIM" "------------------------------------------------------------" "$C_RESET" | tee -a "$LOG_FILE"; }
section() {
    hr
    printf '%s%s== %s ==%s\n' "$C_BOLD" "$C_CYA" "$*" "$C_RESET" | tee -a "$LOG_FILE"
    hr
}

# Run a command, log full command + stdout + exit code. Always returns 0 from
# the wrapper itself so set -e doesn't kill us; caller inspects $LAST_EXIT.
LAST_EXIT=0
run() {
    local cmd_str="$*"
    printf '%s %s$%s %s\n' "$(ts)" "$C_DIM" "$C_RESET" "$cmd_str" | tee -a "$LOG_FILE"
    local out
    out="$("$@" 2>&1)"
    LAST_EXIT=$?
    if [ -n "$out" ]; then
        printf '%s\n' "$out" | sed "s/^/$(ts)   | /" | tee -a "$LOG_FILE"
    fi
    printf '%s %s(exit=%s)%s\n' "$(ts)" "$C_DIM" "$LAST_EXIT" "$C_RESET" | tee -a "$LOG_FILE"
    LAST_OUT="$out"
    return 0
}

# Pretty banner
section "Spotlight repair — Script Kit GPUI"
log "log file:           $LOG_FILE"
log "mode:               $([ $DIAGNOSE -eq 1 ] && echo diagnose-only || echo repair)"
log "erase + rebuild:    $([ $ERASE -eq 1 ] && echo yes || echo no)"
log "wait for reindex:   $([ $WAIT_FOR_REINDEX -eq 1 ] && echo yes || echo no)"
log "probe word:         $PROBE"

# ---------- 0. Environment ----------
section "Environment"
run hostname
run sw_vers
run uname -a
run id -un
run df -h /
run df -h /System/Volumes/Data 2>/dev/null || true
run df -h "$HOME"

# ---------- 1. Detect the home volume ----------
section "Detect volume hosting \$HOME"
HOME_VOLUME="$(df -P "$HOME" | awk 'NR==2 {print $NF}')"
info "df reports \$HOME ($HOME) lives on mount point: $HOME_VOLUME"

# On modern macOS, df typically reports "/" because /Users is a firmlink onto
# /System/Volumes/Data. Spotlight is configured per-volume on the *Data*
# volume, so prefer that when it exists.
INDEX_TARGET="$HOME_VOLUME"
if [ -d /System/Volumes/Data ] && [ "$HOME_VOLUME" = "/" ]; then
    INDEX_TARGET="/System/Volumes/Data"
    info "macOS firmlinks /Users onto the Data volume — switching mdutil target to: $INDEX_TARGET"
fi
ok "indexing target: $INDEX_TARGET"

# ---------- 2. BEFORE: capture state ----------
section "BEFORE — capture current Spotlight state"

info "All volumes:"
run mdutil -sa

info "Target volume detail:"
run mdutil -s "$INDEX_TARGET"

info "Probe via mdfind for kMDItemFSName *${PROBE}*:"
# Use a temp file so we get an exact count without subshell pipefail nuance
PROBE_TMP_BEFORE="$(mktemp -t spotlight-before.XXXXXX)"
mdfind "kMDItemFSName == \"*${PROBE}*\"c" >"$PROBE_TMP_BEFORE" 2>>"$LOG_FILE" || true
BEFORE_TOTAL=$(wc -l < "$PROBE_TMP_BEFORE" | tr -d ' ')
BEFORE_USER=$(grep -c "^${HOME}/" "$PROBE_TMP_BEFORE" || true)
BEFORE_SYSTEM=$(grep -c '^/System/' "$PROBE_TMP_BEFORE" || true)
info "  total mdfind hits for *${PROBE}*: ${BEFORE_TOTAL}"
info "  hits under \$HOME ($HOME):        ${BEFORE_USER}"
info "  hits under /System/...:           ${BEFORE_SYSTEM}"
log  "  first 5 hits:"
head -n 5 "$PROBE_TMP_BEFORE" | sed "s/^/$(ts)   | /" | tee -a "$LOG_FILE" || true
rm -f "$PROBE_TMP_BEFORE"

info "Reality on disk (find under \$HOME, depth ≤ 4, skipping noisy dirs):"
ON_DISK=$(find "$HOME" -maxdepth 4 -iname "*${PROBE}*" \
            -not -path "*/Library/*" -not -path "*/.Trash/*" \
            -not -path "*/node_modules/*" -not -path "*/.git/*" \
            2>/dev/null | wc -l | tr -d ' ')
info "  filesystem find count for *${PROBE}*: ${ON_DISK}"

# Decide whether there's a real problem
NEEDS_REPAIR=0
TARGET_STATE="$(mdutil -s "$INDEX_TARGET" 2>&1)"
log "raw target state: $TARGET_STATE"
if printf '%s' "$TARGET_STATE" | grep -qiE 'disabled|unknown|error'; then
    warn "Target volume reports disabled/unknown indexing state."
    NEEDS_REPAIR=1
fi
if [ "$BEFORE_USER" -eq 0 ] && [ "$ON_DISK" -gt 0 ]; then
    warn "mdfind returns 0 user files but $ON_DISK files exist on disk — index is stale or off."
    NEEDS_REPAIR=1
fi
if [ "$NEEDS_REPAIR" -eq 0 ]; then
    ok "Spotlight looks healthy: index enabled and probe returned $BEFORE_USER user-file hits."
fi

# ---------- 3. Diagnose-only stops here ----------
if [ $DIAGNOSE -eq 1 ]; then
    section "Diagnose-only run — stopping before any sudo action"
    info "Re-run without --diagnose to repair."
    exit 0
fi

if [ $NEEDS_REPAIR -eq 0 ] && [ $ERASE -eq 0 ]; then
    section "No repair needed"
    ok "Nothing to do. Re-run with --erase to force a full rebuild anyway."
    exit 0
fi

# ---------- 4. sudo preflight ----------
section "Preflight — request sudo"
info "About to run privileged commands. You will be prompted for your password."
info "Commands that will run:"
log "  sudo mdutil -i on $INDEX_TARGET"
[ $ERASE -eq 1 ] && log "  sudo mdutil -E  $INDEX_TARGET   (erase + rebuild)"
log "  sudo mdutil -s $INDEX_TARGET   (post-check)"
echo
if ! sudo -v; then
    err "sudo authentication failed. Aborting."
    exit 1
fi
ok "sudo authenticated; cached for this session."

# Keep sudo alive in the background while we work.
( while true; do sudo -n true; sleep 50; kill -0 "$$" 2>/dev/null || exit; done ) >/dev/null 2>&1 &
SUDO_KEEPALIVE_PID=$!
trap 'kill $SUDO_KEEPALIVE_PID 2>/dev/null || true' EXIT

# ---------- 5. Enable indexing ----------
section "Enable indexing on $INDEX_TARGET"
run sudo mdutil -i on "$INDEX_TARGET"
if [ "$LAST_EXIT" -ne 0 ]; then
    err "mdutil -i on failed (exit=$LAST_EXIT). Continuing for visibility but reindex may fail."
fi

# ---------- 6. Optional erase + rebuild ----------
if [ $ERASE -eq 1 ]; then
    section "Erase and rebuild index on $INDEX_TARGET"
    warn "This wipes the existing index; first searches afterward will be slow."
    run sudo mdutil -E "$INDEX_TARGET"
    if [ "$LAST_EXIT" -ne 0 ]; then
        err "mdutil -E failed (exit=$LAST_EXIT)."
    fi
fi

# ---------- 7. Confirm and (optionally) wait ----------
section "Post-action state"
run mdutil -s "$INDEX_TARGET"
run mdutil -sa

if [ $WAIT_FOR_REINDEX -eq 1 ]; then
    section "Watching reindex progress"
    info "Polling mdfind for *${PROBE}* under \$HOME every 15s."
    info "Will exit when ≥1 user-file hit appears, or after 60 minutes."
    DEADLINE=$(( $(date +%s) + 60 * 60 ))
    POLL=0
    LAST_HITS=-1
    while :; do
        POLL=$((POLL + 1))
        NOW=$(date +%s)
        if [ "$NOW" -ge "$DEADLINE" ]; then
            warn "Hit 60-minute poll cap. Reindex may still be running in the background."
            break
        fi
        HITS=$(mdfind -onlyin "$HOME" "kMDItemFSName == \"*${PROBE}*\"c" 2>/dev/null | wc -l | tr -d ' ')
        STATE=$(mdutil -s "$INDEX_TARGET" 2>&1 | tr -d '\n\t')
        if [ "$HITS" != "$LAST_HITS" ]; then
            info "[poll #$POLL] user-file hits=$HITS  state=\"$STATE\""
            LAST_HITS=$HITS
        else
            log  "[poll #$POLL] user-file hits=$HITS  state=\"$STATE\""
        fi
        if [ "$HITS" -gt 0 ]; then
            ok "Index is producing user results. Done waiting."
            break
        fi
        sleep 15
    done
fi

# ---------- 8. AFTER: summary ----------
section "AFTER — summary"
PROBE_TMP_AFTER="$(mktemp -t spotlight-after.XXXXXX)"
mdfind "kMDItemFSName == \"*${PROBE}*\"c" >"$PROBE_TMP_AFTER" 2>>"$LOG_FILE" || true
AFTER_TOTAL=$(wc -l < "$PROBE_TMP_AFTER" | tr -d ' ')
AFTER_USER=$(grep -c "^${HOME}/" "$PROBE_TMP_AFTER" || true)
rm -f "$PROBE_TMP_AFTER"

info "Probe (*${PROBE}*) — before vs after:"
log  "  total mdfind hits:   ${BEFORE_TOTAL}  ->  ${AFTER_TOTAL}"
log  "  hits under \$HOME:    ${BEFORE_USER}   ->  ${AFTER_USER}"
log  "  files actually on disk (depth ≤ 4): ${ON_DISK}"

if [ "$AFTER_USER" -gt 0 ]; then
    ok  "Spotlight is now returning user-file results. Restart Script Kit (or just retype the query) to see them in the launcher."
elif [ $WAIT_FOR_REINDEX -eq 0 ]; then
    info "Reindex started; rerun this script with --diagnose later to recheck."
else
    warn "Probe still returns 0 user hits. Possible causes:"
    log  "  - reindex is still running (can take hours on large volumes)"
    log  "  - $INDEX_TARGET is in System Settings → Spotlight → Search Privacy"
    log  "  - Full Disk Access not granted to /System/Library/CoreServices/Spotlight"
fi

ok  "Full log: $LOG_FILE"
