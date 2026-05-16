# Menu Syntax — Progress Log

Append-only. One entry per Oracle iteration. Newest at the bottom so `cat` reads chronologically.


```
## Iter NNN — <iso-utc>
```

---


  - Parser lives in NEW `src/menu_syntax/` module — do NOT add to `special_entry_from_script_list_filter`. Parse later in result computation so stdin/automation paths stay consistent.
  - Skills stay AI-oriented; don't surface PluginSkill for non-AI capture yet.


  - Cleanup commit addressing all six review corrections + new test coverage.
  - Five example `.ts` scripts shipped.
  - JSON schema at `schemas/menu-syntax/payload-v1.schema.json`.


  - Add a raw-guarded `MenuSyntaxMode` struct in `src/menu_syntax/mode.rs`. Parse at input-change boundaries (`handle_filter_input_change`, `set_filter_text_immediate`), guard consumption by raw-equality so a stale parse never applies to newer input via the 8ms coalescer.
  - Commit 1 `83fc5fb94` — `MenuSyntaxMode`, raw guard, `free_text_for_search`, `prefix_span_for_input` (15 new tests).
  - Commit 2 `c7b71ae52` — `get_grouped_results_with_input_history_and_query` + validation variant + ScriptIssue suppression (8 new tests).
  - Commit 3 `3291eb248` — `build_capture_mode_results` (5 new tests).
  - Commit 4 `3e9201501` — `build_menu_syntax_hint_results` (2 new tests).
  - Commit 8 `957db4d1e` — fallback-state suppression for menu-syntax; detached-spawn for capture handlers (bypass SDK preload).
  - GPUI adapter in `src/app_execute/menu_syntax_execution.rs` — blocked on user's in-progress bin-level compile errors (`render_setup.rs`, `prompt_handler/mod.rs`, `render_prompts/arg/tests.rs`). Land after those fixes.
  - Plugin-owned skills do not yet participate in capture mode (Oracle's gate).
  - `argv.v1` (`!`) stays behind `KIT_MENU_SYNTAX_ARGV=1` until real handler fixtures exist.




  - New `src/menu_syntax/trigger_picker.rs` (818 lines).
  - `TriggerPickerMode` enum (`AdvancedQuery`, `Capture`) — one picker with two modes.
  - `TriggerPickerRowKind` enum including reserved `CaptureHandler` / `CaptureArtifact` variants for commits 2 and 3.
  - `TriggerPickerAction` enum with `InsertToken { keep_open }`, `ReplaceInput`, `FixQualifier`, `CreateHandler`, `OpenHelp`, plus reserved `ExecuteCaptureHandler` / `OpenCaptures` under `#[allow(dead_code)]` to lock the action enum shape now (avoids churn in commits 2/7/8).
  - `TriggerPickerRow` / `TriggerPickerSnapshot` / `TriggerPickerContext` structs.
  - `UnknownQualifierFix` rows driven by a within-one-edit detector that handles single-char edits AND adjacent transpositions (`typ` → `type`, `tpye` → `type`). Does NOT change parser behavior — only surfaces a fix row. `meta.<path>` qualifiers are excluded from typo scan.
  - Session-only recent-query rows filtered to strings whose raw parse is `AdvancedQuery` / `BareQueryPrefix`. No disk persistence.
  - Capture target rows for the five shipped targets (`+todo`, `+cal`, `+note`, `+social`, `+link`) with title/detail/example metadata.
  - `src/menu_syntax/mod.rs` re-exports the new types.
  - `removed-docs` gains a `Trigger Picker` section.
  - legacy triggers `~ ~/Desktop / @ > ?` return `None`
  - unknown `+` heads (`+github`, `+1`, `+react component`) return `None`
  - bare `+` builds 5 target rows in spec order
  - `+todo buy milk` focuses to 1 target row with `target = Some("todo")`
  - `+todo` incomplete still focuses to 1 target row
  - footer `create-handler` carries the focused target
  - row ids are unique within every snapshot
  - `within_one_edit` detector unit test passes all expected cases


  - **Legacy triggers `~ / @ > ?` still win** — raw-guard must survive.


  - New function `build_trigger_picker_grouped_results(snapshot)` in `src/scripts/grouping.rs` plus `build_trigger_picker_for_target` and `format_trigger_picker_row_label` helpers. Consumes the iter 005 `TriggerPickerSnapshot` and emits a mode-appropriate section header (`Filter qualifiers` / `Capture targets` / `Capture <target>`) followed by one `SectionHeader` per non-footer row formatted as `<token>  ·  <title>` (title-only fallback). The footer row is always appended last with a `(coming soon)` suffix so the UI isn't dead.
    1. Capture complete (existing `build_capture_mode_results`) — unchanged, preserves `+todo buy milk` handler row.
  - Updated `removed-docs` Trigger Picker section with iter 008 wiring note.
  - `.test-screenshots/iter008-plus.png` — CAPTURE chip + Capture targets section + 5 target rows + `Create capture handler... (coming soon)` footer.
  - `.test-screenshots/iter008-plus-todo.png` — TODO chip + existing `Capture Todo Inbox` handler row (confirms old capture-complete path still works).
  - `.test-screenshots/iter008-plus-todo-incomplete.png` — TODO chip + focused `+todo Todo inbox` row + `Create capture handler for +todo... (coming soon)` footer.
  - `.test-screenshots/iter008-localhost.png` — no chip, fallback rows (parser boundary preserved).
  - Picker rows are non-selectable. Users can SEE available qualifiers/targets but Enter/Tab/Esc don't activate them yet — that's commit 3.
  - Footer row label reads `(coming soon)` — visible but deliberately dead UI. Commit 6 wires scaffold-handler action, which will drop the suffix.
  - ACP slash picker not verified this tick with a screenshot — but zero ACP code was touched, so regression is not possible from this commit alone.


  - New `src/menu_syntax/trigger_picker_keys.rs` (520 lines).
  - Selectable-index helpers (`first_selectable_index`, `last_selectable_index`, `next_selectable_index`, `prev_selectable_index`) skip `FooterAction` rows and wrap at list edges. Footer rows are visible but explicit navigation (MoveEnd, PageDown) is still required to reach them — Enter on them never fires by default, matching Oracle's "footer must never be dead UI but also never an accidental selection" rule.
  - Module registered in `src/menu_syntax/mod.rs` with public re-exports guarded by `#[allow(unused_imports)]` since GPUI wiring lands in a later tick.
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
  - GPUI event wiring (ArrowUp/ArrowDown/Tab/Enter/Escape/Cmd+N/Cmd+P) inside `src/app_impl/filter_input_change.rs` / `startup_new_tab.rs` / `render_script_list/mod.rs` is deferred to a user-present tick. Tab specifically competes with `startup_new_tab.rs` AI routing — routing it via the menu-syntax classifier without breaking AI tab cycling is the most delicate piece.
  - `OpenHelp`, `OpenCaptures`, `CreateHandler` outcomes have no owner yet. That's intentional — they get owners in commits 5–7.


  - New `src/menu_syntax/handler_index.rs` (pure lib).
  - `HandlerScore { exact_target, default_handler, user_authored, accepts_boost }` — lexicographic tuple, higher sorts first. `accepts_boost` caps at `MAX_ACCEPTS_BOOST = 3` so it can only break ties within a priority bucket, never cross them (Oracle iter 004 explicit rule).
  - `RankedHandler { script, spec, score }` — one entry per (script, spec) pair. A script that declares both an exact-target spec and a wildcard spec appears twice in the ranking (tests lock this).
  - `rank_handlers_for_target(scripts, invocation) -> Vec<RankedHandler>` iterates every script's `menuSyntax` entries (via existing `script_menu_syntax_specs`), filters to `family == "capture.v1"`, scores matches by `(exact_target, default_handler, user_authored, accepts_boost)`, and sorts descending with script name as the stable tie-break.
  - `rank_scripts_handling_capture(scripts, invocation) -> Vec<Arc<Script>>` — dedup-by-path convenience for callers that already dedupe on script identity.
  - `KNOWN_ACCEPTS = ["date", "url", "tag", "tags", "priority", "duration", "kv"]` — any other token in a handler's `accepts` list is silently ignored (permissive classifier, not a parser).
  - `invocation_has(accept, invocation)` — a declared accept counts when the matching field of the `CaptureInvocation` is populated (non-empty `date_phrases` for "date", `Some` for `url`/`priority`/`duration`, etc.).
  - `src/menu_syntax/mod.rs` re-exports `rank_handlers_for_target`, `rank_scripts_handling_capture`, `HandlerScore`, `RankedHandler` under `#[allow(unused_imports)]` so later commits (5 handler actions) can consume them without churning the public surface.
  - `removed-docs` Capture Handler Filtering section gains a paragraph referencing `rank_handlers_for_target` and `rank_scripts_handling_capture` with the lexicographic score semantics, tie-break rule, and accept-boost cap.
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
- `cargo build --lib` — clean.
- `source checks` — all checks passed.
  - Multi-handler fixtures for agentic-testing are not yet scaffolded. That's a natural pairing with commit 6 (Cmd+N scaffold), which creates extra user-authored handlers for free.
  - `build_capture_mode_results` is the only live consumer switched to the ranked variant in this commit. When commit 5 wires capture-handler rows into the trigger picker snapshot, it will also consume `rank_handlers_for_target` directly.
  - `KNOWN_ACCEPTS` is intentionally small — only accepts tokens that map to an observable `CaptureInvocation` field. Expanding the list (e.g. "markdown", "ics") is a per-target design call, not a handler_index change.


    - `build_capture_snapshot(target, ctx)` — now receives the context so it can call `capture_handler_rows` when a target is known.
    - `handler_command_id(script)` — encodes plugin + file stem so `ExecuteCaptureHandler { command_id }` has a deterministic identity. Downstream execution code lives in `menu_syntax_execution.rs` (commit 7 or later will connect the last wire).
    - `probe_invocation(target)` — builds a minimal `CaptureInvocation` with empty body/tags/dates so the ranker can run before the user has typed the body. Accepts-boost is therefore 0 in incomplete mode, which is correct — we don't want to guess the payload before the user types it. Complete captures (already routed to `build_capture_mode_results`) still get the real invocation with populated fields.
  - `removed-docs` Trigger Picker section — added a paragraph describing `capture_handler_rows`, the 5-row cap, and the `TriggerPickerContext.scripts` field.
  - `capture_handler_rows_empty_when_no_scripts` — empty catalog → no handler rows.
  - `capture_handler_rows_surface_for_known_target` — single user-authored default handler shows `default` badge + plugin-scoped command id.
  - `capture_handler_rows_capped_at_max` — 8 handlers → exactly 5 rendered.
  - `capture_handler_rows_preserve_ranked_order` — user-default > shipped-default > user-plain > shipped-plain order holds end-to-end from picker snapshot.
  - `wildcard_only_handler_row_flags_wildcard_badge` — wildcard-only handler carries `wildcard` badge.
  - `capture_handler_rows_skip_when_no_matching_specs` — unrelated target handlers never appear.
  - `bare_capture_mode_has_no_handler_rows_even_with_scripts` — bare `+` still shows targets only.
- `cargo build --lib` clean; `cargo build --bin script-kit-gpui` clean.
- `cargo fmt --package script-kit-gpui` formatted the new code.
- `source checks` passed.
  - `ExecuteCaptureHandler { command_id }` action has no dispatcher yet. The action enum variant keeps its `#[allow(dead_code)]` until a user-present tick wires keyboard routing through `apply_intent` into an execution helper.
  - The help/typo/recent rewrite half of Oracle commit 5 is still outstanding. That needs to live in `filter_input_change.rs` / `filter_input_updates.rs` and should ship with a screenshot pass.


  - New `src/menu_syntax/templates.rs` with `render_capture_handler_template(target, slug) -> String`. Output is a TypeScript source string that a caller can drop at `~/.scriptkit/plugins/main/scripts/capture-<target>-<slug>.ts`.
    - Leading filename comment (`// capture-<slug>.ts`).
    - Header doc explaining when the handler fires and pointing at `removed-docs Payload`.
    - `KIT_MENU_SYNTAX_PAYLOAD_PATH` env reader with an actionable error message.
    - `SK_PATH` fallback to `~/.scriptkit`; writes `<$SK_PATH>/menu-syntax/<artifact>.jsonl`.
    - Emitted body echoes `target`, `body`, `tags`, `priority`, `url`, `duration`, `dates`, `raw`, `createdAt` — an easy starting point the author can trim down.
    - `slug_or_target(target, slug)` — normalizes user input to kebab-case alphanumerics; falls back to the target string when the slug is empty or all non-alphanumeric.
    - `display_name_from_slug(target, slug)` — title-cases the slug into a human-readable name (`"jira sync"` → `"Capture Jira Sync"`).
    - `artifact_hint_for(target)` — maps known targets to the same filenames used by shipped examples (`todos.jsonl`, `events.jsonl`, `notes.jsonl`, `drafts.jsonl`, `bookmarks.jsonl`); unknown targets fall back to `entries.jsonl`.
    - `accepts_hint_for(target)` — known targets get the same `accepts` list as the shipped handler so picker scoring stays consistent; unknown targets get a generic `["tags", "date", "url", "kv"]`.
  - `src/menu_syntax/mod.rs` registers `pub mod templates` and re-exports `render_capture_handler_template` under `#[allow(unused_imports)]` (dispatcher lands later).
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
- `cargo fmt`, `source checks` both pass.
  - Scripts produced from this template are not yet auto-imported into the launcher's script catalog. The user has to restart Script Kit (or trigger a reload) after saving the scaffold — acceptable for AFK; revisit when wiring the authoring flow.
  - `accepts` hint duplicates values from shipped examples. If the shipped examples' accepts lists change, these hints will drift; centralizing them behind a shared table is worth doing when there are more than 2 readers.


  - New `src/menu_syntax/artifacts.rs` with `CaptureArtifactKind` (`Todo`, `CalendarEvent`, `Note`, `SocialDraft`, `Bookmark`, `Payload`), `CaptureArtifact`, and `ReadArtifactReport`.
  - `read_jsonl_artifact(path, kind)` — line-by-line JSONL reader. Missing files yield an empty report (not a warning — a user who hasn't captured a note yet shouldn't see a warning). Unreadable files surface one warning and bump `skipped`. Malformed lines bump `skipped` and push a warning (capped at `MAX_WARNINGS = 10`). Blank lines are skipped silently. Non-object scalars (`"string"`, `42`) are still included with best-effort snippets.
  - `read_payload_dir(path)` — enumerates `capture_v1-*.json` tempfiles only; other files in the directory are ignored silently. Missing directory yields an empty report.
  - `extract_created_at` reads `createdAt` (shipped examples + template) or `timestamp` (payload tempfiles) without privileging either.
  - `src/menu_syntax/mod.rs` registers `pub mod artifacts;` and re-exports `CaptureArtifact`, `CaptureArtifactKind`, `ReadArtifactReport`, `read_all_artifacts`, `read_jsonl_artifact`, `read_payload_dir` under `#[allow(unused_imports)]`.
  - `removed-docs` gains a `Captures Inverse Browser` section above `Shipped Examples` documenting the tolerant-reader contract, the warning cap, the snippet truncation rule, and why `Payload` is excluded from `BROWSER_ORDER`.
  - `read_jsonl_artifact_returns_all_valid_entries`
  - `read_jsonl_artifact_skips_malformed_lines_with_warning`
  - `read_jsonl_artifact_handles_missing_file_gracefully` (no warning for missing)
  - `read_jsonl_artifact_ignores_blank_lines`
  - `read_jsonl_artifact_truncates_snippet_for_long_bodies` (ends with `…`)
  - `snippet_falls_back_to_raw_when_body_is_missing`
  - `non_object_top_level_json_is_still_included`
  - `read_payload_dir_returns_only_capture_v1_files` (unrelated files silently skipped; bad `capture_v1-*.json` counts as skipped)
  - `read_payload_dir_handles_missing_dir_gracefully`
  - `read_all_artifacts_counts_warnings_across_files`
  - `warning_cap_prevents_unbounded_accumulation` (30 dirty rows → 10 warnings, all counted as skipped)
  - `artifact_filename_for_matches_templates_and_shipped_examples`
  - `browser_order_excludes_payload` (regression guard against a future refactor accidentally surfacing payload rows to users)
- `cargo build --lib` clean; `cargo fmt` OK; `source checks` passed after trimming the new section's leading paragraph to ≤250 chars.
  - No built-in view consumes `read_all_artifacts` yet. The Captures inverse browser ships in a later tick that pairs the reader with a new builtin and wires it into the picker's `OpenCaptures` outcome (currently deferred per iter 009 commit 3).
  - No file-system write paths in this module. Retention + deletion belong to commit 8.
  - `read_all_artifacts` does not yet expose per-kind counts (for a header like "Captures (42)"). Callers can compute that from `entries.iter().filter(|e| e.kind == K).count()`; if a hotter path needs it, add a `counts` field to `ReadArtifactReport`.


    - `PayloadListing { path, created_at_unix }` — caller-enumerated entry shape. The retention module never enumerates files itself, so user JSONL / markdown / `.ics` / social drafts are untouchable by construction.
    - `RetentionPlan { keep, delete }` ordered newest-first.
    - `apply_retention_plan(plan) -> AppliedRetention { deleted, failed }` — thin FS helper; treats missing paths as successful no-ops so concurrent passes don't spuriously fail.
  - `src/menu_syntax/mod.rs` registers `pub mod retention` and re-exports `plan_retention`, `apply_retention_plan`, `PayloadListing`, `RetentionConfig`, `RetentionPlan`, `AppliedRetention`, and the three default constants under `#[allow(unused_imports)]`.
  - `removed-docs` gains a `Payload Retention` section above `Captures Inverse Browser` documenting the policy, the newest-250 floor invariant, and the pure/FS split.
  - `empty_listing_yields_empty_plan`
  - `listing_under_keep_newest_keeps_everything_regardless_of_age` (50 ancient files all kept under the 250 floor)
  - `age_rule_only_triggers_outside_newest_floor` (250 young + 10 old → all 250 kept, 10 old deleted)
  - `old_entries_inside_newest_floor_are_still_kept` (50 ancient files all kept — invariant regression guard)
  - `hard_cap_trims_excess_even_when_young` (1050 young → 1000 kept, 50 deleted by hard cap)
  - `newest_250_invariant_never_violated_even_when_all_old` (300 old → 250 kept, 50 deleted)
  - `age_rule_cutoff_is_strictly_greater_than` (boundary file at exactly 30d is kept)
  - `plan_is_deterministic_for_identical_inputs` (order-independence)
  - `tie_break_on_timestamp_uses_path_ordering` (identical timestamps → alphabetical rank)
  - `keep_order_is_newest_first` (plan.keep is sorted for streaming consumers)
  - `default_constants_match_oracle_iter_004_numbers` (250 / 1000 / 30 locked in)
- `cargo build --lib` clean; `cargo fmt`, `source checks` pass.
  - No caller invokes `plan_retention` yet. The hook belongs alongside a successful capture execution — Oracle iter 004 said "run opportunistically after successful payload write, not in a daemon." Wiring that up is a one-liner in the execution path but best done with the user available to watch for file-system surprises.
  - Retention only targets payloads. If a future handler starts writing capture artifacts into the payload dir, the pattern filter (`capture_v1-*.json`, caller-enforced) keeps the policy safe.
  - GPUI event wiring for ArrowUp/ArrowDown/Tab/Enter/Escape/Cmd+N/Cmd+P — Tab conflicts with AI routing in `src/app_impl/startup_new_tab.rs`; Enter conflicts with the focused Input submit path.
  - Handler/picker row *selection* so CaptureHandler, FixQualifier, OpenHelp, and create-handler footer rows stop rendering as non-selectable SectionHeaders.
  - Filesystem writer + editor-open flow for scaffolded handlers (`capture-<target>-<slug>.ts`).
  - Captures inverse-browser builtin view that consumes `read_all_artifacts`.
  - `apply_retention_plan` invocation hook after successful payload writes, plus HUD copy improvements.
  - Optional shared renderer extraction (`src/components/inline_picker.rs`) per Oracle option C — still on the table but gated on the keyboard/selection landing first.


  - **Commit D** — Land the menu-syntax popup. New `src/app_impl/menu_syntax_trigger_popup.rs` (owner entity, snapshot sync, selection by row id, key dispatch bridge to `apply_intent`, `adapt_trigger_picker_row`). Delete `build_trigger_picker_grouped_results` in `src/scripts/grouping.rs`. Drop the trigger-picker branch in `src/app_impl/filtering_cache.rs`. Wire keyboard dispatch (Arrow/Tab/Enter/Escape) in `filter_input_updates.rs` + `startup_new_tab.rs` (Tab guard must come before AI routing).
  - **No closures, no domain actions.** Owners map `id` → their own accept logic.
  - `Tab` / `Shift+Tab` — intercept in `src/app_impl/startup_new_tab.rs` **before** AI routing. `CompleteSelected` / `MovePrevious` (or `CompletePrevious` if intent exists).
  - `Enter` — intercept in `filter_input_updates.rs` before input submit. Add backstop in `filter_input_change.rs` if GPUI submit bypasses keydown. `AcceptSelected`.
  - `Escape` — intercept in `filter_input_updates.rs` before launcher clear-filter. First Escape closes popup only (filter unchanged); second Escape clears filter; third Escape hides window per existing flow.
  ```rust
      if starts_with_legacy_trigger(filter) { close_menu_syntax_trigger_popup(app, cx); return; }
      match build_trigger_picker_snapshot(filter) {
          Some(snapshot) => show_or_update_menu_syntax_trigger_popup(app, snapshot, cx),
          None => close_menu_syntax_trigger_popup(app, cx),
      }
  }
  ```
  - Typed `+` must remain in filter text; only delete the rendered chip, not the user input.
  - Grep for existing `InlineDropdown`/shared row primitives before creating `inline_picker.rs`; extend or wrap if equivalents exist.
  - `TriggerPickerRow.id` must be stable across rebuilds; verify before depending on id-based selection persistence.
  - Unicode highlight ranges must be char-boundary-safe (UTF-8) — bad byte slicing will panic.
  - Do **not** update popup state from `filtering_cache.rs`; popup lifecycle belongs in input update/change handling.


  - `src/components/mod.rs` registers `pub mod inline_popup_window`.
  - `src/ai/acp/popup_window.rs` rewritten as a thin compatibility facade. Every `DENSE_PICKER_*` / `dense_picker_*` / `popup_*` name is kept via `pub(crate) use ... as old_name;` aliases, so `picker_popup.rs`, `model_selector_popup.rs`, `history_popup.rs`, `view.rs`, `src/storybook/playground_overlay_metrics.rs`, and the source-text audit tests in `src/ai/acp/tests.rs` compile without edits. The ACP-flavored convenience `dense_picker_height(item_count)` stays local so ACP callers keep passing a bare count and get `CONTEXT_PICKER_ROW_HEIGHT` applied automatically.
  - `removed-docs` gains a doc link pointing at `src/components/inline_popup_window.rs` and a new sentence under `Popup behavior` explaining the shared-module + facade pattern.
- `cargo build --lib --package script-kit-gpui` clean; `source checks` passes.
  - The tracing event name changed from `acp_inline_dropdown_popup_attached` to `inline_popup_attached`. This is the only non-trivial behavior diff; it's intentional since the helper now serves multiple surfaces. Telemetry dashboards filtering on the old event name will need updating.
  - The compatibility facade `src/ai/acp/popup_window.rs` can be removed entirely once all ACP callers are migrated to the neutral names. Deferring that rename sweep until after commit D lands so the popup pivot plan doesn't blow its blast radius.
  - `history_popup.rs` has a second, local copy of `popup_ns_window` / `attach_popup_to_parent_window` / `set_popup_window_bounds` (introduced independently). Not unified here to keep commit B's blast radius minimal; worth de-duping after the pivot plan completes.


    - `InlinePickerRowId = SharedString`.
    - `InlinePickerRow { id, kind, title, token, subtitle, detail, example, leading, badges, accessory, highlights, enabled, disabled_reason }`.
    - `InlinePickerRowKind = Context | SlashCommand | TextTrigger | Action | Custom(SharedString)` — visual classification, no behavioral inference.
    - `InlinePickerLeadingVisual = Glyph(SharedString) | IconName(SharedString)`.
    - `InlinePickerAccessory = Text | Shortcut | Token`.
    - `InlinePickerHighlights { title, token, subtitle, detail }` with `Vec<Range<usize>>` per slot.
    - `#[allow(dead_code)]` on every pub surface since the first consumer (menu-syntax trigger popup) lands in commit D.
  - `src/components/mod.rs` registers `pub mod inline_picker`.
- **No changes to `src/ai/acp/picker_popup.rs`, `model_selector_popup.rs`, `history_popup.rs`, `view.rs`, or `src/scripts/grouping.rs` in this commit.** ACP rendering remains unchanged; menu-syntax's existing inline-SectionHeader takeover remains in place so users are not left without guidance between C and D.
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
- `source checks` — all checks passed (fixed one broken doc link `[[src/components/inline_dropdown]]` → `[[src/components/inline_dropdown/mod.rs]]`).
  - Commit D will need to decide whether menu-syntax renders through the existing `InlineDropdown` surface (mapping `InlinePickerRow` → primitive args for `render_soft_compact_picker_row`) or a menu-syntax-specific renderer. The former keeps consistency with ACP; the latter gives flexibility for trigger-row-specific affordances (inline examples, badges, `coming soon` footers). Favor the former unless a specific affordance demands divergence.
  - If automation wants a unified `InlinePickerRow` view of ACP popup state (today automation reads `AcpMentionPopupSnapshot` directly and builds custom shapes), the adapter `adapt_context_picker_item` lands in that tick — NOT in this commit.
  - Create `src/app_impl/menu_syntax_trigger_popup.rs` owner entity (singleton slot, show/update/close lifecycle, snapshot cache, selection persisted by row id).
  - Delete `build_trigger_picker_grouped_results` in `src/scripts/grouping.rs`; drop the trigger-picker branch in `src/app_impl/filtering_cache.rs`.
    - Arrow Up/Down in `filter_input_updates.rs` BEFORE main-list nav; skip disabled rows via `inline_picker_next_enabled_index` / `_previous_enabled_index`.
    - Tab/Shift+Tab in `startup_new_tab.rs` BEFORE the AI routing branch.
    - Enter in `filter_input_updates.rs` BEFORE Input submit.


    - `TriggerPopupTransition = NoChange | Close | Open { snapshot, selected_row_id } | Update { snapshot, selected_row_id }` — what the owner should do this tick.
      - `starts_with_legacy_trigger(raw_filter)` → `Close` (or `NoChange` when already closed).
      - `build_trigger_picker_snapshot == None` → `Close` or `NoChange`.
      - `Some(snapshot)` with current open → `NoChange` when snapshot equals previous AND selection is unchanged; otherwise `Update { snapshot, selected_row_id }` preserving selection by row id when that row still exists and is enabled, falling back to the first enabled row otherwise.
    - `preserve_or_pick_first_enabled(snapshot, previous_id)` — private helper driving selection persistence.
    - `adapt_trigger_picker_row(&TriggerPickerRow) -> InlinePickerRow` — neutral-shape adapter.
      - `String` fields → `SharedString` copies.
      - Preserves `enabled` flag verbatim.
      - `leading`, `accessory`, `highlights`, `disabled_reason` → `None`/default (menu-syntax rows do not carry those yet; commit D2 decides whether they should).
  - `src/app_impl/mod.rs` registers `mod menu_syntax_trigger_popup;`.
  - `src/lib.rs` adds a `#[path] pub mod menu_syntax_trigger_popup;` re-export using the same pattern as `path_action` and `routes`, so the pure state-machine tests run under `cargo test --lib`. (The `src/app_impl/` tree is binary-only via `include!()`; the lib re-export lets agents verify these transitions without compiling the binary test target, which has pre-existing unrelated errors.)
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
- `cargo build --lib --package script-kit-gpui` clean (same 2 pre-existing warnings).
- `source checks` — all checks passed.
  - Singleton slot + `OnceLock<Mutex<Option<slot>>>` lifecycle (mirror `ACP_MENTION_POPUP_WINDOW` in `src/ai/acp/picker_popup.rs`).
  - `sync_menu_syntax_trigger_popup_for_filter(filter, app, cx)` — GPUI-aware entry point that calls `plan_trigger_popup_transition` and dispatches the transition to the window owner.
  - Wire-in from `src/app_impl/filter_input_change.rs` so every filter update runs the state machine.
    - Arrow Up/Down in `src/app_impl/filter_input_updates.rs` (use `inline_picker_next_enabled_index` / `_previous_enabled_index`).
    - Tab / Shift+Tab in `src/app_impl/startup_new_tab.rs` BEFORE the AI routing branch.
  - Remove `#[allow(dead_code)]` from this module's pub surface once D2 consumes every item.
  - `removed-docs` gets a new `Menu Syntax Trigger Popup` section describing the state machine + adapter + GPUI owner.


    - Removed `#[allow(dead_code)]` from `MenuSyntaxTriggerPopupState`, `TriggerPopupTransition`, and `starts_with_legacy_trigger` (now consumed by `filter_input_change.rs`).
    - Added scoped `#[allow(dead_code)]` with justification comment to `plan_trigger_popup_transition` and `preserve_or_pick_first_enabled` — the lib-crate copy of this module has no consumer (the lib re-export exists only so tests run under `cargo test --lib`), but the binary-crate copy is consumed. The allow silences the lib-side `dead_code` warning that would otherwise fire.
  - `cargo test --lib menu_syntax` — 197 passed (unchanged — 180 D1 baseline + 17 trigger_popup tests still green).
- `source checks` — all checks passed.
  - No user-visible change. The existing SectionHeader takeover still renders menu-syntax rows inline. Commit D2b swaps that for the real popup.


  - **Registered in `src/app_impl/mod.rs`** alongside the pure state-machine module.
  - `cargo test --lib menu_syntax` — 197 passed (unchanged from D2a).
  - `cargo build --lib --package script-kit-gpui` — clean, same 2 pre-existing warnings.
  - Broader `cargo test --lib` — 12327 passed, same 23 pre-existing failures (12 tab_ai_mode_*, 6 dialog_builtin_validation + theme + scripts/frecency audits). Zero regressions — verified by `git stash` baseline run showing identical failure list.
- `source checks` — all checks passed.
  - ACP `/` and `@` popups are untouched (the shared renderer was extracted in commits B/C without behavior change, and D2b only adds a new consumer).
  - Arrow Up/Down intercept in `src/app_impl/filter_input_updates.rs` ahead of main-list nav.
  - Tab / Shift+Tab intercept in `src/app_impl/startup_new_tab.rs` ahead of the `try_route_plain_tab_to_acp_context_capture` branch.
  - Enter intercept in `src/app_impl/filter_input_updates.rs` ahead of input submit.
  - `is_menu_syntax_trigger_popup_window_open()` already exists and is the intercept predicate.
  - Agentic-testing screenshots at `.test-screenshots/iter022-*.png`.
  - Removal of remaining `#[allow(dead_code)]` on consumed items in `src/components/inline_picker.rs`.


  - `cargo test --lib menu_syntax` — 197 passed (unchanged).
  - Full `cargo test --lib` — 12327 passed, same 23 pre-existing failures (12 tab_ai_mode_* + 11 other audits). Zero regressions.
- `cargo build --lib --package script-kit-gpui` clean (same 2 pre-existing warnings).
- `cargo build --bin script-kit-gpui` clean (same 12 pre-existing bin + 2 pre-existing lib warnings).
- `source checks` — all checks passed.
  - Commit A — `e5dca4779` — remove mode chip.
  - Commit B — `e76af941a` — extract `components/inline_popup_window.rs`.
  - Commit C — `cb83f0b0c` — neutral `InlinePickerRow` shape + selection helpers.
  - Commit D1 — `0a6b2a6f7` — pure trigger-popup state machine + row adapter (17 new tests).
  - Commit D2a — `e7c7a2b7a` — wire state machine into live filter events + tracing.
  - Commit D2b — `99bd1bea2` — GPUI popup window + SectionHeader takeover removal.
  - Commit D2c — pending SHA — keyboard dispatch via `InlinePickerKeyIntent` + `apply_intent` at four intercept sites.
  - 4 `cargo test --bin` compile errors in `src/render_prompts/arg/tests.rs` about missing `PromptFooterColors` fields (`selected_alpha`, `text_primary`).
  - `src/ai/acp/history_popup.rs` still carries a local copy of `popup_ns_window` / `attach_popup_to_parent_window` / `set_popup_window_bounds` that predates the `components/inline_popup_window.rs` extraction. Should dedupe to the shared helpers in a follow-up janitorial.
  - The agentic-testing simulateKey path cannot drive the real macOS keystroke pipeline for the Input state — a separate infrastructure task is needed before detached popup keyboard flows can be covered by the automated screenshot harness.
