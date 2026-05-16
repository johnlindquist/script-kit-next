# 012 Root Unified Source Filters / Source Chips / Lazy Paging

This chapter maps the launcher source-filter grammar, source-chip state rows, source-only browse flows, and Files lazy paging behavior.

Raw Oracle reference: [answer](../raw-oracle/012-root-source-filters/answer.md), [prompt](../raw-oracle/012-root-source-filters/prompt.md), [bundle map](../raw-oracle/012-root-source-filters/bundle-map.md), [full log](../raw-oracle/012-root-source-filters/output.log), [session metadata](../raw-oracle/012-root-source-filters/session.json).

## Executive Summary

Root Unified Source Filters let users scope ordinary launcher search by typing source heads such as `files:`, `f:`, `clipboard:`, `c:`, `notes:`, `n:`, `tabs:`, `t:`, `history:`, `h:`, `apps:`, `a:`, `scripts:`, `s:`, `commands:`, `cmd:`, `conversations:`, `ai:`, `vault:`, `v:`, `dictation:`, `d:`, and `windows:`. The parser strips committed source tokens from the query, records include/exclude filters, and keeps the launcher in `ScriptList`.

The feature is a routing contract, not just text decoration. A positive source head explicitly opts that source into the current stripped query even when the source is disabled for ordinary passive root search. Source-filter mode suppresses unrelated sources and fallback rows, exposes non-selectable source status rows, blocks launcher input-history recall, and keys async frames by both stripped query and source-filter set.

Files have extra source-chip behavior. Explicit `files:` / `f:` filters can browse Recent Files with empty stripped text, can run one-character ASCII alphanumeric queries such as `f:s`, and can expand the visible source-chip page when keyboard selection nears the bottom without requiring Enter or snapping the list.

## What Users Can Do

| Capability | Example | Result |
|---|---|---|
| Search a source with an attached query. | `c:skip`, `files:s`, `h:https://example.com` | The source head is removed from visible search text and the query is routed to that source. |
| Search a source with spaced syntax. | `c: skip`, `files: png` | Equivalent to the attached form after parsing. |
| Put a standalone source token anywhere. | `png files:`, `invoice n:` | The standalone token scopes the query while the remaining words rank rows. |
| Browse a source with no stripped query. | `f:`, `n:`, `c:`, `t:`, `h:`, `ai:`, `d:` | Shows that source's default browse rows where implemented. |
| Combine sources. | `n: c: invoice` | Allows rows from both included sources and suppresses unrelated rows. |
| Exclude a source. | `-files: png` | Allows every source except Files and removes fallback rows. |
| Resolve include/exclude conflict. | `files: -files: png` | Exclusion wins for Files. |
| Filter primary rows. | `a: terminal`, `s: deploy`, `cmd: settings`, `w: chrome` | Shows primary source rows that match the requested source. |
| Discover sources. | Leading `:` | Opens source/filter discovery behavior; it is not committed source syntax. |
| Keep literal text. | `"files:"`, `unknown: query`, currently `processes:` | Quoted, unknown, and uncommitted heads remain search text. |

## Source Head Matrix

| Source | Heads | Behavior | Notes |
|---|---|---|---|
| Files | `files:`, `f:` | Root Files rows plus Recent Files browse; explicit source filters enable Files even if passive/global search is disabled. | Owns one-character search and lazy source-chip paging. |
| Notes | `notes:`, `n:` | Metadata-only Notes rows and pinned/recent browse. | Opening a row routes to Notes without toggling the existing note window closed. |
| Clipboard History | `clipboard:`, `c:` | Metadata-only clipboard rows and recent browse. | Ordinary passive root clipboard is opt-in; explicit source filters enable direct lookup. |
| Browser Tabs | `tabs:`, `t:` | Open-tab metadata rows and browse where provider data exists. | No page content or cookies are exposed. |
| Browser History | `history:`, `h:` | Local history title/URL/visit metadata. | Browse and search depend on available local snapshot data. |
| Apps | `apps:`, `a:` | Primary app rows via root unified source filtering. | No dedicated default browse semantics beyond matching app rows are proven here. |
| Scripts | `scripts:`, `s:` | Primary script rows via root unified source filtering. | Preserves existing script execution/action ownership. |
| Commands | `commands:`, `cmd:` | Config-backed command rows via root unified source filtering. | Preserves command namespace and shortcut/action behavior. |
| AI Conversations | `conversations:`, `ai:` | Saved conversation metadata and resume rows. | Uses local history metadata, not full transcript content in receipts. |
| AI Vault | `vault:`, `v:` | Head/route exists; grouping currently emits disabled status. | Searchable rows are not proven in this pass. |
| Dictation History | `dictation:`, `d:` | Dictation metadata rows and recent browse. | Transcript content loads only after explicit selection/action. |
| Windows | `windows:`, `w:` | Window metadata rows. | Proof can use test providers to avoid accessibility dependence. |
| Processes | `processes:`, `p:` | Descriptor exists, but parser/tests treat it as uncommitted until rows exist. | Agents should not document it as a committed source filter yet. |

## Core Concepts

| Concept | Meaning | Contract |
|---|---|---|
| Source head | A known source token ending in `:`. | Standalone known heads commit a source filter; leading `:` is discovery instead. |
| Stripped query | User input after committed source tokens are removed. | Exposed as `computedSearchText` / `free_text` and used for ranking. |
| Include filter | Positive source head such as `c:`. | Explicitly enables the source for the active query. |
| Exclude filter | Negative source head such as `-files:`. | Exclusion wins over inclusion and suppresses matching rows. |
| Source-only browse | Empty stripped text plus positive include and no advanced predicate. | Produces default browse rows where implemented. |
| Source-filter frame key | Async/cache identity including stripped query and source filters. | Prevents stale provider results from bleeding across modes. |
| Source status row | Metadata row for loading/empty/disabled/exhausted state. | Visible to automation as status, but not selectable or executable. |
| Source chip page | Initial Files filtered page budget. | Expands near bottom selection without Enter. |
| Menu syntax hint boundary | Source filters are menu syntax, but source heads must not open unrelated power hints. | Completed heads suppress the unrelated hint path. |

## Entry Points

| Entry | User action | Result |
|---|---|---|
| Main launcher filter | Type a query containing committed source heads. | `ScriptList` remains active with scoped root rows. |
| Source discovery | Type leading `:`. | Opens/refines discovery without committing a source filter. |
| Source-only browse | Type only a positive source head, with optional whitespace. | Shows source browse/default rows and status. |
| Agentic automation | `setFilter`, `simulateKey`, `waitFor`, `getState`, `getElements`, `batch`. | Receipts expose stripped text, filters, status rows, selected row, and scroll state. |
| Cmd+K on filtered row | Open actions for selected root result. | Handed off to root source actions from feature 011. |

## User Workflows

### Clipboard Search

The user types `c:skip` or `clipboard: skip`. The parser records a Clipboard include filter and strips the search text to `skip`. The launcher performs a direct metadata lookup for Clipboard History even if ordinary passive clipboard search is disabled. Visible rows are Clipboard rows plus non-selectable status rows; unrelated fallback rows are suppressed.

### Files One-character Search

The user types `f:s`. Because Files was explicitly requested, the one-character stripped query `s` is valid for Files. A plain `s` remains below the ordinary global file threshold. This keeps broad passive file search conservative while making explicit file intent fast.

### Source-only Browse

The user types `n:`, `f:`, `c:`, `t:`, `h:`, `ai:`, or `d:` without stripped search text. The parser creates a positive include filter and leaves `computedSearchText` empty. The source browse path returns default rows where implemented, such as Recent Files, pinned/recent Notes, recent Clipboard metadata, current tabs, history metadata, saved conversations, or dictation metadata.

### Files Lazy Paging

The user types `f:` or `f:s`, then navigates downward. Files starts with a source-chip page budget. When selection approaches the last few Files rows, `maybe_expand_root_file_source_chip_page` expands the page, updates the list count, preserves selection by stable key, and schedules a footer-safe reveal. It must not require Enter, move the selected row under the footer, or snap the list to the top.

### Combined And Negative Filters

The user types `n: c: invoice` to search Notes and Clipboard together. The allowed source set is the union of positive filters. If the user types `files: -files: png`, the negative filter wins and Files rows are excluded. A negative-only query such as `-files: png` allows all other sources but still activates source-filter mode, which suppresses fallback rows.

### Primary Source Filters

The user types `a: terminal`, `s: deploy`, `cmd: settings`, or `w: chrome`. The root unified primary sources are filtered to matching Apps, Scripts, Commands, or Windows rows. Existing execution/action ownership remains with the underlying source or the root action feature.

### Discovery And Literal Text

The user types leading `:` to discover or refine source filters. This path is distinct from committed source syntax. Quoted tokens such as `"files:"`, unknown heads, and currently uncommitted `processes:` / `p:` remain literal query text.

### Input History Boundary

While source-filter mode is active, Up/Down are row-navigation keys. Launcher input-history recall is blocked so `c:`, `f:`, and related filters do not unexpectedly rewrite the filter text.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Search clipboard metadata. | `c:skip` | `ScriptList`, source-filter mode. | Type. | `parse_filter_query`, passive clipboard lookup, grouping filter. | Clipboard rows and status only. | `root-source-filter-clipboard.ts`, `getState.sourceFilters`. |
| Browse recent clipboard. | `c:` | Empty stripped text. | Type. | Source browse config and clipboard provider. | Recent clipboard metadata rows. | `root-source-filter-matrix.ts`. |
| Search Files with one char. | `f:s` | Files source chip. | Type. | File explicit intent lowers threshold. | Files rows for `s`. | `root-source-filter-matrix.ts --query s`. |
| Browse Recent Files. | `f:` | Files browse. | Type. | Root file browse target, source-chip page. | Initial Recent Files rows. | `source-chip-pagination-proof.ts`. |
| Expand Files page. | `f:` or `f:s` | Selection near bottom. | Down. | `maybe_expand_root_file_source_chip_page`, selection restore. | More Files rows without Enter. | `root-source-filter-lazy-scroll.ts`. |
| Search Notes. | `n: invoice` | Notes-only rows. | Type. | Notes passive source enabled by explicit filter. | Metadata rows, no note bodies in receipt. | Source filter audits and matrix script. |
| Search browser history. | `h: docs` | History rows. | Type. | History metadata provider. | URL/title rows. | `root-source-filter-history-up.ts`. |
| Filter apps. | `a: terminal` | Primary app rows. | Type. | `SearchResult::root_unified_source`. | Apps only. | Source filters contract. |
| Filter scripts. | `s: deploy` | Primary script rows. | Type. | Root unified source filtering. | Scripts only. | Source filters contract. |
| Filter commands. | `cmd: settings` | Command rows. | Type. | Config-backed command source. | Commands only. | Config schema parity audit. |
| Exclude Files. | `-files: png` | Source-filter mode. | Type. | Exclusion application after parsing. | Non-Files rows only. | Menu syntax source filter tests. |
| Discover source filters. | `:` | Discovery/refine path. | Type. | Menu syntax discovery branch. | No committed source filter. | Menu syntax tests. |
| Keep unknown head literal. | `unknown: png` | Ordinary search. | Type. | Parser rejects unknown source head. | `unknown:` remains search text. | Parser tests. |
| Keep source status non-actionable. | Any source with status row. | Status visible. | Down/Enter/Cmd+K. | Status row excluded from result/action subject. | Cannot select or execute status. | `getElements`, source audits. |

## State Machine

| State | Trigger | Next state | Notes |
|---|---|---|---|
| Ordinary root search | No committed source filter. | Primary/passive/default root rows. | Passive sources obey ordinary config thresholds. |
| Source discovery | Leading `:`. | Discovery/refine UI state. | Does not create `sourceFilters`. |
| Source-filter parse | Known source head committed. | Source-filter mode. | Sets include/exclude filters and stripped query. |
| Source-only browse | Positive filter, empty stripped text, no advanced predicate. | Browse rows for that source. | Negative-only and non-empty queries do not use browse defaults. |
| Source query | Positive filter plus stripped text. | Source search rows. | Explicit source filters can enable disabled passive defaults. |
| Source loading | Provider pending for active frame. | Status row. | Pending results for old frame must not mutate current frame. |
| Source exhausted | Page or provider complete. | Exhausted status. | Files may show "No more results". |
| Files page expansion | Selection approaches page end. | Larger source-chip page. | Preserve selection key and scroll safely. |
| Actions | Cmd+K on result row. | MainList actions dialog. | Status rows must not become action subjects. |

## Visual And Focus States

| State | Visible result | Focus owner | Automation signal |
|---|---|---|---|
| Parsed source filter. | Source-filter chips/indicators plus scoped rows. | Main filter/list. | `sourceFilters`, `filterIndicators`, `computedSearchText`. |
| Source-only browse. | Empty filter text after stripping, browse rows, status rows. | Main filter/list. | Empty `computedSearchText` with positive source include. |
| Source status. | Loading/disabled/empty/exhausted metadata row. | Not focusable. | `getElements` row with `role:"status"` / `kind:"sourceStatus"`. |
| Files lazy page. | More Files rows appended within same list. | Selection remains on stable row. | `mainListScroll`, selected key, visible fingerprint. |
| Actions on result. | Actions dialog opens for selected result. | Actions dialog. | `actionsDialog` receipt from feature 011. |
| Input history blocked. | Source filter text remains in filter input. | Main filter/list. | Up/Down changes row selection, not input text. |

## Keystrokes And Commands

| Input | Behavior |
|---|---|
| Printable source heads | Parsed into include/exclude filters when known and unquoted. |
| Space after source head | Allows spaced syntax and source-only browse. |
| Up/Down | Navigates rows; source-filter mode blocks input-history recall. |
| Enter | Executes selected result; status rows are not executable. |
| Cmd+K | Opens root actions only for real result rows. |
| Escape | Uses normal launcher close/back behavior; source filters do not create a separate window. |
| `setFilter` | Automation route for typing source-filter queries directly. |
| `waitFor` | Waits for source-filter state, rows, status, or scroll conditions. |

## Actions And Menus

Source-filter mode does not create a separate action catalog. It scopes which root rows exist. Cmd+K actions are owned by the selected row's source and the root source actions feature.

Status rows are intentionally excluded from SearchResults, selected-row resolution, row counts, scroll height, Enter execution, and action subjects. Agents should verify action behavior against result rows, then separately verify that status rows remain metadata-only.

## Automation And Protocol Surface

| Surface | Fields/behavior |
|---|---|
| `mainWindowPreflight` | `filterText`, `computedSearchText`, `sourceFilters`, `filterIndicators`, selected index/key/role, visible results, fingerprints, root passive frame, warnings, and action receipts. |
| `rootPassiveFrame` | Frame query, source filters, per-source enabled/cache/loading/refresh status, and source-specific snapshot identity. |
| `getElements` | Exposes visible source status rows as `role:"status"` / `kind:"sourceStatus"` without payload-heavy local data. |
| `waitFor` | Can assert source filters, stripped text, row source names, status rows, stable fingerprints, and scroll/paging changes. |
| `batch` | Preferred for source-filter proof scripts to avoid timing gaps between `setFilter`, navigation, and state reads. |
| `mainListScroll` | Required for Files lazy paging proof, including footer-safe reveal and non-snap behavior. |

Runtime proof scripts named in the raw pass:

```bash
bun scripts/agentic/root-source-filter-stability.ts
bun scripts/agentic/root-source-filter-clipboard.ts
bun scripts/agentic/root-source-filter-history-up.ts --timeout 12000
bun scripts/agentic/source-chip-pagination-proof.ts --timeout 16000
bun scripts/agentic/root-source-filter-matrix.ts --query s --timeout 16000
bun scripts/agentic/root-source-filter-lazy-scroll.ts
```

## Data, Storage, And Privacy Boundaries

- Clipboard root rows and receipts stay metadata-first; full clipboard content is loaded only through explicit selection or source-owned actions.
- Dictation rows expose transcript-safe metadata; transcript content loads only after explicit paste/copy/attach/create-note actions.
- Notes rows expose note metadata, not note body content, in source-filter receipts.
- Browser Tabs and Browser History expose local title/URL/visit metadata only; they do not expose page content, cookies, downloads, network data, or favicons.
- ACP conversation rows expose saved conversation metadata and resume identifiers, not the full transcript in generic source-filter receipts.
- Passive snapshots and frame keys must avoid stale async updates crossing from one source filter to another.
- Status rows should remain content-light and non-actionable even when the provider is loading, disabled, unavailable, empty, or exhausted.

## Error, Empty, Loading, And Disabled States

| State | Expected behavior |
|---|---|
| Loading | Show source status row such as "Loading more..." for the active frame only. |
| Empty | Show source-specific empty state, not unrelated launcher fallback rows. |
| Exhausted | Show "Showing N of M" or "Showing N of M - No more results" for paged sources where available. |
| Disabled | Show disabled/source unavailable status when a routed source exists but cannot produce rows. |
| Provider unavailable | Keep scoped UI stable and expose provider-unavailable status. |
| AI Vault disabled | `vault:` / `v:` shows disabled status until searchable rows are implemented/proven. |
| Unknown head | Treat as literal search text. |
| Quoted head | Treat as literal search text. |
| Processes head | Treat as uncommitted/planned unless current parser/tests change. |
| Stale async result | Ignore if frame key does not match active stripped query and source-filter set. |

## Code Ownership

| Area | Source anchors |
|---|---|
| Source heads and descriptors | `src/menu_syntax/source_heads.rs` |
| Query parsing | `src/menu_syntax/query.rs`, `src/menu_syntax/payload.rs` |
| Source-only browse config | `src/menu_syntax/source_filter_browse.rs` |
| Menu syntax hints | `src/menu_syntax/main_hint.rs` |
| Filtering cache and passive frames | `src/app_impl/filtering_cache.rs` |
| Root Files search and source-chip paging | `src/app_impl/root_file_search.rs` |
| Filter input changes/history boundary | `src/app_impl/filter_input_core.rs`, `src/app_impl/filter_input_change.rs` |
| Grouping and suppression | `src/scripts/grouping.rs`, `src/scripts/types.rs`, `src/scripts/search/unified.rs` |
| Status row rendering | `src/list_item/mod.rs` |
| Selection/scroll preservation | `src/scrolling/selection_owned.rs` |
| Automation receipts | `src/main_window_preflight/types.rs`, `src/main_window_preflight/build.rs` |
| Source config/schema | `src/config/types.rs`, `src/config/defaults.rs` |
| Parser and source audits | `tests/menu_syntax_source_filters.rs`, `tests/source_audits/*source_filter*`, `tests/source_audits/*root_file*` |
| Runtime proof | `scripts/agentic/root-source-filter-*.ts`, `scripts/agentic/source-chip-pagination-proof.ts` |

## Invariants And Regression Risks

- Leading `:` remains discovery/refine syntax, not a committed source filter.
- Known standalone source heads commit include/exclude filters; quoted, unknown, and uncommitted heads stay literal.
- Attached query syntax strips correctly for `c:skip`, `files:s`, and similar forms.
- Positive filters explicitly enable their source for the active stripped query.
- Exclusion wins over inclusion.
- Source-filter mode suppresses disallowed sources and fallback rows.
- Source-only browse requires empty stripped text, positive include, and no advanced predicate.
- Source status rows are not SearchResults, are not selectable, do not count toward scroll height/action subjects, and remain visible to automation.
- Root Files and passive frame keys include source filters to prevent async bleed.
- Explicit Files filters allow one-character ASCII alphanumeric search; ordinary root search does not inherit that threshold.
- Files source-chip paging expands near bottom without Enter, snap-to-top, or footer overlap.
- Source-filter mode blocks launcher input-history recall.
- AI Vault and Processes must stay documented as disabled/uncommitted until tests prove committed rows.

## Verification Recipes

### Source And Unit Contracts

Run:

```bash
cargo test --test menu_syntax_source_filters -- --nocapture
cargo test --test source_audits root_unified_source_filters_contract -- --nocapture
cargo test --test source_audits root_unified_source_filter_browse_contract -- --nocapture
cargo test --test source_audits root_file_search_contract -- --nocapture
cargo test --test source_audits root_unified_passive_snapshot_contract -- --nocapture
cargo test --test source_audits root_unified_search_stability_contract -- --nocapture
cargo test --test source_audits root_unified_config_schema_parity_contract -- --nocapture
cargo test --lib source_filter_files_empty_browse_uses_browse_target_not_recent_render_cap -- --nocapture
cargo test --test source_audits root_recent_file_seed_pool_exceeds_empty_render_cap -- --nocapture
```

Check:

- Source heads parse and strip correctly.
- Discovery, quoted, unknown, and uncommitted heads stay separate.
- Include/exclude semantics and fallback suppression hold.
- Source-only browse uses browse targets.
- Passive frames and cache fingerprints include source filters.
- Files one-character explicit search and paging contracts hold.

### Runtime State Proof

Run:

```bash
bun scripts/agentic/root-source-filter-stability.ts
bun scripts/agentic/root-source-filter-clipboard.ts
bun scripts/agentic/root-source-filter-history-up.ts --timeout 12000
bun scripts/agentic/source-chip-pagination-proof.ts --timeout 16000
bun scripts/agentic/root-source-filter-matrix.ts --query s --timeout 16000
bun scripts/agentic/root-source-filter-lazy-scroll.ts
```

For each proof, capture:

- `filterText` and `computedSearchText`.
- `sourceFilters` include/exclude set.
- Visible row source names and selected stable key.
- `getElements` source status rows.
- `mainListScroll` and selection after Files page expansion.
- Absence of fallback rows and executable status rows.

### Hygiene

Run:

```bash
cargo check --lib
cargo fmt --check
lat check
git diff --check -- .goals/feature_map.md feature-map FEATURE_MAP.md
```

## Agent Notes

- Treat this feature as a parser-to-provider-to-receipt contract. A visual chip without stripped-query/source-filter state is incomplete.
- Prefer state-first proof. Screenshots are useful only for footer overlap or row placement issues that state cannot answer.
- Use `batch` to combine filter changes, key navigation, and receipts when proving lazy paging.
- Do not assert full content access from source-filter rows. Most rows intentionally expose metadata until an explicit action or Enter route.
- Keep 011 Root Unified Search Result Actions separate: source filters decide which rows exist; feature 011 decides which actions those rows expose.

## Related Features

- [001 Main Menu / Script List / Menu Syntax / Actions / Shortcut Assignment](./001-main-menu.md)
- [005 Built-in Filterable Surfaces](./005-built-in-filterable-surfaces.md)
- [007 Root Unified Search Notes](./007-root-notes.md)
- [008 Root Unified Search Clipboard History](./008-root-clipboard-history.md)
- [009 Root Unified Search Dictation History](./009-root-dictation-history.md)
- [010 Root Unified Search ACP History](./010-root-acp-history.md)
- [011 Root Unified Search Result Actions](./011-root-source-actions.md)

## Open Questions And Gaps

- `processes:` / `p:` appears in descriptors but is not committed by the parser/tests until root process rows exist.
- `vault:` / `v:` is routed far enough to show disabled status, but searchable AI Vault rows are not proven.
- Primary source default browse for `apps:`, `scripts:`, and `commands:` should be treated as filtered primary rows unless future tests prove dedicated browse defaults.
- Browser Tabs and Browser History availability depends on local provider snapshots.
- Display names in receipts can drift between source labels, such as `clipboard` versus Clipboard History; agents should assert stable source ids where available.
