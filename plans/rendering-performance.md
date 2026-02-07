# Rendering and UI Performance Audit

Date: 2026-02-07  
Agent: `codex-perf-rendering`  
Scope: `src/render_prompts/**/*.rs`, `src/prompts/**/*.rs`, `src/components/**/*.rs`

## Summary

The largest rendering costs are in prompt render paths that do expensive parsing/cloning work on every frame or keystroke:

1. `DivPrompt` reparses HTML on every render and repeatedly clones render context.
2. `ArgPrompt`, `PathPrompt`, and `EnvPrompt` perform allocation-heavy string splitting/cloning in hot paths.
3. `SelectPrompt` renders all rows non-virtualized and uses O(n) membership checks.
4. Some render paths mutate state every render (`EditorPrompt`/`TermPrompt`) or perform render-time side effects (`render_path_prompt` search sync).

The codebase already has a strong pattern in `Markdown` prompt caching (`src/prompts/markdown.rs:6-9`, `src/prompts/markdown.rs:720-729`), and similar caching/virtualization patterns should be applied in the hotspots below.

## Findings

## P0 - Reparse/Rebuild Work in Render Hot Paths

### 1) `DivPrompt` reparses HTML and recreates callback/context every render

- Evidence:
  - `src/prompts/div.rs:887` reparses `parse_html(&self.html)` inside `render`.
  - `src/prompts/div.rs:892-914` creates a new `Arc` link callback every render.
  - `src/prompts/div.rs:325-330` clones `RenderContext` per element.
  - `src/prompts/div.rs:755-845` deep recursive `ctx.clone()` calls in inline rendering.
  - `src/prompts/div.rs:976-992` reapplies Tailwind class parsing in render.
- Impact:
  - Parsing and context cloning scale with document size and happen on each rerender (focus changes, cursor blink, key events).
- Recommendation:
  - Cache parsed HTML AST in `DivPrompt` state with a dirty flag keyed by `self.html`.
  - Cache precomputed styled blocks for unchanged `tailwind`/`container_classes`.
  - Store link callback once (e.g., in `DivPrompt`) and reuse.
  - Refactor `RenderContext` to avoid cloning `Arc` in deep recursion (pass refs where possible).

### 2) `ArgPrompt` does avoidable filtering/cloning each render

- Evidence:
  - `src/render_prompts/arg.rs:150` computes `self.filtered_arg_choices()` into unused `_filtered`.
  - `src/render_prompts/arg.rs:365` clones owned filtered data for render closure.
  - `src/render_prompts/arg.rs:389-395` per-row `choice.name.clone()` and `choice.description.clone()`.
  - `src/render_prompts/arg.rs:60`, `src/render_prompts/arg.rs:89-91`, `src/render_prompts/arg.rs:116-117` build `Vec<char>` and new `String`s per render.
- Impact:
  - Frequent key events trigger repeated allocations and full filtered snapshot copies.
- Recommendation:
  - Remove unused `_filtered` call.
  - Maintain `filtered_indices: Vec<usize>` and render from source choices by index.
  - Cache row view-models when filter text changes, not on every render.
  - Replace char-vec slicing path with shared helper that avoids rebuilding full `Vec<char>` each frame.

### 3) `PathPrompt` uses cloning + locking in render path

- Evidence:
  - `src/prompts/path.rs:260-269` clones entries and lowercases each name per filter pass.
  - `src/prompts/path.rs:555-559` clones all entry names each render into `entries_for_list`.
  - `src/prompts/path.rs:572-576` allocates emoji strings per visible row.
  - `src/render_prompts/path.rs:82-94` writes shared search text mutex during render.
  - `src/prompts/path.rs:605-609` reads+clones shared search text mutex during render.
- Impact:
  - Extra allocations and mutex work in render can cause frame jitter, especially with large directories.
- Recommendation:
  - Store lowercase name in `PathEntry` once and filter on it.
  - Keep filtered indices instead of cloning `PathEntry`.
  - Use static/shared icon constants rather than per-row `to_string()`.
  - Move actions-search synchronization out of render into action-dialog event handlers.

## P1 - Missing/Fragile Notify and Unnecessary Render-time Mutations

### 4) Potential missing `cx.notify()` after closing path actions on Enter

- Evidence:
  - In Enter handler branch, state is mutated (`show_actions_popup`, `actions_dialog`, `path_actions_showing`) at `src/render_prompts/path.rs:176-181` without an immediate `cx.notify()`.
  - Other close paths do call notify (`src/render_prompts/path.rs:139`, `src/render_prompts/path.rs:213`, `src/render_prompts/path.rs:289`).
- Impact:
  - UI update may depend on unrelated subsequent notifications, causing stale dialog visibility in some action paths.
- Recommendation:
  - Call `cx.notify()` immediately after close-state mutation in the Enter/Return close branch (before/independent of action execution path).

### 5) `EditorPrompt` / `TermPrompt` update child state on every render

- Evidence:
  - `src/render_prompts/editor.rs:15-17` always runs `entity.update` to set `editor.suppress_keys`.
  - `src/render_prompts/term.rs:22-24` always runs `entity.update` to set `term.suppress_keys`.
- Impact:
  - Can trigger unnecessary child invalidation/update churn every frame even when value is unchanged.
- Recommendation:
  - Guard update with equality check and only update when `show_actions_popup` changes.
  - Optionally track previous suppress state in parent (`ScriptListApp`) to avoid cross-entity reads.

## P1 - Non-virtualized or O(n) Selection Work

### 6) `SelectPrompt` renders full list and uses O(n) selection checks

- Evidence:
  - Full render loop over all filtered rows: `src/prompts/select.rs:731-770`.
  - Membership check per row: `src/prompts/select.rs:734` uses `self.selected.contains(&choice_idx)`.
- Impact:
  - Large choice sets cause heavy rebuild cost and O(n^2) behavior when many selections exist.
- Recommendation:
  - Replace row loop with `uniform_list` virtualization.
  - Store selected items in `HashSet<usize>` for O(1) membership.
  - Precompute `ChoiceDisplayMetadata` and highlighted title when filter changes.

## P2 - Recomputed Derived Data in Render

### 7) `TemplatePrompt` recomputes full preview string each render

- Evidence:
  - `src/prompts/template.rs:453` calls `preview_template()` during render.
  - `src/prompts/template.rs:287-300` repeatedly `replace()` across all placeholders.
- Impact:
  - Scales with template size and input count; unnecessary when no input changed.
- Recommendation:
  - Cache preview with dirty flag updated in `set_input`, `handle_char`, and `handle_backspace`.

### 8) `ChatPrompt` clones turn cache snapshot per render and clones turn data per row

- Evidence:
  - `src/prompts/chat.rs:3030` clones `conversation_turns_cache` into render snapshot.
  - `src/prompts/chat.rs:2241`, `src/prompts/chat.rs:2247`, `src/prompts/chat.rs:2271`, `src/prompts/chat.rs:2302` clone row fields in render.
- Impact:
  - Cost grows with conversation length and frequency of rerenders.
- Recommendation:
  - Keep conversation turn fields as `SharedString`/`Arc<str>` where possible.
  - Build row view-models only when turns cache is marked dirty.
  - Consider avoiding per-render snapshot clone by using stable shared immutable data structure.

## P2 - Component-level Allocation Hotspots

### 9) `UnifiedListItem` highlight splitting allocates many small strings

- Evidence:
  - `src/components/unified_list_item/render.rs:401`, `src/components/unified_list_item/render.rs:411`, `src/components/unified_list_item/render.rs:422`, `src/components/unified_list_item/render.rs:433`.
  - Creates a `Vec<Div>` and `slice.to_string()` fragments per render.
- Impact:
  - Significant with large virtualized lists and frequent filter updates.
- Recommendation:
  - Precompute highlighted segments with filtered results.
  - Use compact container (`SmallVec`) and shared strings where possible.

### 10) `FormTextField` / `FormTextArea` clone/slice strings in render

- Evidence:
  - `src/components/form_fields.rs:645-646` slice-by-char each render.
  - `src/components/form_fields.rs:658`, `src/components/form_fields.rs:671` allocate `to_string()` fragments.
  - `src/components/form_fields.rs:1157` clones textarea text each render.
- Impact:
  - Acceptable for small forms but spikes with many fields and rapid typing.
- Recommendation:
  - Reuse a shared cursor/text rendering helper with reduced intermediate allocations.
  - Store display fragments as cached derived state updated on input mutation.

### 11) `EnvPrompt` repeats ArgPrompt-style char vector slicing in render

- Evidence:
  - `src/prompts/env.rs:244-245`, `src/prompts/env.rs:263-265`, `src/prompts/env.rs:283-285`.
- Impact:
  - Repeated per-keystroke allocations in input-heavy flow.
- Recommendation:
  - Reuse optimized input rendering helper used by other prompts.

## Implementation Plan (Recommended Order)

1. Eliminate clear hot-path waste:
   - Remove `_filtered` in arg render.
   - Add missing notify in path Enter-close branch.
   - Guard `suppress_keys` updates in editor/term render.
2. Add data-shape optimizations:
   - `PathPrompt` and `ArgPrompt` switch to index-based filtered lists.
   - `SelectPrompt` migrate to `uniform_list` + `HashSet` selection.
3. Add cache layers:
   - `DivPrompt` parsed HTML + class-application cache.
   - `TemplatePrompt` preview cache.
   - Optional: `ChatPrompt` row view-model cache.
4. Component-level polish:
   - Optimize `UnifiedListItem` highlight fragments.
   - Optimize form/env input text split helpers.

## Validation Plan

After each optimization batch:

1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. Runtime sanity via stdin protocol:
   - `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
   - For prompt-specific checks, run corresponding smoke scripts through stdin JSON protocol.
5. Compare logs for reduced render churn:
   - fewer repetitive render-time side-effect logs
   - stable key handling latency under rapid navigation/typing

