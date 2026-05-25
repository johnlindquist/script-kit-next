#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="${SCRIPT_KIT_REPO_ROOT:-/Users/johnlindquist/dev/script-kit-gpui}"
STATE_DIR="${SCRIPT_KIT_WATCHER_STATE_DIR:-$HOME/Library/Application Support/script-kit-gpui/disk-space-cargo-watcher}"
THRESHOLD_GIB="${SCRIPT_KIT_FREE_THRESHOLD_GIB:-25}"
TARGET_FREE_GIB="${SCRIPT_KIT_TARGET_FREE_GIB:-35}"
APPLY=0
REASON="manual"

while [ "$#" -gt 0 ]; do
    case "$1" in
        --apply) APPLY=1; shift ;;
        --repo) REPO_ROOT="$2"; shift 2 ;;
        --state-dir) STATE_DIR="$2"; shift 2 ;;
        --threshold-gib) THRESHOLD_GIB="$2"; shift 2 ;;
        --target-free-gib) TARGET_FREE_GIB="$2"; shift 2 ;;
        --reason) REASON="$2"; shift 2 ;;
        --help|-h)
            cat <<EOF
Usage: $0 --apply --repo /path/to/repo --threshold-gib 25 --target-free-gib 35 --state-dir /path/to/state
EOF
            exit 0
            ;;
        *) echo "[cargo-clean] unknown argument: $1" >&2; exit 2 ;;
    esac
done

PATH="/Users/johnlindquist/.local/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
export PATH

cd "$REPO_ROOT"

log() { echo "[cargo-clean] $(date '+%Y-%m-%dT%H:%M:%S%z') $*" >&2; }

free_gib() {
    df -Pk "$REPO_ROOT" | awk 'NR==2 { printf "%.1f", $4 / 1048576 }'
}

ge_float() { awk -v a="$1" -v b="$2" 'BEGIN { exit(a >= b ? 0 : 1) }'; }
lt_float() { awk -v a="$1" -v b="$2" 'BEGIN { exit(a < b ? 0 : 1) }'; }

show_report() {
    log "free=$(free_gib)GiB threshold=${THRESHOLD_GIB}GiB target=${TARGET_FREE_GIB}GiB"
    du -sh \
        target \
        target/debug \
        target/debug/incremental \
        target-agent \
        target-agent/pools \
        target-agent/agents \
        target-agent/runtime \
        2>/dev/null || true
    if [ -d target-agent/.locks ]; then
        log "agent locks:"
        find target-agent/.locks -mindepth 1 -maxdepth 2 -type f -name pid \
            -print -exec sh -c 'printf "  "; cat "$1"; printf "\n"' sh {} \; \
            2>/dev/null || true
    fi
}

run_prune() {
    local prune_time_days="$1"
    local prune_agent_days="$2"
    local prune_incremental_days="$3"

    if [ ! -x ./scripts/agentic/prune-cargo-targets.sh ]; then
        log "missing executable ./scripts/agentic/prune-cargo-targets.sh"
        return 1
    fi

    if [ "$APPLY" = "1" ]; then
        log "running prune apply PRUNE_TIME_DAYS=${prune_time_days} PRUNE_AGENT_DAYS=${prune_agent_days} PRUNE_INCREMENTAL_DAYS=${prune_incremental_days}"
        PRUNE_TIME_DAYS="$prune_time_days" \
        PRUNE_AGENT_DAYS="$prune_agent_days" \
        PRUNE_INCREMENTAL_DAYS="$prune_incremental_days" \
            ./scripts/agentic/prune-cargo-targets.sh --apply || true
    else
        log "dry-run prune only; pass --apply to delete"
        PRUNE_TIME_DAYS="$prune_time_days" \
        PRUNE_AGENT_DAYS="$prune_agent_days" \
        PRUNE_INCREMENTAL_DAYS="$prune_incremental_days" \
            ./scripts/agentic/prune-cargo-targets.sh || true
    fi
}

command_for_pid() { ps -o command= -p "$1" 2>/dev/null | head -n 1 || true; }
comm_for_pid() {
    local raw
    raw="$(ps -o comm= -p "$1" 2>/dev/null | head -n 1 || true)"
    basename "$raw" 2>/dev/null || printf "%s" "$raw"
}

pid_cwd_under_repo() {
    local pid="$1"
    local cwd
    cwd="$(lsof -a -p "$pid" -d cwd -Fn 2>/dev/null | sed -n 's/^n//p' | head -n 1 || true)"
    case "$cwd" in
        "$REPO_ROOT"|"$REPO_ROOT"/*) return 0 ;;
        *) return 1 ;;
    esac
}

collect_tree() {
    local root="$1"
    local child
    echo "$root"
    for child in $(pgrep -P "$root" 2>/dev/null || true); do
        collect_tree "$child"
    done
}

terminate_tree() {
    local root="$1"
    local why="$2"
    local pids p cmd
    [ -n "$root" ] || return 0
    case "$root" in "$$"|"$PPID") return 0 ;; esac
    if ! kill -0 "$root" 2>/dev/null; then return 0; fi
    cmd="$(command_for_pid "$root")"
    log "terminating pid=$root why=$why cmd=${cmd}"
    pids="$(collect_tree "$root" | awk 'NF && !seen[$1]++' | sort -rn)"

    if [ "$APPLY" != "1" ]; then
        log "dry-run would TERM tree: $(echo "$pids" | tr '\n' ' ')"
        return 0
    fi

    for p in $pids; do
        case "$p" in "$$"|"$PPID") continue ;; esac
        kill -TERM "$p" 2>/dev/null || true
    done
    sleep 4
    for p in $pids; do
        case "$p" in "$$"|"$PPID") continue ;; esac
        if kill -0 "$p" 2>/dev/null; then
            kill -KILL "$p" 2>/dev/null || true
        fi
    done
}

terminate_agent_lock_holders() {
    local pid_file pid lock_name
    shopt -s nullglob
    for pid_file in "$REPO_ROOT"/target-agent/.locks/*.lock/pid; do
        pid="$(tr -dc '0-9' < "$pid_file" || true)"
        lock_name="$(basename "$(dirname "$pid_file")")"
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            terminate_tree "$pid" "target-agent lock holder $lock_name"
        fi
    done
    shopt -u nullglob
}

should_kill_pid() {
    local pid="$1" cmd comm
    case "$pid" in "$$"|"$PPID") return 1 ;; esac
    cmd="$(command_for_pid "$pid")"
    [ -n "$cmd" ] || return 1
    case "$cmd" in *disk-space-cargo*) return 1 ;; esac
    comm="$(comm_for_pid "$pid")"

    case "$cmd" in
        *"$REPO_ROOT/dev.sh"*|*" ./dev.sh"*|*"scripts/agentic/dev-cycle.sh"*|*"scripts/agentic/agent-cargo.sh"*)
            if pid_cwd_under_repo "$pid" || [[ "$cmd" == *"$REPO_ROOT"* ]]; then return 0; fi
            ;;
    esac
    case "$comm" in
        cargo-watch|cargo|rustc)
            if pid_cwd_under_repo "$pid"; then return 0; fi
            ;;
    esac
    case "$cmd" in
        *"cargo watch"*)
            if pid_cwd_under_repo "$pid" || [[ "$cmd" == *"$REPO_ROOT"* ]]; then return 0; fi
            ;;
    esac
    return 1
}

discover_known_dev_cargo_pids() {
    local pid
    ps -axo pid= | while read -r pid; do
        [ -n "$pid" ] || continue
        if should_kill_pid "$pid"; then echo "$pid"; fi
    done | sort -u
}

terminate_known_dev_cargo_processes() {
    local pid
    log "looking for Script Kit dev/cargo processes using repo"
    for pid in $(discover_known_dev_cargo_pids); do
        terminate_tree "$pid" "repo dev/cargo process"
    done
}

emergency_delete_agent_targets() {
    if [ ! -d target-agent ]; then return 0; fi
    log "emergency deleting target-agent subdirectories except .locks"
    if [ "$APPLY" = "1" ]; then
        find target-agent -mindepth 1 -maxdepth 1 -type d ! -name ".locks" -print -exec rm -rf {} +
        mkdir -p target-agent/pools target-agent/agents target-agent/.locks
    else
        find target-agent -mindepth 1 -maxdepth 1 -type d ! -name ".locks" -print
    fi
}

emergency_delete_incremental() {
    if [ ! -d target/debug/incremental ]; then return 0; fi
    log "emergency deleting target/debug/incremental contents"
    if [ "$APPLY" = "1" ]; then
        find target/debug/incremental -mindepth 1 -maxdepth 1 -print -exec rm -rf {} +
    else
        find target/debug/incremental -mindepth 1 -maxdepth 1 -print
    fi
}

# --- Main ---

log "start reason=${REASON} repo=${REPO_ROOT} apply=${APPLY}"
show_report

# Phase 1: normal prune
run_prune 14 7 14

if ge_float "$(free_gib)" "$TARGET_FREE_GIB"; then
    log "target free reached after normal prune"
    show_report
    exit 0
fi

# Phase 2: terminate dev processes + aggressive prune
log "still below target free after normal prune; terminating known dev cargo processes"
terminate_agent_lock_holders
terminate_known_dev_cargo_processes
run_prune 3 1 3

if ge_float "$(free_gib)" "$TARGET_FREE_GIB"; then
    log "target free reached after aggressive prune"
    show_report
    exit 0
fi

# Phase 3: emergency delete bounded cache dirs
log "still below target free; deleting bounded cargo cache directories"
emergency_delete_agent_targets
emergency_delete_incremental
run_prune 1 0 1

show_report

if lt_float "$(free_gib)" "$THRESHOLD_GIB"; then
    log "free disk remains below threshold after cleanup"
    exit 2
fi

log "cleanup complete"
exit 0
