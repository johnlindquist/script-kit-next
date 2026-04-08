# Fast dev commands — `make test` runs 10k+ tests in ~12s via nextest

.PHONY: test check lint verify ship-check run bundle test-all test-system test-slow smoke-main-menu

# Default: fast parallel tests (nextest)
test:
	cargo nextest run --lib

# Compilation check
check:
	cargo check

# Lint (lib only, deny warnings)
lint:
	cargo clippy --lib -- -D warnings

# Canonical validation gate used by release tags
verify:
	bash scripts/verify.sh --skip-bundle

# Full local ship gate (validation + bundle sanity)
ship-check:
	bash scripts/verify.sh

# Run the app
run:
	echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Release bundle (explicit binary target — matches CI)
bundle:
	cargo bundle --release --bin script-kit-gpui

# All tests including ignored system tests
test-all:
	cargo nextest run --lib --run-ignored all

# System tests only (require permissions/hardware)
test-system:
	cargo test --features system-tests

# Slow tests only
test-slow:
	cargo test --features slow-tests

# Plugin/skill smoke: discovery → search → ACP staging contract
smoke-main-menu:
	cargo nextest run --test smoke_main_menu --test plugin_inventory --test plugin_skill_search --test plugin_skill_launch --test plugin_skill_main_menu --test agent_workspace_contract
