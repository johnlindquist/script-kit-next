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

## Root Recent File Seed Pool

Root recent-file seed-pool changes are verified with source audits and grouping checks because they only change app-layer hydration and pure grouping.

Hydration must use the deeper seed limit, empty-root rendering must use the render limit, grouping must remain provider-free, and non-empty global recent seeds must keep filename-token eligibility.

Directory-context recent seed changes must assert that ordered parent-directory plus filename-token recents seed non-empty global root searches, while path-only, reversed-order, and unsafe short-parent matches stay excluded.

## Root File Directory Context Ranking

Root file directory-context ranking is verified with pure query-builder and ranking tests because it changes retrieval and scoring math without adding UI state.

Run `cargo test --lib root_file_path_context`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. If the known local SIGBUS failure interrupts tests before execution, keep the failure log and use `cargo check --lib` plus targeted source inspection as the proof path.

## Computer-use native-window capture

Native-window capture proof goes through the real MCP path and treats the JSON receipt as the primary oracle.

For `computer/capture_native_window`, first call `computer/list_native_windows` and select a row whose `observation.captureSelectionCandidate.status` is `candidate`; then call `computer/capture_native_window` with `pid`, `nativeWindowId`, and `expectedBundleId`. The primary proof is the receipt: `status:"captured"`, stable `correlationId`, non-empty SHA-256, positive byte length/dimensions, and `pixelAudit.blankLike:false`. When `includeImage:true`, decode `pngBase64`, verify PNG magic bytes, decoded byte length, and SHA-256. Negative proof should include wrong `expectedBundleId` -> `ownershipMismatch`, stale or missing `nativeWindowId` -> `windowNotFound`, unknown input fields -> `invalid_arguments`, and a non-candidate listed row -> `notCaptureCandidate` when the current runtime exposes one; all negative capture receipts must keep `capture:null`.

## Oracle Bundle Context

Oracle review bundles should carry the same process context local agents use, so remote review does not miss repo-specific grounding or verification rules.

Include `CLAUDE.md`/`AGENTS.md`, the owning skill file, and relevant `lat.md/` pages in Script Kit GPUI Oracle bundles. Include this [[verification]] page whenever implementation or review checks are part of the prompt, and make the required `lat.md` update plus `lat check` expectations visible to Oracle.

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
