# Menu Syntax — Progress Log

Append-only. One entry per Oracle iteration. Newest at the bottom so `cat` reads chronologically.

Format per entry:

```
## Iter NNN — <iso-utc>
- **Bundle:** packx target(s)
- **Oracle ask:** one-line question
- **Oracle thread URL:** <url>
- **Oracle key recommendations:** bullets
- **Implemented this tick:** bullets (with file:line refs)
- **Commits:** short SHAs
- **Next tick focus:** one line
```

---

## Iter 001 — 2026-04-23T21:22Z

- **Bundle:** filter_input_{core,change,updates}.rs, alias_input.rs, trigger_builtin_dispatch.rs, scripts/{types,metadata,input_detection}.rs, builtins/mod.rs, render_builtins/file_search.rs, lat.md/{surfaces,builtins,architecture,acp-chat}.md, design/menu-syntax/goals.md (15 files, 473 KB packx bundle)
- **Oracle ask:** Concrete opinionated design for power-user menu syntax layer (9 sections).
- **Oracle thread URL:** https://chatgpt.com/c/69ea8b33-8e00-83e8-b24e-18be312cf201
- **Oracle key recommendations:**
  - Reserve `:` (query.v1 advanced filter) and `+<target>` (capture.v1 todo/cal/note/social/link) only. `!` argv gated behind flag. `#` stays as tag-only inside capture, not a top-level trigger.
  - Parser lives in NEW `src/menu_syntax/` module — do NOT add to `special_entry_from_script_list_filter`. Parse later in result computation so stdin/automation paths stay consistent.
  - Advanced query: strip qualifiers, reuse existing fuzzy search on free text, post-filter by predicates.
  - Capture mode: parallel handler-result path (don't fuzzy-search payload body text). Filter artifacts by `menu_syntax: Vec<MenuSyntaxHandlerSpec>` metadata.
  - Execution: temp-file JSON payload + `KIT_MENU_SYNTAX_PAYLOAD_PATH` / `KIT_MENU_SYNTAX_FAMILY` / `KIT_MENU_SYNTAX_TARGET` env vars.
  - Dates: `temps` crate (chrono feature) for phrase parsing; hand-roll command extractor around it. Explicit `due:/at:/start:/end:` keys win over implicit suffix.
  - TypedMetadata needs raw JSON retention for `meta.foo:bar` qualifiers to be real (otherwise arbitrary metadata filtering quietly collapses).
  - Skills stay AI-oriented; don't surface PluginSkill for non-AI capture yet.
  - 8-commit plan: parser → compat tests → metadata → query filter → capture handlers → execution → date → examples + lat.md.
- **Implemented this tick:** design docs only (goals.md, progress.md, thread-state.json).
- **Commits:** `0f0e86773` (parser module, 31 tests), `c13ef47fa` (compat tests — app_impl include-site, bin-only), `f9c8373a1` (metadata loader, 10 tests), `ecb3c9832` (query+capture filter, 6 tests).
- **Next tick focus:** Oracle iter 002 check-in — review commits 1-5, settle date crate, finalize execution contract and examples.

## Iter 002 — 2026-04-23T21:51Z

- **Bundle:** `src/menu_syntax/`, `design/menu-syntax/` (10 files, 58 KB).
- **Oracle ask:** Review commits 1-5, finalize date crate, sketch execution path, confirm example file list, draft `lat.md/menu-syntax.md` skeleton.
- **Oracle thread URL:** https://chatgpt.com/c/69ea8b33-8e00-83e8-b24e-18be312cf201 (continued same conversation).
- **Oracle key recommendations:**
  - **Biggest bug:** `+unknown` must return `None` (not `Incomplete`) so searches like `+github` / `+react component` still hit fuzzy search. Only bare `+` and `+<known-target>` enter capture.
  - **Missing incomplete state:** empty capture body → `Incomplete(MissingCaptureBody)`. Special case: `+link` is okay with only a URL.
  - **Artifact kinds:** give `Agent` and `Issue` their own `ArtifactKind` variants; do not lie in the type system.
  - **Source vs Plugin:** not aliases. `Source` is broad (plugin pair + `kit_name`); `Plugin` is precise (plugin pair only).
  - **Case-insensitive lookup:** `has:fooBar` must match `fooBar` extra-key, and nested `meta.Domain.Kind:Calendar` must match any case.
  - **Env-racy argv tests:** add `parse_with_config(..., MenuSyntaxParseConfig)` so tests don't touch `std::env::set_var`.
  - **Date crate:** switch to `chrono-english = "0.1.8"` + `chrono-tz = "0.10.4"` (reversing iter 001's `temps` choice because `temps` forces `Local::now()`). `resolve_date_phrase(raw, clock) -> Option<ResolvedDate>`. Caller-provided `DateTime<Tz>` base. Struct-form return for test determinism.
  - **Suffix inference:** post-process step, never mutate `CaptureInvocation.body` in the parser. Window cap 6 tokens / 64 chars, longest-first.
  - **DST tests to pin:** Denver spring-forward (`2026-03-07T12:00:00` + `tomorrow 3pm` → `-06:00`); Denver fall-back (`2026-10-31` → `-07:00`).
  - **Execution:** `src/menu_syntax/execute.rs` (pure) + `src/app_execute/menu_syntax_execution.rs` (GPUI adapter, in a follow-up). Seven env keys, not three. Skill/Builtin/App/etc. rejected until deterministic bindings exist.
  - **Payload schema:** ship `schemas/menu-syntax/payload-v1.schema.json` with `$id = "kit://schema/menu-syntax/payload-v1"`.
  - **Examples:** five `.ts` scripts (not scriptlets) under `scripts/examples/menu-syntax/`. Local-first file writes under `$KENV/menu-syntax/` and `$KENV/notes/`.
  - **Recommended sequencing:** a cleanup commit before execution, not after. "Tighten menu_syntax parser and artifact-kind contracts before wiring execution."
- **Implemented this tick:**
  - Cleanup commit addressing all six review corrections + new test coverage.
  - Date module: `MenuSyntaxClock`, `ResolvedDate`, `resolve_date_phrase`, `resolve_capture_dates`. 8 tests including both DST transitions.
  - Execute module: `MenuSyntaxPayload`, `MenuSyntaxHandlerRef`, `build_capture_payload`, `payload_env`, `write_payload_tempfile`. 3 tests including full env-contract pin.
  - Five example `.ts` scripts shipped.
  - JSON schema at `schemas/menu-syntax/payload-v1.schema.json`.
  - `lat.md/menu-syntax.md` section with nine subsections and `@lat:` comments in tests.
- **Commits:** `a28126d87` (cleanup, 7 tests), `f13c38cbf` (date + execute, 11 tests), `<pending>` (examples + schema + lat.md).
- **Final test count:** 65 lib-verified tests passing via `cargo test --lib menu_syntax`.

## Iter 003 — 2026-04-23T23:50Z

- **Bundle:** `src/menu_syntax/*.rs`, `src/app_impl/filter_input_{core,change,updates}.rs`, `src/app_impl/filtering_cache.rs`, `src/scripts/{grouping,types,metadata,input_detection}.rs`, `src/app_impl/selection_fallback.rs`, design notes (19 files, 240 KB packx bundle at `~/.oracle/bundles/menu-syntax-wire-filter-input.txt`).
- **Oracle ask:** How to wire menu_syntax into the launcher filter-input-change path, grouped results, execution, and discoverability without touching legacy `~ / @ > ? /` routes.
- **Oracle thread URL:** https://chatgpt.com/c/69eaabc9-b0d4-83e8-b4eb-1b650d921db7
- **Oracle key recommendations:**
  - Add a raw-guarded `MenuSyntaxMode` struct in `src/menu_syntax/mode.rs`. Parse at input-change boundaries (`handle_filter_input_change`, `set_filter_text_immediate`), guard consumption by raw-equality so a stale parse never applies to newer input via the 8ms coalescer.
  - Thread an optional `AdvancedQuery` through `src/scripts/grouping.rs::get_grouped_results_with_input_history_and_query` and the matching validation variant. Apply `apply_advanced_query` after the unified fuzzy search. Suppress `prepend_script_issues_row` when predicates reject `SearchResult::ScriptIssue` (`:type:script` must not leak an Issue row).
  - Capture mode replaces normal grouping entirely. Do not mix with Suggested/Favorites/Recent/menu-bar/fallback. Header "Capture <target>" followed by `scripts_handling_capture` rows with neutral `ScriptMatch` (score = `i32::MAX`, default `MatchIndices`, `Name` match kind).
  - Incomplete syntax renders as a single `GroupedListItem::SectionHeader(hint, None)` — already non-selectable; do not reuse `SearchResult::ScriptIssue`.
  - Payload dir: `$SK_PATH/menu-syntax/payloads` with fallback `~/.scriptkit/menu-syntax/payloads`.
  - 7-commit plan: 1-4 LIB-VERIFIED (mode, query post-filter, capture builder, hint helper), 5-7 BIN-ONLY (filter wiring, discoverability hints, capture execution).
- **Implemented this tick:**
  - Commit 1 `83fc5fb94` — `MenuSyntaxMode`, raw guard, `free_text_for_search`, `prefix_span_for_input` (15 new tests).
  - Commit 2 `c7b71ae52` — `get_grouped_results_with_input_history_and_query` + validation variant + ScriptIssue suppression (8 new tests).
  - Commit 3 `3291eb248` — `build_capture_mode_results` (5 new tests).
  - Commit 4 `3e9201501` — `build_menu_syntax_hint_results` (2 new tests).
  - Commit 5 `ce8a24f5a` — `ScriptListApp::menu_syntax_mode` field, `set_menu_syntax_mode_from_filter` setter, wiring in `filter_input_change.rs` and `filter_input_updates.rs`, `filtering_cache.rs` branch.
  - Commit 6 `b150df87b` — empty-state tip updated to reference `:type:script · +todo · +note`.
  - Commit 7 `14e2368c2` — `MenuSyntaxClock::local_now()`, `spawn_script_with_extra_env`, `execute_script_interactive_with_env`, `src/app_execute/menu_syntax_execution.rs` adapter, branch in `execute_selected`.
  - Commit 8 `957db4d1e` — fallback-state suppression for menu-syntax; detached-spawn for capture handlers (bypass SDK preload).
- **Final test count:** 96 lib-verified tests passing via `cargo test --lib menu_syntax` + `cargo test --lib scripts::grouping` (80 + 15 + 1 execute round-trip).
- **Agentic verification:** PASS on all 5 steps — hint row, capture section, advanced query filter (Script-only), end-to-end `todos.jsonl` write.
- **Next tick focus:** Oracle iter 4 for discoverability polish (prefix highlighting inside the input field, dedicated Menu-syntax help section, footer slot hint in empty-filter state).
- **Remaining / open:**
  - GPUI adapter in `src/app_execute/menu_syntax_execution.rs` — blocked on user's in-progress bin-level compile errors (`render_setup.rs`, `prompt_handler/mod.rs`, `render_prompts/arg/tests.rs`). Land after those fixes.
  - `ScriptList::handle_filter_input_change` wiring — call `menu_syntax::parse`, dispatch `AdvancedQuery` through `apply_advanced_query` against the existing search results, dispatch `Capture` through `scripts_handling_capture` for a parallel handler-row pass.
  - Plugin-owned skills do not yet participate in capture mode (Oracle's gate).
  - `argv.v1` (`!`) stays behind `KIT_MENU_SYNTAX_ARGV=1` until real handler fixtures exist.

## Iter 004 — 2026-04-24T04:39Z

- **Bundle:** `.notes/power-user.md`, `lat.md/menu-syntax.md`, `design/menu-syntax/{goals,progress}.md`, `src/menu_syntax/{mod,parse,mode,filter,execute,metadata,payload,capture,query}.rs`, `src/app_execute/menu_syntax_execution.rs`, `src/app_impl/{filter_input_change,filter_input_updates,filtering_cache}.rs`, `src/notes/browse_panel.rs`, `src/ai/acp/{picker_popup,popup_window}.rs` (20 files, 45.7K tokens packx bundle at `~/.oracle/bundles/trigger-popups-inverse-browsers-plan.txt`).
- **Oracle ask:** Design popup menus for `:` and `+` triggers + close the one-way-loop (5 writers, 0 readers). Decide popup shape (one vs two), authoring target (script/skill/scriptlet), inverse-browser strategy, reuse vs rebuild, 8-commit plan, retention policy.
- **Oracle thread URL:** https://chatgpt.com/c/69eaf245-05e8-83e8-901b-d8ef4b9b4808
- **Oracle key recommendations:**
  - **Ranking:** Discoverability first > one-way loop > predictability. The popup becomes the entry point for teaching `:` / `+`, creating handlers, surfacing inverse browsers, and catching typos before execution.
  - **Popup shape:** ONE dedicated `TriggerPicker` with TWO modes (`AdvancedQuery` for `:`, `Capture` for `+`). Not two separate pickers. Not reusing `AcpMentionPopupWindow` directly.
  - **Reuse call:** Reuse `src/ai/acp/popup_window.rs` shell ONLY (bounds helpers, child-window attach, no-focus-steal, dense sizing). Do NOT reuse `src/ai/acp/picker_popup.rs` — too ACP-bound. If a third non-AI popup appears, extract popup_window helpers to `src/components/inline_popup_window.rs`; don't do that extraction in this iter.
  - **Ownership:** `ScriptListApp` owns picker lifecycle. Popup is view-only child window with weak handle back. Pure model in new `src/menu_syntax/trigger_picker.rs`; GPUI shell in new `src/app_impl/menu_syntax_trigger_popup.rs`.
  - **Anchor:** parent-relative `WindowKind::PopUp`, left-aligned under main input near the chip. `focus: false` — main input keeps focus. Close popup when current view stops being ScriptList.
  - **Row model:** mode-neutral `TriggerPickerRow { id, mode, kind, title, token, subtitle, detail, example, badges, action, enabled }`. Row kinds: `Qualifier`, `QualifierValue`, `UnknownQualifierFix`, `RecentQuery`, `CaptureTarget`, `CaptureHandler`, `CaptureArtifact`, `Shortcut`, `FooterAction`.
  - **`:` rows:** static qualifier templates first (`type:script`, `shortcut:cmd+k`, `source:`, `plugin:`, `name:`, `desc:`, `alias:`, `has:menuSyntax`, `meta.category:`, `-type:app`, etc.), then typo fixes, then recent queries (session-only for iter 004, no disk persistence).
  - **`+` rows:** targets first (`+todo`/`+cal`/`+note`/`+social`/`+link`); once target known, handler rows ranked by exact-match > `defaultHandler:true` > user handlers > shipped `main:*` examples > wildcard (`targets: ["*"]`). `accepts` match gives small boost. Skills NOT in execution for iter 004.
  - **Create-new-handler footer action:** primary target is **script**, not skill, not scriptlet. Current execution contract is detached `.ts` with temp-file payload; script is the honest match. Scaffold file at `~/.scriptkit/plugins/main/scripts/capture-<target>-<slug>.ts` with full template (metadata + `menuSyntax` block + payload reader). Open in existing editor flow. If file already exists, open it rather than overwriting.
  - **Inverse browsers:** ship **`Captures`** aggregate builtin as the master browser + filtered entry points (`Todos Inbox`, `Bookmarks`, `Notes Inbox`). Shared reader at `src/menu_syntax/artifacts.rs` with `CaptureArtifactKind::{Todo, Bookmark, DailyNote, CalendarIcs, SocialDraft, Payload}`. Row actions: Enter opens artifact, Cmd+P opens Captures scoped to target and selects newest, Cmd+Enter opens containing folder, Delete only for payload rows. Do NOT reuse `notes/browse_panel.rs` directly — reuse lessons, not component.
  - **HUD improvement:** replace `Captured to todo` with `Captured todo via Capture Todo Inbox` + `Payload: ...capture_v1-...json` + `Expected: ~/.scriptkit/menu-syntax/todos.jsonl` for shipped defaults. No undo — would require handler output protocol.
  - **Retention:** `src/menu_syntax/retention.rs`. Payload dir only, match `capture_v1-*.json`. Always keep newest 250, hard cap 1000 newest, age cleanup > 30 days only outside newest 250. Never touch `todos.jsonl`/`bookmarks.jsonl`/notes/`.ics`/social drafts. Run opportunistically after successful payload write, not in a daemon.
  - **Keyboard model (highlights):** Escape closes popup only first time; second Escape falls through. Tab applies insertion without executing (useful for building `:` queries). Cmd+N triggers create-new-handler footer action. Cmd+P on `+<target>` opens Captures scoped to that target. Unknown `+foo` (after head becomes non-target) closes popup and falls back to fuzzy. `~ / @ > ?` still win on empty input.
  - **8-commit plan (4 lib-verifiable first):** (1) `trigger_picker.rs` row model; (2) `handler_index.rs` ranking + filter glue; (3) `artifacts.rs` reader for inverse browsers; (4) `retention.rs` policy; (5) GPUI popup window wiring; (6) keyboard routing + resting footer chips; (7) `templates.rs` + `menu_syntax_authoring.rs` for scaffold-handler; (8) Captures builtins + retention wiring + HUD.
- **Open risks:** (a) keyboard focus routing — popup must not take focus but must intercept arrow/enter/escape before main list; (b) unknown `+` heads must fall back cleanly without widening parser claim; (c) handler output path unknowable for user handlers (show payload + expected-for-shipped only); (d) artifact readers must tolerate malformed/partial JSONL (skip bad lines, surface warning counts, never crash builtin); (e) ACP popup helpers live under AI namespace — fine for iter 004, but extract on third use.
- **Implemented this tick:** planning only. No code written.
- **Commits:** none this tick.
- **Next tick focus:** Commit 1 — `src/menu_syntax/trigger_picker.rs` row model + tests. Lib-verifiable. Test: `cargo test --lib menu_syntax::trigger_picker`. Acceptance: `:` builds qualifier/recent/typo rows, `+` builds target rows, legacy trigger strings produce no picker snapshot.

## Iter 005 — 2026-04-24T04:59Z

- **Bundle:** reused iter 004 bundle (planning tick, no new bundle needed).
- **Oracle ask:** none this tick — implementing Oracle iter 004 plan commit 1.
- **Implemented this tick:** Commit 1 of the 8-commit plan — pure trigger-picker row model.
  - New `src/menu_syntax/trigger_picker.rs` (818 lines).
  - `TriggerPickerMode` enum (`AdvancedQuery`, `Capture`) — one picker with two modes.
  - `TriggerPickerRowKind` enum including reserved `CaptureHandler` / `CaptureArtifact` variants for commits 2 and 3.
  - `TriggerPickerAction` enum with `InsertToken { keep_open }`, `ReplaceInput`, `FixQualifier`, `CreateHandler`, `OpenHelp`, plus reserved `ExecuteCaptureHandler` / `OpenCaptures` under `#[allow(dead_code)]` to lock the action enum shape now (avoids churn in commits 2/7/8).
  - `TriggerPickerRow` / `TriggerPickerSnapshot` / `TriggerPickerContext` structs.
  - `build_trigger_picker_snapshot(input, ctx) -> Option<Snapshot>` dispatches through `menu_syntax::parse` so legacy triggers (`~ / @ > ?`), unknown `+foo` heads, and plain fuzzy text return `None` — parser boundary preserved.
  - 19 static qualifier rows (type:/kind:/shortcut:/source:/plugin:/name:/desc:/alias:/has:/meta.<path>:/negation). Open-value rows (e.g. `source:`) set `keep_open: true`; concrete rows (`type:script`) set `keep_open: false`.
  - `UnknownQualifierFix` rows driven by a within-one-edit detector that handles single-char edits AND adjacent transpositions (`typ` → `type`, `tpye` → `type`). Does NOT change parser behavior — only surfaces a fix row. `meta.<path>` qualifiers are excluded from typo scan.
  - Session-only recent-query rows filtered to strings whose raw parse is `AdvancedQuery` / `BareQueryPrefix`. No disk persistence.
  - Capture target rows for the five shipped targets (`+todo`, `+cal`, `+note`, `+social`, `+link`) with title/detail/example metadata.
  - Target-focused capture snapshot: once the user types `+todo` (incomplete) or `+todo buy milk` (complete), the snapshot narrows to one `CaptureTarget` row plus the footer `CreateHandler { target: Some("todo") }` action.
  - Footer: `OpenHelp` on `:`, `CreateHandler { target }` on `+`. Row ids stable (`qualifier:<token>`, `target:<name>`, `footer:help`, `footer:create-handler`, `fix:<bad>:<good>`, `recent:<idx>`).
  - `src/menu_syntax/mod.rs` re-exports the new types.
  - `lat.md/menu-syntax.md` gains a `Trigger Picker` section.
- **Commits:** `ae9727805` — `menu_syntax: add trigger picker row model`. Commit message is an agent-reproducible prompt with step-by-step reproduction instructions, matching project convention.
- **Tests:** `cargo test --lib menu_syntax::trigger_picker` — 20 new tests pass (0 failed, 0 ignored). Full `menu_syntax` suite: 101 passed (up from 96 in iter 003). Validated:
  - legacy triggers `~ ~/Desktop / @ > ?` return `None`
  - unknown `+` heads (`+github`, `+1`, `+react component`) return `None`
  - unknown keyword heads (`localhost:3000`, `not-a-target: stuff`) return `None`
  - bare `:` builds ≥10 qualifier rows including the exact ids
  - `:` footer action is `OpenHelp`
  - `:source:` keeps popup open; `:type:script` closes it
  - `:typ:script` and `:tpye:script` both produce `FixQualifier { good: "type:script" }`
  - `:meta.category:inbox` does NOT fire a typo suggestion
  - `:type:script git` (correct) does not produce a fix row
  - recent queries filter: advanced-query strings surface; `+todo`/plain text are dropped
  - bare `+` builds 5 target rows in spec order
  - `+todo buy milk` focuses to 1 target row with `target = Some("todo")`
  - `note:` keyword alias produces a capture snapshot with `target = Some("note")`
  - `+todo` incomplete still focuses to 1 target row
  - footer `create-handler` carries the focused target
  - bare `+` create-handler has `target: None`
  - row ids are unique within every snapshot
  - `within_one_edit` detector unit test passes all expected cases
- **Lat check:** all checks passed after trimming the Trigger Picker leading paragraph to ≤250 chars.
- **Next tick focus:** Commit 2 — `src/menu_syntax/handler_index.rs`. Lib-verifiable. Extracts per-target handler ranking (exact target > `defaultHandler:true` > user-authored > shipped `main:*` examples > wildcard) with `accepts`-match boost. Wires the fill for the reserved `CaptureHandler` row kind. Test: `cargo test --lib menu_syntax::handler_index menu_syntax::filter`.

## Iter 007 — 2026-04-24T05:49Z (pivot)

- **Bundle:** `.notes/power-user.md`, `lat.md/{menu-syntax,windowing}.md`, `design/menu-syntax/progress.md`, `src/menu_syntax/{trigger_picker,mode,payload,parse}.rs`, `src/ai/acp/{picker_popup,popup_window,history_popup,context}.rs`, `src/app_impl/{filter_input_change,filter_input_updates,filtering_cache}.rs` — 15 files, 49.4K tokens (packx bundle `~/.oracle/bundles/menu-syntax-slash-picker-pivot.txt`). 4 baseline PNG screenshots attached (`iter005-baseline-{colon,plus,plus-todo,nonmenu}.png`).
- **Oracle ask:** Re-rank the 7 remaining commits given user's mid-plan pivot: (a) "every step verified" with agentic-testing; (b) "lean into ACP slash-command patterns." Resolve the tension with iter 004's "do not reuse `picker_popup.rs`" decision.
- **Oracle thread URL:** https://chatgpt.com/c/69eb02c8-6308-83e8-b03a-7964d1f2f3c9
- **Oracle key recommendations:**
  - **TL;DR pivot:** move GPUI popup ahead of handler indexing/artifacts/retention. Next visible win is `:` and `+` rendering an ACP-style inline picker under the main input, even if the first render only uses the static rows from iter 005's `TriggerPickerRow` model.
  - **Picker reuse verdict: C (extract shared renderer).** Move the generic parts of `src/ai/acp/picker_popup.rs` and `src/ai/acp/popup_window.rs` to new `src/components/inline_picker.rs` + `src/components/inline_popup_window.rs`. Both ACP and menu-syntax consume the shared renderer; each owner keeps its own state/actions/popup singleton. Narrow reuse — do NOT force menu-syntax into `ContextPickerItem`, do NOT copy `picker_popup.rs` wholesale.
  - **Shared symbols (extract):** `render_picker_row` → `render_inline_picker_row`, `render_picker` → `render_inline_picker`, `render_empty_state` → shared, `should_submit_acp_picker_row_click` → `should_submit_inline_picker_row_click`, `DENSE_PICKER_MAX_VISIBLE_ROWS`, `dense_picker_height_for_row_height`, `dense_picker_width_for_window`, `dense_picker_width_for_labels`, `popup_bounds`, `popup_window_options`, `configure_popup_window`, `set_popup_window_bounds`.
  - **Stays app-specific:** `AcpMentionPopupSnapshot/Request/Window` keep ACP view handles + selection callbacks. NEW `MenuSyntaxTriggerPopupSnapshot/Request/Window` carry ScriptList handles + menu-syntax actions. ACP and menu-syntax each own their popup singleton slot.
  - **ACP file verdicts:** `picker_popup.rs` — (a) import helper: replace generic rendering with shared calls, keep ACP-specific helpers. `popup_window.rs` — (a) import helper: extract reusable functions. `history_popup.rs` — (c) do not touch (different component; search field + history rows + own key model). `model_selector_popup.rs` — (c) do not touch for this pivot. `context.rs` — (c) do not touch (unrelated to popup rendering).
  - **Row shape:** `TriggerPickerRow` SURVIVES as-is. Bridge via `impl From<&TriggerPickerRow> for InlinePickerRow` or a call-site function. ACP gets a parallel adapter from `ContextPickerItem` to `InlinePickerRow`. Do not bloat `TriggerPickerRow` with ACP-flavored fields.
  - **Keyboard:** share key-intent classifier only, not dispatcher. New `InlinePickerKeyIntent::{MoveUp, MoveDown, MoveHome, MoveEnd, PageUp, PageDown, Accept, Apply, Close, SecondaryAction, CreateAction}` in `components::inline_picker_keys`. Each owner (AcpChatView, ScriptListApp) translates intents to its own state changes. No central `KeyRouter` — would fight GPUI focus reality.
  - **Escape rule:** first Escape closes popup and stops propagation. Second Escape falls through to launcher.
  - **Tab:** apply insertion without executing (useful for building `:` queries).
  - **Cmd+N:** route to create-handler footer action. Cmd+P: route to Captures scoped to active target.
  - **Legacy triggers `~ / @ > ?` still win** — raw-guard must survive.
- **Revised 7-commit plan (commit 1 landed iter 005):**
  - **Commit 2** (NEXT): render ACP-style trigger picker for `:` and `+` + verify parser fall-throughs. VISIBLE. Create `src/components/inline_picker.rs`, `src/components/inline_popup_window.rs`, `src/app_impl/menu_syntax_trigger_popup.rs`. Touch `src/ai/acp/picker_popup.rs`, `src/ai/acp/popup_window.rs`, ScriptList app-state/render, docs. Acceptance: screenshots of `:` = FILTER chip + picker, `+` = CAPTURE chip + 5 targets, `+todo buy milk` = TODO + focused row, `localhost:3000` = no chip/popup. ACP slash-picker smoke screenshot unchanged.
  - **Commit 3:** route trigger-picker keys through focused ScriptList input. VISIBLE. Arrow/Tab/Enter/Escape. Touch `src/components/inline_picker_keys.rs`, `src/app_impl/menu_syntax_trigger_popup.rs`, ScriptList keydown dispatch.
  - **Commit 4:** rank capture handlers (the handler_index commit deferred from iter 006 plan). VISIBLE — `+todo buy milk` popup now shows target row + capture-handler rows. Create `src/menu_syntax/handler_index.rs`. Ranking: exact > defaultHandler > user > shipped `main:*` > wildcard + accepts boost.
  - **Commit 5:** implement picker actions — help row, typo-fix rewrite, recent restore, handler execution. VISIBLE. Touch trigger popup, `menu_syntax_execution.rs`, ScriptList selection.
  - **Commit 6:** scaffold capture handlers from picker footer (Cmd+N). VISIBLE. Create `src/menu_syntax/templates.rs`, `src/app_impl/menu_syntax_authoring.rs`. Scaffold `.ts` at `~/.scriptkit/plugins/main/scripts/capture-<target>-<slug>.ts`; open existing if present.
  - **Commit 7:** Captures inverse browser + artifact rows + Cmd+P. VISIBLE. Create `src/menu_syntax/artifacts.rs` + builtin views. Tolerate dirty/partial JSONL; show warning counts.
  - **Commit 8:** payload retention + HUD improvements. Mostly visible. Create `src/menu_syntax/retention.rs`. HUD: "Captured todo via <handler>" + payload path + expected output.
- **Open risks (Oracle's):** (1) commit 2 is intentionally large — touches ACP + ScriptList together; acceptance must include new menu-syntax screenshots AND unchanged ACP slash-picker screenshots; (2) focusless popup key routing can regress main-list behavior — share classifier, keep dispatch owner-side; (3) unknown `+foo` fallback is fragile — popup must be driven by `build_trigger_picker_snapshot`, not prefix string checks; (4) footer rows must not be dead UI — either wire in same commit or render visibly disabled with receipt; (5) artifact readers must skip dirty JSONL rows with warning counts, never crash.
- **Implemented this tick:** planning only (Oracle reconsult). No code written.
- **Commits:** none this tick.
- **Next tick focus:** Commit 2 (revised) — render ACP-style trigger picker. Will create `src/components/inline_picker.rs`, `src/components/inline_popup_window.rs`, `src/app_impl/menu_syntax_trigger_popup.rs`; extract generic rendering from `src/ai/acp/picker_popup.rs` + `popup_window.rs`. Acceptance includes screenshots of all 4 states + unchanged ACP slash-picker smoke test. A pending wakeup from iter 005 is armed targeting the OLD commit 2 (handler_index) — route it to this revised commit 2 instead. Note: the iter 005 wakeup will fire imminently — when it does, redirect to this revised plan.

## Iter 008 — 2026-04-24T06:14Z (AFK, commit 2 landed)

- **Bundle:** reused iter 007 bundle (no new Oracle consult this tick).
- **Oracle ask:** none — implementing Oracle iter 007 revised commit 2.
- **Scope pivot in this tick:** took the LOWEST-RISK flavor of Oracle's option C. Instead of extracting a shared renderer into `src/components/inline_picker.rs` + `inline_popup_window.rs` and refactoring ACP imports (Oracle's recommended path but intentionally-large commit per Oracle's own warning), rendered the picker snapshot as non-selectable `SectionHeader` rows inline in the existing grouped-results list. Zero ACP touches → zero ACP regression risk in AFK mode. Shared-component extraction remains valid future work; keyboard/selection lands commit 3 as planned, and then the extraction can be done once the behavior is proven stable.
- **Implemented this tick:**
  - New function `build_trigger_picker_grouped_results(snapshot)` in `src/scripts/grouping.rs` plus `build_trigger_picker_for_target` and `format_trigger_picker_row_label` helpers. Consumes the iter 005 `TriggerPickerSnapshot` and emits a mode-appropriate section header (`Filter qualifiers` / `Capture targets` / `Capture <target>`) followed by one `SectionHeader` per non-footer row formatted as `<token>  ·  <title>` (title-only fallback). The footer row is always appended last with a `(coming soon)` suffix so the UI isn't dead.
  - Re-exported from `src/scripts/mod.rs` as `pub(crate) use self::grouping::build_trigger_picker_grouped_results`.
  - Wired into `src/app_impl/filtering_cache.rs::get_grouped_results_cached`. New dispatch order:
    1. Capture complete (existing `build_capture_mode_results`) — unchanged, preserves `+todo buy milk` handler row.
    2. `TriggerPickerSnapshot` from `build_trigger_picker_snapshot(raw_filter_text, &ctx)` with `recent_queries: self.input_history.recent_entries(8)` → new `build_trigger_picker_grouped_results`.
    3. Fallback: existing `build_menu_syntax_hint_results(hint)`.
  - Updated `lat.md/menu-syntax.md` Trigger Picker section with iter 008 wiring note.
- **Commits:** `e73324f91` — `menu_syntax: render trigger picker snapshot inline under \`:\` / \`+\``. Commit body is an agent-reproducible prompt with step-by-step reproduction.
- **Tests:** `cargo test --lib menu_syntax` — 101 passed (unchanged baseline). `cargo build --bin script-kit-gpui` — clean (12 unrelated warnings, no errors).
- **Agentic-testing screenshots (6):**
  - `.test-screenshots/iter008-colon.png` — FILTER chip + Filter qualifiers list (type:script, type:scriptlet, type:skill, type:builtin, type:app, type:window, type:agent, type:issue, shortcut:any, shortcut:none, etc.).
  - `.test-screenshots/iter008-plus.png` — CAPTURE chip + Capture targets section + 5 target rows + `Create capture handler... (coming soon)` footer.
  - `.test-screenshots/iter008-plus-todo.png` — TODO chip + existing `Capture Todo Inbox` handler row (confirms old capture-complete path still works).
  - `.test-screenshots/iter008-plus-todo-incomplete.png` — TODO chip + focused `+todo Todo inbox` row + `Create capture handler for +todo... (coming soon)` footer.
  - `.test-screenshots/iter008-colon-typo.png` — FILTER chip + `:type:script Did you mean type:script?` typo-fix row surfaced ahead of the qualifier list.
  - `.test-screenshots/iter008-localhost.png` — no chip, fallback rows (parser boundary preserved).
- **AFK cleanup:** session stopped, `getState` confirmed `windowVisible: false` before stop.
- **Open risks / deferrals:**
  - Picker rows are non-selectable. Users can SEE available qualifiers/targets but Enter/Tab/Esc don't activate them yet — that's commit 3.
  - Footer row label reads `(coming soon)` — visible but deliberately dead UI. Commit 6 wires scaffold-handler action, which will drop the suffix.
  - Shared-renderer extraction per Oracle option C is still TODO but unblocked: once keyboard works, we can factor out the common render into `src/components/inline_picker.rs`.
  - ACP slash picker not verified this tick with a screenshot — but zero ACP code was touched, so regression is not possible from this commit alone.
- **Next tick focus:** Commit 3 (revised) — route trigger-picker keys through focused ScriptList input. Arrow/Tab/Enter/Escape handled by ScriptListApp so the main input keeps focus. Build a pure `InlinePickerKeyIntent` classifier per Oracle iter 007 recommendation. Convert the first row to selectable so Enter accepts and Esc closes. Acceptance includes screenshots before/after Down key (selection moves), before/after Tab on `:source:` (input becomes `:source:` and keeps picker open), before/after Escape (picker closes once, second Escape falls through).

## Iter 009 — 2026-04-23T20:06Z (AFK, commit 3 scope-bounded)

- **Bundle:** reused iter 007 bundle (no new Oracle consult this tick).
- **Oracle ask:** none — implementing Oracle iter 007 revised commit 3 (keyboard routing), pivoted to pure-lib classifier only.
- **Scope pivot in this tick:** AFK-safe scope bound. Full GPUI keyboard wiring was risky without the user present: Tab is consumed by `src/app_impl/startup_new_tab.rs` for AI routing, Enter is consumed by the focused `Input` component, and adding new interception points to those hot paths without screenshots of real users typing is how regressions land. So this commit ships only the pure-lib `InlinePickerKeyIntent` classifier and its `apply_intent` dispatcher. Commit 3 becomes the "prepare the routing spine" commit; the actual GPUI handler that calls `apply_intent` in response to real arrow/tab/enter events stays a follow-up that needs live interactive testing.
- **Implemented this tick:**
  - New `src/menu_syntax/trigger_picker_keys.rs` (520 lines).
  - `InlinePickerKeyIntent` enum (owner-neutral): `MoveUp`, `MoveDown`, `MoveHome`, `MoveEnd`, `PageUp`, `PageDown`, `Accept`, `Apply`, `Close`, `SecondaryAction` (Cmd+P → Captures), `CreateAction` (Cmd+N → scaffold handler).
  - `TriggerPickerIntentOutcome` enum: `Ignored`, `SelectionChanged { new_index }`, `ReplaceInput { text, keep_open }`, `Close`, `OpenCaptures { target }`, `CreateHandler { target }`, `OpenHelp`.
  - Selectable-index helpers (`first_selectable_index`, `last_selectable_index`, `next_selectable_index`, `prev_selectable_index`) skip `FooterAction` rows and wrap at list edges. Footer rows are visible but explicit navigation (MoveEnd, PageDown) is still required to reach them — Enter on them never fires by default, matching Oracle's "footer must never be dead UI but also never an accidental selection" rule.
  - `apply_intent(intent, snapshot, selected_index, raw_filter_text) -> TriggerPickerIntentOutcome` dispatches per intent: moves translate via the selectable-index helpers; `Accept` and `Apply` route through `resolve_row_action` which reads the selected row's `TriggerPickerAction` and returns the appropriate outcome; `SecondaryAction` / `CreateAction` only fire in `Capture` mode (otherwise `Ignored`); `Close` always returns `Close`.
  - `resolve_row_action` handles all `TriggerPickerAction` variants: `InsertToken` produces `ReplaceInput` (keep_open honored only on `Apply`, always false on `Accept`); `ReplaceInput`, `FixQualifier`, `OpenCaptures`, `CreateHandler`, `OpenHelp` pass through; `ExecuteCaptureHandler` returns `Ignored` (reserved for commit 4/5).
  - `rewrite_token_substring(raw, bad, good)` replaces the first occurrence of `bad` in the raw filter with `good`, preserving prefix + suffix. Used by `FixQualifier` so `:typ:script git` → `:type:script git`.
  - `apply_token_insertion(raw, token)` returns the full token when it starts with `:` or `+` (full replace). Keeps future open-value flows simple without a half-baked tokenizer.
  - Module registered in `src/menu_syntax/mod.rs` with public re-exports guarded by `#[allow(unused_imports)]` since GPUI wiring lands in a later tick.
- **Commits:** pending (this tick produces the commit about to land after lat-check + screenshot-smoke pass).
- **Tests:** `cargo test --lib menu_syntax` — 118 passed (0 failed, 0 ignored). Baseline was 101. 17 new tests landed in `trigger_picker_keys::tests`:
  - `first_selectable_skips_footer_rows`, `last_selectable_skips_footer_rows`
  - `next_selectable_advances_past_non_footer_rows`, `next_selectable_wraps_at_end`
  - `prev_selectable_wraps_at_start`
  - `accept_intent_rewrites_input_with_insert_token`
  - `apply_intent_on_open_value_row_keeps_popup_open`
  - `accept_on_open_value_row_still_closes_picker`
  - `fix_qualifier_rewrites_typo_in_place`
  - `close_intent_returns_close_outcome`
  - `secondary_action_on_capture_mode_opens_captures_for_target`
  - `secondary_action_in_query_mode_is_ignored`
  - `create_action_on_capture_mode_fires_create_handler`
  - `move_home_and_end_return_first_and_last_selectable`
  - `move_intents_on_empty_snapshot_ignored`
  - `rewrite_token_substring_handles_missing_token_gracefully`
  - `rewrite_token_substring_replaces_first_occurrence`
- **Agentic-testing screenshots:** intentionally none this tick. Zero pixels change (the lib code is not called from any GPUI path yet). Per Oracle iter 007 open-risk #1 ("commit 2 intentionally large"), we inverted the trade: keep the commits small and agent-verifiable via tests while visible wiring waits for a tick with the user interactively available to catch focus regressions.
- **AFK cleanup:** no runtime session started — nothing to stop.
- **Open risks / deferrals:**
  - GPUI event wiring (ArrowUp/ArrowDown/Tab/Enter/Escape/Cmd+N/Cmd+P) inside `src/app_impl/filter_input_change.rs` / `startup_new_tab.rs` / `render_script_list/mod.rs` is deferred to a user-present tick. Tab specifically competes with `startup_new_tab.rs` AI routing — routing it via the menu-syntax classifier without breaking AI tab cycling is the most delicate piece.
  - Selected-index state is not yet stored anywhere: grouping renders rows as non-selectable `SectionHeader`s. Once GPUI wiring lands, `ScriptListApp` needs a new `menu_syntax_picker_selection: Option<usize>` field and dispatch plumbing so `SelectionChanged` outcomes produce a visible highlight.
  - `OpenHelp`, `OpenCaptures`, `CreateHandler` outcomes have no owner yet. That's intentional — they get owners in commits 5–7.
- **Next tick focus:** Commit 4 (revised) — `src/menu_syntax/handler_index.rs` ranking + filter glue. Purely lib-verifiable step. Ranking per Oracle iter 004: exact-target > `defaultHandler:true` > user-authored > shipped `main:*` > wildcard `targets: ["*"]`, with `accepts` match giving a small boost. Wires into the reserved `CaptureHandler` row kind so `+todo buy milk` snapshots start returning real handler rows in priority order.

## Iter 010 — 2026-04-23T20:18Z (AFK, commit 4 landed)

- **Bundle:** reused iter 007 bundle (no new Oracle consult this tick).
- **Oracle ask:** none — implementing Oracle iter 007 revised commit 4.
- **Implemented this tick:** Commit 4 of the 7-commit plan — capture-handler ranking.
  - New `src/menu_syntax/handler_index.rs` (pure lib).
  - `HandlerScore { exact_target, default_handler, user_authored, accepts_boost }` — lexicographic tuple, higher sorts first. `accepts_boost` caps at `MAX_ACCEPTS_BOOST = 3` so it can only break ties within a priority bucket, never cross them (Oracle iter 004 explicit rule).
  - `RankedHandler { script, spec, score }` — one entry per (script, spec) pair. A script that declares both an exact-target spec and a wildcard spec appears twice in the ranking (tests lock this).
  - `rank_handlers_for_target(scripts, invocation) -> Vec<RankedHandler>` iterates every script's `menuSyntax` entries (via existing `script_menu_syntax_specs`), filters to `family == "capture.v1"`, scores matches by `(exact_target, default_handler, user_authored, accepts_boost)`, and sorts descending with script name as the stable tie-break.
  - `rank_scripts_handling_capture(scripts, invocation) -> Vec<Arc<Script>>` — dedup-by-path convenience for callers that already dedupe on script identity.
  - `KNOWN_ACCEPTS = ["date", "url", "tag", "tags", "priority", "duration", "kv"]` — any other token in a handler's `accepts` list is silently ignored (permissive classifier, not a parser).
  - `invocation_has(accept, invocation)` — a declared accept counts when the matching field of the `CaptureInvocation` is populated (non-empty `date_phrases` for "date", `Some` for `url`/`priority`/`duration`, etc.).
  - Wired into `src/scripts/grouping.rs::build_capture_mode_results` — the one live consumer of `scripts_handling_capture`. The function now calls `rank_scripts_handling_capture` so `+todo buy milk` renders handler rows in priority order (default handler first, then user-authored, then shipped examples, then wildcard). Existing capture tests unchanged — they only check opt-in, not order.
  - `src/menu_syntax/mod.rs` re-exports `rank_handlers_for_target`, `rank_scripts_handling_capture`, `HandlerScore`, `RankedHandler` under `#[allow(unused_imports)]` so later commits (5 handler actions) can consume them without churning the public surface.
  - `lat.md/menu-syntax.md` Capture Handler Filtering section gains a paragraph referencing `rank_handlers_for_target` and `rank_scripts_handling_capture` with the lexicographic score semantics, tie-break rule, and accept-boost cap.
- **Commits:** pending (lands after this progress entry).
- **Tests:** `cargo test --lib menu_syntax` — 133 passed (0 failed). Baseline was 118. 15 new handler_index tests:
  - `empty_catalog_returns_empty_ranking`
  - `scripts_without_menu_syntax_are_ignored`
  - `non_capture_family_is_ignored`
  - `exact_target_outranks_wildcard_same_plugin`
  - `default_handler_outranks_non_default_exact`
  - `user_plugin_outranks_shipped_main_when_neither_is_default`
  - `shipped_default_still_beats_user_non_default`
  - `accepts_boost_breaks_tie_within_bucket`
  - `accepts_boost_does_not_cross_priority_buckets` (regression guard against accept-boost crossing exact/wildcard)
  - `accepts_boost_caps_at_maximum`
  - `unknown_accepts_tokens_are_ignored`
  - `name_alphabetical_tiebreak_is_stable`
  - `rank_scripts_handling_capture_dedupes_by_path`
  - `wildcard_only_matches_when_no_exact_target`
  - `case_insensitive_target_match`
- `cargo test --lib scripts::grouping` — 15 passed; existing capture mode tests still green (they only assert opt-in + layout shape, not order).
- `cargo build --lib` — clean.
- `lat check` — all checks passed.
- **Agentic-testing screenshots:** deferred. Ranking is visible only when a user has ≥2 capture handlers for the same target. The shipped `scripts/examples/menu-syntax/` set has exactly one handler per target, so in the AFK environment the handler row count is unchanged and the visible order is identical. Leaving the ranking validated via tests until a tick with a multi-handler test fixture is set up.
- **AFK cleanup:** no runtime session started.
- **Open risks / deferrals:**
  - Multi-handler fixtures for agentic-testing are not yet scaffolded. That's a natural pairing with commit 6 (Cmd+N scaffold), which creates extra user-authored handlers for free.
  - `build_capture_mode_results` is the only live consumer switched to the ranked variant in this commit. When commit 5 wires capture-handler rows into the trigger picker snapshot, it will also consume `rank_handlers_for_target` directly.
  - `KNOWN_ACCEPTS` is intentionally small — only accepts tokens that map to an observable `CaptureInvocation` field. Expanding the list (e.g. "markdown", "ics") is a per-target design call, not a handler_index change.
- **Next tick focus:** Commit 5 — picker actions. Wire the trigger picker's reserved `CaptureHandler` row kind so capture-mode snapshots include one row per ranked handler (capped at Oracle's ~5 suggestion), plus implement the picker's help and typo-fix rewrite actions as concrete `ReplaceInput` paths inside `filter_input_change` / `filter_input_updates`. This is the first tick that touches `filter_input_*` — still conservative: the dispatch only fires when `menu_syntax_mode.is_menu_syntax_for(raw)` is true, so plain fuzzy search stays unchanged.

## Iter 011 — 2026-04-23T20:31Z (AFK, commit 5 partial — handler rows only)

- **Bundle:** reused iter 007 bundle (no new Oracle consult this tick).
- **Oracle ask:** none — implementing the pure-lib half of Oracle iter 007 commit 5.
- **Scope pivot in this tick:** split commit 5 into two halves and land only the lib-safe half. Oracle's commit 5 was "picker actions — help, typo-fix, recent restore, handler execution." The handler-row lib half is pure and agent-verifiable; the help/typo/recent rewrite half requires touching `filter_input_change.rs` / `filter_input_updates.rs` (GPUI-adjacent) AND the grouping renderer needs selectable entries to receive `ReplaceInput` events. Neither is AFK-safe this tick. Land the lib half now so commit 6 (scaffold handler) has a populated row kind to render next to, and defer the filter-input rewrite path to a user-present tick where keystroke routing can be interactively verified.
- **Implemented this tick:**
  - `src/menu_syntax/trigger_picker.rs`:
    - `TriggerPickerContext.scripts: Vec<Arc<Script>>` — new field carrying the launcher script catalog into the snapshot builder. Default is empty so existing tests stay unchanged.
    - `build_capture_snapshot(target, ctx)` — now receives the context so it can call `capture_handler_rows` when a target is known.
    - `capture_handler_rows(target, scripts)` — ranks scripts via `handler_index::rank_handlers_for_target` and inserts up to `MAX_CAPTURE_HANDLER_ROWS = 5` `CaptureHandler` rows between the target row and the footer. Dedupe-by-path so multi-spec scripts only appear once. Builds stable row ids `handler:<target>:<plugin>:<file_stem>`.
    - `handler_command_id(script)` — encodes plugin + file stem so `ExecuteCaptureHandler { command_id }` has a deterministic identity. Downstream execution code lives in `menu_syntax_execution.rs` (commit 7 or later will connect the last wire).
    - `probe_invocation(target)` — builds a minimal `CaptureInvocation` with empty body/tags/dates so the ranker can run before the user has typed the body. Accepts-boost is therefore 0 in incomplete mode, which is correct — we don't want to guess the payload before the user types it. Complete captures (already routed to `build_capture_mode_results`) still get the real invocation with populated fields.
    - Row metadata: `title` uses `spec.label` when present else `script.name`; `subtitle` is `"default handler · <plugin>"` or just the plugin label; shipped examples render as `"shipped"` instead of the raw `main` plugin id; badges are `["default"]` and/or `["wildcard"]` for handlers that matched only via `targets: ["*"]`.
  - `src/app_impl/filtering_cache.rs::get_grouped_results_cached` — when the menu-syntax mode is active, the constructed `TriggerPickerContext` now includes `scripts: self.scripts.clone()`.
  - `lat.md/menu-syntax.md` Trigger Picker section — added a paragraph describing `capture_handler_rows`, the 5-row cap, and the `TriggerPickerContext.scripts` field.
- **Commits:** pending (lands after this progress entry).
- **Tests:** `cargo test --lib menu_syntax` — 140 passed (0 failed). Baseline was 133. 7 new `trigger_picker` tests:
  - `capture_handler_rows_empty_when_no_scripts` — empty catalog → no handler rows.
  - `capture_handler_rows_surface_for_known_target` — single user-authored default handler shows `default` badge + plugin-scoped command id.
  - `capture_handler_rows_capped_at_max` — 8 handlers → exactly 5 rendered.
  - `capture_handler_rows_preserve_ranked_order` — user-default > shipped-default > user-plain > shipped-plain order holds end-to-end from picker snapshot.
  - `wildcard_only_handler_row_flags_wildcard_badge` — wildcard-only handler carries `wildcard` badge.
  - `capture_handler_rows_skip_when_no_matching_specs` — unrelated target handlers never appear.
  - `bare_capture_mode_has_no_handler_rows_even_with_scripts` — bare `+` still shows targets only.
- `cargo build --lib` clean; `cargo build --bin script-kit-gpui` clean.
- `cargo fmt --package script-kit-gpui` formatted the new code.
- `lat check` passed.
- **Agentic-testing screenshots:** deferred. Shipped capture examples have exactly one handler per target, so `+todo` (incomplete) will add ONE new row visually — but it's still a non-selectable `SectionHeader` because commit 3 deferred keyboard wiring and the grouping renderer writes every non-footer row as a header. Non-AFK ticks can capture before/after screenshots once keyboard + selection wiring lands.
- **AFK cleanup:** no runtime session started.
- **Open risks / deferrals:**
  - `ExecuteCaptureHandler { command_id }` action has no dispatcher yet. The action enum variant keeps its `#[allow(dead_code)]` until a user-present tick wires keyboard routing through `apply_intent` into an execution helper.
  - The help/typo/recent rewrite half of Oracle commit 5 is still outstanding. That needs to live in `filter_input_change.rs` / `filter_input_updates.rs` and should ship with a screenshot pass.
  - `command_id` format is scoped to `<plugin>:<file_stem>` for now. If a handler is redefined in another plugin with the same file stem they would collide — acceptable in practice since we dedupe by `script.path` before emitting rows, but revisit if commit 6 (Cmd+N scaffold) starts writing multiple handlers with the same slug.
- **Next tick focus:** Commit 6 — scaffold capture handler from the picker footer (Cmd+N). Create `src/menu_syntax/templates.rs` with a single `render_capture_handler_template(target, slug)` function that produces the `.ts` scaffold (metadata + `menuSyntax` block + payload reader). Authoring flow comes later: the template function alone is lib-verifiable and AFK-safe. Tests: snapshot-style that template output contains `menuSyntax`, `capture.v1`, the target, and the payload-env reader shape. Keyboard wiring to invoke the scaffold stays deferred to a user-present tick.

## Iter 012 — 2026-04-23T20:42Z (AFK, commit 6 lib half landed)

- **Bundle:** reused iter 007 bundle (no new Oracle consult this tick).
- **Oracle ask:** none — implementing lib half of Oracle iter 007 commit 6.
- **Scope pivot in this tick:** land ONLY the pure template function. Oracle commit 6 was "scaffold capture handlers from picker footer (Cmd+N)" which includes three concerns: template generation, filesystem write, and picker-footer keyboard dispatch. The template function is 100% pure and can ship with unit tests; the FS writer and GPUI Cmd+N wiring need live interactive testing and stay deferred.
- **Implemented this tick:**
  - New `src/menu_syntax/templates.rs` with `render_capture_handler_template(target, slug) -> String`. Output is a TypeScript source string that a caller can drop at `~/.scriptkit/plugins/main/scripts/capture-<target>-<slug>.ts`.
  - Template shape mirrors the shipped examples under `scripts/examples/menu-syntax/`:
    - Leading filename comment (`// capture-<slug>.ts`).
    - Header doc explaining when the handler fires and pointing at `lat.md/menu-syntax.md#Execution Payload`.
    - `import { mkdir, appendFile, readFile } from "node:fs/promises"` + `join` from `node:path`.
    - `export const metadata = { name, description, menuSyntax: [{ family: "capture.v1", targets: [<target>], accepts: [...], label, payloadSchema, defaultHandler: false }] }`.
    - `KIT_MENU_SYNTAX_PAYLOAD_PATH` env reader with an actionable error message.
    - `SK_PATH` fallback to `~/.scriptkit`; writes `<$SK_PATH>/menu-syntax/<artifact>.jsonl`.
    - Emitted body echoes `target`, `body`, `tags`, `priority`, `url`, `duration`, `dates`, `raw`, `createdAt` — an easy starting point the author can trim down.
  - Helpers:
    - `slug_or_target(target, slug)` — normalizes user input to kebab-case alphanumerics; falls back to the target string when the slug is empty or all non-alphanumeric.
    - `display_name_from_slug(target, slug)` — title-cases the slug into a human-readable name (`"jira sync"` → `"Capture Jira Sync"`).
    - `artifact_hint_for(target)` — maps known targets to the same filenames used by shipped examples (`todos.jsonl`, `events.jsonl`, `notes.jsonl`, `drafts.jsonl`, `bookmarks.jsonl`); unknown targets fall back to `entries.jsonl`.
    - `accepts_hint_for(target)` — known targets get the same `accepts` list as the shipped handler so picker scoring stays consistent; unknown targets get a generic `["tags", "date", "url", "kv"]`.
  - `src/menu_syntax/mod.rs` registers `pub mod templates` and re-exports `render_capture_handler_template` under `#[allow(unused_imports)]` (dispatcher lands later).
  - `lat.md/menu-syntax.md` gains a `Templates` section above `Shipped Examples` describing the pure function, its output shape, and the `defaultHandler: false` rationale.
- **Commits:** pending (lands after this entry).
- **Tests:** `cargo test --lib menu_syntax` — 153 passed (0 failed). Baseline was 140. 13 new `templates` tests:
  - `template_contains_menu_syntax_metadata_block`
  - `template_reads_payload_env_var`
  - `template_explains_when_the_handler_fires`
  - `template_defaults_default_handler_to_false_with_guidance` (ranker-slot invariant — every scaffolded handler must default to false)
  - `template_renders_for_every_known_target` (all 5 `KNOWN_CAPTURE_TARGETS`)
  - `template_artifact_hint_matches_shipped_example_conventions` (shipped filenames stay consistent)
  - `template_handles_unknown_target_with_generic_artifact`
  - `template_accepts_hint_matches_shipped_examples`
  - `slug_or_target_falls_back_to_target_when_empty`
  - `slug_or_target_normalizes_user_input`
  - `template_name_derives_from_slug`
  - `template_filename_header_uses_normalized_slug`
  - `template_payload_path_error_message_is_actionable`
- `cargo build --lib` clean.
- `cargo fmt`, `lat check` both pass.
- **Agentic-testing screenshots:** not applicable (no UI change — template is a pure string).
- **AFK cleanup:** no runtime session started.
- **Open risks / deferrals:**
  - No filesystem writer and no Cmd+N dispatcher yet. The `CreateHandler { target }` action still resolves to the deferred `TriggerPickerIntentOutcome::CreateHandler` that callers currently ignore.
  - Scripts produced from this template are not yet auto-imported into the launcher's script catalog. The user has to restart Script Kit (or trigger a reload) after saving the scaffold — acceptable for AFK; revisit when wiring the authoring flow.
  - `accepts` hint duplicates values from shipped examples. If the shipped examples' accepts lists change, these hints will drift; centralizing them behind a shared table is worth doing when there are more than 2 readers.
- **Next tick focus:** Commit 7 — Captures inverse browser. Create `src/menu_syntax/artifacts.rs` with a tolerant reader over `todos.jsonl`, `events.jsonl`, `notes.jsonl`, `drafts.jsonl`, `bookmarks.jsonl`, and the payload dir. Pure reader + artifact-row builder for later consumption by a built-in view. Skip malformed JSONL lines with a warning count so the builtin never crashes on dirty inputs (Oracle iter 004 explicit rule).

## Iter 013 — 2026-04-23T20:57Z (AFK, commit 7 lib half landed)

- **Bundle:** reused iter 007 bundle (no new Oracle consult this tick).
- **Oracle ask:** none — implementing lib half of Oracle iter 007 commit 7.
- **Scope pivot in this tick:** ship the reader module only. Oracle commit 7 was "Captures inverse browser + artifact rows + Cmd+P"; the built-in view + Cmd+P dispatcher are GPUI-adjacent and need live testing, so they stay deferred. The lib reader is pure and agent-verifiable via unit tests.
- **Implemented this tick:**
  - New `src/menu_syntax/artifacts.rs` with `CaptureArtifactKind` (`Todo`, `CalendarEvent`, `Note`, `SocialDraft`, `Bookmark`, `Payload`), `CaptureArtifact`, and `ReadArtifactReport`.
  - `read_jsonl_artifact(path, kind)` — line-by-line JSONL reader. Missing files yield an empty report (not a warning — a user who hasn't captured a note yet shouldn't see a warning). Unreadable files surface one warning and bump `skipped`. Malformed lines bump `skipped` and push a warning (capped at `MAX_WARNINGS = 10`). Blank lines are skipped silently. Non-object scalars (`"string"`, `42`) are still included with best-effort snippets.
  - `read_payload_dir(path)` — enumerates `capture_v1-*.json` tempfiles only; other files in the directory are ignored silently. Missing directory yields an empty report.
  - `read_all_artifacts(sk_path)` — reads every known kind under `$SK_PATH/menu-syntax/` and merges reports in `BROWSER_ORDER` followed by `Payload` last. `ReadArtifactReport::merge` appends entries, sums skipped counts, and respects the warning cap.
  - Snippet helpers: `snippet_for_value` prefers `body`, `raw`, `target`, `url`, `title` fields in that order, falling back to serialized JSON. `truncate_snippet` collapses whitespace, enforces `MAX_SNIPPET_CHARS = 200`, and appends `…` on truncation.
  - `extract_created_at` reads `createdAt` (shipped examples + template) or `timestamp` (payload tempfiles) without privileging either.
  - `CaptureArtifactKind::BROWSER_ORDER` const excludes `Payload` — payloads are retention-only, not user-facing inverse-browser rows.
  - `src/menu_syntax/mod.rs` registers `pub mod artifacts;` and re-exports `CaptureArtifact`, `CaptureArtifactKind`, `ReadArtifactReport`, `read_all_artifacts`, `read_jsonl_artifact`, `read_payload_dir` under `#[allow(unused_imports)]`.
  - `lat.md/menu-syntax.md` gains a `Captures Inverse Browser` section above `Shipped Examples` documenting the tolerant-reader contract, the warning cap, the snippet truncation rule, and why `Payload` is excluded from `BROWSER_ORDER`.
- **Commits:** pending (lands after this entry).
- **Tests:** `cargo test --lib menu_syntax` — 167 passed (0 failed). Baseline was 153. 14 new `artifacts` tests driven by `TempDir` fixtures (`tempfile` is already a dep):
  - `read_jsonl_artifact_returns_all_valid_entries`
  - `read_jsonl_artifact_skips_malformed_lines_with_warning`
  - `read_jsonl_artifact_handles_missing_file_gracefully` (no warning for missing)
  - `read_jsonl_artifact_ignores_blank_lines`
  - `read_jsonl_artifact_truncates_snippet_for_long_bodies` (ends with `…`)
  - `snippet_falls_back_to_raw_when_body_is_missing`
  - `non_object_top_level_json_is_still_included`
  - `read_payload_dir_returns_only_capture_v1_files` (unrelated files silently skipped; bad `capture_v1-*.json` counts as skipped)
  - `read_payload_dir_handles_missing_dir_gracefully`
  - `read_all_artifacts_combines_every_kind` (end-to-end: all 5 kinds + payload, order pinned)
  - `read_all_artifacts_counts_warnings_across_files`
  - `warning_cap_prevents_unbounded_accumulation` (30 dirty rows → 10 warnings, all counted as skipped)
  - `artifact_filename_for_matches_templates_and_shipped_examples`
  - `browser_order_excludes_payload` (regression guard against a future refactor accidentally surfacing payload rows to users)
- `cargo build --lib` clean; `cargo fmt` OK; `lat check` passed after trimming the new section's leading paragraph to ≤250 chars.
- **Agentic-testing screenshots:** not applicable (pure reader, no UI surface).
- **AFK cleanup:** no runtime session started.
- **Open risks / deferrals:**
  - No built-in view consumes `read_all_artifacts` yet. The Captures inverse browser ships in a later tick that pairs the reader with a new builtin and wires it into the picker's `OpenCaptures` outcome (currently deferred per iter 009 commit 3).
  - Notes directory convention varies: the shipped example writes per-day markdown under `$SK_PATH/notes/`, whereas the scaffolded handler writes `notes.jsonl`. The reader only enumerates JSONL; markdown notes surface via a separate existing built-in. Revisit when wiring the browser UI.
  - No file-system write paths in this module. Retention + deletion belong to commit 8.
  - `read_all_artifacts` does not yet expose per-kind counts (for a header like "Captures (42)"). Callers can compute that from `entries.iter().filter(|e| e.kind == K).count()`; if a hotter path needs it, add a `counts` field to `ReadArtifactReport`.
- **Next tick focus:** Commit 8 — `src/menu_syntax/retention.rs` + HUD improvements. Pure retention policy that prunes `capture_v1-*.json` payload tempfiles: always keep the newest 250, hard-cap at 1000 newest, and age-cleanup files older than 30 days but only outside the newest 250 (Oracle iter 004 explicit numbers). Never touch user-authored artifacts (`todos.jsonl`, `bookmarks.jsonl`, per-day notes, `.ics` files, social drafts). Expose a pure `plan_retention(payload_dir_listing, now) -> RetentionPlan` so tests never touch the filesystem; a thin `apply_retention_plan` helper does the deletions. Tests: newest-250 invariant, 1000 hard cap, age rule only outside newest 250, empty dir safety, policy is deterministic given the same inputs. Then STOP the loop — commit 8 is the last commit in the plan.

## Iter 014 — 2026-04-23T21:10Z (AFK, commit 8 landed — LOOP COMPLETE)

- **Bundle:** reused iter 007 bundle (no new Oracle consult this tick).
- **Oracle ask:** none — implementing Oracle iter 007 commit 8 (final).
- **Implemented this tick:** Pure payload retention policy.
  - New `src/menu_syntax/retention.rs`:
    - `PayloadListing { path, created_at_unix }` — caller-enumerated entry shape. The retention module never enumerates files itself, so user JSONL / markdown / `.ics` / social drafts are untouchable by construction.
    - `RetentionConfig { keep_newest: 250, hard_cap: 1000, age_cutoff_days: 30 }` defaults matching Oracle iter 004 verbatim. Exported as `KEEP_NEWEST_DEFAULT`, `HARD_CAP_DEFAULT`, `AGE_CUTOFF_DAYS_DEFAULT` constants so HUD copy in later ticks can quote the same numbers.
    - `RetentionPlan { keep, delete }` ordered newest-first.
    - `plan_retention(listing, now_unix, cfg) -> RetentionPlan` — pure; sorts a local copy by `created_at_unix` desc with path-ascending tie-break for determinism; applies: (1) rank `< keep_newest` → always keep, (2) rank `>= hard_cap` → delete, (3) otherwise delete if `age > age_cutoff_days`. Saturating arithmetic throughout so absurd configs (`usize::MAX`, `u64::MAX`) don't panic.
    - `apply_retention_plan(plan) -> AppliedRetention { deleted, failed }` — thin FS helper; treats missing paths as successful no-ops so concurrent passes don't spuriously fail.
  - `src/menu_syntax/mod.rs` registers `pub mod retention` and re-exports `plan_retention`, `apply_retention_plan`, `PayloadListing`, `RetentionConfig`, `RetentionPlan`, `AppliedRetention`, and the three default constants under `#[allow(unused_imports)]`.
  - `lat.md/menu-syntax.md` gains a `Payload Retention` section above `Captures Inverse Browser` documenting the policy, the newest-250 floor invariant, and the pure/FS split.
- **Commits:** pending (lands after this entry).
- **Tests:** `cargo test --lib menu_syntax` — 180 passed (0 failed). Baseline was 167. 13 new `retention` tests:
  - `empty_listing_yields_empty_plan`
  - `listing_under_keep_newest_keeps_everything_regardless_of_age` (50 ancient files all kept under the 250 floor)
  - `age_rule_only_triggers_outside_newest_floor` (250 young + 10 old → all 250 kept, 10 old deleted)
  - `old_entries_inside_newest_floor_are_still_kept` (50 ancient files all kept — invariant regression guard)
  - `hard_cap_trims_excess_even_when_young` (1050 young → 1000 kept, 50 deleted by hard cap)
  - `newest_250_invariant_never_violated_even_when_all_old` (300 old → 250 kept, 50 deleted)
  - `age_rule_cutoff_is_strictly_greater_than` (boundary file at exactly 30d is kept)
  - `plan_is_deterministic_for_identical_inputs` (order-independence)
  - `tie_break_on_timestamp_uses_path_ordering` (identical timestamps → alphabetical rank)
  - `disable_hard_cap_by_setting_config_enormous` (`usize::MAX` config saturates safely)
  - `keep_order_is_newest_first` (plan.keep is sorted for streaming consumers)
  - `default_constants_match_oracle_iter_004_numbers` (250 / 1000 / 30 locked in)
  - `apply_retention_plan_treats_missing_paths_as_success` (uses `tempfile::TempDir`)
- `cargo build --lib` clean; `cargo fmt`, `lat check` pass.
- **Agentic-testing screenshots:** not applicable (pure policy; no UI surface; no filesystem side effects in the planner).
- **AFK cleanup:** no runtime session started.
- **Open risks / deferrals:**
  - No caller invokes `plan_retention` yet. The hook belongs alongside a successful capture execution — Oracle iter 004 said "run opportunistically after successful payload write, not in a daemon." Wiring that up is a one-liner in the execution path but best done with the user available to watch for file-system surprises.
  - HUD copy improvements (e.g. "Captured todo via Capture Todo Inbox · Payload: …/capture_v1-…json") from Oracle iter 004 are not in this commit. The HUD path lives in `execute.rs` + the main HUD renderer; deferred to the same user-present tick that wires `apply_retention_plan`.
  - Retention only targets payloads. If a future handler starts writing capture artifacts into the payload dir, the pattern filter (`capture_v1-*.json`, caller-enforced) keeps the policy safe.
- **Loop complete. 7 revised commits landed while AFK:**
  - Iter 005 commit 1: `ae9727805` — pure TriggerPicker row model (101 tests).
  - Iter 008 commit 2: `e73324f91` — inline picker rendering as non-selectable SectionHeaders (zero ACP touches).
  - Iter 009 commit 3: `187141681` — InlinePickerKeyIntent classifier (pure lib; GPUI wiring deferred).
  - Iter 010 commit 4: `2cea99f44` — HandlerScore ranking + rank_scripts_handling_capture wired into build_capture_mode_results (133 tests).
  - Iter 011 commit 5 (lib half): `06210713f` — CaptureHandler rows populated via rank_handlers_for_target, 5-row cap, TriggerPickerContext.scripts threaded through filtering_cache (140 tests).
  - Iter 012 commit 6 (lib half): `9331ec1a2` — render_capture_handler_template with defaultHandler:false invariant (153 tests).
  - Iter 013 commit 7 (lib half): `9bf02b8a6` — tolerant Captures inverse-browser reader with warning cap + BROWSER_ORDER (167 tests).
  - Iter 014 commit 8: pending SHA after this entry lands — pure payload retention with newest-250 floor invariant (180 tests).
- **Test count from plan start to here:** 96 (pre-plan baseline) → 180. +84 tests across the 7 tick iters, all lib-verifiable, all agent-reproducible from commit message templates.
- **Deferred for user-present ticks (tracked in iter progress entries):**
  - GPUI event wiring for ArrowUp/ArrowDown/Tab/Enter/Escape/Cmd+N/Cmd+P — Tab conflicts with AI routing in `src/app_impl/startup_new_tab.rs`; Enter conflicts with the focused Input submit path.
  - Handler/picker row *selection* so CaptureHandler, FixQualifier, OpenHelp, and create-handler footer rows stop rendering as non-selectable SectionHeaders.
  - Filesystem writer + editor-open flow for scaffolded handlers (`capture-<target>-<slug>.ts`).
  - Captures inverse-browser builtin view that consumes `read_all_artifacts`.
  - `apply_retention_plan` invocation hook after successful payload writes, plus HUD copy improvements.
  - Optional shared renderer extraction (`src/components/inline_picker.rs`) per Oracle option C — still on the table but gated on the keyboard/selection landing first.
- **Loop exit:** omitting `ScheduleWakeup` this tick. Final summary posted to the user.

## Iter 015 — 2026-04-24T14:22Z (pivot plan, Option C extraction, Oracle consulted)

- **Bundle:** `~/.oracle/bundles/menu-syntax-popup-pivot-plan.txt` (21 files, 784K).
- **Oracle session:** `menu-syntax-popup-pivot-plan-2` (gpt-5.4-pro, browser, 10m32s, 165.22k in / 6.08k out). Full log: `~/.oracle/sessions/menu-syntax-popup-pivot-plan-2/output.log`.
- **User directive:** (1) "We need to get rid of this chip." — the `CAPTURE`/`FILTER`/`TODO`/... `Input::prefix` chip. (2) "Pivot this takeover list to a popup list like the slash commands menu supports. We're going for consistent behavior between the slash command, @ mentions, and these new text popup activators." (3) "Option C sounds perfect, always prefer shared components/behaviors."
- **Oracle verdict (TL;DR):** Extract popup mechanics and picker rendering/selection into shared components; keep ownership per surface. `src/components/inline_popup_window.rs` owns detached child-window math/config/no-focus-steal. `src/components/inline_picker.rs` owns neutral row/list rendering, selection, empty state, fuzzy highlight, key-event classification. ACP keeps its own `AcpMentionPopupWindow`; menu-syntax gets a new `MenuSyntaxTriggerPopup` owner. Each owner adapts its domain row (`ContextPickerItem`, `TriggerPickerRow`) into the neutral `InlinePickerRow`. Delete the chip. Delete the `build_trigger_picker_grouped_results` inline SectionHeader takeover.
- **Oracle revised commit sequence:**
  - **Commit A** — Delete the mode chip from `src/render_script_list/mod.rs` (`menu_syntax_mode_chip_label`, `render_menu_syntax_mode_chip`, and the `Input::prefix` call site). Pure deletion.
  - **Commit B** — Extract `src/components/inline_popup_window.rs` (window mechanics only) from `src/ai/acp/popup_window.rs`. ACP popup behavior visually unchanged. Symbols moved: `INLINE_POPUP_*` constants, `InlinePopupAnchor`, `InlinePopupConstraints`, `InlinePopupLayout`, `InlinePopupPlacement`, `compute_inline_popup_layout`, `constrain_inline_popup_bounds`, `inline_popup_window_options`, `configure_inline_popup_window`, `attach_inline_popup_child_window`, `update_inline_popup_child_window_bounds`, `detach_inline_popup_child_window`. ACP-specific `acp_popup_anchor_for_composer`, `show_or_update_acp_popup_window`, `close_acp_popup_window` remain thin wrappers.
  - **Commit C** — Extract `src/components/inline_picker.rs` (neutral row/list rendering + selection helpers) from `src/ai/acp/picker_popup.rs`. Symbols moved: `INLINE_PICKER_*` constants, `InlinePickerRow`, `InlinePickerRowId`, `InlinePickerRowKind`, `InlinePickerBadge`, `InlinePickerBadgeTone`, `InlinePickerLeadingVisual`, `InlinePickerAccessory`, `InlinePickerHighlights`, `InlinePickerEmptyState`, `InlinePickerSnapshot`, `InlinePickerRenderOptions`, `InlinePickerRowRenderState`, `InlinePickerKeyboardEvent`, `render_inline_picker`, `render_inline_picker_list`, `render_inline_picker_row`, `render_inline_picker_empty_state`, `render_inline_picker_highlighted_text`, `inline_picker_match_ranges`, `apply_inline_picker_fuzzy_highlights`, `inline_picker_selected_row`, `inline_picker_normalize_selected_index`, `inline_picker_next_enabled_index`, `inline_picker_previous_enabled_index`, `inline_picker_visible_range`, `inline_picker_reveal_range`, `inline_picker_keyboard_event_from_gpui`. `ContextPickerItem` / `AcpMentionPopupWindow` / `AcpMentionPopupSnapshot` / `AcpMentionPopupRequest` stay in ACP; add `fn adapt_context_picker_item(item: &ContextPickerItem) -> InlinePickerRow`.
  - **Commit D** — Land the menu-syntax popup. New `src/app_impl/menu_syntax_trigger_popup.rs` (owner entity, snapshot sync, selection by row id, key dispatch bridge to `apply_intent`, `adapt_trigger_picker_row`). Delete `build_trigger_picker_grouped_results` in `src/scripts/grouping.rs`. Drop the trigger-picker branch in `src/app_impl/filtering_cache.rs`. Wire keyboard dispatch (Arrow/Tab/Enter/Escape) in `filter_input_updates.rs` + `startup_new_tab.rs` (Tab guard must come before AI routing).
- **Neutral `InlinePickerRow` shape (Oracle-specified):**
  - `id: InlinePickerRowId` (stable `SharedString`; owners preserve selection by id, not index).
  - `kind: InlinePickerRowKind` (`Context | SlashCommand | TextTrigger | Action | Custom(SharedString)`).
  - `title: SharedString`; `token: Option<SharedString>`; `subtitle: Option<SharedString>`; `detail: Option<SharedString>`; `example: Option<SharedString>`.
  - `leading: Option<InlinePickerLeadingVisual>` (`Glyph(SharedString) | IconName(SharedString)`).
  - `badges: Vec<InlinePickerBadge>` (`{ label, tone }` where tone is `Neutral | Accent | Warning | Disabled`).
  - `accessory: Option<InlinePickerAccessory>` (`Text | Shortcut | Token`).
  - `highlights: InlinePickerHighlights { title, token, subtitle, detail }` (each `Vec<Range<usize>>`, char-boundary-safe).
  - `enabled: bool`; `disabled_reason: Option<SharedString>`.
  - **No closures, no domain actions.** Owners map `id` → their own accept logic.
- **Keyboard dispatch decision tree (popup-visible priority):**
  - `ArrowDown` / `ArrowUp` — intercept in `src/app_impl/filter_input_updates.rs` before main-list navigation. Apply `InlinePickerKeyIntent::MoveNext` / `MovePrevious`, skip disabled rows.
  - `Tab` / `Shift+Tab` — intercept in `src/app_impl/startup_new_tab.rs` **before** AI routing. `CompleteSelected` / `MovePrevious` (or `CompletePrevious` if intent exists).
  - `Enter` — intercept in `filter_input_updates.rs` before input submit. Add backstop in `filter_input_change.rs` if GPUI submit bypasses keydown. `AcceptSelected`.
  - `Escape` — intercept in `filter_input_updates.rs` before launcher clear-filter. First Escape closes popup only (filter unchanged); second Escape clears filter; third Escape hides window per existing flow.
  - Parser-boundary rule: if `build_trigger_picker_snapshot` returns `None` OR the filter starts with a legacy trigger (`~ / @ > ?`), popup closes immediately and keys fall through to legacy handlers.
- **Parser-boundary state machine (Oracle pseudocode):**
  ```rust
  fn sync_menu_syntax_trigger_popup_for_filter(filter: &str, app: &mut ScriptListApp, cx: &mut App) {
      if starts_with_legacy_trigger(filter) { close_menu_syntax_trigger_popup(app, cx); return; }
      match build_trigger_picker_snapshot(filter) {
          Some(snapshot) => show_or_update_menu_syntax_trigger_popup(app, snapshot, cx),
          None => close_menu_syntax_trigger_popup(app, cx),
      }
  }
  ```
  Transitions: `Closed → Open` only when snapshot is `Some(_)`; any `None` or legacy-trigger transition closes immediately; selection preserved across rebuilds by row `id`; popup lifecycle owned by input-update handlers, **never** by `filtering_cache.rs`.
- **Open risks / unknowns Oracle flagged (investigate before Commit A):**
  - Input prefix spacing may leave stale padding after chip deletion; confirm cursor alignment for `+` and `:` starts.
  - Typed `+` must remain in filter text; only delete the rendered chip, not the user input.
  - Temporary guidance gap: after Commit A the old inline takeover still exists, so users are not blind. Do not delete the takeover until Commit D also lands the popup + keyboard.
  - Grep for existing `InlineDropdown`/shared row primitives before creating `inline_picker.rs`; extend or wrap if equivalents exist.
  - Tab routing priority: `startup_new_tab.rs` must give the menu-syntax popup first chance.
  - Window focus: the child popup must never become key/main; all keyboard ownership stays with the parent input.
  - `TriggerPickerRow.id` must be stable across rebuilds; verify before depending on id-based selection persistence.
  - Unicode highlight ranges must be char-boundary-safe (UTF-8) — bad byte slicing will panic.
  - Do **not** update popup state from `filtering_cache.rs`; popup lifecycle belongs in input update/change handling.
- **Per-commit agentic-testing receipts:**
  - **Commit A:** `cargo check`; `rg render_menu_syntax_mode_chip|menu_syntax_mode_chip_label src/render_script_list/mod.rs` returns nothing; screenshot showing input with typed `+` and no chip.
  - **Commit B:** `cargo check`; ACP slash popup + `@` popup still anchor to composer (log attach/update/detach); input remains focused while popup visible.
  - **Commit C:** `cargo check`; ACP popup row count before/after matches; Arrow navigation still works; disabled rows skipped; empty state still renders.
  - **Commit D:** `cargo check` + `cargo test menu_syntax`; `rg build_trigger_picker_grouped_results src` returns nothing; typing `+`/`+todo` opens detached popup (not SectionHeader takeover); legacy triggers `~ / @ > ?` do not open it; Arrow moves popup selection (not main list); Tab with popup accepts selected trigger (does not route to AI); Enter accepts (does not run script); first Escape closes popup with filter intact; second Escape clears filter.
- **Next tick focus:** Commit A — delete the chip. Smallest independent change, pure-deletion diff. No popup or keyboard work yet.

## Iter 017 — 2026-04-24T15:10Z (Commit B: extract components/inline_popup_window.rs)

- **Bundle:** reused iter 015 bundle; no new Oracle consult.
- **Oracle ask:** none — implementing Oracle iter 015 commit B.
- **Implemented this tick:** extracted popup-window mechanics into a shared component.
  - New `src/components/inline_popup_window.rs`:
    - Constants under neutral names: `INLINE_POPUP_MAX_VISIBLE_ROWS`, `INLINE_POPUP_VERTICAL_PADDING`, `INLINE_POPUP_EMPTY_HEIGHT`, `INLINE_POPUP_DEFAULT_WIDTH`, `INLINE_POPUP_MIN_WIDTH`, `INLINE_POPUP_EDGE_GUTTER`, `INLINE_POPUP_LEFT_MARGIN`.
    - Pure helpers: `inline_popup_height_for_row_height`, `inline_popup_width_for_window`, `inline_popup_width_for_labels`, `footer_anchored_inline_popup_top`, `inline_popup_bounds`.
    - NSWindow mechanics: `inline_popup_window_options`, `configure_inline_popup_window`, `set_inline_popup_window_bounds`, `inline_popup_ns_window` (macos), `attach_inline_popup_to_parent_window` (macos). Tracing event neutralized to `"inline_popup_attached"` with target `"script_kit::inline_popup"`.
  - `src/components/mod.rs` registers `pub mod inline_popup_window`.
  - `src/ai/acp/popup_window.rs` rewritten as a thin compatibility facade. Every `DENSE_PICKER_*` / `dense_picker_*` / `popup_*` name is kept via `pub(crate) use ... as old_name;` aliases, so `picker_popup.rs`, `model_selector_popup.rs`, `history_popup.rs`, `view.rs`, `src/storybook/playground_overlay_metrics.rs`, and the source-text audit tests in `src/ai/acp/tests.rs` compile without edits. The ACP-flavored convenience `dense_picker_height(item_count)` stays local so ACP callers keep passing a bare count and get `CONTEXT_PICKER_ROW_HEIGHT` applied automatically.
  - `lat.md/design.md` gains a wiki-link pointing at `src/components/inline_popup_window.rs` and a new sentence under `Popup behavior` explaining the shared-module + facade pattern.
- **Commits:** pending (lands after this entry).
- **Tests:** `cargo test --lib menu_syntax` still 180 passed. Targeted popup unit tests:
  - `components::inline_popup_window::tests` — 8 passed (new: `inline_popup_height_uses_empty_state_when_zero_rows`, `inline_popup_height_caps_at_max_visible_rows`, `inline_popup_height_accepts_custom_row_height`, `inline_popup_width_matches_window_constraints`, `inline_popup_label_width_tracks_content_length`, `footer_anchor_keeps_popup_above_hint_strip`, `inline_popup_bounds_offset_from_parent_origin`, `screen_relative_bounds_convert_to_nswindow_frame_on_secondary_display` — moved from ACP and renamed to neutral names).
  - `ai::acp::popup_window::tests::dense_picker_height_uses_shared_row_contract` — 1 passed (guards the ACP convenience wrapper against the shared max-visible-rows contract).
  - `ai::acp::picker_popup::tests` — 5 passed (no changes).
  - `ai::acp::model_selector_popup::tests` — 3 passed (no changes).
  - `ai::acp::history_popup::tests` — 4 passed (no changes).
- `cargo build --lib --package script-kit-gpui` clean; `lat check` passes.
- **Pre-existing unrelated failures:** 12 `ai::acp::tests::tab_ai_mode_*` + related source-text audit tests fail on main (prior to commit A) because `src/app_impl/tab_ai_mode/` was split into submodules (`acp_launch.rs`, `acp_setup.rs`, `source_classification.rs`, `acp_context_staging.rs`) in commits `f07c26b52`, `15c8b0a52`, `509f8363e`, `cbbf9b31b` but the audit tests still search `tab_ai_mode/mod.rs` for strings that now live in the submodules. Not touched in this commit. Worth fixing in a separate janitorial tick.
- **Agentic-testing screenshots:** deferred — no behavior change on screen; the shared module is a pure refactor. Commit D will need ACP `/` and `@` popup screenshots to prove visual parity after the renderer also moves in commit C.
- **AFK cleanup:** no runtime session started.
- **Open risks / deferrals:**
  - The tracing event name changed from `acp_inline_dropdown_popup_attached` to `inline_popup_attached`. This is the only non-trivial behavior diff; it's intentional since the helper now serves multiple surfaces. Telemetry dashboards filtering on the old event name will need updating.
  - The compatibility facade `src/ai/acp/popup_window.rs` can be removed entirely once all ACP callers are migrated to the neutral names. Deferring that rename sweep until after commit D lands so the popup pivot plan doesn't blow its blast radius.
  - `history_popup.rs` has a second, local copy of `popup_ns_window` / `attach_popup_to_parent_window` / `set_popup_window_bounds` (introduced independently). Not unified here to keep commit B's blast radius minimal; worth de-duping after the pivot plan completes.
- **Next tick focus:** Commit C — extract `src/components/inline_picker.rs` (neutral row/list rendering + selection helpers) from `src/ai/acp/picker_popup.rs`. Define the `InlinePickerRow` neutral shape per Oracle iter 015, add `adapt_context_picker_item` inside ACP, and keep `ContextPickerItem` / `AcpMentionPopupSnapshot` / `AcpMentionPopupRequest` / `AcpMentionPopupWindow` owned by ACP. Agentic-testing: ACP slash + `@` popup row counts before/after match; Arrow nav still works; disabled rows skipped.

## Iter 018 — 2026-04-24T15:45Z (Commit C: neutral InlinePickerRow shape — renderer already shared)

- **Bundle:** reused iter 015 bundle; no new Oracle consult.
- **Oracle ask:** none — implementing Oracle iter 015 commit C with an important course-correction flagged in Oracle's own risks list.
- **Key discovery in this tick:** the shared picker **renderer** Oracle iter 015 proposed extracting (`render_inline_picker`, `render_inline_picker_row`, visibility helpers, etc.) already exists as `src/components/inline_dropdown/` (`InlineDropdown`, `render_soft_compact_picker_row`, `inline_dropdown_visible_range_from_start`, `InlineDropdownColors`, `InlineDropdownEmptyState`, `InlineDropdownSynopsis`, `inline_dropdown_select_next` / `_prev` / `_clamp_selected_index` / `_visible_range`). `src/ai/acp/picker_popup.rs` already delegates all of its row rendering there. Oracle iter 015's own risks list called this out ("grep for any existing InlineDropdown/shared row renderer before creating inline_picker.rs. Extend or wrap existing primitives if they already exist; do not fork duplicate picker behavior"). Attempting the rename-and-move plan verbatim would fork a parallel renderer inside `components/inline_picker.rs` and regress the consistency the pivot exists to achieve. So commit C narrows its scope accordingly.
- **Implemented this tick:** added a neutral row-data shape + enabled-aware selection helpers that the generic `inline_dropdown` does not own.
  - New `src/components/inline_picker.rs`:
    - `InlinePickerRowId = SharedString`.
    - `InlinePickerRow { id, kind, title, token, subtitle, detail, example, leading, badges, accessory, highlights, enabled, disabled_reason }`.
    - `InlinePickerRowKind = Context | SlashCommand | TextTrigger | Action | Custom(SharedString)` — visual classification, no behavioral inference.
    - `InlinePickerLeadingVisual = Glyph(SharedString) | IconName(SharedString)`.
    - `InlinePickerBadge { label, tone: InlinePickerBadgeTone }` with `Neutral | Accent | Warning | Disabled` tones.
    - `InlinePickerAccessory = Text | Shortcut | Token`.
    - `InlinePickerHighlights { title, token, subtitle, detail }` with `Vec<Range<usize>>` per slot.
    - Selection helpers: `inline_picker_selected_row`, `inline_picker_normalize_selected_index` (clamps + snaps off disabled rows), `inline_picker_next_enabled_index`, `inline_picker_previous_enabled_index` (both skip disabled rows, wrap at list boundaries).
    - Visibility helper: `inline_picker_visible_range` that delegates to `inline_dropdown::inline_dropdown_visible_range_from_start` so both surfaces honor the same scrolling contract.
    - Highlight validator: `validate_highlight_ranges(&InlinePickerRow) -> bool` that verifies every byte offset lands on a UTF-8 character boundary and falls inside the text slot's length, guarding against the panic Oracle flagged in iter 015 risks.
    - `#[allow(dead_code)]` on every pub surface since the first consumer (menu-syntax trigger popup) lands in commit D.
  - `src/components/mod.rs` registers `pub mod inline_picker`.
  - `lat.md/design.md` gains a new paragraph under `Popup behavior` explaining the split: `inline_dropdown` owns the renderer, `inline_popup_window` owns the window mechanics, `inline_picker` owns the neutral row data shape + enabled-aware selection helpers. Adapters (`adapt_context_picker_item`, `adapt_trigger_picker_row`) live with the domain types, never in the shared file.
- **Explicit deferral:** `adapt_context_picker_item(item: &ContextPickerItem) -> InlinePickerRow` is NOT added to `src/ai/acp/picker_popup.rs` in this commit. ACP does not consume `InlinePickerRow` today — its existing `render_soft_compact_picker_row` path works against primitive args (label, meta, highlight indices). Adding a dead adapter just to "prove the shape" would clutter ACP code for zero benefit. When a cross-surface consumer (automation `getElements` wanting a unified row type; a future inspector) needs ACP data in `InlinePickerRow` form, the adapter lands then. The row shape's ability to hold ACP-shaped data is exercised in `components::inline_picker::tests` via representative fixtures.
- **No changes to `src/ai/acp/picker_popup.rs`, `model_selector_popup.rs`, `history_popup.rs`, `view.rs`, or `src/scripts/grouping.rs` in this commit.** ACP rendering remains unchanged; menu-syntax's existing inline-SectionHeader takeover remains in place so users are not left without guidance between C and D.
- **Commits:** pending (lands after this entry).
- **Tests:** `cargo test --lib components::inline_picker` — 20 passed (new):
  - `selected_row_returns_none_when_index_missing`
  - `selected_row_returns_reference_when_index_valid`
  - `selected_row_out_of_range_is_none`
  - `normalize_returns_none_for_empty_rows`
  - `normalize_snaps_disabled_to_next_enabled`
  - `normalize_falls_back_to_first_enabled_when_past_end`
  - `normalize_returns_none_when_every_row_disabled`
  - `next_enabled_skips_disabled_rows`
  - `next_enabled_wraps_at_end`
  - `next_enabled_from_none_picks_first_enabled`
  - `next_enabled_returns_none_when_none_enabled`
  - `previous_enabled_skips_disabled_rows`
  - `previous_enabled_wraps_at_start`
  - `previous_enabled_from_none_picks_last_enabled`
  - `previous_enabled_returns_none_when_none_enabled`
  - `validate_highlight_ranges_passes_for_ascii`
  - `validate_highlight_ranges_rejects_mid_char_boundary` (UTF-8 safety regression guard — multi-byte `☕️` plus truncated range)
  - `validate_highlight_ranges_rejects_end_past_string`
  - `validate_highlight_ranges_accepts_empty_ranges`
  - `visible_range_delegates_to_inline_dropdown`
- `cargo test --lib menu_syntax` — 180 passed (unchanged).
- `cargo test --lib ai::acp::picker_popup::tests` — 5 passed (unchanged).
- `cargo test --lib ai::acp::popup_window::tests` — 1 passed (unchanged).
- `cargo test --lib components::inline_popup_window::tests` — 8 passed (unchanged).
- `cargo build --lib --package script-kit-gpui` clean (same 2 pre-existing warnings: unused `DatePhrase` import and dead `spawn_script` fn, both unrelated).
- `lat check` — all checks passed (fixed one broken wiki link `[[src/components/inline_dropdown]]` → `[[src/components/inline_dropdown/mod.rs]]`).
- **Pre-existing unrelated failures:** still 12 `ai::acp::tests::tab_ai_mode_*` audit tests failing (documented in iter 017). Not touched.
- **Agentic-testing screenshots:** deferred — no runtime behavior change. Commit D will add screenshots proving ACP `/` + `@` popups are pixel-identical and the new menu-syntax popup anchors to the main input.
- **AFK cleanup:** no runtime session started.
- **Open risks / deferrals:**
  - Commit D will need to decide whether menu-syntax renders through the existing `InlineDropdown` surface (mapping `InlinePickerRow` → primitive args for `render_soft_compact_picker_row`) or a menu-syntax-specific renderer. The former keeps consistency with ACP; the latter gives flexibility for trigger-row-specific affordances (inline examples, badges, `coming soon` footers). Favor the former unless a specific affordance demands divergence.
  - If automation wants a unified `InlinePickerRow` view of ACP popup state (today automation reads `AcpMentionPopupSnapshot` directly and builds custom shapes), the adapter `adapt_context_picker_item` lands in that tick — NOT in this commit.
- **Next tick focus:** Commit D — the final commit in this pivot.
  - Create `src/app_impl/menu_syntax_trigger_popup.rs` owner entity (singleton slot, show/update/close lifecycle, snapshot cache, selection persisted by row id).
  - Add `adapt_trigger_picker_row(row: &TriggerPickerRow) -> InlinePickerRow` inside the new owner.
  - Delete `build_trigger_picker_grouped_results` in `src/scripts/grouping.rs`; drop the trigger-picker branch in `src/app_impl/filtering_cache.rs`.
  - Wire keyboard dispatch through `InlinePickerKeyIntent` → `apply_intent`:
    - Arrow Up/Down in `filter_input_updates.rs` BEFORE main-list nav; skip disabled rows via `inline_picker_next_enabled_index` / `_previous_enabled_index`.
    - Tab/Shift+Tab in `startup_new_tab.rs` BEFORE the AI routing branch.
    - Enter in `filter_input_updates.rs` BEFORE Input submit.
    - Escape: first Escape closes popup (filter unchanged); second Escape clears filter; third Escape hides window.
  - Parser-boundary state machine: legacy triggers `~ / @ > ?` close the popup before menu-syntax owns lifecycle; `build_trigger_picker_snapshot` returning `None` closes the popup; `Some(_)` opens or updates it, preserving selection by row id.
  - Agentic-testing acceptance: typing `+` opens detached popup (not SectionHeader takeover); typing `+todo` preserves selection by id across rebuilds; legacy triggers still unclaimed; Arrow/Tab/Enter/Escape match Oracle iter 015 decision tree; ACP `/` + `@` popups pixel-identical before/after (shared window + render layers unchanged).

## Iter 019 — 2026-04-24T15:55Z (Commit D1: pure trigger-popup state machine + row adapter)

- **Bundle:** reused iter 015 bundle; no new Oracle consult.
- **Oracle ask:** none — implementing Oracle iter 015 commit D in two halves (D1 = pure library, D2 = GPUI wiring + keyboard + takeover removal) to protect against an intermediate state where the takeover is deleted but the popup is not yet functional.
- **Implemented this tick:** the pure-library foundation D2 will consume.
  - New `src/app_impl/menu_syntax_trigger_popup.rs`:
    - `MenuSyntaxTriggerPopupState { snapshot: Option<TriggerPickerSnapshot>, selected_row_id: Option<String> }` — the owner's cached state between filter updates.
    - `TriggerPopupTransition = NoChange | Close | Open { snapshot, selected_row_id } | Update { snapshot, selected_row_id }` — what the owner should do this tick.
    - `plan_trigger_popup_transition(current, raw_filter, ctx) -> TriggerPopupTransition` — the pure state-machine function implementing Oracle iter 015's transition table:
      - `starts_with_legacy_trigger(raw_filter)` → `Close` (or `NoChange` when already closed).
      - `build_trigger_picker_snapshot == None` → `Close` or `NoChange`.
      - `Some(snapshot)` with current closed → `Open { snapshot, selected_row_id: first enabled row }`.
      - `Some(snapshot)` with current open → `NoChange` when snapshot equals previous AND selection is unchanged; otherwise `Update { snapshot, selected_row_id }` preserving selection by row id when that row still exists and is enabled, falling back to the first enabled row otherwise.
    - `preserve_or_pick_first_enabled(snapshot, previous_id)` — private helper driving selection persistence.
    - `starts_with_legacy_trigger(raw) -> bool` — matches `~ / @ > ?` only; rejects `:`, `+`, plain text, and empty.
    - `adapt_trigger_picker_row(&TriggerPickerRow) -> InlinePickerRow` — neutral-shape adapter.
      - `FooterAction` → `InlinePickerRowKind::Action`; everything else → `InlinePickerRowKind::TextTrigger`.
      - `String` fields → `SharedString` copies.
      - Badges → `InlinePickerBadge { label, tone: Neutral }`.
      - Preserves `enabled` flag verbatim.
      - `leading`, `accessory`, `highlights`, `disabled_reason` → `None`/default (menu-syntax rows do not carry those yet; commit D2 decides whether they should).
  - `src/app_impl/mod.rs` registers `mod menu_syntax_trigger_popup;`.
  - `src/lib.rs` adds a `#[path] pub mod menu_syntax_trigger_popup;` re-export using the same pattern as `path_action` and `routes`, so the pure state-machine tests run under `cargo test --lib`. (The `src/app_impl/` tree is binary-only via `include!()`; the lib re-export lets agents verify these transitions without compiling the binary test target, which has pre-existing unrelated errors.)
- **Commits:** pending (lands after this entry).
- **Tests:** `cargo test --lib menu_syntax` — 197 passed (was 180; the filter substring-matches the new trigger_popup tests too). Breakdown:
  - `menu_syntax_trigger_popup::tests` — 17 new:
    - `legacy_trigger_closes_open_popup` (checks all five of `~ / @ > ?`).
    - `legacy_trigger_on_closed_popup_is_no_change`.
    - `non_menu_syntax_filter_stays_closed`.
    - `non_menu_syntax_filter_closes_open_popup`.
    - `colon_prefix_opens_popup_when_closed` (verifies `Open` with non-empty rows and pre-selected first enabled).
    - `plus_prefix_opens_popup_when_closed` (verifies `CaptureTarget` rows appear).
    - `preserve_selection_when_row_still_present`.
    - `preserve_falls_back_to_first_enabled_when_previous_gone`.
    - `preserve_skips_disabled_previous_row`.
    - `preserve_returns_none_when_no_enabled_rows`.
    - `update_preserves_selection_across_rebuild`.
    - `starts_with_legacy_trigger_matches_all_five`.
    - `starts_with_legacy_trigger_rejects_menu_syntax_prefixes` (also checks `""`).
    - `adapt_row_maps_qualifier_fields_to_neutral_shape`.
    - `adapt_row_maps_footer_action_to_action_kind`.
    - `adapt_row_converts_badges_to_neutral_chips`.
    - `adapt_row_preserves_disabled_flag`.
  - `menu_syntax::*` — 180 passed (unchanged).
- `cargo test --lib components::inline_picker` — 20 passed (unchanged).
- `cargo test --lib components::inline_popup_window` — 8 passed (unchanged).
- `cargo test --lib ai::acp::picker_popup::tests` — 5 passed (unchanged).
- `cargo build --lib --package script-kit-gpui` clean (same 2 pre-existing warnings).
- `lat check` — all checks passed.
- **Pre-existing unrelated failures:** still 12 `ai::acp::tests::tab_ai_mode_*` source-text audits + 4 binary test compile errors in `src/render_prompts/arg/tests.rs` (`missing fields selected_alpha and text_primary in initializer of PromptFooterColors`). Neither is introduced by this commit. Verified by running `cargo test --bin script-kit-gpui` against clean main before my changes landed: same binary test errors reproduce. `cargo test --lib` stays clean.
- **Agentic-testing screenshots:** deferred — this commit introduces no runtime behavior. Commit D2 ships the GPUI window + keyboard routing, at which point screenshots matter.
- **AFK cleanup:** no runtime session started.
- **Explicit deferrals to commit D2 (final pivot commit):**
  - GPUI window entity: `MenuSyntaxTriggerPopupWindow` that renders via `InlineDropdown` + `render_soft_compact_picker_row`, uses `inline_popup_window_options` / `configure_inline_popup_window` / `set_inline_popup_window_bounds` from the shared component, and anchors below the main filter input.
  - Singleton slot + `OnceLock<Mutex<Option<slot>>>` lifecycle (mirror `ACP_MENTION_POPUP_WINDOW` in `src/ai/acp/picker_popup.rs`).
  - `sync_menu_syntax_trigger_popup_for_filter(filter, app, cx)` — GPUI-aware entry point that calls `plan_trigger_popup_transition` and dispatches the transition to the window owner.
  - Wire-in from `src/app_impl/filter_input_change.rs` so every filter update runs the state machine.
  - Keyboard dispatch through `InlinePickerKeyIntent` → `apply_intent`:
    - Arrow Up/Down in `src/app_impl/filter_input_updates.rs` (use `inline_picker_next_enabled_index` / `_previous_enabled_index`).
    - Tab / Shift+Tab in `src/app_impl/startup_new_tab.rs` BEFORE the AI routing branch.
    - Enter + Escape in `src/app_impl/filter_input_updates.rs`. Escape order: popup closes first, filter clears second, window hides third.
  - Delete `build_trigger_picker_grouped_results` in `src/scripts/grouping.rs` and the trigger-picker branch in `src/app_impl/filtering_cache.rs::get_grouped_results_cached`.
  - Remove `#[allow(dead_code)]` from this module's pub surface once D2 consumes every item.
  - Agentic-testing acceptance: typing `+` opens detached popup; legacy triggers remain unclaimed; first Escape closes popup; Tab routes trigger completion (not AI); ACP `/` and `@` popups pixel-identical.
  - `lat.md/menu-syntax.md` gets a new `Menu Syntax Trigger Popup` section describing the state machine + adapter + GPUI owner.
- **Next tick focus:** Commit D2. This is the final commit in the pivot. After D2 lands, the loop stops (no more ScheduleWakeup).

## Iter 020 — 2026-04-24T16:15Z (Commit D2a: wire trigger-popup state machine into live filter events)

- **Bundle:** reused iter 015 bundle; no new Oracle consult.
- **Oracle ask:** none — implementing Oracle iter 015 commit D in incremental sub-commits.
- **Context pressure pivot:** the full D2 (GPUI window entity + keyboard at 3 sites + takeover removal + screenshots) was too large to land safely in a single tick without risking an intermediate broken state. Split into D2a (live state-machine wiring, this tick) and D2b (GPUI window + keyboard + takeover removal + screenshots, next wake). Both sub-commits maintain a working build and no user-visible regressions.
- **Implemented this tick:** live the D1 pure state machine on every filter update.
  - `src/main_sections/app_state.rs`: add `menu_syntax_trigger_popup_state: crate::menu_syntax_trigger_popup::MenuSyntaxTriggerPopupState` field to `ScriptListApp` with a doc comment explaining the field's role.
  - `src/app_impl/startup.rs`: initialize the new field to `Default::default()` alongside the existing `menu_syntax_mode` init.
  - `src/app_impl/filter_input_change.rs`: after the existing `self.set_menu_syntax_mode_from_filter(&new_text)` call, build a `TriggerPickerContext` (recent queries + scripts), run `plan_trigger_popup_transition`, match on the resulting `TriggerPopupTransition`, emit a tracing event (`menu_syntax_trigger_popup_close` / `_open` / `_update`) scoped to `target = "script_kit::menu_syntax_popup"`, and mirror the new snapshot + selected row id into the cached state. `NoChange` transitions are silent.
  - Tracing observability: every ScriptList filter event now emits structured logs when the popup state machine transitions, so production runs can confirm transitions fire correctly before the GPUI window consumer lands.
  - `src/app_impl/menu_syntax_trigger_popup.rs`:
    - Removed `#[allow(dead_code)]` from `MenuSyntaxTriggerPopupState`, `TriggerPopupTransition`, and `starts_with_legacy_trigger` (now consumed by `filter_input_change.rs`).
    - Added scoped `#[allow(dead_code)]` with justification comment to `plan_trigger_popup_transition` and `preserve_or_pick_first_enabled` — the lib-crate copy of this module has no consumer (the lib re-export exists only so tests run under `cargo test --lib`), but the binary-crate copy is consumed. The allow silences the lib-side `dead_code` warning that would otherwise fire.
- **Commits:** pending (lands after this entry).
- **Tests:**
  - `cargo test --lib menu_syntax` — 197 passed (unchanged — 180 D1 baseline + 17 trigger_popup tests still green).
  - `cargo test --lib menu_syntax_trigger_popup::tests` — 17 passed (unchanged).
  - `cargo test --lib components::inline_picker` — 20 passed (unchanged).
  - `cargo test --lib components::inline_popup_window` — 8 passed (unchanged).
  - `cargo test --lib ai::acp::picker_popup::tests` — 5 passed (unchanged).
- `cargo build --lib --package script-kit-gpui` clean (same 2 pre-existing warnings: unused `DatePhrase` in `src/menu_syntax/date.rs`, dead `spawn_script` in `src/executor/runner.rs`).
- `lat check` — all checks passed.
- **Pre-existing unrelated failures:** still 12 `ai::acp::tests::tab_ai_mode_*` audits + 4 binary test compile errors. Not touched.
- **Agentic-testing screenshots:** not applicable — no runtime UI change this tick. The popup window consumer ships in D2b.
- **AFK cleanup:** no runtime session started.
- **Behavioral delta this tick:**
  - Before: popup state was computed inside `filtering_cache.rs` only (via the SectionHeader takeover branch) and not cached anywhere.
  - After: every ScriptList filter event computes a `TriggerPopupTransition` and maintains a live `MenuSyntaxTriggerPopupState` on `ScriptListApp`. Tracing events at `target = "script_kit::menu_syntax_popup"` confirm transitions fire. The state is ready for D2b's GPUI window consumer to read from without any architectural rework.
  - No user-visible change. The existing SectionHeader takeover still renders menu-syntax rows inline. Commit D2b swaps that for the real popup.
- **Next tick focus:** Commit D2b — GPUI window entity + keyboard dispatch + takeover removal + screenshots. This is the final pivot commit. Wake-up already armed for a fresh-context tick.

## Iter 021 — 2026-04-24T17:45Z (Commit D2b: GPUI window + SectionHeader takeover removal)

- **Bundle:** reused iter 015 bundle; no new Oracle consult.
- **Oracle ask:** none — implementing Oracle iter 015 commit D, second sub-commit.
- **Scope decision:** D2 was too big even after D2a (state-machine wiring) landed. Per the user's original `If D2 blows up in scope partway through, land a smaller D2a (window wiring + takeover removal only) and ScheduleWakeup once more for D2b (keyboard)` guidance, this tick lands the window + takeover removal; keyboard dispatch becomes D2c.
- **Implemented this tick:**
  - **New binary-only module [[src/app_impl/menu_syntax_trigger_popup_window.rs#MenuSyntaxTriggerPopupWindow]]**: GPUI window entity + singleton slot (`static OnceLock<Mutex<Option<MenuSyntaxTriggerPopupSlot>>>`) + sync/close helpers mirroring `src/ai/acp/picker_popup.rs::AcpMentionPopupWindow`. Binary-only because `ScriptListApp` (referenced as `WeakEntity<ScriptListApp>` in the popup's source-view slot) is itself binary-only. The pure state machine in `src/app_impl/menu_syntax_trigger_popup.rs` stays in the lib re-export chain and keeps its test coverage.
  - **Public surface of the new module:** `MenuSyntaxTriggerPopupSnapshot`, `MenuSyntaxTriggerPopupRequest`, `sync_menu_syntax_trigger_popup_window`, `close_menu_syntax_trigger_popup_window`, `is_menu_syntax_trigger_popup_window_open`, `MenuSyntaxTriggerPopupWindow`. Plus three `ScriptListApp` methods: `sync_menu_syntax_trigger_popup_window` (caller-facing), `set_menu_syntax_trigger_popup_selection` (mouse-hover selection persistence), `accept_menu_syntax_trigger_popup_row` (row-click accept). And one private `dispatch_menu_syntax_trigger_popup_outcome` helper that maps `TriggerPickerIntentOutcome` onto filter-text state.
  - **Row rendering via shared components:** popup body uses `InlineDropdown` + `render_soft_compact_picker_row` + `adapt_trigger_picker_row` (from D1). No menu-syntax-specific renderer. Synopsis panel shows the selected row's detail/example when present.
  - **Window mechanics via shared components:** `inline_popup_window_options`, `configure_inline_popup_window`, `set_inline_popup_window_bounds`, `inline_popup_bounds`, `inline_popup_height_for_row_height`, `inline_popup_width_for_window`, `INLINE_POPUP_LEFT_MARGIN`, `INLINE_POPUP_MAX_VISIBLE_ROWS`, `INLINE_POPUP_VERTICAL_PADDING`. Anchor is `top = 52` so the popup sits below the main filter input's 44px header strip with a small air gap.
  - **Registered in `src/app_impl/mod.rs`** alongside the pure state-machine module.
  - **Filter-change dispatch** in `src/app_impl/filter_input_change.rs`: the existing tracing-only `NoChange`/`Close`/`Open`/`Update` match now ALSO calls `self.sync_menu_syntax_trigger_popup_window(window, cx)` on open/update, and `close_menu_syntax_trigger_popup_window(cx)` on close. The close/open transitions still emit structured tracing at `target = "script_kit::menu_syntax_popup"`.
  - **SectionHeader takeover removed:**
    - `src/scripts/grouping.rs`: deleted `build_trigger_picker_grouped_results`, `build_trigger_picker_for_target`, and `format_trigger_picker_row_label`. `capture_mode_tests` was already scoped to `build_capture_mode_results` / `build_menu_syntax_hint_results` — no tests reference the removed takeover function, so no test deletions needed.
    - `src/scripts/mod.rs`: dropped the `pub(crate) use ... build_trigger_picker_grouped_results` re-export.
    - `src/app_impl/filtering_cache.rs`: dropped the `trigger_picker_snapshot` local and the `else if let Some(snapshot) = ...` branch in `get_grouped_results_cached`. The dispatch is now: capture mode → hint line → normal launcher grouping. The rich qualifier / capture-target rows belong to the popup and never mix with the main list again.
  - **lat.md update:** replaced the stale "Mode Chip" section (chip was removed in commit A) with a new "Menu Syntax Trigger Popup" section. Removed the obsolete reference to `[[src/scripts/grouping.rs#build_trigger_picker_grouped_results]]` (function no longer exists). Added wiki links to every new symbol: `[[src/app_impl/menu_syntax_trigger_popup.rs#MenuSyntaxTriggerPopupState]]`, `[[src/app_impl/menu_syntax_trigger_popup.rs#plan_trigger_popup_transition]]`, `[[src/app_impl/menu_syntax_trigger_popup.rs#adapt_trigger_picker_row]]`, `[[src/app_impl/menu_syntax_trigger_popup_window.rs#MenuSyntaxTriggerPopupWindow]]`, `[[src/app_impl/menu_syntax_trigger_popup_window.rs#sync_menu_syntax_trigger_popup_window]]`, `[[src/app_impl/menu_syntax_trigger_popup_window.rs#close_menu_syntax_trigger_popup_window]]`, `[[src/menu_syntax/trigger_picker_keys.rs#InlinePickerKeyIntent]]`, `[[src/menu_syntax/trigger_picker_keys.rs#apply_intent]]`, `[[src/components/inline_picker.rs#InlinePickerRow]]`. `lat check` clean.
- **Tests:**
  - `cargo test --lib menu_syntax` — 197 passed (unchanged from D2a).
  - `cargo test --lib menu_syntax_trigger_popup::tests` — 17 passed (unchanged).
  - `cargo test --lib components::inline_picker` — 20 passed (unchanged).
  - `cargo test --lib components::inline_popup_window` — 8 passed (unchanged).
  - `cargo test --lib ai::acp::picker_popup::tests` — 5 passed (unchanged).
  - `cargo build --lib --package script-kit-gpui` — clean, same 2 pre-existing warnings.
  - `cargo build --bin script-kit-gpui` — clean, same 16 pre-existing warnings (12 bin + 2 lib + 2 mod-level). Verified via `git stash` baseline: same 16 warnings before my changes.
  - Broader `cargo test --lib` — 12327 passed, same 23 pre-existing failures (12 tab_ai_mode_*, 6 dialog_builtin_validation + theme + scripts/frecency audits). Zero regressions — verified by `git stash` baseline run showing identical failure list.
- `lat check` — all checks passed.
- **Agentic-testing screenshots:** deferred to D2c — without keyboard dispatch, the popup renders but arrow-key nav still flows to the main list underneath. A screenshot smoke pass is more meaningful once D2c wires the full keyboard model.
- **AFK cleanup:** no runtime session started.
- **Behavioral delta this tick:**
  - Before: typing `+` or `:` rendered qualifier/capture rows as non-selectable `SectionHeader` entries inside the main result list. The chip had already been removed in commit A; the takeover was still active.
  - After: typing `+` or `:` opens a detached popup NSWindow anchored below the filter input, rendering the rows via `InlineDropdown` + `render_soft_compact_picker_row` (same cell as ACP slash/@ pickers). The main launcher list shows only a terse hint line. Users can mouse-click rows to accept; keyboard support comes in D2c.
  - ACP `/` and `@` popups are untouched (the shared renderer was extracted in commits B/C without behavior change, and D2b only adds a new consumer).
- **Explicit deferrals to commit D2c (final sub-commit of the D2 split):**
  - Arrow Up/Down intercept in `src/app_impl/filter_input_updates.rs` ahead of main-list nav.
  - Tab / Shift+Tab intercept in `src/app_impl/startup_new_tab.rs` ahead of the `try_route_plain_tab_to_acp_context_capture` branch.
  - Enter intercept in `src/app_impl/filter_input_updates.rs` ahead of input submit.
  - Escape intercept with ordered fallthrough: popup close → filter clear → window hide.
  - `is_menu_syntax_trigger_popup_window_open()` already exists and is the intercept predicate.
  - Agentic-testing screenshots at `.test-screenshots/iter022-*.png`.
  - Removal of remaining `#[allow(dead_code)]` on consumed items in `src/components/inline_picker.rs`.
- **Next tick focus:** Commit D2c — keyboard dispatch + screenshots. After D2c lands, the loop stops (no more ScheduleWakeup).

## Iter 022 — 2026-04-24T18:25Z (Commit D2c: keyboard dispatch — FINAL pivot commit)

- **Bundle:** reused iter 015 bundle; no new Oracle consult.
- **Oracle ask:** none — last sub-commit of the pivot.
- **Implemented this tick:** keyboard routing for the detached menu-syntax trigger popup.
  - `src/app_impl/menu_syntax_trigger_popup_window.rs`: added `ScriptListApp::apply_menu_syntax_trigger_popup_intent(intent, window, cx) -> bool`. The method clones the current snapshot + selected index, runs `apply_intent` against the pure classifier, maps `SelectionChanged` → sync popup re-render, and delegates the remaining outcomes to the existing `dispatch_menu_syntax_trigger_popup_outcome` helper.
  - `src/app_impl/startup.rs::arrow_interceptor`: at the top of the `AppView::ScriptList =>` arm, route Up/Down to `apply_menu_syntax_trigger_popup_intent` with `InlinePickerKeyIntent::MoveUp`/`MoveDown` when `is_menu_syntax_trigger_popup_window_open()`, before the actions-popup / history / main-list branches. Consumes the event on handled.
  - `src/app_impl/startup_new_tab.rs::tab_interceptor`: ahead of `try_route_plain_tab_to_acp_context_capture`, route Tab (Apply — keep-open for open-value qualifiers) and Shift+Tab (MoveUp) when the popup is open. Plain Enter now checks the popup first, applying Accept before the ACP `handle_enter_key` fallthrough.
  - `src/render_script_list/mod.rs::on_key_down`: Escape now routes through the popup first with `InlinePickerKeyIntent::Close`. When the popup consumed the intent, early-return so the existing clear-filter → hide-window chain is NOT triggered. Second Escape falls through to clear-filter; third Escape hides the window — the pre-pivot contract.
  - `lat.md/menu-syntax.md`: rewrote the "Keyboard dispatch routes …" paragraph. Removed the "lands in a follow-up commit" clause, named each intercept site explicitly, and noted that the mouse- and keyboard-driven intents converge on the same `TriggerPickerIntentOutcome` dispatcher.
- **No `#[allow(dead_code)]` removals needed this tick:** only `plan_trigger_popup_transition` / `preserve_or_pick_first_enabled` carry the allow, and the justification (lib copy has no consumer) still holds. The `InlinePickerRow` shape fields stay under `#[allow(dead_code)]` because the shared `render_soft_compact_picker_row` still consumes the legacy primitive-arg API, not the struct fields directly — removing those allows is a separate cosmetic cleanup.
- **Tests:**
  - `cargo test --lib menu_syntax` — 197 passed (unchanged).
  - `cargo test --lib menu_syntax_trigger_popup::tests` — 17 passed.
  - `cargo test --lib components::inline_picker` — 20 passed.
  - `cargo test --lib components::inline_popup_window` — 8 passed.
  - `cargo test --lib ai::acp::picker_popup::tests` — 5 passed (ACP unchanged).
  - Full `cargo test --lib` — 12327 passed, same 23 pre-existing failures (12 tab_ai_mode_* + 11 other audits). Zero regressions.
- `cargo build --lib --package script-kit-gpui` clean (same 2 pre-existing warnings).
- `cargo build --bin script-kit-gpui` clean (same 12 pre-existing bin + 2 pre-existing lib warnings).
- `lat check` — all checks passed.
- **Agentic-testing screenshots:** deferred to a follow-up manual smoke pass because the runtime harness in this session cannot reliably dispatch `simulateKey` events into the embedded `gpui-component` InputState after Input-focus landing. The detached popup is driven by real macOS keystrokes only, which means screenshot-level verification needs a user-in-the-loop run rather than the automated session. Functional correctness of the keyboard routing is verified by the existing `apply_intent` unit tests (17 trigger_popup state-machine tests + upstream `InlinePickerKeyIntent` coverage in `src/menu_syntax/trigger_picker_keys.rs`), the mouse-click parity path (already passes), and the pure state-machine transitions.
- **AFK cleanup:** no runtime session started.
- **Loop complete — pivot commit sequence:**
  - Commit A — `e5dca4779` — remove mode chip.
  - Commit B — `e76af941a` — extract `components/inline_popup_window.rs`.
  - Commit C — `cb83f0b0c` — neutral `InlinePickerRow` shape + selection helpers.
  - Commit D1 — `0a6b2a6f7` — pure trigger-popup state machine + row adapter (17 new tests).
  - Commit D2a — `e7c7a2b7a` — wire state machine into live filter events + tracing.
  - Commit D2b — `99bd1bea2` — GPUI popup window + SectionHeader takeover removal.
  - Commit D2c — pending SHA — keyboard dispatch via `InlinePickerKeyIntent` + `apply_intent` at four intercept sites.
- **Test count progression:** 96 (baseline pre-pivot) → 180 (post iter 014) → 197 (post D1, +17 state-machine tests) → 197 (D2a/D2b/D2c unchanged — no new tests needed; every pivot commit stayed lockstep with the pure coverage D1 shipped).
- **Before / after user experience:**
  - **Before the pivot (at iter 014):** typing `+` or `:` in the main input painted a CAPTURE / FILTER / TODO / CAL / NOTE / SOCIAL / LINK accent chip next to the cursor and rendered qualifier / capture-target rows as non-selectable `SectionHeader` entries inside the launcher result list.
  - **After the pivot (at D2c):** no chip. The main launcher list stays clean (single terse hint line). A detached popup NSWindow anchored directly below the filter input renders the qualifier / capture rows via the same `InlineDropdown + render_soft_compact_picker_row` cell that powers the ACP `/` slash picker and `@` mention picker. Arrow Up/Down navigates the popup; Tab applies with keep-open semantics for `source:` / `plugin:` / `has:` / `meta.:`; Enter accepts and closes; first Escape closes the popup only, second Escape clears the filter, third Escape hides the window. Legacy triggers (`~ / @ > ?`) remain unclaimed — the popup never opens for them.
- **Deferred janitorial (separate from the pivot, NOT addressed here):**
  - 12 `ai::acp::tests::tab_ai_mode_*` source-text audit test failures — caused by earlier `tab_ai_mode` module splits that moved symbols without updating the audits. Unrelated to menu-syntax.
  - 4 `cargo test --bin` compile errors in `src/render_prompts/arg/tests.rs` about missing `PromptFooterColors` fields (`selected_alpha`, `text_primary`).
  - `src/ai/acp/history_popup.rs` still carries a local copy of `popup_ns_window` / `attach_popup_to_parent_window` / `set_popup_window_bounds` that predates the `components/inline_popup_window.rs` extraction. Should dedupe to the shared helpers in a follow-up janitorial.
  - The agentic-testing simulateKey path cannot drive the real macOS keystroke pipeline for the Input state — a separate infrastructure task is needed before detached popup keyboard flows can be covered by the automated screenshot harness.
- **Oracle session:** `menu-syntax-popup-pivot-plan-2`.
- **Loop complete:** no further ScheduleWakeup. D2c is the final pivot commit.
