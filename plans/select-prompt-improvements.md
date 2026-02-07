# Select Prompt Improvements Audit

Date: 2026-02-07  
Agent: `codex-select-prompt`  
Scope: `src/prompts/select.rs`

## Executive Summary

`SelectPrompt` works for small lists, but it has correctness and scalability issues that become obvious as choice counts grow.

Top issues:

1. The list is not virtualized and not scroll-tracked, so large result sets are clipped and expensive to render.
2. Filtering does heavy recomputation on each keystroke (metadata parsing, repeated lowercase conversions, repeated fuzzy work).
3. Selection behavior has UX inconsistencies (`Space` cannot be typed into search, single-select `Enter` can submit an empty selection).
4. Grouping is not implemented in the prompt data model/render path.

## Current Behavior Map

`src/prompts/select.rs` currently provides:

1. Metadata extraction from `Choice.description` (`src/prompts/select.rs:69`).
2. Scored filtering across `name`, `description`, `value`, and inferred metadata (`src/prompts/select.rs:322`).
3. Keyboard routing for navigation/toggle/submit/cancel (`src/prompts/select.rs:609`).
4. Flat row rendering with `UnifiedListItem` (`src/prompts/select.rs:731`).
5. Unit tests for metadata parsing/scoring helpers and UTF-8 highlight range conversion (`src/prompts/select.rs:790`).

## Findings (Ranked)

### P0: Virtual scrolling and list rendering are not production-ready for large lists

Evidence:

1. Render builds every filtered row each frame with a full loop (`src/prompts/select.rs:731`).
2. List container uses `.overflow_y_hidden()` and has no scroll handle (`src/prompts/select.rs:718`).
3. Up/down navigation only changes `focused_index`; no `scroll_to_item(...)` path exists (`src/prompts/select.rs:550`, `src/prompts/select.rs:558`).

Impact:

1. Large lists are clipped visually.
2. Focus can move to off-screen rows with no visual feedback.
3. Render cost scales with filtered length, not viewport size.

Recommendation:

1. Migrate to `uniform_list` + `UniformListScrollHandle` and call `scroll_to_item(..., ScrollStrategy::Nearest)` on focus changes.
2. Keep fixed row height (`LIST_ITEM_HEIGHT`) and render only visible range.
3. Add keyboard + smoke tests that verify focus remains visible while navigating long lists.

### P0: Search/filter hot path does repeated expensive work per keystroke

Evidence:

1. Refilter runs on each char and backspace (`src/prompts/select.rs:567`, `src/prompts/select.rs:575`).
2. Metadata is reparsed per choice during filtering (`src/prompts/select.rs:327`) and again during render (`src/prompts/select.rs:735`).
3. `score_field` lowercases every haystack per score call (`src/prompts/select.rs:309`).
4. Tie-break sorting lowercases names inside comparator (`src/prompts/select.rs:492`).
5. Title highlighting reruns fuzzy matching during render (`src/prompts/select.rs:743`, `src/prompts/select.rs:398`).

Impact:

1. Typing latency grows with list size and description complexity.
2. Avoidable CPU/alloc churn in the most user-visible path.

Recommendation:

1. Build a cached `SelectChoiceIndex` once in `new()` with:
   - parsed metadata
   - lowercase searchable fields
   - precomputed semantic id fallback
2. Refilter should output index+score+highlight metadata into a reusable buffer.
3. Sort using cached lowercase keys, not inline `to_lowercase()`.
4. Add refilter timing spans with `correlation_id` to compare before/after latency.

### P1: Multi-select/search key behavior blocks common queries

Evidence:

1. `space` is hard-bound to toggle selection (`src/prompts/select.rs:631`).
2. Character input explicitly rejects `' '` (`src/prompts/select.rs:638`).

Impact:

1. Users cannot type multi-word queries (for example `open logs`).
2. Search quality feels broken even when scoring is good.

Recommendation:

1. Let `Space` insert a space in filter text by default.
2. Move toggle-selection to `Ctrl+Space` / `Cmd+Space` (or `Tab`) to preserve keyboard efficiency.
3. Add regression test: `test_select_prompt_accepts_space_in_filter_query`.

### P1: Single-select and Enter semantics are inconsistent

Evidence:

1. In single-select mode, selection is only updated on explicit toggle (`src/prompts/select.rs:523`).
2. `Enter` always submits `self.selected` (`src/prompts/select.rs:632`, `src/prompts/select.rs:533`).

Impact:

1. Navigating then pressing Enter can return `[]` if user never toggled.
2. Behavior diverges from expected launcher-style "Enter selects focused item" UX.

Recommendation:

1. In single-select mode, `Enter` should submit the focused item if nothing is toggled.
2. In multi-select mode, decide product behavior explicitly:
   - Option A: submit focused item when selection set is empty
   - Option B: require explicit toggle and show helper text

### P1: Selection state model has correctness and performance problems

Evidence:

1. `selected` is `Vec<usize>` (`src/prompts/select.rs:42`).
2. Row membership checks are O(n) via `.contains()` (`src/prompts/select.rs:734`).
3. Toggle removal scans linearly (`src/prompts/select.rs:518`).
4. Cmd/Ctrl+A checks only `len` equality (`src/prompts/select.rs:619`) and `select_all()` overwrites selection (`src/prompts/select.rs:585`).

Impact:

1. Selection operations get slower as selected count grows.
2. `len`-only equality can produce wrong toggle-all behavior.
3. Filtered select-all can unexpectedly drop prior off-filter selections.

Recommendation:

1. Use `HashSet<usize>` for membership and toggle operations.
2. Track deterministic submit order explicitly (for example sort by source index at submit).
3. Replace `len` check with set containment for filtered indices.
4. Make select-all additive over current selection (or explicitly document replace behavior).

### P1: Selection UX visuals and automation IDs are unstable

Evidence:

1. List item state sets `is_selected` from focus (`src/prompts/select.rs:760`) while selected state is rendered separately via accent bar (`src/prompts/select.rs:766`).
2. Fallback semantic id uses `display_idx`, which changes as filtering/sorting changes (`src/prompts/select.rs:738`).

Impact:

1. Focus vs selected semantics are visually muddled.
2. Semantic IDs are unstable for automated UI targeting.

Recommendation:

1. Decouple focused and selected styling explicitly (focused row highlight + selected checkmark/accent).
2. Generate fallback semantic IDs from stable source index/key, not display index.

### P2: Grouping is not implemented in SelectPrompt

Evidence:

1. Filter output is a flat `Vec<usize>` (`src/prompts/select.rs:44`, `src/prompts/select.rs:499`).
2. No grouped row model or header rendering exists in `SelectPrompt`.

Impact:

1. Cannot present large option sets in structured sections.
2. No header-aware navigation behavior.

Recommendation:

1. Add optional grouping metadata to select choices/protocol.
2. Build grouped rows using existing section-header patterns (`GroupedListItem`/`GroupedListState`) and skip headers during keyboard navigation.
3. Support group-level select-all where multi-select is enabled.

### P2: Test coverage is too narrow for prompt behavior

Evidence:

1. Existing unit tests only cover metadata/scoring helpers and UTF-8 ranges (`src/prompts/select.rs:800`).
2. Coverage matrix marks `select` as partial and calls out missing multi-select coverage (`tests/protocol-coverage-matrix.ts:145`).

Recommendation (TDD test names):

1. `test_select_prompt_accepts_space_in_filter_query`
2. `test_select_prompt_submit_uses_focused_item_in_single_mode_when_none_toggled`
3. `test_select_prompt_cmd_a_toggles_only_when_all_filtered_items_are_selected`
4. `test_select_prompt_select_all_preserves_existing_off_filter_selection_when_configured_additive`
5. `test_select_prompt_generates_stable_semantic_id_when_filter_order_changes`
6. `test_select_prompt_scrolls_to_keep_focused_item_visible`
7. `test_select_prompt_virtual_list_renders_visible_range_only`

## Prioritized Roadmap

### Phase 1: Correctness and UX

1. Fix space handling for search input.
2. Define and implement Enter semantics for single/multi select.
3. Fix Cmd/Ctrl+A logic (set-based, filtered-aware).
4. Stabilize semantic ID fallback generation.

### Phase 2: Performance foundations

1. Add cached choice index (parsed metadata + lowercase fields).
2. Move selection membership to `HashSet`.
3. Precompute highlight ranges during refilter.

### Phase 3: Virtualization and grouping

1. Convert rows to `uniform_list` + scroll handle.
2. Add focus-driven `scroll_to_item`.
3. Introduce grouped row model and header-aware navigation.

### Phase 4: Validation and telemetry

1. Add targeted unit/smoke tests from above.
2. Add structured latency logs (`refilter_ms`, `render_rows`, `visible_rows`) with `correlation_id`.

## Risks / Known Gaps

1. Grouping likely needs protocol/type changes beyond `src/prompts/select.rs`.
2. Selection order semantics (toggle order vs source order) need product decision before changing submit payload ordering.
3. Keybinding changes for toggle/search (`Space` behavior) may require migration notes for users with muscle memory.
