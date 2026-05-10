# Verification

This repo prefers the smallest runtime-backed verification that proves a change. UI work should verify the real surface; logic work should stay on the narrowest relevant checks.

## Main menu and footer

`make smoke-main-menu` is the repo's fast launcher and footer smoke target. Use it for main window, footer, built-in menu, and plugin-skill routing changes.

Native footer click-box changes need a real native click on empty space inside a visible footer item, plus a negative click or wheel over non-button footer background. Source-contract tests alone are not sufficient because AppKit hit testing can fail before GPUI state changes.

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

## Root Unified Search Safety Controls

Root unified-search safety is verified with grouping, config, and selection-key checks before runtime screenshots.

Files must remain passive by default: they can beat fallback handoff rows, but not command, script, app, skill, or window rows unless an explicit exact-only promotion policy allows it. Global root file provider completion must not mutate the active visible frame for the same filter text.

Use `cargo test --test source_audits root_file_search_contract -- --nocapture`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. If `cargo test --lib root_file` hits the known local SIGBUS before tests execute, keep the failure log and rely on the focused source-audit proof plus `cargo check --lib`.

## Root Unified Search Frame Stability

Root unified-search frame stability prevents late passive rows from changing the selected command or click target.

Checks must prove that global root file search does not publish partial provider rows into the active visible frame, provider completion warms a bounded per-query cache, grouped-result cache reads return before refreshing recent-file seeds, ScriptList typing does not notify/render between `filter_text` and `computed_filter_text` updates, selection snapshots use `SearchResult::stable_selection_key` instead of input-history keys, fallback rows have stable selection keys without becoming history targets, and main-window preflight receipts expose the selected key plus a visible row fingerprint.

Use `cargo test --test source_audits root_unified_search_stability_contract -- --nocapture`, `cargo test --lib stable_selection_key`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. Run `bun scripts/agentic/root-search-frame-stability.ts` for the state-first runtime proof against the real main menu; it compares `selected_result_key`, Enter action, and `visible_result_key_fingerprint` before and after root file provider completion for the same filter text.

## Root Unified Passive Snapshot Caches

Passive snapshot caches and query-frame latches keep slow local providers from changing an active root-search frame.

Checks must prove that Browser Tabs and Browser History foreground search only fuzzy-filters cached metadata snapshots, that stale or missing snapshots start background refreshes only after source eligibility passes, and that their hit vectors flow through a frozen per-query passive frame before grouping. Saved ACP and Dictation history must reuse mtime-backed JSONL indexes while invalidating after local writes/deletes. Refresh completion must never call `cx.notify`, invalidate grouped results, or publish rows into the active frame for the same filter text.

Use `cargo test --test source_audits root_unified_passive_snapshot_contract -- --nocapture` with the browser-tabs, browser-history, passive-frame, JSONL index, and root-stability audits, plus `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. Runtime proof should use `bun scripts/agentic/root-passive-frame-stability.ts` with preflight/state receipts rather than screenshots.

## Root Unified Search Config Parity

Root unified-search source additions must keep user controls, defaults, docs, and audits in lockstep.

Each root source listed in `UnifiedSearchConfig` needs a Rust config struct, default constants, a section-options accessor with clamps or an explicit promotion policy, a `config.ts` schema interface, a grouping append function, a source-audit module, and a verification section. This parity guard should run before adding new passive sources so slower providers cannot enter the root menu without the controls and proofs that prevent ranking or target-shift regressions.

Use `cargo test --test source_audits root_unified_config_schema_parity_contract -- --nocapture`, the source-specific root unified-search audit, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. Runtime proof is unnecessary for parity-only edits, but behavior changes still need the relevant state-first proof.

## Root Unified Search ACP History

ACP history root rows are verified with grouping, config, type metadata, execution wiring, and source-audit tests.

The critical regression guard is that adding a second passive source cannot split the Files section or its Search Files continuation row.

Use `cargo test --test source_audits root_unified_acp_history_contract -- --nocapture` with the root file source audit, plus `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`.

## Root Unified Search Clipboard History

Clipboard root rows are verified with metadata search, passive grouping, stable-key, config, and execution source audits.

Checks must prove that root clipboard search is bounded to metadata, disabled by default, excluded from empty root, inserted without splitting Files or fallbacks, keyed by `clipboard-history/{id}`, and executed through the existing clipboard paste helper.

Use `cargo test --test source_audits root_unified_clipboard_history_contract -- --nocapture` with the existing root file and ACP history source audits, plus `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`.

## Root Unified Search Dictation History

Dictation history root rows are verified as opt-in, metadata-only, bounded, local-only, and passive.

Checks must prove that root Dictation History is disabled by default; excludes empty, short, newline, disabled, and advanced queries; scans only the compacted local history loader up to `scanLimit`; does not log raw root query text; carries no full transcript text in root result rows; inserts after Clipboard History and before AI Conversations; uses the shared capped passive-score helper; keys rows by `dictation-history/{id}`; and loads transcript content only after explicit Enter.

Use `cargo test --test source_audits root_unified_dictation_history_contract -- --nocapture` with the existing root stability, passive snapshot, config parity, clipboard history, and ACP history audits, plus `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. Runtime proof should use a synthetic saved dictation entry when validating the live surface.

## Root Unified Search Notes

Notes root rows are verified with metadata-only storage tests, passive grouping, stable-key, config, and non-toggle open wiring.

Checks must prove that root Notes search excludes empty, short, newline, disabled, and advanced queries; searches active notes only; returns metadata without note bodies; inserts after Browser Tabs and before Clipboard History and AI Conversations; keys rows by `note/{id}`; and opens Notes through the non-toggle helper.

Use `cargo test --test source_audits root_unified_notes_contract -- --nocapture` with the existing root file, ACP history, and clipboard history audits, plus `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. Because Enter crosses from the launcher to a separate Notes window, add a narrow state-first runtime proof when validating the live surface.

## Root Unified Search Browser Tabs

Browser Tabs root rows are verified as opt-in, metadata-only, stale-while-revalidate cached, and passive.

Checks must prove that root Browser Tabs search is disabled by default; excludes empty, short, newline, disabled, and advanced queries; reads only current tab title, URL, browser, and tab-location metadata from a cache-only foreground snapshot; performs no favicon, page-content, cookie, download, or network reads; inserts after Files/Recent Files and before Notes; uses the shared capped passive-score helper; keys rows by `browser-tab/...` for selection only; and switches the existing tab through `activate_tab`.

Use `cargo test --test source_audits root_unified_browser_tabs_contract -- --nocapture` with the existing root stability, file, notes, clipboard history, ACP history, and browser history audits, plus `cargo check --lib`, `cargo build`, `cargo fmt --check`, `git diff --check`, and `lat check`. Add a state-first runtime proof when a supported browser is open and `unifiedSearch.browserTabs.enabled` is true.

## Root Unified Search Browser History

Browser History root rows are verified as opt-in, metadata-only, stale-while-revalidate cached, and passive.

Checks must prove that root Browser History search is disabled by default; excludes empty, short, newline, disabled, and advanced queries; foreground search fuzzy-filters only cached local URL/title/visit metadata while background refreshes copy bounded Chromium history DBs; rejects non-HTTP(S) schemes; performs no favicon, cookie, download, content, or network reads; inserts after Browser Tabs, Notes, Clipboard History, and AI Conversations and before fallback handoff rows; keys rows by `browser-history/...`; and opens through the safe URL helper.

Use `cargo test --test source_audits root_unified_browser_history_contract -- --nocapture` with the existing root file, notes, clipboard history, and ACP history audits, plus `cargo check --lib`, `cargo build`, `cargo fmt --check`, `git diff --check`, and `lat check`.

## Computer-use native-window capture

Native-window capture proof goes through the real MCP path and treats the JSON receipt as the primary oracle.

For `computer/capture_native_window`, first call `computer/list_native_windows` and select a row whose `observation.captureSelectionCandidate.status` is `candidate`; then call `computer/capture_native_window` with `pid`, `nativeWindowId`, and `expectedBundleId`. The primary proof is the receipt: `status:"captured"`, stable `correlationId`, non-empty SHA-256, positive byte length/dimensions, and `pixelAudit.blankLike:false`. When `includeImage:true`, decode `pngBase64`, verify PNG magic bytes, decoded byte length, and SHA-256. Negative proof should include wrong `expectedBundleId` -> `ownershipMismatch`, stale or missing `nativeWindowId` -> `windowNotFound`, unknown input fields -> `invalid_arguments`, and a non-candidate listed row -> `notCaptureCandidate` when the current runtime exposes one; all negative capture receipts must keep `capture:null`.

SDK scriptability is pinned separately by `tests/source_audits/sdk_computer_use_contract.rs`: the SDK must expose typed `computer.listNativeWindows()` and `computer.captureNativeWindow()` helpers, discover the app server from `~/.scriptkit/server.json`, call `/rpc` with the bearer token, and keep the public `computer` namespace observation/capture-only.

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
