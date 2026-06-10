#!/bin/bash

# Dev runner script for script-kit-gpui
# Uses cargo-watch to auto-rebuild on Rust file changes.
#
# Visibility: prints a banner at t=0, then a per-iteration heartbeat
# (build/relaunch elapsed seconds) via scripts/agentic/dev-cycle.sh so the
# loop never looks frozen. Screen clearing is opt-in (SCRIPT_KIT_DEV_CLEAR=1).
#
# Log mode:
#   Defaults to SCRIPT_KIT_AI_LOG=1 (compact AI format: SS.mmm|L|C|message)
#   Override with: SCRIPT_KIT_AI_LOG=0 ./dev.sh   (standard verbose logs)
#   Or use:        RUST_LOG=debug ./dev.sh         (debug-level verbose logs)

set -e

# --- Signal cleanup: one Ctrl+C must stop cargo-watch and all helper children ---
SCRIPT_KIT_DEV_CACHE_PID=""
SCRIPT_KIT_DEV_WATCHDOG_PID=""
SCRIPT_KIT_DEV_PI_SIDECAR_PID=""
DEV_SH_CLEANED_UP=0
DEV_SH_EXIT_CODE=0
dev_sh_cleanup() {
    if [ "$DEV_SH_CLEANED_UP" = "1" ]; then
        return 0
    fi
    DEV_SH_CLEANED_UP=1
    trap - INT TERM EXIT

    if [ -n "${SCRIPT_KIT_DEV_CACHE_PID:-}" ]; then
        kill "$SCRIPT_KIT_DEV_CACHE_PID" 2>/dev/null || true
        wait "$SCRIPT_KIT_DEV_CACHE_PID" 2>/dev/null || true
    fi

    if [ -n "${SCRIPT_KIT_DEV_WATCHDOG_PID:-}" ]; then
        kill "$SCRIPT_KIT_DEV_WATCHDOG_PID" 2>/dev/null || true
        wait "$SCRIPT_KIT_DEV_WATCHDOG_PID" 2>/dev/null || true
    fi

    if [ -n "${SCRIPT_KIT_DEV_PI_SIDECAR_PID:-}" ]; then
        kill "$SCRIPT_KIT_DEV_PI_SIDECAR_PID" 2>/dev/null || true
        wait "$SCRIPT_KIT_DEV_PI_SIDECAR_PID" 2>/dev/null || true
    fi

    # Stop cargo-watch, dev-cycle, and any in-flight cargo build.
    pkill -TERM -P "$$" 2>/dev/null || true
    sleep 0.2
    pkill -KILL -P "$$" 2>/dev/null || true

    rm -f "$SCRIPT_KIT_DEV_STAMP_FILE" 2>/dev/null || true

    if [ "$DEV_SH_EXIT_CODE" -ne 0 ]; then
        exit "$DEV_SH_EXIT_CODE"
    fi
}
dev_sh_on_signal() {
    local sig="$1"
    case "$sig" in
        INT) DEV_SH_EXIT_CODE=130 ;;
        TERM) DEV_SH_EXIT_CODE=143 ;;
    esac
    dev_sh_cleanup
}
trap 'dev_sh_on_signal INT' INT
trap 'dev_sh_on_signal TERM' TERM
trap dev_sh_cleanup EXIT

# --- Banner FIRST so the user sees activity within ~1s, before any du scan ---
echo "[dev.sh] start t=$(date '+%Y-%m-%dT%H:%M:%S%z') pid=$$"
echo "[dev.sh] First build may take several minutes; subsequent rebuilds are incremental."
echo "[dev.sh] Build output is visible (--message-format=short). Press Ctrl+C to stop."
echo ""

# Default to compact AI log mode unless explicitly overridden
export SCRIPT_KIT_AI_LOG="${SCRIPT_KIT_AI_LOG:-1}"

# Dev startup profile: optimize for time-to-usable-session during cargo-watch loops.
export SCRIPT_KIT_STARTUP_PROFILE="${SCRIPT_KIT_STARTUP_PROFILE:-dev-fast}"
export SCRIPT_KIT_DEFER_SCHEDULER_STARTUP="${SCRIPT_KIT_DEFER_SCHEDULER_STARTUP:-1}"
export SCRIPT_KIT_STARTUP_READY_LOG="${SCRIPT_KIT_STARTUP_READY_LOG:-1}"
export SCRIPT_KIT_DISABLE_AGENT_CHAT_HOT_PREWARM="${SCRIPT_KIT_DISABLE_AGENT_CHAT_HOT_PREWARM:-0}"
export SCRIPT_KIT_DISABLE_QUICK_TERMINAL_WARM_PTY="${SCRIPT_KIT_DISABLE_QUICK_TERMINAL_WARM_PTY:-1}"

# Agentic session name: dev.sh launches through the reusable session contract.
export SCRIPT_KIT_DEV_SESSION_NAME="${SCRIPT_KIT_DEV_SESSION_NAME:-dev-watch}"
SESSION_DIR_RAW="${SCRIPT_KIT_SESSION_DIR:-/tmp/sk-agentic-sessions}"
mkdir -p "$SESSION_DIR_RAW"
export SCRIPT_KIT_SESSION_DIR="$(cd "$SESSION_DIR_RAW" && pwd -P)"

# --- Launcher self-update stamp ---------------------------------------------
# Record a digest of the launcher + helper scripts at start. dev-cycle.sh will
# compare this against the live files each iteration and suggest a restart if
# they have changed under the running cargo-watch process.
SCRIPT_KIT_DEV_STAMP_DIR="${TMPDIR:-/tmp}/sk-dev-launcher-stamps"
mkdir -p "$SCRIPT_KIT_DEV_STAMP_DIR"
SCRIPT_KIT_DEV_STAMP_FILE="${SCRIPT_KIT_DEV_STAMP_DIR}/$$.stamp"
shasum_of() {
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 1 "$@" 2>/dev/null | awk '{print $1}' | paste -sd, -
    else
        md5 -q "$@" 2>/dev/null | paste -sd, -
    fi
}
shasum_of dev.sh scripts/agentic/dev-cycle.sh scripts/agentic/dev-relaunch.sh > "$SCRIPT_KIT_DEV_STAMP_FILE"
export SCRIPT_KIT_DEV_STAMP_FILE

# --- Async cache-size reporter -----------------------------------------------
# Reporting target/ size synchronously can take 10+ seconds with a 77 GB
# target/, which is the bulk of the "frozen" silence. Run it in the background
# so the heartbeat starts immediately. Result lands on stderr when ready.
SCRIPT_KIT_TARGET_CLEAN_THRESHOLD_GB="${SCRIPT_KIT_TARGET_CLEAN_THRESHOLD_GB:-50}"
SCRIPT_KIT_TARGET_AGENT_THRESHOLD_GB="${SCRIPT_KIT_TARGET_AGENT_THRESHOLD_GB:-50}"
if [ "${SCRIPT_KIT_REPORT_CACHE_SIZE:-1}" = "1" ]; then
    (
        humanize_kib() {
            local kib="$1"
            awk -v k="$kib" 'BEGIN{
                u="K"; v=k+0;
                if (v>=1024){v/=1024;u="M"};
                if (v>=1024){v/=1024;u="G"};
                if (v>=1024){v/=1024;u="T"};
                printf("%.1f%s", v, u);
            }'
        }
        if [ -d target ]; then
            target_kib="$(du -sk target 2>/dev/null | awk '{print $1}')"
            inc_kib="$(du -sk target/debug/incremental 2>/dev/null | awk '{print $1}')"
            target_human="$(humanize_kib "${target_kib:-0}")"
            inc_human="$(humanize_kib "${inc_kib:-0}")"
            echo "[dev.sh] cache target=${target_human} incremental=${inc_human}" >&2

            # SUGGEST only — never auto-clean from dev.sh; that forces a cold
            # rebuild with no progress. Use prune-cargo-targets.sh instead.
            if [[ "$SCRIPT_KIT_TARGET_CLEAN_THRESHOLD_GB" =~ ^[0-9]+$ ]] && [ "$SCRIPT_KIT_TARGET_CLEAN_THRESHOLD_GB" -gt 0 ]; then
                threshold_kib=$((SCRIPT_KIT_TARGET_CLEAN_THRESHOLD_GB * 1024 * 1024))
                if [ -n "$target_kib" ] && [ "$target_kib" -gt "$threshold_kib" ]; then
                    echo "[dev.sh] SUGGEST target/ is ${target_human} (>${SCRIPT_KIT_TARGET_CLEAN_THRESHOLD_GB}G) — run: scripts/agentic/prune-cargo-targets.sh --apply" >&2
                fi
            fi
        fi
        if [ -d target-agent ]; then
            agent_kib="$(du -sk target-agent 2>/dev/null | awk '{print $1}')"
            agent_human="$(humanize_kib "${agent_kib:-0}")"
            echo "[dev.sh] cache target-agent=${agent_human}" >&2
            pools_kib="$(du -sk target-agent/pools 2>/dev/null | awk '{print $1}')"
            runtime_kib="$(du -sk target-agent/runtime 2>/dev/null | awk '{print $1}')"
            [ -n "$pools_kib" ] && echo "[dev.sh] cache target-agent-pools=$(humanize_kib "$pools_kib")" >&2
            [ -n "$runtime_kib" ] && echo "[dev.sh] cache target-agent-runtime=$(humanize_kib "$runtime_kib")" >&2
            if [[ "$SCRIPT_KIT_TARGET_AGENT_THRESHOLD_GB" =~ ^[0-9]+$ ]] && [ "$SCRIPT_KIT_TARGET_AGENT_THRESHOLD_GB" -gt 0 ]; then
                threshold_kib=$((SCRIPT_KIT_TARGET_AGENT_THRESHOLD_GB * 1024 * 1024))
                if [ -n "$agent_kib" ] && [ "$agent_kib" -gt "$threshold_kib" ]; then
                    echo "[dev.sh] SUGGEST target-agent/ is ${agent_human} (>${SCRIPT_KIT_TARGET_AGENT_THRESHOLD_GB}G) — run: scripts/agentic/prune-cargo-targets.sh --apply" >&2
                fi
            fi
        fi
    ) &
    SCRIPT_KIT_DEV_CACHE_PID=$!
fi

# --- Pi sidecar availability ---------------------------------------------------
# Dev runs execute the bare target binary, so the bundled Contents/MacOS/pi
# never resolves. Make sure a Pi binary exists for Agent Chat (cmd+enter):
# fast no-op when one already resolves, otherwise build the repo-local sidecar
# in the background (concurrent with the first cargo build).
if [ "${SCRIPT_KIT_DEV_ENSURE_PI_SIDECAR:-1}" = "1" ]; then
    bash scripts/agentic/ensure-pi-sidecar.sh &
    SCRIPT_KIT_DEV_PI_SIDECAR_PID=$!
fi

# --- Crash watchdog -----------------------------------------------------------
# Supervise the session app pid: loud banner + auto-relaunch on abnormal death,
# incremental-cache wipe on a repeat crash of the same binary, and a stop-and-
# report banner when a clean rebuild still crashes (real bug, not cache rot).
if [ "${SCRIPT_KIT_DEV_CRASH_WATCHDOG:-1}" = "1" ]; then
    bash scripts/agentic/dev-crash-watchdog.sh "$SCRIPT_KIT_DEV_SESSION_NAME" &
    SCRIPT_KIT_DEV_WATCHDOG_PID=$!
fi

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "cargo-watch is not installed"
    echo ""
    echo "Install it with:"
    echo "  cargo install cargo-watch"
    echo ""
    exit 1
fi

echo "[dev.sh] cargo-watch ready"
echo "   Watching: src/, scripts/kit-sdk.ts, Cargo.toml, build.rs"
if [ "$SCRIPT_KIT_AI_LOG" = "1" ]; then
    echo "   Log mode: compact AI (SS.mmm|L|C|message). Override: SCRIPT_KIT_AI_LOG=0 ./dev.sh"
else
    echo "   Log mode: standard verbose"
fi
echo "   Agentic session: ${SCRIPT_KIT_DEV_SESSION_NAME}"
echo "   Startup profile: ${SCRIPT_KIT_STARTUP_PROFILE}"
echo "   Quick Terminal warm PTY: disabled"
echo "   Cargo dev profile: debug=0 incremental=true codegen-units=256"
echo "   Build target: script-kit-gpui only (skips smoke-test, vibrancy-poc, menu-syntax-doctor)"
echo "   Session log: ~/.scriptkit/logs/latest-session.jsonl"
echo "   Clear screen between rebuilds: SCRIPT_KIT_DEV_CLEAR=${SCRIPT_KIT_DEV_CLEAR:-0} (set =1 to enable cargo-watch -c)"
echo "   Crash watchdog: SCRIPT_KIT_DEV_CRASH_WATCHDOG=${SCRIPT_KIT_DEV_CRASH_WATCHDOG:-1} (banner + auto-relaunch on app crash)"
echo ""

# Clear-screen is opt-in. cargo-watch -c wipes the heartbeat output, so default
# it to OFF and let the user opt in with SCRIPT_KIT_DEV_CLEAR=1.
cargo_watch_args=()
if [ "${SCRIPT_KIT_DEV_CLEAR:-0}" = "1" ] && [ -t 1 ] && [ -n "${TERM:-}" ] && [ "${TERM}" != "dumb" ]; then
    cargo_watch_args+=(-c)
fi

# --no-restart prevents killing an in-flight build when the user saves again
# mid-compile. Debounce coalesces rapid save bursts.
cargo_watch_args+=(--no-restart -d "${SCRIPT_KIT_CARGO_WATCH_DELAY:-1.0}")

# Delegate the build+relaunch iteration to dev-cycle.sh so we get:
#   - timestamped "build start"/"build done in Xs"
#   - per-second heartbeat with elapsed seconds
#   - relaunch elapsed + ready marker
#   - skip-relaunch when binary mtime unchanged and session is healthy
cargo watch "${cargo_watch_args[@]}" \
    -s "bash scripts/agentic/dev-cycle.sh" \
    -w src/ \
    -w scripts/kit-sdk.ts \
    -w Cargo.toml \
    -w Cargo.lock \
    -w build.rs \
    -i 'src/bin/storybook.rs' \
    -i 'src/bin/smoke-test.rs' \
    -i 'src/storybook/*' \
    -i 'src/stories/*' \
    -i 'src/*_tests.rs' \
    -i 'tests/*' \
    -i '*.md' \
    -i 'docs/*' \
    -i 'expert-bundles/*' \
    -i 'audit-docs/*' \
    -i 'audits/*' \
    -i '.test-screenshots/*' \
    -i 'test-screenshots/*' \
    -i '.hive/*' \
    -i '.mocks/*' \
    -i 'storybook.sh' \
    -i 'tasks/*' \
    -i 'plan/*' \
    -i 'security-audit/*' \
    -i 'ai/*' \
    -i 'hooks/*' \
    -i 'kit-init/*' \
    -i 'rules/*'
