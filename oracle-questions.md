# Oracle-packx Running Question Queue

Self-pacing loop: pick top unanswered question → build focused packx bundle → `mcp__oracle__consult` → implement recommendation → mark `done` → next.

Keep this file dense. Each entry: why it matters + concrete scope + target files + success criteria. Refine as answers land.

## Conventions

- Status: `queued`, `in-flight`, `done`, `superseded`
- Each Oracle session uses a 3-5 word slug
- Record session slug when firing so we can trace commits back
- Keep bundles 50k-150k tokens; GPT-5.4 Pro handles large context fine but Oracle prefers ≤128k for best quality

---

## Q1 — `protocol-builtin-reliability-plan` — DONE (session: `protocol-builtin-reliabilit-plan`)

Structural fix for `triggerBuiltin unknown-name` log-spam class. Three duplicated dispatchers; no startup-validated registry.

**Outcome:** Oracle prescribed `BuiltinId` + `BuiltinSpec` + `registry()` OnceLock, single `dispatch_trigger_builtin` helper, protocolVersion pre-check with deprecated-name fallback, new `src/protocol_stats.rs` counters, rate-limited unknown logs, contract tests. Implementing now.

## Q2 — `acp-chat-state-machine-audit` — PR1 DONE (session: `acp-chat-state-machine-audit`, commit `956ff8eff`); PR2 QUEUED

ACP chat (`src/ai/acp/**`, `src/app_impl/tab_ai_mode.rs`) has grown many implicit states (context-capture, attachment-portal, auto-approve, slash-menu, streaming, tool-use). `tab_ai_mode.rs` is 37k tokens alone and holds special-case Tab/Shift+Tab + Cmd+Enter routing with attachment-portal guards.

**Question:** Give me an explicit state machine for the ACP chat surface. Enumerate every state, transition trigger, and invariant. Flag states that are currently implicit (conditional branches) that should be explicit enum variants. Propose the smallest refactor PR to make the state machine enforceable by the compiler without breaking `is_in_attachment_portal` guards.

**Bundle:** `src/ai/acp/**/*.rs`, `src/ai/harness/**/*.rs`, `src/app_impl/tab_ai_mode.rs`, `src/app_impl/attachment_portal.rs`, `lat.md/acp-chat.md`, `lat.md/ai-context.md`

**Target files:** `src/ai/acp/state.rs` (new), `src/app_impl/tab_ai_mode.rs`, `src/app_impl/attachment_portal.rs`

## Q3 — `shortcuts-hud-grid-dismiss-logic` — PR1 DONE (session: `shortcuts-hud-grid-dismiss-logic`); PR2 QUEUED

`src/app_impl/shortcuts_hud_grid.rs` contains `is_dismissable_view()` — a string/enum match that grew organically. Risk: adding a new non-dismissable view means editing this file or silently breaking click-outside behavior.

**Question:** Design a per-view `DismissPolicy` contract so every `AppView` variant declares its own dismiss behavior exhaustively. The compiler should reject `AppView` additions that forget to declare a policy.

**Bundle:** `src/app_impl/shortcuts_hud_grid.rs`, `src/main_sections/app_view_state.rs`, `src/main_sections/render_impl.rs`, sampled AppView consumers via grep

**Target files:** `src/main_sections/app_view_state.rs`, `src/app_impl/shortcuts_hud_grid.rs`

## Q4 — `window-activation-invariants-guard` — PR1 DONE (session: `window-activation-invariants-guard`)

NSPanel/NonactivatingPanel + cursor-in-background + collection-behavior config (per MEMORY.md) is fragile — any change to `src/platform/visibility_focus.rs`, `src/platform/app_window_management.rs`, or `src/platform/cursor.rs` can silently break non-activating behavior.

**Question:** What runtime invariant-check can we add at every `show_main_window*` boundary to fail fast if the window level, collection behavior, or cursor swizzle are wrong? Give a ~30-line diagnostic helper that asserts the expected state. Where should it hook (every show? once per process? debug-only?).

**Bundle:** `src/platform/visibility_focus.rs`, `src/platform/app_window_management.rs`, `src/platform/cursor.rs`, `src/main_entry/app_run_setup.rs` (window_kind path), memory/`window_activation_config.md` if it exists

**Target files:** new `src/platform/panel_invariants.rs`, edits at each show_main_window call site

## Q5 — `mcp-resource-doc-drift-tests` — PR1 DONE (session: `mcp-resource-doc-drift-tests`)

`src/mcp_resources/mod.rs` hand-writes SDK/scripts/scriptlets reference content. Risk: Rust types drift from documented fields.

**Question:** Design a generic "resource vs code truth" test harness. For every MCP resource that lists stdin verbs, builtin ids, or prompt APIs, the test should derive the expected set from code and diff against the resource payload. Fail with a precise "missing X, unexpected Y" message. Show concrete code for 2 resources.

**Bundle:** `src/mcp_resources/**/*.rs`, `src/builtins/mod.rs`, `src/stdin_commands/mod.rs`, `scripts/kit-sdk.ts`

**Target files:** `src/mcp_resources/tests/drift.rs` (new), per-resource descriptor exports

## Q6 — `logging-observability-next-pass` — PR1 DONE (session: `logging-observabil-next-pass`); PR2–4 QUEUED

Recent AFK audits keep finding log-spam regressions (O(untrusted-input) log lines). The 120-char cap was point fix. What's the systemic guard?

**Question:** Design a logging lint rule + runtime budget. (a) A CI/static scan that flags `logging::log` / `tracing::*!` calls using `{name}` or `{e}` interpolation where the value comes from untrusted input without a length cap. (b) A runtime-budget wrapper (e.g., `log_user_value()` helper) that all untrusted-value logs MUST go through. Give concrete Rust code and a grep-based CI check.

**Bundle:** `src/logging/**`, `src/main_entry/runtime_stdin*.rs`, random sample of tracing! callers

**Target files:** new `src/logging/safe_user_value.rs`, CI addition, grep check script

## Q7 — `renderer-hot-path-alloc-audit` — PR1 DONE (session: `renderer-hot-path-alloc-audit`); PR2 VERIFIED-NOT-WORTH

Per MEMORY.md, Metal video rendering uses zero-copy CVPixelBuffer — but the list/prompt render paths (`src/render_prompts/*`, `src/app_render/*`) are less audited. Suspect per-frame allocations (String clones, Vec::new per list item, SharedString churn).

**Question:** Audit the hot render path for per-frame allocations and interning opportunities. Rank the top 5 offenders by `(cost per frame × frequency)`. For each, give the 1-line-diff fix (Cow, interned SharedString, cached Entity, etc.). Do NOT recommend a rewrite — recommend cheap wins only.

**Bundle:** `src/app_render/**/*.rs`, `src/render_prompts/**/*.rs`, `src/scrolling/**/*.rs`, sampled `lat.md/*` rendering docs

**Target files:** TBD based on hot path analysis

## Q8 — `script-metadata-validation-fail-fast` — PR1 DONE (session: `script-metadata-validation-fail-fast`); PR2-4 QUEUED

Script Kit scripts have metadata (shortcut, trigger, schedule, kenv, pass, etc.) parsed from TS source. Invalid metadata currently fails silently or at runtime on trigger.

**Question:** Design a startup-time metadata validation pass. Every loaded script should be validated before it lands in `SCRIPTS_INDEX`; bad scripts land in a `FAILED_SCRIPTS` list surfaced via MCP resource + menu bar warning. Show the split-point in the current loader and the error struct. What errors should be fatal vs warning?

**Bundle:** `src/scripts/**/*.rs`, `src/script_metadata/**/*.rs`, sample of `scripts/kit-sdk.ts`

**Target files:** `src/scripts/validation.rs` (new), `src/scripts/mod.rs` (split-point), MCP surface

## Q9 — `afk-audit-loop-effectiveness` — QUEUED

The AFK audit loop (visible in recent commits as Run 7 Pass #1-9) keeps finding regressions but each fix is point-style. What's the systemic improvement to the audit loop itself so passes produce structural fixes by default, not surgical patches?

**Question:** Review the AFK audit log (`audits/afk/log.md` + `audits/afk/stories.md` if present) and the Pass-style commit messages. Diagnose why passes are surgical instead of structural. Propose a concrete change to the audit skill prompt and/or pass-evaluation criteria that would bias toward structural fixes. Draft the revised prompt.

**Bundle:** `audits/afk/**`, `.claude/skills/afk-audit-loop/**`, recent audit-log commits

**Target files:** `.claude/skills/afk-audit-loop/**`, possibly a new `audits/afk/ladder.md` (surgical → structural progression)

## Q10 — `notes-acp-embed-contract` — QUEUED

Notes window can host embedded ACP chat (per `lat.md/overview.md`). What's the contract between the Notes host and the ACP embed? Risk: they share state through free-form calls, breakage when either evolves.

**Question:** Define the formal embed contract. Inputs (theme, width constraints, focus coordination), outputs (focus changes, size requests, escape-to-parent), lifecycle (mount/unmount/restart). Show the trait signature and the smallest refactor to get there.

**Bundle:** `src/notes/**`, `src/ai/acp/**/*.rs` (embed integration), `lat.md/notes.md`, `lat.md/acp-chat.md`

**Target files:** new `src/notes/acp_embed.rs` or `src/ai/acp/embed.rs` trait, Notes window glue

## Q11 — `massive-files-next-slices` — DONE (session: `massive-files-next-slices`)

Remaining massive files still hide behavior ownership across ACP automation state, Tab AI context capture, prompt-handler automation, builtin execution, MCP catalogs, and dialog actions. Need next slices that improve agent understanding without a broad rewrite.

**Question:** Review a focused packx bundle of `src/prompt_handler/mod.rs`, `src/app_execute/builtin_execution.rs`, `src/app_impl/tab_ai_mode.rs`, `src/ai/acp/view.rs`, `src/actions/dialog.rs`, `src/mcp_resources/mod.rs`, and the active AURP docs. Rank the next refactor slices by regression-prevention value.

**Outcome:** Oracle ranked: finish App Launcher Tab AI row projection; apply AURP-18 projection helpers to Process Manager; promote MCP-backed SDK/template catalog rows to explicit projection owners; split ACP automation state snapshots into named builders; name automation batch target capabilities before extracting runners. It explicitly deferred browser/window-dependent matrix work, whole-`prompt_handler` batch rewrites, dictation/model-download behavior, and broad ActionsDialog consolidation.

**Bundle:** `~/.oracle/bundles/massive-files-next-slices-minified.txt`

**Target files:** `src/app_impl/tab_ai_mode.rs`, `src/render_builtins/app_launcher.rs`, `src/prompt_handler/mod.rs`, `src/mcp_resources/mod.rs`, `src/ai/acp/view.rs`

---

## Work log

- **2026-04-18** Q1 fired (30m54s), implementing
- **2026-04-18** Queue written with 10 candidates; ordering reflects impact+readiness
- **2026-04-18** Q1 done (commit `80a716905`): protocolVersion gate + builtinId/name normalization
- **2026-04-18** Q2 fired and answered (35m57s); PR1 done (commit `956ff8eff`): AcpSurfaceState placement machine collapsing 4-field conjunction into one explicit enum; PR2 (AcpOverlayState key-owner enum in AcpChatView) queued
- **2026-04-18** Q3 fired and answered (7m41s); PR1 done: `AppView::dismiss_policy()` + `DismissPolicy`/`DismissTrigger`/`DismissEffect` types collapsing the HUD/grid negative `matches!` into an exhaustive `AppView` method (no wildcard, no `Default`). PR2 (`MainSemanticSurface` migration of `semantic_surface_for_main_view`) queued
- **2026-04-18** Q4 fired and answered; PR1 done: `src/platform/panel_invariants.rs` runtime audit + `platform::ensure_main_panel_configured` centralizing helper + moved `PANEL_CONFIGURED` one-shot to the lib/bin crate roots so only the helper flips it after `assert_main_panel_invariants(..).ok()`. Six duplicated inline configure sequences collapsed across `app_run_setup.rs`, `runtime_stdin.rs`, `runtime_stdin_match_core.rs`, `window_visibility.rs`. 6 source-audit tests (`tests/panel_invariants_contract.rs`) + 7 unit tests for `collection_behavior_ok`. Pin: level 101 (NSPopUpMenuWindowLevel) not 3 — Oracle caught the fragile assumption.
- **2026-04-18** Q6 fired and answered (7m42s); PR1 done: byte-capped `logging::log_user_value` (`src/logging/safe_user_value.rs`) with UTF-8 char-boundary walk-back + ellipsis inside the budget (200 bytes default) + `LogSafe` metadata (raw_bytes / safe_bytes / truncated); complementary time-window `logging::log_rate_limit` (`src/logging/rate_limit.rs`) keyed on `(category, key.len(), hash(key))` — never stores raw string — 30s emit window, 120s stale GC, 2048-key cap with auto-prune; migrated all three `trigger_builtin_dispatch.rs` log functions (`log_unknown_trigger_builtin`, `log_deprecated_trigger_builtin_name`, `log_invalid_trigger_builtin`) off `UNKNOWN_NAME_PREVIEW_CHAR_LIMIT` + `chars().take(120)` + `should_log_occurrence` onto shared helpers with full structured-field set (name_preview/name_bytes/name_safe_bytes/name_truncated/suppressed/occurrences_total). 15 new unit tests (9 safe_user_value + 6 rate_limit). New `lat.md/logging.md` documents the contract. PR2 (migrate remaining call sites in `runtime_stdin.rs`, `runtime_stdin_match_core.rs`, `dictation/history.rs`, `ai/acp/handlers.rs`, `ai/window/interactions.rs`) queued; PR3 (grep CI guard `scripts/check-log-user-values.sh`) queued; PR4 (minimal `kit://log-budget` MCP resource) queued
- **2026-04-18** Q5 fired and answered (7m30s); PR1 done: drift-audit harness for hand-written MCP reference resources. Added `stdin_commands::all_external_command_verbs()` + `trigger_registry::all_trigger_builtin_command_ids()` (single runtime source of truth, unit-pinned exhaustively against the dispatch match arms); two new MCP resources `kit://stdin-commands` + `kit://trigger-builtins` emitting markdown prose with `<!-- drift-audit:<marker>:start -->` marker blocks; `tests/mcp_resource_drift.rs` with `ResourceDriftAudit` trait + `DriftReport` diff-friendly output. 5 drift tests + 2 new unit tests. Key design: tests never scrape Rust source and never duplicate match arms — everything keys off the accessors. Oracle ranking: 1) harness, 2) trigger-builtin accessor, 3) stdin-verb accessor, 4) marker blocks — all landed in PR1.
- **2026-04-18** Q7 fired and answered (6m53s, extended); PR1 done (commit `770b9b4b2`): three verified render-path alloc fixes in `src/list_item/mod.rs`. (a) Shortcut-token path now keeps `Option<Cow<'a, [String]>>` end-to-end — dropped `.map(|cow| cow.into_owned())` (Oracle #2, ~9k allocs/sec saved at 50 rows × 60fps). (b) Name+description highlight loops now call `indices.contains(&char_idx)` directly on the `Vec<usize>` instead of building per-row `HashSet<usize>` (Oracle #4, ~3-6k allocs/sec). (c) Tooltip/StyledText name paths use `self.name.clone()` (SharedString Arc bump) instead of `self.name.to_string()` (Oracle #5, ~3k allocs/sec). All 29 list_item tests pass.
- **2026-04-18** Q7 PR2 verified-not-worth: Oracle tagged items #1 + #3 as VERIFY-FIRST. Verified `src/app_impl/filtering_cache.rs#get_grouped_results_cached` already returns `(Arc<[GroupedListItem]>, Arc<[SearchResult]>)` — the clones on `preview_panel.rs:101-102` are Arc bumps, not deep clones, so Oracle's #1 concern is moot (Oracle explicitly said "drop it from the top 5" when the accessor returns Arc handles). Item #3's `HighlightedSpan.text` is `String` (`src/syntax.rs:80`), so swapping the cache to `Arc<[HighlightedLine]>` + `SharedString` spans would save allocs, but only when a preview is visible and only on renders triggered by preview state change — not every frame. Net estimated savings well under Oracle's own threshold of 1000 allocs/sec. Deferred until a concrete perf complaint surfaces.
- **2026-04-18** Q8 fired and answered (5m29s, extended); PR1 done: `src/scripts/validation.rs` foundation. Added `ValidationSeverity` / `BindingKind` / `MetadataField` / `ScriptValidationKind` / `RelatedScript` / `ScriptValidationIssue` / `FailedScript` / `ValidationReport` / `ScriptCatalogReport` types (all Serialize/Deserialize for the eventual `kit://failed-scripts` MCP resource). Implemented `detect_binding_collisions(&[Arc<Script>])` exhaustively catching duplicate `shortcut` / `alias` / `keyword` / `trigger` declarations with lowercase+whitespace-normalized shortcut comparison so "Cmd Shift K" and "cmd  shift k" collide. Every colliding script becomes a `FailedScript` with `related` pointers at its peers, kept catalog excludes them so dispatch never races. `validate_script_catalog(Vec<Arc<Script>>)` is the single entry point; `loader::read_scripts_report()` is the new additive API that wraps `read_scripts()` + validation into one `Arc<ScriptCatalogReport>`. Existing `read_scripts()` untouched — PR2 will migrate startup/hotkeys/mcp_server call sites and plumb typed-metadata parse errors (Oracle's #1 ranked item). 10 unit tests: empty catalog, single script with bindings, duplicate shortcut with case normalization, duplicate alias, duplicate keyword via TypedMetadata, trigger collision via `extra["trigger"]`, three-way collision lists all peers, kind-scoped buckets don't cross-collide (shortcut vs alias with same text), empty-string values skipped, JSON serialization shape. Ranked remaining work: parse-error plumbing (PR2), atomic snapshot + startup migration (PR3), `kit://failed-scripts` MCP resource + menu-bar badge (PR4). Cron/schedule and kenv/pass/preview validation flagged verify-first by Oracle.
- **2026-04-23** Q11 fired and answered (9m07s); tracker updated in `lat.md/agent-understanding-regression-plan.md`. Immediate next work is App Launcher Tab AI projection, then Process Manager row projection, before AURP-19 browser/window-sensitive receipts.
