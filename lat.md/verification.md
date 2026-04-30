# Verification

This repo prefers the smallest runtime-backed verification that proves a change. UI work should verify the real surface; logic work should stay on the narrowest relevant checks.

## Main menu and footer

`make smoke-main-menu` is the repo's fast launcher and footer smoke target. Use it for main window, footer, built-in menu, and plugin-skill routing changes.

## Targeted checks

Use the smallest check that exercises the touched code:

- `make check` or `cargo check` for compile validation
- `lat check` for lattice, markdown, or validation-contract changes
- `make lint` or `cargo clippy --lib -- -D warnings` for lint-sensitive Rust changes
- `make test` or `cargo nextest run --lib` for library changes
- `make test-all`, `make test-system`, or `make test-slow` only when the touched area justifies them
- Autonomous loop verify commands should avoid known-red repo-wide suites; prefer compile validation plus docs such as `cargo check --lib && lat check`, then add the narrowest relevant tests or runtime proof separately.
- `SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui` for direct runtime inspection when you need the app open
- `bun scripts/agentic/filterable-surface-matrix.ts --session <name>` for migrated filterable launcher surfaces whose `getState.visibleChoiceCount` must match `getElements` list rows.
- Tray-opened current-app command work should use the real agentic runtime path and cover app-switch mid-interaction, same-bundle relaunches that change PID, PID-aware tracker cache invalidation and republish, cold live captures that must discard stale results and retry boundedly, refresh-on-filter, guarded execution, and the existing empty/no-match states.

Shell helper changes outside the Rust app should keep their proof narrow too. For zsh helpers such as `cpath`, prefer the dedicated sourced-shell test plus one real-shell smoke check that covers raw paths, directory-plus-term search like `cpath .notes scroll`, and `ls -l` or `eza -l` style listing input before calling the work done.

For UI changes outside the main launcher/footer path, use the project's agentic runtime verification flow against the real surface instead of guessing from unit tests alone.

## Release gates

`make verify` is the broad validation gate. Use it for release work, CI debugging, or when the change touches shared build/test infrastructure.

The gate runs `lat check` before compile, lint, Rust tests, and SDK tests so broken lattice links or missing code references fail with the same priority as source validation. CI also runs a dedicated lattice job and no longer ignores markdown-only changes.

`make ship-check` is human-only release validation and should not be run by an AI agent.

## Default nextest profile

The default nextest profile is the CI fast lane; it excludes system-dependent and known-stale source-contract suites until those contracts are refreshed.

The filter lives in [.config/nextest.toml](../.config/nextest.toml). Keep newly stale generated contract suites out of the default profile only when they are already failing on main or block unrelated build health; prefer updating the contract tests when the behavior itself changed.

The CI Rust test job installs the repo-pinned Bun version before `nextest` because config-backed preference tests read the generated `config.ts` through the same Bun loader used by the app.

## Legacy sources

These docs and commands seeded the verification summary and remain in place while the lattice absorbs the durable rules.

- [CLAUDE.md](../CLAUDE.md)
- [Makefile](../Makefile)
