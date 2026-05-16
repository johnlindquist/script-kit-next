# 008 Root Unified Search Clipboard History

This chapter maps Clipboard History rows in root launcher search, where clipboard entries appear as passive metadata results.

## Executive Summary

Root Clipboard History lets users intentionally search or browse recent clipboard metadata from the main launcher. Ordinary passive clipboard rows are opt-in and disabled by default because clipboard history is sensitive. Explicit source filters such as `c:` and `clipboard:` opt Clipboard History into the current query or browse frame.

This feature owns `SearchResult::ClipboardHistory(ClipboardHistoryMatch)`, root clipboard metadata search, source-filter behavior, passive frame isolation, row identity, content-light preflight receipts, and the Enter path that copies the selected entry, hides the launcher, and simulates paste.

It does not own the full Clipboard History built-in surface, clipboard capture/retention/dedupe internals, image previews, OCR rendering, pin/delete UI, Quick Look UI, sequential paste UI, or generic MainList action-dialog infrastructure.

## Human Capabilities

| Capability | User story | Contract |
|---|---|---|
| Opt-in passive search | Enable root Clipboard History and type a non-empty root query. | Matching metadata appears under Clipboard History as passive rows. |
| Explicit source search | Type `c:skip`, `c: skip`, `clipboard:skip`, or `clipboard: skip`. | Clipboard History is searched directly for stripped text even when ordinary passive rows are disabled. |
| Explicit source browse | Type `c:` or `clipboard:` with no stripped text. | Bounded recent eligible clipboard metadata rows appear. |
| Recognize clipboard rows | See Clipboard History section/source and Clipboard type label. | Rows expose content-light preview/title/subtitle only. |
| Paste selected entry | Press Enter on a root clipboard row. | Entry is copied, launcher hides/resets, then Cmd+V is simulated into the previously focused app. |
| Privacy boundary | Root search does not show raw content, image payloads, OCR, or full clipboard previews. | Root rows use metadata only; full payload loads only in activation/action paths. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| Root clipboard row | `SearchResult::ClipboardHistory`. | Passive launcher result with stable key `clipboard-history/{id}`. |
| Metadata search | `search_root_clipboard_history_meta*`. | Uses bounded metadata such as id, content type, timestamp, pinned, text preview, dimensions, byte size, OCR marker fields. |
| Eligible content type | Text, Link, File, Color. | Image entries are excluded from root rows. |
| Ordinary passive config | `unifiedSearch.clipboardHistory`, global unified search, and `builtIns.clipboardHistory`. | Ordinary root clipboard is disabled by default. |
| Explicit source head | `clipboard:` or `c:`. | Enables Clipboard History for this frame and sets explicit-source options. |
| Passive frame | Frozen row vector keyed by query, advanced-query flag, source filters, and clipboard options. | Warm cache completion must not mutate the active visible frame. |
| Paste route | `execute_root_clipboard_history_paste`. | Reuses existing copy helper, hides launcher, then delayed CG paste. |
| Non-bindable identity | Stable key exists, launcher command id is `None`. | Selection is stable but rows cannot become shortcut/alias commands. |

## Entry Points

| Entry | Example | Result |
|---|---|---|
| Ordinary enabled query | `skip`. | Passive Clipboard History section may appear if enabled and eligible. |
| Spaced source query | `c: skip`, `clipboard: skip`. | Source-filtered Clipboard History search for `skip`. |
| Attached source query | `c:skip`, `clipboard:skip`. | Same stripped query behavior. |
| Source-only browse | `c:`, `c: `, `clipboard:`. | Bounded recent eligible metadata rows. |
| Row activation | Enter on selected Clipboard row. | Copy selected entry, hide/reset launcher, simulate paste. |
| Root action popup | Cmd+K on selected Clipboard row. | Adjacent root result actions; paste must delegate to same helper as Enter. |

## State Model

| State | Meaning |
|---|---|
| Main launcher ScriptList | Root search input, grouped rows, selected row, source filters, preflight receipts. |
| Ordinary passive clipboard | Clipboard rows may append only when configured on, query is eligible, and cached hits exist. |
| Source-filter mode | `c:` / `clipboard:` makes Clipboard History an allowed source for stripped text. |
| Source-filter browse | Empty stripped text with explicit source returns recent metadata rows. |
| Advanced predicate mode | Passive Clipboard History rows are suppressed. |
| Full Clipboard History view | Separate dedicated built-in surface, not owned here. |
| Metadata cache | Ordinary foreground path is cache-only; background warmers prepare future frames. |
| Activation path | Uses existing clipboard copy helper and selected-text paste simulation. |

## Query Eligibility

| Query/config state | Ordinary behavior | Explicit `c:` / `clipboard:` behavior |
|---|---|---|
| Default config | No rows; disabled by default. | Explicit source may opt in for this query. |
| Empty root query | No rows. | Browse bounded recent metadata. |
| Query below `min_query_chars` | No rows. | Explicit source can set `min_query_chars=0`. |
| Newline-containing query | Ineligible. | Should remain source-safe and not leak rows unexpectedly. |
| Advanced predicate query | No passive clipboard rows. | Source-filter parsing remains separate from unrelated predicates. |
| Image-only match | No root row. | Still excluded. |
| Cold ordinary cache | No rows in current frame; warms future cache. | Direct source path can read metadata immediately. |
| No source-filter hits | No Clipboard item rows. | Show source-filter no-results/status chrome. |

## User Workflows

### Ordinary Passive Search

The user enables the ordinary root Clipboard History source and types a non-empty eligible query. Matching recent clipboard metadata appears in the Clipboard History passive section according to passive order and budget.

Rows show a preview title from metadata, a Clipboard type label, source name `Clipboard History`, and a subtitle with pinned marker, content type, and relative time where available. They do not include raw clipboard content.

### Explicit Clipboard Source Search

The user types `c:skip`, `c: skip`, `clipboard:skip`, or `clipboard: skip`. The source head is stripped, search text becomes `skip`, Clipboard History is enabled for the current frame, and other sources are suppressed unless separately included.

Quoted or unknown heads remain literal. Multiple source heads are additive. Exclusion wins if a source is both included and excluded.

### Source-Only Browse

The user types `c:` or `clipboard:` without stripped text. This intentionally browses bounded recent eligible clipboard metadata. It does not enable ordinary empty-root clipboard recents outside source-filter mode.

### Paste Selected Clipboard Row

The user selects a root Clipboard History row and presses Enter. The root path routes to `execute_root_clipboard_history_paste(&entry.id, cx)`, copies the entry through `copy_entry_to_clipboard(entry_id)`, hides/resets the main launcher, waits briefly, and calls `selected_text::simulate_paste_with_cg()`.

State-first proof can prove row selection, route, and hide/reset. Actual insertion into another app is macOS focus/permission dependent and needs targeted native proof.

## Interaction Matrix

| Interaction | Context | Expected behavior | Proof |
|---|---|---|---|
| Type ordinary eligible query. | Clipboard source enabled. | Clipboard History section appears passively. | Preflight row role/source/type/stable key. |
| Type ordinary query with default config. | Clipboard source disabled. | No Clipboard History rows. | No visible result with source `Clipboard History`. |
| Type `c:skip`. | Explicit source. | `computed_search_text="skip"`, Clipboard source active. | Source filter receipt. |
| Type `c:`. | Explicit empty source. | Browse recent eligible metadata. | Empty stripped text and Clipboard rows/status. |
| Type short `s`. | Ordinary root. | No clipboard rows if below min length. | Eligibility test/receipt. |
| Type short `c:s`. | Explicit source. | Search may run with source min length zero. | Source-filter options receipt/source audit. |
| Type advanced predicate. | Ordinary root. | No passive clipboard rows. | Grouping/source-audit test. |
| Wait for cold cache. | Ordinary cache empty. | Current frame stays stable; future frames may show warmed hits. | Visible fingerprint unchanged. |
| Press Enter. | Selected root clipboard row. | Copy, hide/reset, delayed paste simulation. | Source audit plus state/native proof as needed. |
| Cmd+K. | Selected root clipboard row. | Adjacent root actions; paste delegates to same helper. | Root result actions receipts. |

## Visual States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| Ordinary enabled matches. | `Clipboard History` section, Clipboard rows with preview metadata. | Main launcher input/list. | `visibleResults[].role=rootPassive`, `typeLabel=Clipboard`, `sourceName=Clipboard History`, key `clipboard-history/{id}`. |
| Clipboard source filter. | Clipboard-only rows plus source-filter/status chrome. | Root input/list. | `source_filters`, filter indicators, cached source statuses. |
| Source-filter browse. | Recent eligible metadata rows with empty stripped text. | Root input/list. | `computed_search_text=""`, source includes Clipboard History. |
| No source-filter matches. | No Clipboard item rows; source-filter no-results/status state. | Root input/list. | Status row with shown/loaded counts; disallowed sources absent. |
| Selected clipboard row. | Highlighted row, action text `Paste Clipboard`. | MainList selected item. | `selectedResultKey`, `selectedResultRole=rootPassive`, enter action kind. |
| After Enter success. | Launcher hidden/reset; target app receives paste if OS allows. | Previously frontmost app. | Protocol can prove hide/reset; native proof proves actual insertion. |

## Data, Storage, And Privacy Boundaries

- Root search uses clipboard metadata only: bounded table/cache reads and `text_preview`.
- Root grouping must not call `get_entry_content()` during search.
- Eligible root rows are Text, Link, File, and Color. Images/OCR/raw payloads are excluded.
- Ordinary passive Clipboard History is disabled by default.
- Explicit `c:` / `clipboard:` is treated as intentional user selection and may enable the source for that frame.
- Activation can load/copy the actual clipboard item because the user selected an explicit row action.
- Stable keys and preflight receipts must not include raw clipboard content.
- Cold cache warming must not repaint the active frame and move selection.

## Error, Empty, Loading, And Disabled States

| State | Expected behavior |
|---|---|
| Default ordinary config. | No passive clipboard rows. |
| Ordinary empty root. | No clipboard recents. |
| Ordinary short query. | No rows below threshold. |
| Newline query. | Ineligible. |
| Advanced predicate. | Passive clipboard rows suppressed. |
| Explicit source no hits. | Source-filter status/no-results state. |
| Cold ordinary cache. | Current frame empty/stable; background refresh warms future frames. |
| Copy failure. | Should not simulate paste or paste stale clipboard content. |
| Simulated paste failure. | Logged after copy/hide path; native focus/permission dependent. |
| Image metadata reaching grouping. | Should be excluded before visible root row; defensive filtering is an open question. |

## Code Ownership

| Area | Source anchors |
|---|---|
| Metadata search | `src/clipboard_history/database.rs#search_root_clipboard_history_meta` |
| Metadata cache | `src/clipboard_history/cache.rs#search_root_clipboard_history_meta_cached` |
| Root options | `src/clipboard_history/types.rs#RootClipboardHistorySectionOptions` |
| Eligibility | `src/clipboard_history/types.rs#root_clipboard_entry_is_eligible` |
| Grouping | `src/scripts/grouping.rs#append_root_clipboard_history_section` |
| Source filters | `src/menu_syntax/source_heads.rs`, `src/menu_syntax/payload.rs`, `src/app_impl/filtering_cache.rs` |
| Paste path | `src/app_impl/selection_fallback.rs#execute_root_clipboard_history_paste`, `src/clipboard_history/clipboard.rs#copy_entry_to_clipboard`, `src/selected_text.rs#simulate_paste_with_cg` |
| Result identity | `src/scripts/types.rs#ClipboardHistoryMatch`, `SearchResult::ClipboardHistory` |
| Config/schema | `UnifiedSearchClipboardHistoryConfig`, `src/config/defaults.rs`, `scripts/config-schema.ts` |
| Tests/source audits | `tests/source_audits/root_unified_clipboard_history_contract.rs`, `root_unified_source_filters_contract.rs`, `root_unified_passive_snapshot_contract.rs`, `tests/menu_syntax_source_filters.rs` |

## Invariants And Regression Risks

- Root clipboard search is metadata-only and bounded.
- Ordinary passive Clipboard History is opt-in and disabled by default.
- `clipboard:` / `c:` explicitly opts Clipboard History into the current query or browse frame.
- Source heads support spaced and attached query text.
- Source-only `c:` browses bounded recent metadata; ordinary empty root does not.
- Images/OCR/raw content are excluded from root Clipboard History rows.
- Passive grouping preserves Files, fallback, and primary result boundaries.
- Row identity is stable but non-bindable.
- Enter reuses the existing clipboard copy plus simulated paste contract.
- Foreground ordinary passive grouping is cache-only.
- Source-filter active state suppresses disallowed sources and exposes content-light status.

## Verification Recipes

### Source And Config Contracts

Run:

```bash
cargo test --test source_audits root_unified_clipboard_history_contract -- --nocapture
cargo test --test source_audits root_unified_source_filters_contract -- --nocapture
cargo test --test source_audits root_unified_config_schema_parity_contract -- --nocapture
cargo test --test source_audits root_unified_passive_snapshot_contract -- --nocapture
cargo test --test source_audits root_unified_search_stability_contract -- --nocapture
cargo test --test menu_syntax_source_filters -- --nocapture
```

Check:

- Ordinary clipboard is opt-in and scoped.
- Metadata search is bounded and excludes raw content/images.
- Source filters strip text, isolate frames, and allow explicit browse.
- Cache warmers cannot mutate the active frame.

### Runtime State Proof

Use state-first proof:

- For `c:skip`, assert `computed_search_text=="skip"` and source filter includes Clipboard History.
- For `c:`, assert empty stripped text and root passive Clipboard rows/status only.
- For ordinary default config, assert no source `Clipboard History`.
- For ordinary enabled query, assert role `RootPassive`, type label `Clipboard`, source `Clipboard History`, key `clipboard-history/{id}`.
- Assert no raw clipboard content fields appear in receipts.
- Assert visible row fingerprints stay stable across cold cache settlement.

### Enter Paste Proof

Use a two-tier proof:

- State/static tier: prove selected root clipboard row, Enter action route, reuse of paste helper, and launcher hide/reset.
- Native tier only when needed: use a controlled paste target to prove clipboard write, focus handoff, 100 ms delay, and CG paste insertion.

## Agent Notes

- Do not use the dedicated Clipboard History view as proof for root Clipboard History rows.
- Treat any raw clipboard payload in launcher/preflight/elements as a privacy regression.
- Treat ordinary empty-root clipboard recents as a regression unless source-filter mode is active.
- Treat cache-warmed row movement in the same query frame as a selection stability regression.
- Keep root clipboard action catalog questions in [011 Root Result Actions](./011-root-source-actions.md) unless the action is Enter paste.

## Related Features

- [001 Main Menu](./001-main-menu.md) owns root grouping, selection, source filters, and fallback boundaries.
- [005 Built-in Filterable Surfaces](./005-built-in-filterable-surfaces.md) owns the full dedicated Clipboard History browser.
- [011 Root Result Actions](./011-root-source-actions.md) owns root action palettes beyond default Enter paste.

## Raw Oracle References

- [Prompt](../raw-oracle/008-root-clipboard-history/prompt.md)
- [Bundle map](../raw-oracle/008-root-clipboard-history/bundle-map.md)
- [Answer](../raw-oracle/008-root-clipboard-history/answer.md)
- [Full output log](../raw-oracle/008-root-clipboard-history/output.log)
- [Session metadata](../raw-oracle/008-root-clipboard-history/session.json)

## Open Questions And Gaps

- Root Clipboard action ownership needs reconciliation: some excerpts list attach, pin/unpin, Quick Look, and delete, while first-pass docs scoped root rows primarily to paste.
- Paste failure UX needs exact source confirmation so copy failure cannot hide and paste stale clipboard contents.
- The relationship among `unifiedSearch.enabled`, `builtIns.clipboardHistory`, ordinary passive defaults, and explicit source heads should be documented as a privacy decision.
- Producer-side image exclusion is pinned, but grouping may need defensive eligibility filtering if future callers pass image metadata.
- Native paste proof is OS/focus dependent and should stay separate from routine source/protocol regression checks.
