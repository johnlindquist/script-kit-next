# Verification

This repo prefers the smallest runtime-backed verification that proves a change. UI work should verify the real surface; logic work should stay on the narrowest relevant checks.

Keyboard and mouse SDK helper tests should assert explicit unsupported rejection. UI behavior tests must use state-first receipts instead of `keyboard.*` or `mouse.*` helper calls, because those helpers do not provide native input receipts.

System feedback SDK tests should distinguish protocol serialization from behavior. Platform feedback dispatch must resolve from app-originated `systemFeedbackResult` receipts, while unimplemented UI promises should return `ERR_UNSUPPORTED_SDK_FEATURE`.

Shortcut assignment proof should cover config writes, save-time live route conflict checks, dynamic unregister route removal/no-ops, and recoverable registration failures. Use source audits plus `cargo check --lib` unless OS-level delivery is the target.

SDK `hotkey()` proof must stay separate from shortcut assignment. Use `cargo test --test hotkey_prompt_contract -- --nocapture` for source proof that the SDK routes to `ShowHotkey`, the prompt submits `HotkeyInfo` without config or registry mutation, and simulateKey dispatch plus getElements receipts are wired. Runtime proof should launch `hotkey("Press a keyboard shortcut")`, assert `getState.promptType:"hotkey"` and `getElements` capture rows, capture a modifier chord with `simulateKey`, verify the resolved JSON, then separately prove Escape/Cmd-W cancellation without `config.ts` fingerprint changes.

## ACP Existing-Chat Mutations

ACP existing-chat mutation proof must show SDK promises settle and conversation reads observe the stored result.

Use `cargo test --test acp_existing_chat_mutation_contract -- --nocapture` for source proof that `aiAppendMessage`, `aiSendMessage`, and `aiSetSystemPrompt` route through direct storage handlers, validate invalid/deleted chats, reject typed SDK errors, and write messages that `aiGetConversation` can read back. Pair with `cargo check --lib`, `bun scripts/check-sdk-types.ts`, `cargo fmt --check`, `git diff --check`, and `lat check`.

EnvPrompt secret-store proof should distinguish missing files from read, decrypt/format, parse, and cache failures. Source audits must prove result-returning lookup APIs, EnvPrompt storage-error propagation, redacted `getElements` status kinds, and `cargo check --lib`; direct lib tests may be blocked by unrelated dirty-tree `cfg(test)` failures.

## DropPrompt Native Drop

DropPrompt native-drop verification must prove both the GPUI file-drop hook and the redacted automation receipt boundary.

Use `cargo test --test drop_prompt_native_drop_contract -- --nocapture` for source proof that `DropPrompt` wires `.on_drop` through `ExternalPaths`, `getState.drop` carries only `{index,name,size}`, `getElements` no longer exposes paths as row values, and SDK submit remains full-fidelity `FileInfo[]`. Runtime proof should start empty, assert Submit disabled through `activeFooter`, assert `getState.drop.fileCount:0` and no dropped-file elements, perform a native file drop onto `window:drop`, then assert redacted state/elements, enabled Submit, full SDK submit payload, and Escape cancel behavior. Use `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check` before closing the slice.

## PathPrompt Filesystem Edges

PathPrompt filesystem-edge verification must prove directory load states through prompt-owned receipts, not by inferring from an empty list.

Use `cargo test --lib prompts::path::prompt::tests:: -- --nocapture` for direct load-entry proof of missing paths, non-directory starts, empty directories, hidden-dotfile policy, symlink rows, and simulated permission denial. Use `cargo test --test path_prompt_filesystem_edges_contract -- --nocapture` for source proof that `stateResult.path`, `getElements` path-status rows, SDK `PathPromptState`, and stable status copy remain wired. Runtime proof should launch `path({ startPath })` against a missing path, an empty temp directory, and a permission-denied or chmod-simulated directory, then assert `getState.path.status.kind`, `getState.path.status.message`, `visibleEntryCount`, and `getElements` `path-status.statusKind` before cleanup. Use `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check` before closing the slice.

## TemplatePrompt Automation Parity

TemplatePrompt automation parity proves the protocol path owns the same submit, cancel, navigation, ForceSubmit, and actions behavior as the visible footer advertises.

Use `cargo test --test template_prompt_parity_contract -- --nocapture` for source proof that every stdin simulateKey dispatcher has a TemplatePrompt arm, ForceSubmit includes TemplatePrompt in direct and batch paths, and the TemplatePrompt footer Actions button has a live `ActionsDialogHost::TemplatePrompt` mapping with focus restore. Runtime proof should launch `template("Hello {{name}}")`, assert `getState.promptType:"template"` plus `getElements` template rows, fill with `batch.setInput`, submit with `simulateKey Enter`, separately cancel with `simulateKey Escape`, open actions with `simulateKey Cmd+K`, and prove `batch.forceSubmit` resolves a provided value. Use `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check` before closing the slice.

## FieldsPrompt Automation Parity

FieldsPrompt automation parity proves `fields()` is a production prompt surface, not a coming-soon route.

Use `cargo test --test source_audits fields_prompt_contract -- --nocapture` for source proof that `Message::Fields` routes to `PromptMessage::ShowFields`, reuses `FormPromptState::from_fields`, reports prompt type `fields`, exposes `fields-fields` and `input:fields-*` elements, submits values as a JSON array in definition order, and has explicit submit/cancel/navigation simulateKey arms. Runtime proof should launch `fields()` with at least one text field and one typed validation field, assert `getState.promptType:"fields"` and `activeFooter.activeSurface:"form_prompt"`, assert `getElements` field rows, fill through `batch.setInput` / `batch.selectBySemanticId`, verify invalid email blocks Enter, verify valid Enter submits, prove explicit `batch.forceSubmit` array resolution, open actions with Cmd+K under the FormPrompt host, and cancel with Escape. Use `bun scripts/agentic/fields-prompt-parity.ts`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check` before closing the slice.

## SDK find unsupported boundary

`find()` verification is a negative SDK contract, not a UI prompt proof.

Checks must prove the SDK rejects with `UnsupportedSdkFeatureError` / `ERR_UNSUPPORTED_SDK_FEATURE` before `nextId`, `addPending`, or `send`, that no stale `FindMessage` / `type:"find"` SDK message shape remains, that the SDK Reference marks `find` as Unsupported, and that a Bun smoke call rejects without stdout JSONL writes. Use `fileSearch(query, { onlyin })` for the supported non-interactive file-search route.

## Main menu and footer

`make smoke-main-menu` is the repo's fast launcher and footer smoke target. Use it for main window, footer, built-in menu, and plugin-skill routing changes.

Native footer click-box changes need a real native click on empty space inside a visible footer item, plus a negative click or wheel over non-button footer background. Source-contract tests alone are not sufficient because AppKit hit testing can fail before GPUI state changes.

## Storybook Adoption Contract

Storybook adoption hardening is verified through source contracts first, then state-first runtime proofs when the visual lab is launched.

Run `cargo test --test storybook_adoption_contract`, `cargo test --test storybook_main_menu_render_path_contract`, `cargo test --test storybook_footer_contract`, `cargo test --test storybook_compare_contract`, `cargo test --test storybook_context_picker_contract`, `cargo test --test storybook_lifecycle_contract`, and `cargo test --test storybook_adoption_audit` for the static contract layer. The matching agentic proof scripts are `bun scripts/agentic/storybook_main_menu_parity.ts`, `bun scripts/agentic/storybook_lifecycle_theme.ts`, and `bun scripts/agentic/storybook_context_picker_parity.ts`.

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
- Current-app launcher-label or menu-shortcut parsing changes should at least run `cargo test --test current_app_commands -- --nocapture`, `cargo check --lib`, `git diff --check`, and `lat check`; narrow filters are acceptable while iterating on `current_app_commands_launcher_label` or `keyboard_shortcut`.

Shell helper changes outside the Rust app should keep their proof narrow too. For zsh helpers such as `cpath`, prefer the dedicated sourced-shell test plus one real-shell smoke check that covers raw paths, directory-plus-term search like `cpath .notes scroll`, and `ls -l` or `eza -l` style listing input before calling the work done.

For UI changes outside the main launcher/footer path, use the project's agentic runtime verification flow against the real surface instead of guessing from unit tests alone.

## Root Recent File Seed Pool

Root recent-file seed-pool changes are verified with source audits and grouping checks because they only change app-layer hydration and pure grouping.

Hydration must use the deeper seed limit, empty-root rendering must use the render limit, grouping must remain provider-free, and non-empty global recent seeds must keep filename-token eligibility.

Directory-context recent seed changes must assert that ordered parent-directory plus filename-token recents seed non-empty global root searches, while path-only, reversed-order, and unsafe short-parent matches stay excluded.

## Agentic Surface Audits

Agentic surface tooling is verified with integration/source-audit tests plus Bun runtime proofs on this host, avoiding the known `cargo test --lib` SIGBUS path.

Run `cargo test --test source_audits verify_shot_pixel_audit_contract`, `cargo test --test source_audits timestamp_formatting_contract`, `cargo test --test acp_mention_popup_registry_lifecycle_contract --test acp_popup_automation_parity_contract --test kit_store_visible_rows_contract --test settings_visible_rows_contract --test agentic_surface_navigator_inventory_contract`, `bun scripts/agentic/verify-shot-blank-rejection-matrix.ts`, `bun scripts/agentic/verify-shot-live-dark-surface.ts`, `bun scripts/agentic/surface-navigator-inventory-audit.ts --json`, `cargo fmt --check`, `git diff --check`, and `lat check`.

## Agentic Hard Scenarios

Hard-scenario tooling proves new CLI recipes, receipt fields, and failure modes before relying on live UI runs.

Run `cargo test --test agentic_hard_scenarios_contract --test agentic_loop_two_contract --test agentic_loop_three_contract --test agentic_loop_four_contract --test agentic_loop_five_contract --test agentic_loop_six_contract --test agentic_loop_seven_contract --test agentic_loop_eight_contract --test agentic_loop_nine_contract --test agentic_loop_ten_contract --test agentic_loop_eleven_contract --test agentic_loop_twelve_contract --test agentic_loop_thirteen_contract --test agentic_loop_fourteen_contract --test agentic_loop_fifteen_contract --test agentic_loop_sixteen_contract -- --nocapture` after changing `scripts/agentic/index.ts`, `scripts/agentic/scenario.ts`, `scripts/agentic/target-thread.ts`, or visual proof routing. Runtime proof should include the existing hard-scenario commands plus loop-ten through loop-sixteen proof with `bun scripts/agentic/index.ts screenshot-semantics-visual-consistency-stress --session <name> --group filterable-main --case clipboard-history-visible-rows --json`, `bun scripts/agentic/index.ts visible-text-clipping-overlap-stress --session <name> --surfaces main,actionsDialog,promptPopup,acpDetached --json`, `bun scripts/agentic/index.ts layout-measurement-regression-stress --session <name> --surfaces main,actionsDialog,acpDetached --json`, `bun scripts/agentic/index.ts modal-stack-arbitration-stress --session <name> --host acpChat --json`, `bun scripts/agentic/index.ts cross-surface-export-provenance-stress --session <name> --source file-search --destination acp-composer --export-mode copy --query AGENTS.md --json`, `bun scripts/agentic/index.ts dev-session-recovery-stale-target-stress --session <name> --entry clipboard-history-actions --kind actionsDialog --restart-mode stop-start --json`, `bun scripts/agentic/index.ts menu-syntax-ambiguity-diagnostics-stress --session <name> --query '>open @file !bad ~AGENTS.md' --json`, `bun scripts/agentic/index.ts ime-composition-input-boundary-stress --session <name> --json`, `bun scripts/agentic/index.ts accessibility-selected-text-fallback-stress --session <name> --json`, `bun scripts/agentic/index.ts display-migration-visual-bounds-stress --session <name> --surfaces main,actionsDialog,promptPopup,acpDetached,notes --from-display primary --to-display external --json`, `bun scripts/agentic/index.ts native-picker-external-return-focus-stress --session <name> --origin acp --handoff file-picker --foreign-app Finder --json`, `bun scripts/agentic/index.ts drag-cancel-payload-scope-stress --session <name> --source file-search --hover-target drop-prompt --cancel escape --json`, `bun scripts/agentic/index.ts runtime-appearance-churn-focused-input-stress --session <name> --surface acp-composer --churn scale,font,theme --cycles 6 --json`, `bun scripts/agentic/index.ts power-resume-window-generation-stress --session <name> --surface main --event sleep-wake --json`, `bun scripts/agentic/index.ts menu-tray-notification-modal-interruption-stress --session <name> --host acpChat --active-surface actionsDialog --interruptions tray-menu,app-menu,notification --json`, `bun scripts/agentic/index.ts stream-progress-cancel-visual-stability-stress --session <name> --surface acp-composer --updates 40 --cancel-at 25 --json`, `bun scripts/agentic/index.ts dictation-media-permission-readiness-churn-stress --session <name> --target acp-composer --churn microphone-permission,model-readiness --json`, `bun scripts/agentic/index.ts animation-frame-capture-determinism-stress --session <name> --surfaces main,actionsDialog,promptPopup --frames 6 --interval-ms 80 --json`, `bun scripts/agentic/index.ts accessibility-tree-semantic-parity-stress --session <name> --surfaces main,actionsDialog,promptPopup --json`, `bun scripts/agentic/index.ts rtl-bidi-emoji-text-rendering-stress --session <name> --surface acp-composer --text 'abc שלום 👩🏽‍💻 é مرحبا 123' --json`, and `bun scripts/agentic/index.ts high-volume-virtualized-list-stability-stress --session <name> --surface clipboard-history --fixture-count 5000 --filter-cycles 8 --scroll-cycles 12 --json`, followed by `session.sh stop` and status verification.

Loop seventeen adds `--test agentic_loop_seventeen_contract` plus fail-closed runtime proof for `bun scripts/agentic/index.ts input-modality-transition-ownership-stress --session <name> --surface main --interleave pointer-hover,keyboard-nav,trackpad-scroll,wheel-scroll,shortcut --cycles 8 --json`, `bun scripts/agentic/index.ts multi-context-attachment-dedupe-provenance-stress --session <name> --origins file,screenshot,selected-text,mcp-resource,clipboard-snippet --destinations acp-composer,notes --reorder-cycles 3 --json`, and `bun scripts/agentic/index.ts visual-contrast-readable-state-stress --session <name> --surfaces main,actionsDialog,promptPopup,acp-composer,notes --themes light,dark --scale-factors 1,1.25,1.5 --states active,inactive,disabled,focused,error,loading --json`.

Loop eighteen adds `--test agentic_loop_eighteen_contract` plus fail-closed runtime proof for `bun scripts/agentic/index.ts empty-error-retry-state-ux-stress --session <name> --surfaces main,clipboard-history,emoji-picker,file-search --query 'agentic-loop-eighteen-no-results-zzzz' --json`, `bun scripts/agentic/index.ts form-validation-inline-recovery-stress --session <name> --surface fields-prompt --fields email,required-text,number --invalid email:not-an-email,required-text:,number:not-a-number --valid email:ada@example.com,required-text:Ada,number:42 --json`, and `bun scripts/agentic/index.ts navigation-back-stack-history-stress --session <name> --origin main --surfaces clipboard-history,emoji-picker,file-search,actionsDialog --transitions triggerBuiltin,cmd-k,escape,back --json`.

Loop nineteen adds `--test agentic_loop_nineteen_contract` plus fail-closed runtime proof for long text wrapping/resizing UX, actions/command discoverability no-op UX, and dense list/detail preview readability. Run `bun scripts/agentic/index.ts long-text-wrap-resize-surface-stress --session <name> --surfaces main,clipboard-history,emoji-picker,file-search,actionsDialog --widths mini,narrow,full --fixtures long-name,long-path,long-description,multiline-snippet --json`, `bun scripts/agentic/index.ts actions-command-discoverability-noop-stress --session <name> --hosts main,clipboard-history,emoji-picker,file-search,app-launcher --states actionable,disabled,no-op --json`, and `bun scripts/agentic/index.ts dense-list-detail-preview-readability-stress --session <name> --surfaces file-search,sdk-reference,script-template-catalog --query agentic-loop-nineteen-preview --filter-cycles 4 --selection-cycles 8 --resize-cycles 3 --json`.

## Cargo Test SIGBUS Guard

Plain Rust tests must not resolve `#[test]` to GPUI's proc-macro test harness through `use gpui::*`.

`gpui` intentionally avoids re-exporting the proc-macro named `test` from its prelude surface. Otherwise lib-test compilation can feed ordinary unit tests into `gpui_macros::test`, recurse through large function bodies in `syn`, and SIGBUS before normal compile errors are reported.

## Agent Chat Codex Setup

Codex default setup changes require source-contract tests plus a state-first setup receipt.

Run the focused ACP config/preflight tests, `cargo test --test acp_onboarding`, and `lat check`. Runtime proof should prefer `bun scripts/agentic/index.ts acp-setup-recovery --select-agent codex-acp --json` so the receipt proves `selectedAgentId: "codex-acp"`, catalog membership, compatible agents, and idempotent `performAcpSetupAction(selectAgent)` without screenshots.

## Root File Directory Context Ranking

Root file directory-context ranking is verified with pure query-builder and ranking tests because it changes retrieval and scoring math without adding UI state.

Run `cargo test --lib root_file_path_context`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. If the known local SIGBUS failure interrupts tests before execution, keep the failure log and use `cargo check --lib` plus targeted source inspection as the proof path.

## Root Unified Search Safety Controls

Root unified-search safety is verified with grouping, config, and selection-key checks before runtime screenshots.

Files must remain passive by default: they can beat fallback handoff rows, but not command, script, app, skill, or window rows unless an explicit exact-only promotion policy allows it. Global root file provider completion must not mutate the active visible frame for the same filter text.

Explicit source heads may raise that source's visible rows and expose non-selectable source status metadata, but status must stay out of `SearchResult` execution, actions subjects, list item counts, scroll height, and selection coercion. `getElements` should expose status metadata so runtime proof can assert source-only paging without relying on screenshots.

Use `cargo test --test source_audits root_file_search_contract -- --nocapture`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. If `cargo test --lib root_file` hits the known local SIGBUS before tests execute, keep the failure log and rely on the focused source-audit proof plus `cargo check --lib`.

## Root Unified Search Frame Stability

Root unified-search frame stability prevents late passive rows from changing the selected command or click target.

Checks must prove that global root file search does not publish partial provider rows into the active visible frame, provider completion updates only provider status and cache, grouped-result cache reads return before refreshing recent-file seeds, ScriptList typing installs `computed_filter_text`, root-file state, and grouped rows before notifying, selection snapshots use `SearchResult::stable_selection_key` instead of input-history keys, fallback rows have stable selection keys without becoming history targets, and main-window preflight receipts expose selected identity, visible results, and a visible row fingerprint.

Use `cargo test --test source_audits root_unified_search_stability_contract -- --nocapture`, `cargo test --lib stable_selection_key`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. Run `bun scripts/agentic/root-search-frame-stability.ts` for the state-first runtime proof against the real main menu; it uses a deterministic delayed provider fixture and fails if any intermediate same-query sample changes the active row projection. When the report needs visual evidence, run `bun scripts/agentic/root-search-visual-stability.ts --query fix` and inspect its `.test-output/` contact sheet plus app log; the visual receipt fails on same-input fingerprint changes and on grouping or handler latency over budget. To prove real file rows are visibly participating, run the visual script with `--no-fixture --warm-provider --expect-visible-file-results`; that first warms the real provider, then retypes the query and fails unless every full-query frame includes root-file receipts.

## Root Unified Search Passive Ranking Receipt

Root passive ranking receipts prove actual visible row roles instead of inferring intent from action labels.

`mainWindowPreflight.visibleResults` exposes content-light row receipts with grouped index, visible rank, stable key, role, action kind, type label, and source name. Roles classify visible rows as primary launcher intent, root file, root passive, fallback, script issue, or agent, so runtime proofs can assert that passive rows exist and stay below commands, scripts, apps, skills, and windows for collision queries. Passive source order checks must use these receipts rather than row-label guesses.

Use `cargo test --test source_audits root_unified_search_stability_contract -- --nocapture`, `cargo test --lib root_passive_source_order -- --nocapture`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. If the known local `gpui_macros` SIGBUS interrupts the lib-test binary before tests execute, keep the failure log and rely on the source-audit receipt proof plus `cargo check --lib`.

## Root Unified Passive Snapshot Caches

Passive snapshot caches and query-frame latches keep slow local providers from changing an active root-search frame.

Checks must prove that Browser Tabs and Browser History foreground search only fuzzy-filters cached metadata snapshots, that stale or missing snapshots start background refreshes only after source eligibility passes, and that their hit vectors flow through a frozen per-query passive frame before grouping. Notes, Clipboard History, Dictation History, and ACP History must use cache-only foreground lookups and warm cold SQLite or JSONL data on background threads. Saved ACP and Dictation history must reuse mtime-backed JSONL indexes while invalidating after local writes/deletes, and ACP History must clamp legacy `search_text` before foreground ranking. Refresh completion must never call `cx.notify`, invalidate grouped results, or publish rows into the active frame for the same filter text.

Use `cargo test --test source_audits root_unified_passive_snapshot_contract -- --nocapture` with the browser-tabs, browser-history, passive-frame, JSONL index, ACP bounded-search-text, and root-stability audits, plus `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. Runtime proof should use `bun scripts/agentic/root-passive-frame-stability.ts` with preflight/state receipts rather than screenshots.

## Root Unified Search Config Parity

Root unified-search source additions must keep user controls, defaults, docs, and audits in lockstep.

Each root source listed in `UnifiedSearchConfig` needs a Rust config struct, default constants, a section-options accessor with clamps or an explicit promotion policy, a `config.ts` schema interface, a grouping append function, a source-audit module, and a verification section. The passive source order also needs Rust and `config.ts` schema parity, total enum coverage in grouping, duplicate normalization, and missing-default append behavior. This parity guard should run before adding new passive sources so slower providers cannot enter the root menu without the controls and proofs that prevent ranking or target-shift regressions.

Use `cargo test --test source_audits root_unified_config_schema_parity_contract -- --nocapture`, the source-specific root unified-search audit, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. Runtime proof is unnecessary for parity-only edits, but behavior changes still need the relevant state-first proof.

## Root Unified Search Passive Result Limits

Passive result-limit changes are verified through grouping tests, config/schema parity, and role-based runtime receipts.

Checks must prove that the budget applies only to root-passive rows, is applied after root Files and Recent Files and before passive source iteration, permits zero collision rows, preserves passive source order, and never moves passive rows above primary launcher results. Use `cargo test --test source_audits root_unified_passive_budget_contract -- --nocapture`, `cargo test --test source_audits root_unified_config_schema_parity_contract -- --nocapture`, `cargo test --test source_audits root_unified_search_stability_contract -- --nocapture`, `cargo test --test source_audits root_unified_passive_snapshot_contract -- --nocapture`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, `lat check`, and `bun scripts/agentic/root-passive-frame-stability.ts`.

## Root Unified Search Source Filters

Declarative source filters are verified by parser tests, source audits, and state-first runtime receipts.

Checks must prove that standalone trailing-colon heads such as `files:`/`f:`, `notes:`/`n:`, `clipboard:`/`c:`, `tabs:`/`t:`, `history:`/`h:`, `commands:`/`cmd:`, `conversations:`/`ai:`, `dictation:`/`d:`, and `windows:`/`w:` parse anywhere in ScriptList input; attached source-head queries such as `c:skip`, `clipboard:skip`, `f:s`, `files:s`, `f: s`, `f:sc`, `files: sc`, and `h:https://example.com` strip the known source head and search the selected source for the suffix; `processes:`/`p:` stays uncommitted until root process rows exist; leading `:` remains the discovery trigger and is not committed source syntax; quoted/unknown filter-looking tokens stay literal; capture keyword aliases keep ownership; grouped rows suppress primary/fallback and disallowed sources while active; positive source heads explicitly enable their source for the active stripped query even when that source is disabled for ordinary passive search; source-only trailing-space inputs such as `f: ` and `c: ` keep an empty stripped search text but show that source's default browse rows; explicit source filters expose non-selectable source-chip status metadata for Files, passive sources, base launcher sources, and Windows without adding status to the ScriptList row model; explicit source-filter mode blocks launcher input-history recall so Up and Down remain list navigation; explicit Files source-only browse can raise the recent-file render target without changing the ordinary empty-root Recent Files cap; explicit Files source filters start from a 12-row page and automatically expand near the bottom without requiring Enter; explicit Files source filters allow one-character ASCII alphanumeric stripped queries such as `f:s` and `files:s` without enabling plain `s` root file search; explicit local/passive source heads use direct source lookup so first-use filtered searches produce rows instead of warming a future cache frame; root file/passive frame keys include the source-filter set; preflight receipts expose stripped search text, source filters, and filter indicators; and source-filter-only queries do not render the menu-syntax hint while `;` still opens capture discovery. Use `cargo test --test menu_syntax_source_filters -- --nocapture`, `cargo test --test source_audits root_unified_source_filters_contract -- --nocapture`, `cargo test --test source_audits root_file_source_chip_pages_on_near_bottom_selection -- --nocapture`, `cargo test --lib source_filter_files_empty_browse_uses_browse_target_not_recent_render_cap -- --nocapture`, `cargo test --test source_audits root_recent_file_seed_pool_exceeds_empty_render_cap -- --nocapture`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, `lat check`, `bun scripts/agentic/root-source-filter-stability.ts`, `bun scripts/agentic/root-source-filter-clipboard.ts`, `bun scripts/agentic/root-source-filter-history-up.ts --timeout 12000`, `bun scripts/agentic/source-chip-pagination-proof.ts --timeout 16000`, and `bun scripts/agentic/root-source-filter-matrix.ts --query s --timeout 16000`.

Files source-filter lazy scrolling has an additional footer-safe receipt proof. `stateResult.mainListScroll` exposes scroll top, content height, viewport height, footer height, max scroll top, and selected-row visibility/above-footer flags; `bun scripts/agentic/root-source-filter-lazy-scroll.ts --query s --timeout 20000` seeds recent Files rows plus a delayed root-file provider, moves selection near the bottom, and proves `f:` and `f:s` keep the selected file visible while pages or provider rows are added. The receipt must wait for a measured viewport and selected-row visibility so it catches stale ListState counts, zero-viewport reveal races, and provider-publish selection snaps.

## Root Unified Search Result Actions

Root result actions are verified by typed action catalog tests, source audits, and state-first actions-dialog receipts.

Checks must prove that `ActionsDialogHost::MainList` resolves the focused root unified `SearchResult` before generic script `has_actions()` fallback; Files, Notes, Clipboard History, Browser Tabs, Browser History, AI Conversations, Dictation History, Apps, Commands, Skills, Script Issues, and Windows each expose a typed subject and stable action IDs; scripts, scriptlets, built-in commands, and apps still delegate to existing script action ownership for config-backed shortcut, alias, and deeplink actions; execution uses the pending subject captured on popup open; physical and simulated Enter route action activation before close clears pending root context; File row `Browse Parent Folder` clears stale MainList selection, opens dedicated File Search at the parent folder, and shortens home-prefix display with `~`; File row `Quick Look` uses the captured root-file path, reports controlled OS-helper failures, and never routes through dedicated File Search or clipboard Quick Look state; unknown root action IDs no-op without falling through to `handle_action`; close/reset clears pending subjects; dedicated built-in views keep their current action hosts; and `actionsDialog` receipts include IDs, labels, sections, shortcuts, host, and context stable key without raw local content. Use `cargo test --test source_audits root_unified_source_actions_contract -- --nocapture`, `cargo test --test source_audits copy_deeplink_prefers_command_namespace_for_config_backed_rows -- --nocapture`, `cargo test --test source_audits root_file_action_enter_routes_activation_before_close -- --nocapture`, `cargo test --test source_audits root_file_quick_look -- --nocapture`, `cargo test --test source_audits root_file_browse_parent_folder -- --nocapture`, `cargo test --lib quick_look_missing_path_returns_error_without_panic -- --nocapture`, `cargo test --lib root_file_actions_for_regular_file_displays_parent_folder_with_tilde_home -- --nocapture`, `cargo test --lib parent_folder_search_query_shortens_home_prefix_for_display -- --nocapture`, `cargo test --lib root_unified_result_actions -- --nocapture`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, `lat check`, and `bun scripts/agentic/root-source-actions-matrix.ts` when runtime rows are available.

The actions matrix may set `SCRIPT_KIT_AGENTIC_KEEP_ACTIONS_WINDOW_OPEN=1` so hidden agentic sessions can inspect the detached actions window before focus-driven auto-close runs. That flag is for proof harnesses only; normal actions windows still auto-close when both parent and popup lose focus.

The actions matrix may also set `SCRIPT_KIT_WINDOW_SEARCH_TEST_PROVIDER` with metadata-only window rows. This keeps `w:` proof deterministic in hidden sessions where macOS Accessibility may not expose live windows.

## Root Unified Search ACP History

ACP history root rows are verified with grouping, config, type metadata, execution wiring, and source-audit tests.

The critical regression guard is that adding a second passive source cannot split the Files section or its Search Files continuation row.

Use `cargo test --test source_audits root_unified_acp_history_contract -- --nocapture` with the root file source audit, plus `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`.

## Root Unified Search Clipboard History

Clipboard root rows are verified with metadata search, passive grouping, stable-key, config, and execution source audits.

Checks must prove that root clipboard search is bounded to metadata, disabled by default for ordinary passive search, explicitly enabled by `clipboard:`/`c:` for the active stripped query, excluded from ordinary empty root, inserted without splitting Files or fallbacks, keyed by `clipboard-history/{id}`, executed through the existing clipboard paste helper, and able to show recent clipboard rows for source-only `c: ` without enabling unfiltered empty-root clipboard.

Use `cargo test --test source_audits root_unified_clipboard_history_contract -- --nocapture` with the existing root file and ACP history source audits, plus `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`.

## Selected Text Clipboard Restore

Selected-text replacement is verified with source-audit proof because live Cmd+V depends on native focus.

Checks must prove that `set_selected_text` snapshots `NSPasteboard` items before writing replacement text, restores every saved item/type/data representation after the paste attempt, skips restore on pasteboard change-count drift, returns restore failures explicitly, and logs only content-light summary fields. Use `cargo test --test source_audits selected_text_clipboard_restore -- --nocapture`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. Add manual TextEdit proof only when validating real native focus delivery.

## SDK Image Clipboard

Image clipboard support is verified through source-audit coverage plus a real SDK runtime roundtrip.

Checks must prove that SDK `readImage(): Promise<Buffer>` remains PNG-documented, image writes route to executor-side `set_image`, image reads encode PNG bytes, and image error codes remain distinct. Runtime proof should write a tiny PNG through `clipboard.writeImage()`, read it back through `clipboard.readImage()`, and assert PNG magic plus expected dimensions from the returned Buffer. Use `cargo test --test source_audits clipboard_image_contract -- --nocapture`, `cargo check --lib`, `cargo fmt --check`, `git diff --check`, `lat check`, and the system SDK test with `INCLUDE_SYSTEM_CLIPBOARD_IMAGE=1` when a live clipboard is available.

## Root Unified Search Dictation History

Dictation history root rows are verified as opt-in, metadata-only, bounded, local-only, and passive.

Checks must prove that root Dictation History is disabled by default; excludes ordinary empty, short, newline, disabled, and advanced queries; scans only the compacted local history loader up to `scanLimit`; does not log raw root query text; carries no full transcript text in root result rows; inserts after Clipboard History and before AI Conversations; uses the shared capped passive-score helper; keys rows by `dictation-history/{id}`; loads transcript content only after explicit Enter; and shows recent dictation metadata for source-only `d: `.

Use `cargo test --test source_audits root_unified_dictation_history_contract -- --nocapture` with the existing root stability, passive snapshot, config parity, clipboard history, and ACP history audits, plus `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. Runtime proof should use a synthetic saved dictation entry when validating the live surface.

## Root Unified Search Notes

Notes root rows are verified with metadata-only storage tests, passive grouping, stable-key, config, and non-toggle open wiring.

Checks must prove that root Notes search excludes ordinary empty, short, newline, disabled, and advanced queries; searches active notes only; returns metadata without note bodies; falls back from empty FTS hits to bounded substring matching so attached source-head queries like `n:not` find `Welcome to Notes`; inserts after Browser Tabs and before Clipboard History and AI Conversations; keys rows by `note/{id}`; opens Notes through the non-toggle helper; and shows pinned/recent active note metadata for source-only `n: `.

Use `cargo test --test source_audits root_unified_notes_contract -- --nocapture` with the existing root file, ACP history, and clipboard history audits, plus `cargo check --lib`, `cargo fmt --check`, `git diff --check`, and `lat check`. Because Enter crosses from the launcher to a separate Notes window, add a narrow state-first runtime proof when validating the live surface.

## Root Unified Search Browser Tabs

Browser Tabs root rows are verified as opt-in, metadata-only, stale-while-revalidate cached, and passive.

Checks must prove that root Browser Tabs search is disabled by default; excludes ordinary empty, short, newline, disabled, and advanced queries; reads only current tab title, URL, browser, and tab-location metadata from a cache-only foreground snapshot; performs no favicon, page-content, cookie, download, or network reads; inserts after Files/Recent Files and before Notes; uses the shared capped passive-score helper; keys rows by `browser-tab/...` for selection only; switches the existing tab through `activate_tab`; and shows current tab metadata for source-only `t: `.

Use `cargo test --test source_audits root_unified_browser_tabs_contract -- --nocapture` with the existing root stability, file, notes, clipboard history, ACP history, and browser history audits, plus `cargo check --lib`, `cargo build`, `cargo fmt --check`, `git diff --check`, and `lat check`. Add a state-first runtime proof when a supported browser is open and `unifiedSearch.browserTabs.enabled` is true.

## Root Unified Search Browser History

Browser History root rows are verified as opt-in, metadata-only, stale-while-revalidate cached, and passive.

Checks must prove that root Browser History search is disabled by default; excludes ordinary empty, short, newline, disabled, and advanced queries; foreground search fuzzy-filters only cached local URL/title/visit metadata while background refreshes copy bounded Chromium history DBs; rejects non-HTTP(S) schemes; performs no favicon, cookie, download, content, or network reads; inserts after Browser Tabs, Notes, Clipboard History, and AI Conversations and before fallback handoff rows; keys rows by `browser-history/...`; opens through the safe URL helper; and shows recent safe URL metadata for source-only `h: `.

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

## Design Picker persistence

Design Picker persistence verification keeps the `getState`/`kit/state` `design` envelope and the on-disk catalog id contract honest across previews, explicit commits, and the Cmd+1 cycle path.

The agentic contract tests under `tests/design_picker_state_receipt_contract.rs` and `tests/design_picker_visual_matrix_script_contract.rs` exercise the receipt shape end-to-end: previews leave persisted state untouched, explicit Enter or row-click commits write the canonical catalog id, and Escape/Cmd+W restoration is preview-only. The visual matrix contract additionally pins the ScriptList screenshot for every shipped catalog id under fixed dictation/clipboard fixtures so renderer drift is caught before the receipt drifts. See [[designs#Persistence]] for the receipt fields and the lifecycle each path follows.

## Legacy sources

These docs and commands seeded the verification summary and remain in place while the lattice absorbs the durable rules.

- [CLAUDE.md](../CLAUDE.md)
- [Makefile](../Makefile)
