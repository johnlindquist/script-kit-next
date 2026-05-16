# 007 Root Unified Search Notes

This chapter maps Notes rows in the root launcher search, where notes appear as passive metadata results instead of primary commands.

## Executive Summary

Root Notes lets users find local notes from the main launcher without entering the floating Notes window first. It appends metadata-only Note rows to ordinary eligible root queries and supports explicit source filters such as `notes:` and `n:` for Notes-only search or browse.

This feature owns `SearchResult::Note` root rows, Notes passive grouping, Notes source heads, root note metadata search, cache/frame isolation, stable row identity, and the root Enter path that opens or focuses the floating Notes window with the selected note.

It does not own Markdown editing, Notes Browse, notes-hosted ACP, trash/CRUD, Notes window visual internals, or generic root result actions beyond the primary Open Note behavior.

## Human Capabilities

| Capability | User story | Contract |
|---|---|---|
| Passive Notes search | Type an ordinary non-empty query and see matching notes below primary launcher intent. | Notes never outrank commands, scripts, apps, skills, windows, actions, or root Files. |
| Notes-only search | Type `notes: meeting`, `n: meeting`, or `n:meeting`. | Query text is stripped and only Notes rows/statuses participate. |
| Source-only browse | Type `n:` or `notes:` with no stripped text. | Shows pinned and recently updated active notes. |
| Metadata rows | See note title, pinned/update metadata, character count, source/type, and primary action. | Rows do not expose note bodies. |
| Open selected note | Press Enter on a Note row. | Opens/focuses floating Notes host and selects the note. |
| Existing window reuse | Enter a root Note while Notes is already open. | Reuses/focuses existing Notes window instead of toggling it closed. |
| Content-backed search | Find by title and optionally content when config enables content search. | Search may use content; row payload remains metadata-only. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| Root passive row | Launcher result appended after primary intent. | Classified as `RootPassive` in preflight/state receipts. |
| `RootNoteSearchHit` | Metadata-only note hit. | Carries id, title, updated time, pinned state, character count, score; not full content. |
| Source head | `notes:` or `n:` token in launcher input. | Activates Notes source filtering and strips source syntax from free text. |
| Source-only browse | Notes source filter with empty stripped query. | Returns pinned/recent active notes. |
| Passive frame | Frozen per-query passive source result vector. | Includes source filters and Notes options so cached hits cannot bleed across frames. |
| Non-toggle open | `open_note_in_notes_window(cx, note_id)`. | Opens/focuses/selects without closing an already-open Notes window. |
| Stable row key | `note/{id}`. | Stable for selection/history state, but not a bindable launcher command id. |

## Entry Points

| Entry | Example | Result |
|---|---|---|
| Ordinary root query | `meeting`. | Eligible notes append under Notes section if cache/frame has hits. |
| Spaced source query | `n: meeting`, `notes: meeting`. | Notes-only query for `meeting`. |
| Attached source query | `n:meeting`, `notes:meeting`. | Notes-only query for attached text. |
| Source-only browse | `n:`, `n: `, `notes:`. | Pinned/recent active notes. |
| Row activation | Enter on selected Note row. | Opens/focuses Notes window and selects note. |
| Selection/click | Move to or click a Note row. | Selected row has Open Note primary action and stable key. |

## State Model

| State | Owner | Meaning |
|---|---|---|
| Root launcher ScriptList | Main launcher grouping. | Holds search text, grouped rows, selection, source filters, preflight receipts. |
| Root passive Notes section | `append_root_notes_section`. | Appends metadata rows when query is eligible or source filter explicitly selects Notes. |
| Source-filter active mode | `RootUnifiedSourceFilterSet`. | Suppresses disallowed sources and primary/fallback rows according to source-filter rules. |
| Source-only Notes browse | Notes options with empty stripped query and explicit source. | Uses pinned/recent active metadata with larger explicit-source caps. |
| Floating Notes window | Notes subsystem. | Opens/focuses/selects a note through `open_note_in_notes_window`. |
| Existing Notes window | Notes subsystem. | Reused rather than toggled closed. |
| Notes storage | SQLite/FTS. | Active notes only; deleted notes filtered by `deleted_at IS NULL`. |
| Root passive frame | App state cache. | Freezes `note_hits` and source filters for stable grouping. |

## Query Eligibility

| Query state | Ordinary root behavior | Explicit `n:` / `notes:` behavior |
|---|---|---|
| Empty query | No Notes rows. | Browse pinned/recent active notes. |
| Query below `minQueryChars` | No Notes rows. | Source-specific floor allows explicit source behavior according to config. |
| Multiline query | No Notes rows. | Should stay ineligible unless parser explicitly allows source text. |
| Advanced predicate query | No Notes passive rows. | Source-head parsing remains separate from generic predicate syntax. |
| Disabled passive default | Hidden. | Positive Notes source head opts Notes back in for that query. |
| Eligible non-empty query | Metadata hits append passively. | Notes-only hits/statuses. |
| No hits | No ordinary section. | Source-filter no-results/status surface should be state-verifiable. |

## User Workflows

### Ordinary Passive Search

The user types a normal root query such as `meeting`. If Notes is enabled, the query meets eligibility, and the passive frame has note hits, the launcher appends a Notes section below primary rows and root Files/Browser Tabs according to configured passive order.

The row shows title or `Untitled Note`, pinned/update/character metadata, type/source as Note/Notes, stable key `note/{id}`, and default action `Open Note`. The note body is never included in row text, state, or element receipts.

### Explicit Notes Search

The user types `notes: meeting`, `n: meeting`, or `n:meeting`. The parser strips the source head, computes search text `meeting`, activates the Notes source filter, suppresses other sources, and uses the direct Notes metadata search path.

The attached query form matters: `n:not` should behave like stripped text `not`, including the bounded LIKE fallback when FTS returns no hits.

### Source-Only Browse

The user types `n:` or `notes:` without stripped text. This is an explicit browse request, not an ordinary empty root query. Notes returns pinned and most recently updated active notes, still without body content.

### Open A Note

The user selects a Note row and presses Enter. The root path calls `execute_root_note_open`, which calls `open_note_in_notes_window(cx, note_id)`. On success, the floating Notes window opens or focuses, the note becomes selected, and the launcher hides/resets.

The path must not call the toggle helper `open_notes_window`, because that helper can close an already-open Notes window.

### Stale Or Deleted Note

If a row becomes stale before activation, `open_note_in_notes_window` returns an error. The launcher should remain visible and report a HUD failure such as `Failed to open note`.

## Interaction Matrix

| Interaction | Context | Expected behavior | Proof |
|---|---|---|---|
| Type ordinary eligible query. | Root launcher, Notes enabled. | Notes section appends passively below primary/root-file intent. | Preflight role `RootPassive`, source `Notes`. |
| Type `n: meeting`. | Explicit source filter. | `computed_search_text="meeting"`, only Notes rows/statuses. | Source filters include Notes. |
| Type `n:meeting`. | Attached source query. | Query strips to `meeting`. | Source-filter parser tests. |
| Type `n:`. | Explicit empty Notes source. | Browse pinned/recent active notes. | Note rows with empty stripped text. |
| Select Note row. | Root results include Notes. | Primary action is Open Note; key is `note/{id}`. | Selected result receipt. |
| Press Enter on Note row. | Note exists. | Notes window opens/focuses and selects note; launcher hides. | Notes target state and main hidden state. |
| Press Enter with Notes already open. | Existing Notes host. | Same host remains open and selected note changes. | Before/after Notes window identity. |
| Press Enter on stale row. | Note deleted after result built. | HUD failure and launcher remains. | HUD/state receipt. |
| Switch source filters. | Same free text across Notes/Clipboard/Files. | Notes hits never bleed between frames. | `RootPassiveFrameKey.source_filters`. |

## Visual States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| Ordinary Notes section. | Header `Notes`, Note rows with title/subtitle metadata. | Launcher search/list. | Row role `RootPassive`, source `Notes`, stable key `note/{id}`. |
| Explicit Notes source. | Notes-only rows or source-status/no-results state. | Launcher search/list. | `source_filters=["Notes"]`, stripped query. |
| Source-only browse. | Pinned/recent active note rows. | Launcher search/list. | Empty stripped query plus Notes rows. |
| Selected Note row. | Highlighted metadata row with Open Note action. | Selected root list item. | Selected key and non-bindable command id. |
| Notes window after Enter. | Floating Notes host with selected note. | Notes window/editor. | Notes open-state/selected note receipt. |
| Open failure. | Launcher remains, HUD failure. | Launcher. | HUD text and unchanged launcher visibility. |
| Ineligible ordinary query. | No Notes section. | Launcher. | No `SearchResult::Note`; optional explicit status only in source mode. |

## Data, Storage, And Privacy Boundaries

- Root Notes rows are metadata-only even when search uses note content.
- Deleted notes are excluded with active-only storage filters.
- Ordinary root Notes search uses cache-only foreground paths so SQLite/FTS work does not block typing.
- Cold ordinary caches may show no Notes rows until a future frame is warmed.
- Explicit `n:` / `notes:` search uses a direct path because the user explicitly selected Notes.
- `ROOT_NOTES_SEARCH_CACHE_GENERATION` invalidates the root Notes cache on save, delete, or prune.
- Config values for `unifiedSearch.notes.enabled`, `maxResults`, `minQueryChars`, and `searchContent` clamp through Rust/schema/default parity.

## Error, Empty, Loading, And Disabled States

| State | Expected behavior |
|---|---|
| Ordinary empty query. | No Notes section. |
| Ordinary too-short query. | No Notes section. |
| Ordinary multiline query. | No Notes section. |
| Advanced predicate query. | No Notes passive rows. |
| Disabled Notes passive config. | Hidden during ordinary search. |
| Explicit Notes source while disabled by default. | User intent opts Notes into this query. |
| Source-filter no hits. | Source-filter no-results/status surface, not unrelated rows. |
| Cold ordinary cache. | No blocking spinner implied; future warmed frame may show rows. |
| Missing/deleted note on Enter. | HUD failure and launcher remains visible. |
| Bare malformed source heads. | Must not accidentally activate Notes or the menu-syntax hint. |

## Code Ownership

| Area | Source anchors |
|---|---|
| Notes metadata search | `src/notes/storage.rs#RootNotesSectionOptions`, `RootNoteSearchHit`, `root_notes_query_is_eligible`, `search_root_notes_meta*` |
| Root grouping | `src/scripts/grouping.rs#append_root_notes_section` |
| Selection/open path | `src/app_impl/selection_fallback.rs#execute_root_note_open` |
| Notes non-toggle open | `src/notes/window/window_ops.rs#open_note_in_notes_window` |
| Passive frame keys | `src/main_sections/app_state.rs#RootPassiveFrameKey`, `src/app_impl/filtering_cache.rs#root_passive_frame_for_current_query` |
| Result identity | `src/scripts/types.rs#SearchResult#stable_selection_key`, `history_result_key`, `launcher_command_id` |
| Config/schema/defaults | `UnifiedSearchNotesConfig`, `scripts/config-schema.ts`, `src/config/defaults.rs` |
| Tests/source audits | `tests/source_audits/root_unified_notes_contract.rs`, `root_unified_source_filters_contract.rs`, `root_unified_config_schema_parity_contract.rs`, `root_unified_passive_snapshot_contract.rs` |

## Invariants And Regression Risks

- Root Notes rows must stay metadata-only; note bodies cannot leak into launcher rows, state, or elements.
- Root Notes search must be active-only; soft-deleted notes cannot resurface.
- Ordinary eligibility excludes empty, short, newline, disabled, and advanced predicate queries.
- Explicit `n:` / `notes:` source heads opt into Notes and support source-only browse.
- Notes rows stay passive and never promote above primary launcher intent.
- Passive source order is configurable, but passive rows cannot move ahead of primary rows or root Files.
- Stable key `note/{id}` must not imply shortcut/alias bindability.
- Enter must use `open_note_in_notes_window`, never toggle-style `open_notes_window`.
- Source filters must be part of passive frame keys to prevent cached row bleed.
- Foreground ordinary passive search must stay cache-only.
- Empty FTS hits must fall back to bounded LIKE search so short attached source queries remain useful.
- Config, schema, defaults, and verification docs must stay in parity.

## Verification Recipes

### Source And Config Contracts

Run:

```bash
cargo test --test source_audits root_unified_notes_contract -- --nocapture
cargo test --test source_audits root_unified_source_filters_contract -- --nocapture
cargo test --test source_audits root_unified_config_schema_parity_contract -- --nocapture
cargo test --test source_audits root_unified_passive_snapshot_contract -- --nocapture
cargo test --test menu_syntax_source_filters -- --nocapture
```

Check:

- Notes rows are metadata-only, bounded, active-only, and passive.
- `n:` and `notes:` strip text and isolate source frames.
- Config values clamp and match schema/defaults.

### Storage Unit Coverage

Run the narrow storage tests for:

- `test_root_notes_query_eligibility_respects_config`
- `test_search_root_notes_meta_is_bounded_active_only_and_metadata_only`
- `test_search_root_notes_meta_matches_title_substrings_when_fts_has_no_hit`

Check:

- Query eligibility matches config.
- Deleted notes are filtered out.
- LIKE fallback works when FTS misses.

### Runtime State Proof

Use state-first automation:

- For `n: meeting`, assert stripped query `meeting`, `source_filters` includes Notes, and visible rows are Notes/source-status only.
- For ordinary eligible query, assert a Note row has role `RootPassive`, source `Notes`, type `Note`, key `note/{id}`, action `Open Note`, and no body content.
- For `n:`, assert empty stripped text and pinned/recent metadata rows.
- For empty/short/newline/disabled/advanced ordinary queries, assert no `SearchResult::Note`.
- For Enter on a Note row, assert Notes host opens/focuses with selected note and launcher hides.
- For existing Notes host, assert Enter does not close the host.
- For stale note failure, assert HUD failure and launcher remains visible.

## Agent Notes

- Do not prove root Notes by opening the dedicated Notes Browse surface; this feature is about ScriptList root rows.
- Keep ordinary passive search and explicit source-filter search separate in receipts.
- Treat note body text in launcher row/state as a privacy regression.
- Treat a toggle-close of an already-open Notes window as a root-open-path regression.
- If ordinary Notes hits do not appear on a cold first frame, verify cache warming before calling it a search failure.

## Related Features

- [001 Main Menu](./001-main-menu.md) owns root grouping, source filters, selection, and menu-syntax behavior.
- [006 Notes Window](./006-notes-window.md) owns the floating Notes host after the selected note is opened.
- [011 Root Result Actions](./011-root-source-actions.md) owns non-Enter action palettes for root passive rows.

## Raw Oracle References

- [Prompt](../raw-oracle/007-root-notes/prompt.md)
- [Bundle map](../raw-oracle/007-root-notes/bundle-map.md)
- [Answer](../raw-oracle/007-root-notes/answer.md)
- [Full output log](../raw-oracle/007-root-notes/output.log)
- [Session metadata](../raw-oracle/007-root-notes/session.json)

## Open Questions And Gaps

- `src/main_sections/deeplink.rs` should be audited for note-id routing and the same non-toggle `open_note_in_notes_window` behavior.
- Bare `n:` / `notes:` without trailing space should be explicitly pinned if parser support exists; the raw pass clearly pinned `n: `.
- Cold ordinary cache behavior needs runtime proof for cold to warm to future-frame visibility.
- The exact automation field for selected note id after the Notes window opens should be confirmed so screenshots are not needed.
- Root Note action-subject rows beyond Enter belong to the root result actions chapter and need exact action catalog mapping there.
- Stable selection identity vs query memory/frecency behavior should be documented if agents keep confusing `note/{id}` with bindable command identity.
