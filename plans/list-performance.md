# Unified List Item and Select List Performance Audit

Date: 2026-02-07  
Agent: `codex-list-perf`  
Scope: `src/components/unified_list_item/**/*.rs`, `src/components/unified_list_item_tests.rs`, `src/prompts/select.rs`

## Summary

The current `SelectPrompt` and `UnifiedListItem` implementation is functionally solid, but it has hot-path costs that will scale poorly with large lists:

1. `SelectPrompt` renders every filtered row on each update (`src/prompts/select.rs:731`), with no virtualization.
2. Filtering recomputes metadata, lowercase strings, and fuzzy context per keystroke (`src/prompts/select.rs:471`, `src/prompts/select.rs:480`, `src/prompts/select.rs:327`).
3. Selection checks are O(n) per visible row because selected indices are stored in `Vec<usize>` (`src/prompts/select.rs:42`, `src/prompts/select.rs:734`).
4. `UnifiedListItem` highlighted text rendering allocates many short strings/spans every render (`src/components/unified_list_item/render.rs:394`).
5. The `Custom` variants in `TextContent`, `LeadingContent`, and `TrailingContent` are currently accepted by types but ignored at render time (`src/components/unified_list_item/render.rs:299`, `src/components/unified_list_item/render.rs:344`, `src/components/unified_list_item/render.rs:390`).

## Findings

## P0 - Virtualization and Keystroke Latency

### 1) `SelectPrompt` is non-virtualized and rebuilds the full row tree

- Evidence:
  - Full loop over all filtered rows in render: `src/prompts/select.rs:731`.
  - No `uniform_list` / `track_scroll` usage in this prompt.
  - Each row constructs `UnifiedListItem` plus wrappers (`src/prompts/select.rs:749`).
- Impact:
  - UI work scales with result count, not viewport size.
  - Large choice sets will cause keypress-to-frame latency spikes.
- Recommendation:
  - Migrate to `uniform_list` with a scroll handle, following established patterns already used in prompt/builtin lists (for example `src/render_prompts/arg.rs:382`, `src/render_prompts/arg.rs:408`).
  - Keep `LIST_ITEM_HEIGHT` fixed and render only visible ranges.

### 2) Refilter path does high allocation/recompute work on every key

- Evidence:
  - Refilter runs on every character and backspace (`src/prompts/select.rs:568`, `src/prompts/select.rs:576`).
  - New fuzzy context per refilter (`src/prompts/select.rs:480`).
  - Metadata parsing is rebuilt per choice during scoring (`src/prompts/select.rs:327`, `src/prompts/select.rs:70`).
  - `score_field` lowercases each field repeatedly (`src/prompts/select.rs:309`).
  - Sort tie-breaker lowercases names inside comparator (`src/prompts/select.rs:492`).
- Impact:
  - Unnecessary CPU and heap churn under fast typing.
- Recommendation:
  - Introduce a precomputed `ChoiceSearchIndex` (name lowercased, parsed metadata, optional normalized tie-break key, semantic id).
  - Reuse buffers for score output (`clear` + reserve strategy) instead of allocating new vectors each refilter.
  - Keep query normalization once per keypress.

## P1 - Data Shape and Render Allocation

### 3) Selection state is O(n) for membership and toggle operations

- Evidence:
  - Membership check per row: `self.selected.contains(&choice_idx)` at `src/prompts/select.rs:734`.
  - Toggle remove uses linear search (`src/prompts/select.rs:518`).
  - Select-all clones filtered vector into selected (`src/prompts/select.rs:585`).
- Impact:
  - Behavior trends toward O(rows * selected_count) per render.
- Recommendation:
  - Replace `Vec<usize>` with `HashSet<usize>` (or `FxHashSet`) for membership + toggle.
  - If submission order matters, keep a parallel ordered vector or use `IndexSet<usize>`.

### 4) Render path repeats work already done (or needed) in filtering

- Evidence:
  - Metadata reparsed during row render (`src/prompts/select.rs:735`) even though scoring already parsed metadata.
  - Fuzzy highlight recomputed per row render (`src/prompts/select.rs:743`, `src/prompts/select.rs:398`).
  - Semantic id regenerated per render when absent (`src/prompts/select.rs:737`).
- Impact:
  - Duplicate work on every rerender (focus moves, selection toggles, layout notifications).
- Recommendation:
  - Build a `FilteredRowVm` during `refilter` that stores:
    - source choice index
    - title text + highlight ranges (or prepared text segments)
    - subtitle
    - shortcut badge
    - semantic id
  - Render should only map visible range -> row VM -> element.

### 5) `UnifiedListItem` highlighted text rendering allocates fragments aggressively

- Evidence:
  - `split_text_by_ranges` allocates `Vec<Div>` and `slice.to_string()` fragments (`src/components/unified_list_item/render.rs:394`, `src/components/unified_list_item/render.rs:411`, `src/components/unified_list_item/render.rs:422`, `src/components/unified_list_item/render.rs:433`).
- Impact:
  - Expensive in lists where highlighted rows rerender frequently.
- Recommendation:
  - Precompute highlight fragments in model layer when query changes.
  - Consider `SmallVec` for short segment counts and shared string storage (`SharedString`/`Arc<str>`) for segment payloads.

## P1 - Type-System Correctness and Maintainability

### 6) `Custom` content variants are type-level promises that are currently dropped

- Evidence:
  - `LeadingContent::Custom(_) => None`: `src/components/unified_list_item/render.rs:299`.
  - `TrailingContent::Custom(_) => None`: `src/components/unified_list_item/render.rs:344`.
  - `TextContent::Custom(_) => div()`: `src/components/unified_list_item/render.rs:390`.
- Impact:
  - Silent rendering loss and hard-to-debug behavior.
  - Wasted allocations when callers pass custom elements.
- Recommendation:
  - Either render these variants correctly now, or remove them from public API until supported.
  - If keeping custom content, split presentational API into:
    - fast-path typed variants (cloneable, cacheable)
    - explicit slow-path custom renderer (`fn render_custom(...)`) with clear perf tradeoff.

## P2 - Test Coverage Gaps

### 7) Existing tests validate basic helpers but not performance-sensitive behavior

- Evidence:
  - `src/components/unified_list_item_tests.rs` covers layout constants and UTF-8 boundary assertions only.
  - `src/prompts/select.rs` tests cover metadata parsing/scoring helpers only (`src/prompts/select.rs:800`).
- Missing tests:
  - Virtualized list behavior (visible-range only rendering).
  - Selection membership correctness under `HashSet`/`IndexSet` migration.
  - Deterministic filter ordering with tie-breaks.
  - `Custom` variant rendering behavior (or explicit rejection).
- Recommendation (test names):
  - `test_select_refilter_orders_by_score_then_stable_name_key`
  - `test_select_toggle_selection_is_constant_time_membership_path`
  - `test_select_virtual_list_renders_only_visible_range`
  - `test_unified_list_item_custom_variants_render_or_fail_explicitly`

## Suggested Refactor Plan

1. `SelectPrompt` data model pass:
   - Add `ChoiceSearchIndex` + cached metadata + cached semantic id.
   - Replace `selected: Vec<usize>` with `IndexSet<usize>` (or `HashSet<usize>` + ordered vec for submit).
2. `SelectPrompt` render pass:
   - Switch list rendering to `uniform_list` + scroll handle.
   - Render from cached filtered row VMs.
3. `UnifiedListItem` pass:
   - Remove per-render text-fragment allocations for highlighted content.
   - Resolve `Custom` variant behavior (render or remove).
4. Tests:
   - Add scenario tests above to lock in ranking, virtualization, and custom content semantics.

## Validation Plan

After implementing any batch of the above:

1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. Runtime prompt check via stdin protocol (large list):
   - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
   - `echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/<select-scenario>.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
5. Confirm lower refilter/render churn in logs (add timing spans with `correlation_id`).
