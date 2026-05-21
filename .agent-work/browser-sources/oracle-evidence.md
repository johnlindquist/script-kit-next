# Browser Sources Visibility Evidence

## Prior-State Ledger
- prior_oracle_session: none
- forward_progress_gate: verified
- forward_progress_index: +3 consumed red receipts +3 selected browser sources visibility slice +2 named files/functions/tests/commands +2 added falsifiable DevTools verification = 10

## Goal
Fix Browser History and Browser Tabs visibility in the main list.

User-reported bugs:
- Browser History does not show in the main list at all, even with `history:`.
- Browser Tabs do not show in the main list until `tabs:` has been run at least once.

## Resource Controls
- Runtime receipts were gathered before this Oracle bundle.
- Do not launch app/DevTools/ACP sessions while Oracle is planning.
- Future DevTools sessions must use unique names prefixed `browser-sources-`, one at a time, and must be stopped before Oracle, commit, and final response.
- Keep evidence lightweight; no sandbox HOME/npm/codex caches under `.agent-work`.

## Runtime Red Receipts

### Browser History
Session used: `browser-history-red-0521a` (stopped after collection).

Receipt files:
- `.agent-work/browser-sources/browser-history-red-0521a-state-history-colon.json`
- `.agent-work/browser-sources/browser-history-red-0521a-state-history-colon-settled.json`
- `.agent-work/browser-sources/browser-history-red-0521a-elements-history-colon.json`

Observed state for `history:`:
- `filterText`: `history:`
- `computedSearchText`: empty string
- `sourceFilters`: `["history"]`
- `visibleResultCount`: `0`
- `rootPassiveFrame.browserHistory.enabled`: `true`
- `rootPassiveFrame.browserHistory.frameCount`: `0`
- `rootPassiveFrame.browserHistory.cacheGeneration`: `1`
- `rootPassiveFrame.browserHistory.frameGeneration`: `1`
- `rootPassiveFrame.browserHistory.refreshing`: `true`
- same state after a short settled wait: still 0 rows, still refreshing

### Browser Tabs
Session used: `browser-sources-red-tabs-0521b` (stopped after collection).

Receipt files:
- `.agent-work/browser-sources/browser-sources-red-tabs-0521b-state-tabs-first.json`
- `.agent-work/browser-sources/browser-sources-red-tabs-0521b-state-tabs-settled.json`
- `.agent-work/browser-sources/browser-sources-red-tabs-0521b-state-tabs-second.json`
- `.agent-work/browser-sources/browser-sources-red-tabs-0521b-elements-tabs-first.json`

Observed first `tabs:` state:
- `filterText`: `tabs:`
- `computedSearchText`: empty string
- `sourceFilters`: `["tabs"]`
- `visibleResultCount`: `0`
- `rootPassiveFrame.browserTabs.enabled`: `true`
- `rootPassiveFrame.browserTabs.frameCount`: `0`
- `rootPassiveFrame.browserTabs.cacheGeneration`: `1`
- `rootPassiveFrame.browserTabs.frameGeneration`: `1`
- `rootPassiveFrame.browserTabs.refreshing`: `true`
- after a 2 second wait: still 0 rows, still refreshing

Observed second `tabs:` state in the same session:
- `visibleResultCount`: `11`
- Browser Tabs rows: 11 `rootPassive` rows
- `rootPassiveFrame.browserTabs.frameCount`: `11`
- `rootPassiveFrame.browserTabs.cacheGeneration`: `2`
- `rootPassiveFrame.browserTabs.frameGeneration`: `2`
- `rootPassiveFrame.browserTabs.refreshing`: `true`

This confirms the user symptom: the first explicit tabs query shows no rows, while a later explicit tabs query can show the warmed snapshot.

## Source Context Summary
- `src/app_impl/filtering_cache.rs#root_passive_frame_for_current_query` switches explicit `tabs:` and `history:` to direct lookups.
- `src/browser_tabs.rs#search_root_browser_tabs_meta_direct` calls `ensure_root_browser_tabs_refresh(...)` and immediately reads the cached snapshot.
- `src/browser_history.rs#search_root_browser_history_meta_direct` reads the cached snapshot and triggers `ensure_root_browser_history_refresh(...)` only if the cached candidates are empty.
- `src/main_window_preflight/build.rs#build_root_passive_frame_receipt` exposes per-source `frameCount`, `cacheGeneration`, `frameGeneration`, and `refreshing`.
- `tests/source_audits/root_unified_passive_source_perf_contract.rs` pins that implicit browser tabs/history stay cache-only on the typing path and that cache generation participates in the grouped cache key.

## Current Hypothesis To Evaluate
The explicit source path schedules asynchronous refresh but does not cause the currently visible main list to be recomputed once the refresh completes. The user-visible list remains stuck on the stale frame until a second input event or another cache-key change causes grouping to run again.

## Requested Oracle Output
Please identify the highest-leverage narrow fix that preserves these constraints:
- implicit browser tabs/history search stays cache-first and nonblocking on the ordinary typing path
- explicit `tabs:` and `history:` should surface rows on first use once the background refresh completes, without requiring a second user input
- no broad rewrite of root unified search
- include exact file/function targets, critical code snippets, tests/source audits to add or update, and DevTools verification plan

Return your answer as text in this response only. Do not create, attach, export, or offer any downloadable file. Do not create local project artifacts yourself. The local agent will write any needed files, plans, notes, goals, commits, or verification logs using local tools.
