#!/bin/bash

# Dev runner script for script-kit-gpui
# Uses cargo-watch to auto-rebuild on Rust file changes
# Clears screen between rebuilds for clean output
#
# Only watches files that are actually included in the main app binary.
# Ignores: storybook, stories, tests, benchmarks, docs, etc.
#
# Log mode:
#   Defaults to SCRIPT_KIT_AI_LOG=1 (compact AI format: SS.mmm|L|C|message)
#   Override with: SCRIPT_KIT_AI_LOG=0 ./dev.sh   (standard verbose logs)
#   Or use:        RUST_LOG=debug ./dev.sh         (debug-level verbose logs)

set -e

# Default to compact AI log mode unless explicitly overridden
export SCRIPT_KIT_AI_LOG="${SCRIPT_KIT_AI_LOG:-1}"

# Dev startup profile: optimize for time-to-usable-session during cargo-watch loops.
# Deferred services still start shortly after readiness, so behavior stays intact.
export SCRIPT_KIT_STARTUP_PROFILE="${SCRIPT_KIT_STARTUP_PROFILE:-dev-fast}"
export SCRIPT_KIT_DEFER_SCHEDULER_STARTUP="${SCRIPT_KIT_DEFER_SCHEDULER_STARTUP:-1}"
export SCRIPT_KIT_STARTUP_READY_LOG="${SCRIPT_KIT_STARTUP_READY_LOG:-1}"

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "cargo-watch is not installed"
    echo ""
    echo "Install it with:"
    echo "  cargo install cargo-watch"
    echo ""
    exit 1
fi

echo "Starting dev runner with cargo-watch..."
echo "   Watching: src/, scripts/kit-sdk.ts, Cargo.toml, build.rs"
if [ "$SCRIPT_KIT_AI_LOG" = "1" ]; then
    echo "   Log mode: compact AI (SS.mmm|L|C|message). Override: SCRIPT_KIT_AI_LOG=0 ./dev.sh"
else
    echo "   Log mode: standard verbose"
fi
echo "   Startup profile: ${SCRIPT_KIT_STARTUP_PROFILE}"
echo "   Cargo dev profile: debug=0 incremental=true codegen-units=256"
echo "   Session log: ~/.scriptkit/logs/latest-session.jsonl"
echo "   Copy for AI: cat ~/.scriptkit/logs/latest-session.jsonl | pbcopy"
echo "   Press Ctrl+C to stop"
echo ""

# Only request clear-screen behavior in an interactive terminal that can support it.
cargo_watch_args=()
if [ -t 1 ] && [ -n "${TERM:-}" ] && [ "${TERM}" != "dumb" ]; then
    cargo_watch_args+=(-c)
fi

# IMPORTANT:
# Launch speed for autonomous agents is dominated by "time to usable session",
# not just Rust startup time. Keep watch scope narrow and avoid adding extra
# pre-run shell work here unless it directly improves readiness or rebuild time.
#
# Run cargo watch with auto-rebuild
# -x run: Execute 'cargo run' on file changes
# -c: Clear screen between runs for cleaner output
# -w: Only watch specific directories (disables auto-discovery)
# -i: Ignore patterns that shouldn't trigger rebuilds
cargo watch "${cargo_watch_args[@]}" -x run \
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
