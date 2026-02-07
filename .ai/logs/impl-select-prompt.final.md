Implemented the highest-priority `SelectPrompt` fixes in `src/prompts/select.rs`.

**Changed files**
- `src/prompts/select.rs`

**What changed**
- Switched list rendering to virtualized `uniform_list` with `UniformListScrollHandle` and focus-follow scroll (`ScrollStrategy::Nearest`).
- Added cached per-choice index data (`SelectChoiceIndex`) to avoid repeated metadata parsing/lowercasing in refilter.
- Updated filtering/scoring to use cached lowercase fields and metadata.
- Changed selection state from `Vec<usize>` to `HashSet<usize>` for O(1) membership/toggle.
- Made Cmd/Ctrl+A toggle filtered selection based on set containment (not length), preserving off-filter selections.
- Changed key behavior:
  - `Space` now inserts a space into search.
  - `Cmd/Ctrl+Space` toggles selection.
- Updated single-select Enter semantics: if nothing is toggled, Enter submits the focused item.
- Stabilized fallback semantic IDs using source index (`fallback_select_semantic_id`) instead of display index.
- Added/updated unit tests in `src/prompts/select.rs`:
  - `test_select_prompt_accepts_space_in_filter_query`
  - `test_select_prompt_submit_uses_focused_item_in_single_mode_when_none_toggled`
  - `test_select_prompt_cmd_a_toggles_only_when_all_filtered_items_are_selected`
  - `test_select_prompt_select_all_preserves_existing_off_filter_selection`
  - `test_select_prompt_generates_stable_semantic_id_when_filter_order_changes`
  - Updated scoring tests for cached index path

**Verification run**
- `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`  
  Failed due unrelated in-progress shared-tree errors outside scope:
  - `src/render_prompts/form.rs` (`overflow_y_scroll` method)
  - `src/script_creation.rs` (clippy `needless_as_bytes`)
  - `src/ai/window.rs` (`AiApp::message_body_content` missing)
- `cargo check --lib` passed (confirms `select.rs` compiles in lib build).

**How to test (after shared-tree blockers are fixed)**
1. Run: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
2. Run focused tests: `cargo test --lib select::tests::test_select_prompt_accepts_space_in_filter_query`
3. Runtime check via stdin protocol:
   - Build binary
   - Run a script that opens a select prompt and verify:
     - typing spaces in filter works,
     - Cmd/Ctrl+Space toggles,
     - single-select Enter submits focused row,
     - long lists scroll to keep focus visible.

**Risks / known gaps**
- Full workspace verification is currently blocked by unrelated files above.
- Grouped select rows are still not implemented (out of scope for this edit).
- Multi-select Enter behavior when nothing is selected remains unchanged (still submits empty selection).

Commits made: none.