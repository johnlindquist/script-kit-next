# Fast dev commands — `make test` runs 10k+ tests in ~12s via nextest

.PHONY: test check lint verify run bundle test-all test-system test-slow

# Default: fast parallel tests (nextest)
test:
	cargo nextest run --lib

# Compilation check
check:
	cargo check

# Lint (lib only, deny warnings)
lint:
	cargo clippy --lib -- -D warnings

# Full verification gate (check + lint + test)
verify: check lint test

# Run the app
run:
	echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Release bundle
bundle:
	cargo bundle --release

# All tests including ignored system tests
test-all:
	cargo nextest run --lib --run-ignored all

# System tests only (require permissions/hardware)
test-system:
	cargo test --features system-tests

# Slow tests only
test-slow:
	cargo test --features slow-tests
