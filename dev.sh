#!/bin/bash

# Dev runner script for script-kit-gpui
# Uses cargo-watch to auto-rebuild on Rust file changes
# Clears screen between rebuilds for clean output
#
# Only watches files that are actually included in the main app binary.
# Ignores: storybook, stories, tests, benchmarks, docs, etc.

set -e

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "❌ cargo-watch is not installed"
    echo ""
    echo "Install it with:"
    echo "  cargo install cargo-watch"
    echo ""
    exit 1
fi

echo "🚀 Starting dev runner with cargo-watch..."
echo "   Watching: src/ (excluding storybook/stories), scripts/kit-sdk.ts, Cargo.toml, build.rs"
echo "   Ignoring: tests/, storybook, stories, docs, benchmarks, .md files"
echo "   Press Ctrl+C to stop"
echo ""

# Run cargo watch with auto-rebuild
# -x run: Execute 'cargo run' on file changes
# -c: Clear screen between runs for cleaner output
# -w: Only watch specific directories (disables auto-discovery)
# -i: Ignore patterns that shouldn't trigger rebuilds
cargo watch -c -x run \
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
