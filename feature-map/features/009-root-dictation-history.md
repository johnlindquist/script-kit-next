# 009 Root Unified Search Dictation History

This chapter maps saved dictation rows in root launcher search, where transcripts appear only as metadata until explicit user action.

## Executive Summary

Root Dictation History lets users opt into searching or browsing saved local dictation metadata from the main launcher. It is disabled by default because voice transcripts are sensitive. Explicit source heads such as `dictation:` and `d:` opt the source into the current query or browse frame.

This feature owns `SearchResult::DictationHistory`, metadata-only root rows, source-filter entry, passive frame caching, stable selection identity, and Enter-to-paste transcript loading.

It does not own microphone capture, waveform UI, transcription, device setup, the full Dictation History built-in surface, generic root grouping, generic actions-popup hosting, or OS/native paste implementation.

## Human Capabilities

| Capability | User story | Contract |
|---|---|---|
| Opt-in passive search | Enable root Dictation History and type a normal query. | Matching saved dictation metadata appears in passive root sections. |
| Explicit source search | Type `dictation: standup`, `d: standup`, or `d:standup`. | Dictation History is searched directly for stripped text. |
| Explicit source browse | Type `dictation:` or `d:` with no search text. | Recent saved dictation metadata rows appear. |
| Metadata-only root rows | See preview, target, duration, timestamp, type/source, and Paste Dictation action. | Full transcript is not exposed in root rows or receipts. |
| Paste selected transcript | Press Enter on a Dictation row. | Full transcript loads by id and pastes via text injection. |
| Content-light actions | Open root actions for a Dictation row. | Action context stores id and preview only; transcript loads only after explicit action. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| Saved dictation history | Local `dictation-history.jsonl` under the Kit path. | Root search reads compact local history only. |
| Metadata hit | Root search projection of a saved entry. | Contains id, preview, target, timestamp, duration, matched field, subtitle, score. |
| Full transcript | Sensitive transcript body. | Loaded only after Enter or explicit transcript action by id. |
| Ordinary passive config | `unifiedSearch.dictationHistory`. | Disabled by default; defaults include max/min/scan limits. |
| Explicit source head | `dictation:` or `d:`. | Enables Dictation History for this frame and lowers min chars to zero. |
| Passive frame | Frozen result vector keyed by query, advanced state, source filters, and options. | Cache warmers cannot mutate active visible rows. |
| Stable row key | `dictation-history/{id}`. | Selection stable, but launcher command id is `None`. |

## Entry Points

| Entry | Example | Result |
|---|---|---|
| Ordinary enabled query | `standup`. | Passive Dictation History rows may appear if enabled, eligible, and cached. |
| Spaced source query | `dictation: standup`, `d: standup`. | Direct Dictation source search for `standup`. |
| Attached source query | `dictation:standup`, `d:standup`. | Same stripped query behavior. |
| Source-only browse | `dictation:`, `d:`. | Recent saved metadata rows. |
| Row activation | Enter on selected Dictation row. | Load full transcript by id and paste it. |
| Root actions | Cmd+K on selected Dictation row. | Adjacent action set such as paste/copy/attach/create note/delete. |

## State Model

| State | Meaning |
|---|---|
| Root launcher ScriptList | Search text, source filters, grouped rows, selection, and preflight receipts. |
| Ordinary passive dictation | Rows appear only when config enabled, query eligible, and cached hits exist. |
| Source-filter mode | `dictation:` / `d:` makes Dictation History the active source. |
| Source-only browse | Empty stripped text with explicit source returns recent metadata. |
| Advanced predicate mode | Passive Dictation History rows are suppressed. |
| Full Dictation History view | Dedicated built-in surface, not owned here. |
| JSONL history cache | Mtime/signature-backed index cache over saved local history. |
| Activation path | Loads full entry by id, then uses `TextInjector` to paste transcript. |

## Query Eligibility

| Query/config state | Ordinary behavior | Explicit `d:` / `dictation:` behavior |
|---|---|---|
| Default config | No rows; disabled by default. | Explicit source may opt in. |
| Empty ordinary query | No rows. | Browse recent metadata. |
| Short ordinary query | No rows below clamped min chars. | Explicit source can set min chars to zero. |
| Newline query | Ineligible. | Should not leak rows unexpectedly. |
| Predicated advanced query | Passive rows suppressed. | Source-filter-only browse remains separate. |
| Empty history | No rows. | No rows/status; `kit://dictation` may report unavailable/hidden. |
| Cold ordinary cache | No current-frame hits; warms future frame. | Direct source search reads local metadata. |

## User Workflows

### Ordinary Passive Search

The user enables `unifiedSearch.dictationHistory` and types a normal eligible query. Matching saved dictation metadata appears as passive rows under a Dictation History section, capped by passive budget and source max.

The row title is preview text, the subtitle is built from target, duration, and timestamp, and the default action is Paste Dictation. The transcript is not included in the row model, stable key, preflight receipt, action context, logs, or launcher command id.

### Explicit Dictation Source Search

The user types `dictation: standup`, `d: standup`, `dictation:standup`, or `d:standup`. The parser strips the source head, source filters include Dictation, and the direct metadata search path runs with explicit-source options.

Unknown heads and unrelated tokens such as `project:d` must not parse as Dictation source filters.

### Source-Only Browse

The user types `dictation:` or `d:` with no stripped text. This is an explicit browse request, so recent saved dictation metadata rows can appear even though ordinary empty-root dictation rows are ineligible.

### Paste Transcript

The user selects a Dictation History row and presses Enter. The app calls `execute_root_dictation_history_paste(&id)`, loads the full entry through `dictation::get_history_entry(entry_id)`, creates a `TextInjector`, and calls `paste_text(&transcript)`.

If the entry is missing or stale, the app logs that the root dictation history entry was not found and shows a HUD failure such as `Failed to paste dictation`.

### Root Actions

The root actions dialog can expose Paste Dictation, Copy Transcript, Attach to AI, Create Note from Transcript, and Delete Dictation. The subject should store id plus preview only. Any action that needs the full transcript must load it by id only after explicit selection.

## Interaction Matrix

| Interaction | Context | Expected behavior | Proof |
|---|---|---|---|
| Type ordinary enabled query. | Config enabled and query eligible. | Dictation History section appears passively. | Role/source/type/stable-key receipt. |
| Type ordinary query with default config. | Source disabled. | No Dictation History rows. | No `sourceName=Dictation History`. |
| Type `d:standup`. | Explicit source. | Stripped query `standup`; only allowed Dictation source rows/statuses. | Source filter receipt and parser tests. |
| Type `d:`. | Source-only browse. | Recent metadata rows with no transcript loaded. | Empty stripped text, source filter, content-light rows. |
| Type short ordinary query. | Below min chars. | No passive rows. | Eligibility test/receipt. |
| Type predicated query. | Advanced predicate active. | No passive rows. | Grouping/source audit. |
| Wait for cold cache. | Ordinary cache cold. | Current frame stays stable; future frame may show warmed hits. | Fingerprint unchanged. |
| Press Enter. | Selected Dictation row. | Load transcript by id and paste. | Source audit and targeted runtime/native proof. |
| Open actions. | Selected Dictation row. | Action ids visible; context contains id/preview, not transcript. | `actionsDialog` receipt. |

## Visual States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| No Dictation rows. | Normal root launcher results. | Main input/list. | No visible result with source `Dictation History`. |
| Passive Dictation section. | Header `Dictation History`; preview rows with target/duration/time metadata. | Main list row when selected. | Role `rootPassive`, type `Dictation`, source `Dictation History`, key `dictation-history/{id}`. |
| Source-only browse. | Source chip/status plus recent metadata rows or empty state. | Main input/list. | `source_filters=["dictation"]`, `computed_search_text=""`. |
| Explicit source no results. | No-results/status message for source filter. | Main input/list. | Source status row/chip count. |
| Selected Dictation row. | Preview row, Dictation type tag, Paste Dictation action. | Main list selection. | `selectedResultKey`, enter action kind. |
| Actions dialog. | Paste, Copy Transcript, Attach to AI, Create Note, Delete. | Actions popup/list. | Visible action ids only; no transcript. |
| Paste failure HUD. | HUD `Failed to paste dictation`. | Main app/HUD. | HUD or log receipt. |

## Data, Storage, And Privacy Boundaries

- Root Dictation History is disabled by default because transcripts are sensitive.
- Root rows are metadata-only: id, preview, target, timestamp, duration, matched field, subtitle, score.
- Full transcript is not stored in `DictationHistoryMatch`, root hits, preflight receipts, action context, stable keys, launcher command ids, or logs.
- Full transcript loads only after explicit Enter or explicit transcript action by id.
- Root search is local-only and bounded to compacted JSONL history with `scanLimit`.
- Root search logs query length, not raw query text.
- Non-explicit foreground search is cache-only; background warmers must not mutate active frames.
- Config defaults are clamped: disabled by default, max results bounded, min query chars bounded, scan limit bounded.

## Error, Empty, Loading, And Disabled States

| State | Expected behavior |
|---|---|
| Disabled source. | No root Dictation rows. |
| Ordinary empty query. | No rows. |
| Ordinary short query. | No rows below min chars. |
| Ordinary newline query. | No rows. |
| Predicated advanced query. | No passive rows. |
| Source-only `d:`. | Recent metadata rows if history exists. |
| Empty history. | No rows/status, no transcript exposure. |
| Cold passive cache. | Silent warm for future frames; no spinner required. |
| Missing selected id on Enter. | No paste; log and HUD failure. |
| Native paste failure. | Treat as OS/focus-dependent, verify separately. |

## Code Ownership

| Area | Source anchors |
|---|---|
| Dictation history storage | `src/dictation/history.rs#history_path`, `DictationHistoryEntry`, `search_root_dictation_history*` |
| Query eligibility | `src/dictation/history.rs#root_dictation_history_query_is_eligible` |
| Passive grouping | `src/scripts/grouping.rs#append_root_dictation_history_section` |
| Source filters | `src/menu_syntax/source_heads.rs`, `src/menu_syntax/payload.rs`, `src/app_impl/filtering_cache.rs` |
| Passive frame/cache | `RootPassiveFrameKey`, `RootPassiveFrame.dictation_history_hits`, `search_root_dictation_history_cached` |
| Paste path | `src/app_impl/selection_fallback.rs#execute_root_dictation_history_paste`, `dictation::get_history_entry`, `TextInjector::paste_text` |
| Root actions | `src/app_impl/root_unified_result_actions.rs` |
| Config/schema | `UnifiedSearchDictationHistoryConfig`, `src/config/defaults.rs`, `scripts/config-schema.ts` |
| Tests/source audits | `tests/source_audits/root_unified_dictation_history_contract.rs`, `root_unified_source_filters_contract.rs`, `root_unified_config_schema_parity_contract.rs`, `root_unified_passive_snapshot_contract.rs`, `root_unified_source_actions_contract.rs` |

## Invariants And Regression Risks

- Root Dictation History is disabled by default.
- Root rows are metadata-only; transcript text must not appear in rows, receipts, action context, aliases, command ids, or logs.
- Full transcript loads only after explicit Enter or explicit transcript action by id.
- Search is local-only and bounded to compact JSONL history.
- Root search logs query length instead of raw query text.
- Ordinary empty, short, newline, disabled, and predicated advanced queries do not produce passive rows.
- Source-only `d:` / `dictation:` can browse recent metadata.
- Default passive order places Dictation History after Clipboard History and before AI Conversations.
- Rows use shared passive score caps and budgets.
- Stable key is `dictation-history/{id}` and launcher command id is `None`.
- Non-explicit foreground search is cache-only and must not publish active-frame updates.
- Source-filter parser recognizes standalone/attached `dictation:` and `d:` without triggering unrelated hint popups.

## Verification Recipes

### Source And Config Contracts

Run:

```bash
cargo test --test source_audits root_unified_dictation_history_contract -- --nocapture
cargo test --test source_audits root_unified_source_filters_contract -- --nocapture
cargo test --test menu_syntax_source_filters -- --nocapture
cargo test --test source_audits root_unified_config_schema_parity_contract -- --nocapture
cargo test --test source_audits root_unified_passive_snapshot_contract -- --nocapture
cargo test --test source_audits root_unified_search_stability_contract -- --nocapture
cargo test --test source_audits root_unified_source_actions_contract -- --nocapture
```

Check:

- Defaults keep root Dictation disabled.
- Source filters recognize `dictation:` and `d:`.
- Rows stay metadata-only and non-bindable.
- Cache-only foreground search cannot mutate the current frame.
- Actions context is id/preview-only and transcript-free.

### Runtime State Proof

Use a synthetic saved dictation entry in a test Kit path:

- Enable `unifiedSearch.dictationHistory`.
- For a normal query, assert role `rootPassive`, type `Dictation`, source `Dictation History`, stable key `dictation-history/{id}`, action `Paste Dictation`, and no transcript field.
- For `d:`, assert empty stripped text, source filter contains Dictation, recent metadata rows appear, and no transcript appears.
- For `d: unique-token`, assert selected stable key is the synthetic id and Enter action is present.
- For disabled/short/empty/newline/predicated ordinary queries, assert no Dictation rows.
- For cold cache, assert visible fingerprints do not change while the background warmer runs.
- For actions dialog, assert action ids/labels and absence of transcript.

### Paste Proof

Use source/static proof for routine verification:

- `SearchResult::DictationHistory` routes to `execute_root_dictation_history_paste`.
- The path loads by id through `get_history_entry`.
- It pastes through `TextInjector::paste_text`.
- Missing entries show failure and do not paste.

Use native proof only when validating actual OS paste delivery into an external app.

## Agent Notes

- Do not use the dedicated Dictation History browser as proof for root Dictation rows.
- Treat transcript text in launcher state, row text, action context, or logs as a privacy regression.
- Keep `d:` explicit-source behavior separate from ordinary passive defaults.
- Prove action receipts are content-light before testing transcript-loading actions.
- Native paste proof should be targeted and separate from routine source/protocol checks.

## Related Features

- [001 Main Menu](./001-main-menu.md) owns root grouping, selection, source filters, and fallback boundaries.
- [005 Built-in Filterable Surfaces](./005-built-in-filterable-surfaces.md) covers the dedicated built-in list pattern.
- [011 Root Result Actions](./011-root-source-actions.md) owns the broader root action palette for dictation rows.

## Raw Oracle References

- [Prompt](../raw-oracle/009-root-dictation-history/prompt.md)
- [Bundle map](../raw-oracle/009-root-dictation-history/bundle-map.md)
- [Answer](../raw-oracle/009-root-dictation-history/answer.md)
- [Full output log](../raw-oracle/009-root-dictation-history/output.log)
- [Session metadata](../raw-oracle/009-root-dictation-history/session.json)

## Open Questions And Gaps

- The exact keyboard shortcut for opening root actions was not in the raw bundle; confirm through actions/keyboard docs.
- `SourceHeadSpec` for Dictation may still be marked planned; confirm whether discoverability UI labels an implemented source incorrectly.
- Dictation cache-warm status is less observable than browser snapshot sources; decide whether a content-free cache status receipt is needed.
- Non-paste root actions such as Copy Transcript, Attach to AI, Create Note, and Delete need full ownership mapping in the root actions chapter.
