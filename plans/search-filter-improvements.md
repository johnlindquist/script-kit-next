# Search and Filter Improvements

Date: 2026-02-07
Agent: codex-search-filter
Scope: `src/**/*.rs`

## Summary

Search quality is already strong in the main script list (`nucleo` scoring + prefix filters), but behavior is fragmented across views.

Highest-impact opportunities:

1. Fix relevance/UX mismatches where displayed results and computed counts diverge.
2. Unify ranking and highlighting engines across prompts/dialogs.
3. Remove repeated `to_lowercase().contains(...)` filtering in hot paths.
4. Improve Unicode correctness for highlighting and fallback matching.

## Current Architecture Map

Primary engines and entry points:

- Unified search core: `src/scripts/search.rs`
- Grouping + frecency reorder: `src/scripts/grouping.rs`
- Two-stage coalesced filter pipeline: `src/app_impl.rs`, `src/filter_coalescer.rs`
- File search ranking + render mapping: `src/app_execute.rs`, `src/file_search.rs`, `src/render_builtins.rs`
- Prompt-local filtering:
  - Select prompt: `src/prompts/select.rs`
  - Path prompt: `src/prompts/path.rs`
  - Arg prompt helpers: `src/app_impl.rs`
- Actions dialog filtering: `src/actions/dialog.rs`

## Findings

### SF-001 (High): Unicode highlight path is inconsistent with Unicode ranking path

Evidence:

- Search ranking uses Unicode-safe `nucleo` (`Normalization::Smart`): `src/scripts/search.rs:243`, `src/scripts/search.rs:259`
- Match highlighting for unified search results still uses ASCII-only helper:
  - `src/scripts/search.rs:277`
  - `src/scripts/search.rs:290`
  - `src/scripts/search.rs:337`
  - `src/scripts/search.rs:387`
- ASCII helper explicitly warns it degrades for non-ASCII: `src/scripts/search.rs:27`
- Select prompt highlight also uses ASCII helper: `src/prompts/select.rs:398`, `src/prompts/select.rs:406`

Impact:

- A result can rank highly via Unicode-aware fuzzy matching but show weak or missing highlight spans.
- This creates confusing relevance feedback for accented/non-English script names.

Recommendation:

- Add a Unicode-safe `compute_match_indices_nucleo(...)` utility in `src/scripts/search.rs` and use it for all highlight generation.
- Keep ASCII fast-path only as an optimization fallback when both query and haystack are ASCII.

Suggested tests:

- Add tests in `src/scripts/search.rs` and `src/prompts/select.rs` for accented and non-Latin strings where ranking and highlight spans must agree.

---

### SF-002 (High): FileSearch window sizing count can diverge from displayed fuzzy result count

Evidence:

- Displayed list uses precomputed fuzzy-ranked indices: `src/render_builtins.rs:4015`, `src/render_builtins.rs:4261`
- Those indices come from `filter_results_nucleo_simple(...)`: `src/app_execute.rs:1774`
- Window-size count for FileSearch uses plain substring filter instead:
  - `src/app_impl.rs:3442`
  - `src/app_impl.rs:3450`

Impact:

- Height calculations can be wrong when fuzzy matches exist that are not plain substring matches.
- Selection/scroll heuristics can feel unstable due to count mismatch.

Recommendation:

- In `calculate_window_size_params`, use `self.file_search_display_indices.len()` for `AppView::FileSearchView`.
- Remove parallel substring counting path for this view.

Suggested tests:

- Add a regression test that verifies FileSearch view count equals display-index count for a fuzzy query (for example, matching with transposed/partial characters).

---

### SF-003 (High): Repeated lowercase+contains filtering causes avoidable O(n) allocations and duplicated logic

Evidence:

Repeated filter passes in render and key handlers:

- Clipboard history: `src/render_builtins.rs:352`, `src/render_builtins.rs:428`
- App launcher: `src/render_builtins.rs:1137`, `src/render_builtins.rs:1192`
- Window switcher: `src/render_builtins.rs:1545`, `src/render_builtins.rs:1603`
- Theme chooser helper: `src/render_builtins.rs:2578`, `src/render_builtins.rs:2588`
- Window sizing repeats similar counts: `src/app_impl.rs:3386`, `src/app_impl.rs:3406`, `src/app_impl.rs:3420`, `src/app_impl.rs:3464`
- Arg prompt choice filtering duplicates substring logic: `src/app_impl.rs:7133`, `src/app_impl.rs:7165`

Impact:

- Extra allocations (`to_lowercase`) per item per keystroke.
- Higher latency for large collections.
- Multiple behavior copies drift over time.

Recommendation:

- Introduce a shared `FilterSnapshot` per active view with:
  - normalized query once
  - cached filtered index list
  - optional cached lowercase fields on items
- Reuse snapshot for rendering, key navigation, and sizing.

Suggested tests:

- Add performance-oriented unit tests around helper functions (allocation-free correctness paths).
- Add behavior parity tests: render count == navigation count == sizing count per view.

---

### SF-004 (Medium): Ranking models are fragmented across views, producing inconsistent relevance

Evidence:

- Main search uses weighted mixed model (exact/prefix/substr + nucleo + metadata bonuses): `src/scripts/search.rs:672`, `src/scripts/search.rs:899`, `src/scripts/search.rs:1472`
- Select prompt uses field-weighted `nucleo` scoring: `src/prompts/select.rs:322`
- Actions dialog uses custom heuristic + subsequence matcher (`fuzzy_match`), not `nucleo`: `src/actions/dialog.rs:1013`, `src/actions/dialog.rs:1047`
- Path and Arg prompt filtering are substring-only:
  - `src/prompts/path.rs:259`
  - `src/app_impl.rs:7133`

Impact:

- User sees different ranking quality depending on current view.
- Typo tolerance and partial-match behavior are unpredictable.

Recommendation:

- Create a shared ranking utility (for example `search::rank_fields`) that all views can adopt.
- Standardize baseline behavior: exact > prefix > fuzzy > description/metadata bonus.

Suggested tests:

- Add cross-view fixture tests to ensure common queries produce consistent top ordering for equivalent data.

---

### SF-005 (Medium): FileSearch filtering path still does extra allocations/clones under load

Evidence:

- `filter_results_nucleo_simple` allocates scored tuples and then allocates again to drop scores: `src/file_search.rs:1412`, `src/file_search.rs:1429`
- Render builds cloned `(index, FileResult)` list for closure each render: `src/render_builtins.rs:4186`, `src/render_builtins.rs:4189`
- Directory sort lowercases in comparator repeatedly: `src/app_execute.rs:1738`

Impact:

- More allocator pressure on large directories.
- CPU overhead in repeated sort/filter cycles.

Recommendation:

- Keep a stable `Vec<usize>` ranking index and avoid cloning `FileResult` in render closures.
- Precompute lowercase sort keys for directory entries once per refresh.
- Consider returning `(idx, score)` only from fuzzy filter and dereference results lazily.

Suggested tests:

- Add micro-benchmark style test (unit-level) for ranking helper with large synthetic file lists.

---

### SF-006 (Medium): Frecency second-pass reorder may conflict with unified type-priority ordering

Evidence:

- Unified search sorts by score then type order (BuiltIn > App > Window > Script > Scriptlet > Agent): `src/scripts/search.rs:1551`
- In grouped search mode (non-empty filter), frecency bonus is applied and resorted by boosted score then name only: `src/scripts/grouping.rs:119`, `src/scripts/grouping.rs:164`

Impact:

- Type-order tie-breaking from unified search can be lost after frecency reorder.
- Frequent items can unexpectedly jump categories in ties, creating inconsistent ranking semantics.

Recommendation:

- Preserve unified tie-break policy in frecency sort comparator (score -> type order -> name).
- Consider applying frecency bonus inside the primary scoring pass to avoid double-sort drift.

Suggested tests:

- Add regression test where equal boosted scores preserve expected type priority.

---

### SF-007 (Medium): Match highlighting coverage is incomplete in list UIs

Evidence:

- FileSearch rows render plain name/path text without highlighted matches: `src/render_builtins.rs:4364`, `src/render_builtins.rs:4370`
- Actions dialog renders plain title/description (no match spans): `src/actions/dialog.rs:1734`, `src/actions/dialog.rs:1749`
- Select prompt highlights title only; subtitle remains plain: `src/prompts/select.rs:743`

Impact:

- Users cannot quickly see why an item matched.
- In long lists, scan time increases and relevance feels weaker.

Recommendation:

- Standardize highlight rendering helper for title + secondary field.
- Use field-aware highlight source that matches whichever field contributed the score.

Suggested tests:

- Add rendering-model tests for highlighted ranges with title/description matches.

---

### SF-008 (Low): Scriptlet code fallback search is ASCII-only and expensive for large code bodies

Evidence:

- Code fallback only when query len >= 4 and score == 0: `src/scripts/search.rs:997`
- Uses ASCII-only contains on full `scriptlet.code`: `src/scripts/search.rs:1003`, `src/scripts/search.rs:1005`

Impact:

- Good guardrails already exist, but Unicode code/text queries can miss.
- Worst-case scans still read entire code strings for candidate scriptlets.

Recommendation:

- Add optional capped scan length or token index for scriptlet code fallback.
- Consider Unicode-safe contains fallback when either side is non-ASCII.

## Roadmap

### Phase 1: Correctness and UX parity (quick wins)

1. Fix FileSearch count mismatch (`SF-002`).
2. Add Unicode-safe highlight path for unified results and Select prompt (`SF-001`).
3. Add highlight rendering for FileSearch and Actions dialog (`SF-007`).

### Phase 2: Shared filtering/ranking infrastructure

1. Introduce common `FilterSnapshot` cache for built-in views (`SF-003`).
2. Introduce shared ranking helper and migrate Actions/Path/Arg prompts (`SF-004`).
3. Preserve tie-break semantics when applying frecency boost (`SF-006`).

### Phase 3: Scale optimization

1. Reduce clone/alloc paths in FileSearch display pipeline (`SF-005`).
2. Optimize scriptlet code fallback scanning/indexing (`SF-008`).

## Suggested Success Metrics

- P95 filter-to-render latency under 16ms at 10k items for built-in list views.
- Zero count mismatches between displayed items and window sizing/navigation counts.
- Unicode highlight parity: highlighted ranges present whenever scored matches are visible.
- Cross-view relevance consistency for shared query fixtures.

## Risks and Gaps

- Migrating all views to one ranking helper can change ordering users already rely on.
- Highlight semantics across multiple matched fields need clear precedence to avoid visual noise.
- Some performance improvements likely need benchmark harnesses beyond current unit tests.
- This report is code-audit based; no runtime profiling traces were added in this pass.
