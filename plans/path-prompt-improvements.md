# Path Prompt Improvements Audit

Date: 2026-02-07  
Agent: `codex-path-prompt`  
Scope: `src/render_prompts/path.rs`

## Executive Summary

The current path prompt is functional but still behaves like a minimal directory picker. It is missing advanced completion and filtering behavior expected from a modern path UX, and it does synchronous directory work on the UI path.

Highest-impact improvements:

1. Move directory loading/filtering off the UI thread for large-directory responsiveness.
2. Add real path completion semantics (segment completion, `~`/relative expansion, predictable Tab behavior).
3. Add explicit file-type filtering and hidden-file toggles.
4. Improve navigation model (breadcrumbs/history/parent row) to reduce key friction.

## Responsibility Map (Current)

`src/render_prompts/path.rs` currently:

1. Opens/closes the path actions dialog and executes selected actions (`src/render_prompts/path.rs:7`, `src/render_prompts/path.rs:40`).
2. Syncs actions-search text into shared mutex state during render (`src/render_prompts/path.rs:82`).
3. Intercepts keys at an outer container, routes keys to actions dialog when visible, and toggles Cmd+K (`src/render_prompts/path.rs:96`).
4. Wraps and overlays the `PathPrompt` entity (`src/render_prompts/path.rs:301`).

Core browsing behavior actually lives in `src/prompts/path.rs`:

1. Synchronous `read_dir` loading and sort (`src/prompts/path.rs:203`).
2. `contains`-only filtering (`src/prompts/path.rs:258`).
3. Directory navigation and keyboard handling (`src/prompts/path.rs:444`, `src/prompts/path.rs:486`).
4. Uniform-list rendering with per-render cloned row tuples (`src/prompts/path.rs:554`).

## Findings (Ranked)

### P1: Large directories can stall UI due to synchronous listing and filtering

Evidence:

1. `load_entries` performs blocking `std::fs::read_dir` + per-entry `is_dir` + full sort (`src/prompts/path.rs:203`).
2. Filtering re-walks the full entry list and allocates a new vector on each key press (`src/prompts/path.rs:258`).
3. Render clones all filtered rows into `Vec<(String, bool)>` before `uniform_list` callback (`src/prompts/path.rs:554`).

Impact:

1. Noticeable key latency/freezes when opening directories with thousands of entries.
2. Increased allocations and avoidable CPU work while typing filters.

Recommendation:

1. Reuse streaming directory infrastructure (`list_directory_streaming`) with cancel tokens (`src/file_search.rs:563`).
2. Apply generation guard + stale-result drop pattern already used in file search (`src/app_impl.rs:2836`).
3. Batch UI updates every ~16ms and avoid full list cloning in render.

### P1: Path completion is incomplete and Tab behavior is overloaded

Evidence:

1. Tab/right always attempts directory navigation, never textual completion (`src/prompts/path.rs:517`).
2. Filter text is simple free text; no segment parsing or path expansion.
3. Enter submits raw filter text as a path when no row is selected (`src/prompts/path.rs:368`), without normalization/validation.

Impact:

1. Users cannot use shell-like completion workflows.
2. Higher error rate for partial paths and relative-path input.

Recommendation:

1. Add a path query parser supporting:
   - absolute and relative paths (`./`, `../`)
   - `~` expansion
   - optional environment expansion (`$HOME`, etc.)
2. Define Tab behavior:
   - first: complete current segment (longest common prefix)
   - second (or when segment fully resolved): navigate into selected directory
3. Normalize on submit and show inline validation feedback for invalid paths.

### P1: Navigation model is efficient for experts but weak for mixed keyboard/mouse workflows

Evidence:

1. Parent navigation is only left/shift+tab/backspace-empty (`src/prompts/path.rs:428`, `src/prompts/path.rs:444`).
2. No `..` row, no breadcrumb clicks, no back/forward directory history.
3. `navigate_to` clears filter every time (`src/prompts/path.rs:296`), interrupting scan/search flow.

Impact:

1. Users lose context during directory traversal.
2. Frequent parent/child movement requires repetitive keying.

Recommendation:

1. Preserve filter across directory transitions when still relevant (or make behavior configurable).
2. Add optional first-row `..` entry and breadcrumb segment navigation.
3. Maintain `back_stack`/`forward_stack` for Cmd+[ / Cmd+] (or Alt+Left/Right) navigation.

### P2: File-type filtering is absent

Evidence:

1. `PathEntry` stores only name/path/is_dir (`src/prompts/path.rs:119`).
2. UI only distinguishes folder/file icon; no type classification (`src/prompts/path.rs:571`).
3. Hidden files are always excluded, with no override (`src/prompts/path.rs:219`).

Impact:

1. Hard to quickly narrow to “directories only”, “images only”, etc.
2. Hidden file workflows are impossible from this prompt.

Recommendation:

1. Extend row model with `FileType` classification and metadata (reuse patterns from `src/file_search.rs:29`).
2. Support filter tokens:
   - `type:dir|file|image|doc|audio|video|app`
   - `ext:rs` (repeatable)
   - `hidden:true|false`
3. Add quick toggle chips/actions for common presets.

### P2: `render_prompts/path.rs` still carries avoidable state coupling

Evidence:

1. Search text and visibility are synchronized through `Arc<Mutex<...>>` shared state (`src/render_prompts/path.rs:82`).
2. Cmd+K toggle behavior exists in both outer and inner handlers (`src/render_prompts/path.rs:129`, `src/prompts/path.rs:500`).
3. Outer handler logs key state but without structured context/correlation_id fields.

Impact:

1. Higher complexity and potential divergence between wrapper and prompt key behavior.
2. Harder to diagnose key-routing issues in complex prompt states.

Recommendation:

1. Move actions visibility/search state into a single owner (app-level prompt state or entity state), avoid render-time lock writes.
2. Keep Cmd+K handling in one layer only, with clear precedence.
3. Add structured tracing fields including `correlation_id`, `current_path`, `filtered_count`, `actions_open`.

### P3: Current filtering quality is basic substring matching

Evidence:

1. Filter uses case-insensitive `contains` only (`src/prompts/path.rs:267`).
2. Better fuzzy scoring helper already exists (`filter_results_nucleo_simple`) for file search (`src/file_search.rs:1412`).

Impact:

1. Poor ranking for non-contiguous queries and camel/snake/file-extension patterns.
2. Lower discoverability when many similarly named files exist.

Recommendation:

1. Replace/augment substring filter with fuzzy scoring for ranked results.
2. Keep deterministic tie-breakers: directories first, then score, then alphabetical.

## Proposed Implementation Plan

### Phase 1: Performance + correctness baseline

1. Add async directory listing pipeline to `PathPrompt`:
   - cancel token
   - generation counter
   - batched apply to UI state
2. Introduce `PathEntryView` with cached lowercase name/type to reduce per-keystroke recompute.
3. Remove render-time shared-state lock writes in `render_prompts/path.rs`.

### Phase 2: Completion and navigation UX

1. Implement `PathInputState` parser:
   - active directory
   - segment query
   - filter tokens
2. Add Tab completion state machine (complete segment vs navigate).
3. Add breadcrumb + optional `..` row + history stack.

### Phase 3: Filtering and polish

1. Add tokenized type/extension/hidden filters.
2. Upgrade ranking with fuzzy scoring.
3. Add richer list row descriptions (type badge, size/date optional in compact form).

## TDD Test Plan

1. `test_path_prompt_lists_large_directory_without_blocking_ui_thread`
2. `test_path_prompt_cancels_stale_directory_listing_when_query_changes`
3. `test_path_prompt_tab_completes_current_segment_before_navigation`
4. `test_path_prompt_expands_tilde_and_relative_segments_on_submit`
5. `test_path_prompt_supports_type_and_extension_filter_tokens`
6. `test_path_prompt_preserves_filter_when_navigating_between_related_directories`
7. `test_path_prompt_backspace_empty_moves_to_parent_without_crash_at_root`
8. `test_path_prompt_cmd_k_toggle_is_handled_in_single_layer_only`
9. `test_path_prompt_logs_navigation_with_correlation_id_and_duration_ms`
10. `test_path_prompt_ranks_results_with_fuzzy_score_then_directory_priority`

## Suggested Scenario Smoke Tests (stdin protocol)

1. Large-directory scenario:
   - run path prompt on a directory with >10k entries
   - type rapidly and ensure logs show cancellation + no stalled key handling
2. Completion scenario:
   - input partial nested path, hit Tab repeatedly, verify segment completion order
3. Type filter scenario:
   - apply `type:dir ext:rs` and verify filtered counts/rows

## Risks / Known Gaps

1. Async listing introduces concurrency/state race risk unless generation + cancellation rules are enforced strictly.
2. Tokenized query grammar needs a clear backward-compatible path with existing plain-text filter behavior.
3. Rich metadata (size/date/type detection) can regress first-paint performance if hydrated eagerly; should be lazy for visible rows.
