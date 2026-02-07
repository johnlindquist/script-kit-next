# Data Structure Improvements Audit

Scope: `src/**/*.rs`

Date: 2026-02-07

## Executive Summary

The codebase is generally solid for current scale, but there are several hot paths still using linear scans and re-allocation patterns where indexed structures would materially reduce CPU and allocation churn. The highest-impact opportunities are:

1. Trigger matching in `keyword_matcher` (`O(triggers * trigger_len)` per keystroke).
2. Command/builtin/script lookup paths in app execution (`O(n)` scans + repeated vector construction).
3. Scheduler storage keyed by path currently modeled as `Vec` with repeated linear lookups.

## Findings (Prioritized)

## P0: Keyword matching is linear over all triggers on every keystroke

- Location:
  - `src/keyword_matcher.rs:145`
  - `src/keyword_matcher.rs:152`
  - `src/keyword_matcher.rs:137`
  - `src/keyword_matcher.rs:139`
- Current behavior:
  - `process_keystroke` appends to a rolling `String` and checks every registered trigger with `buffer.ends_with(trigger)`.
  - Buffer trimming rebuilds the full string via `chars().skip(excess).collect()`.
- Data-structure issue:
  - Per keystroke cost grows linearly with number of triggers.
  - Buffer trim is copy-heavy and repeated.
- Recommendation:
  - Replace trigger scan with a suffix matcher structure:
    - Option A: reversed trie keyed by chars from end of buffer.
    - Option B: Aho-Corasick automaton if trigger count grows significantly.
  - Replace `String` front-trim strategy with a fixed-capacity ring (`VecDeque<char>`) and bounded matcher window.
- Expected impact:
  - Significant reduction in per-keystroke CPU and allocations under larger trigger sets.
- Migration risk:
  - Medium. Must preserve exact trigger semantics (Unicode boundaries, delimiter behavior, clear chars).

## P0: Command lookup path mixes O(1) alias registry with multiple O(n) rescans

- Location:
  - `src/app_impl.rs:2364`
  - `src/app_impl.rs:2368`
  - `src/app_impl.rs:2382`
  - `src/app_impl.rs:2396`
  - `src/app_impl.rs:2407`
  - `src/app_impl.rs:6184`
  - `src/app_impl.rs:6200`
  - `src/app_impl.rs:6228`
  - `src/app_impl.rs:6252`
  - `src/app_impl.rs:6092`
  - `src/app_impl.rs:6105`
  - `src/app_execute.rs:1984`
- Current behavior:
  - Alias registry is `HashMap` (`O(1)`), but resolving command IDs repeatedly scans scripts/scriptlets/apps vectors.
  - Builtins are reconstructed (`get_builtin_entries`) and then searched by `.find(...)` in multiple call sites.
- Data-structure issue:
  - Repeated linear scans in hot execution paths.
  - Repeated `Vec` construction for builtins adds avoidable allocation churn.
- Recommendation:
  - Maintain indexed maps refreshed with content updates:
    - `builtin_by_id: HashMap<String, Arc<BuiltInEntry>>`
    - `app_by_bundle_id: HashMap<String, Arc<AppEntry>>`
    - `script_by_path: HashMap<String, Arc<Script>>`
    - `scriptlet_by_name: HashMap<String, Arc<Scriptlet>>`
    - `scriptlet_by_path: HashMap<String, Arc<Scriptlet>>`
  - Reuse a cached builtin collection instead of rebuilding ad hoc for point lookups.
- Expected impact:
  - Lower lookup latency and reduced allocations in alias execution and command dispatch.
- Migration risk:
  - Medium. Requires index invalidation discipline on reload/update events.

## P1: Scheduler stores path-keyed data in `Vec`

- Location:
  - `src/scheduler.rs:63`
  - `src/scheduler.rs:141`
  - `src/scheduler.rs:169`
  - `src/scheduler.rs:284`
- Current behavior:
  - Scripts are stored in `Vec<ScheduledScript>` and updated/removed via linear search.
- Data-structure issue:
  - Path-keyed operations are naturally `HashMap` use-cases.
- Recommendation:
  - Replace with `HashMap<PathBuf, ScheduledScript>`.
  - If ordered rendering/debug output is needed, keep a secondary ordered list or sort at read time.
- Expected impact:
  - Faster add/update/remove paths and simpler keyed updates.
- Migration risk:
  - Low to medium (iteration order changes unless explicitly preserved).

## P1: Notification service uses `Vec` for id-keyed and dedupe-keyed lookups

- Location:
  - `src/notification/service.rs:24`
  - `src/notification/service.rs:88`
  - `src/notification/service.rs:236`
  - `src/notification/service.rs:257`
  - `src/notification/service.rs:390`
- Current behavior:
  - Active notifications are in `Vec<ActiveNotification>`.
  - Dedupe/update/get/dismiss-by-id all scan linearly.
- Data-structure issue:
  - Frequent keyed operations perform repeated O(n) scans.
- Recommendation:
  - Keep `Vec` for display order, add side indexes:
    - `id_to_index: HashMap<NotificationId, usize>`
    - `dedupe_to_id: HashMap<String, NotificationId>` (optional)
  - Rebuild or patch indexes on mutation/removal.
- Expected impact:
  - Lower overhead for progress updates and dismiss/get operations.
- Migration risk:
  - Medium. Index maintenance bugs are possible without strong tests.

## P1: Shortcut matching allocates/sorts on each lookup

- Location:
  - `src/shortcuts/registry.rs:187`
  - `src/shortcuts/registry.rs:194`
  - `src/shortcuts/registry.rs:196`
  - `src/shortcuts/registry.rs:219`
- Current behavior:
  - `find_match` scans every binding in context, allocates `matches: Vec<_>`, then sorts to choose winner.
- Data-structure issue:
  - Cost is `O(contexts * bindings + m log m)` plus per-call allocation.
- Recommendation:
  - Pre-index bindings by `(context, canonical_keystroke)` to pre-resolve highest-priority winner.
  - Keep existing `bindings` vec for deterministic UI/order while hot path uses indexed lookup.
- Expected impact:
  - Lower key-handling latency and less allocator pressure.
- Migration risk:
  - Medium. Requires canonical keystroke normalization consistency.

## P2: Navigation repeatedly rescans grouped list for first/last selectable

- Location:
  - `src/app_navigation.rs:18`
  - `src/app_navigation.rs:26`
  - `src/app_navigation.rs:78`
  - `src/app_navigation.rs:86`
  - `src/app_navigation.rs:133`
  - `src/app_navigation.rs:157`
  - `src/app_navigation.rs:202`
  - `src/app_navigation.rs:246`
- Current behavior:
  - Multiple movement methods call `position`/`rposition` over grouped items.
- Data-structure issue:
  - Repeated full scans in keyboard navigation flow.
- Recommendation:
  - Cache selectable indices when grouped results are rebuilt.
  - Optionally precompute next/prev selectable arrays for O(1) step transitions.
- Expected impact:
  - Smoother navigation in larger result sets.
- Migration risk:
  - Low.

## P2: Keystroke logger uses front-removal on `String`

- Location:
  - `src/keystroke_logger.rs:24`
  - `src/keystroke_logger.rs:75`
  - `src/keystroke_logger.rs:76`
- Current behavior:
  - Keeps recent chars in `String` and removes from front with `remove(0)`.
- Data-structure issue:
  - Front removal in `String` is O(n).
- Recommendation:
  - Use `VecDeque<char>` with max length 10.
- Expected impact:
  - Small but clean hot-path improvement.
- Migration risk:
  - Low.

## P3: Minor scan/allocation opportunities

- `src/scripts/grouping.rs:370`
  - Default suggestion population does repeated `results.iter().position(...)` per default name.
  - Build `HashMap<&str, usize>` name-to-first-index once if list size grows.

- `src/template_variables.rs:208`
  - `extract_variable_names` builds full `Vec<char>` and scans by index.
  - Stream with `char_indices()` to reduce peak allocations for large templates.

## Suggested Implementation Order

1. Build command lookup indexes (app execution path) and replace repeated builtins reconstruction lookups.
2. Refactor `KeywordMatcher` to indexed suffix matching + ring buffer.
3. Convert scheduler storage to map keyed by script path.
4. Add notification side indexes while preserving render order.
5. Optimize shortcut registry hot path with context+keystroke indexing.
6. Apply low-risk P2/P3 micro-optimizations.

## Test Coverage Recommendations (for follow-up implementation)

- Command indexes:
  - alias hit/miss correctness after script/builtin/app reload
  - stale index invalidation behavior
- Keyword matcher:
  - overlapping triggers (`:s`, `:sig`), Unicode input, buffer clear chars, max-buffer trimming behavior
- Scheduler map:
  - add/update/remove idempotency and stable next-run updates
- Notification indexes:
  - dedupe + dismiss + update_progress index consistency after removals
- Shortcut index:
  - override precedence and deterministic winner selection across contexts
