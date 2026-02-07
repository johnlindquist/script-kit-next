Implemented the highest-priority `unified_list_item` list-render hot-path optimization: highlight text fragments are now precomputed once at `TextContent::highlighted(...)` construction time and reused during render.

Changed files:
- `src/components/unified_list_item/types.rs:10`
- `src/components/unified_list_item/render.rs:348`
- `src/components/unified_list_item_tests.rs:57`

What changed:
- Added `HighlightFragment` and cached fragments inside `TextContent::Highlighted` (`src/components/unified_list_item/types.rs:10`, `src/components/unified_list_item/types.rs:32`).
- Added `TextContent::highlight_fragments()` accessor for render-time reuse (`src/components/unified_list_item/types.rs:93`).
- Replaced per-render substring splitting/string allocation with fragment-based rendering (`src/components/unified_list_item/render.rs:373`, `src/components/unified_list_item/render.rs:397`).
- Added tests for fragment precomputation and plain-text no-fragment behavior (`src/components/unified_list_item_tests.rs:57`, `src/components/unified_list_item_tests.rs:73`).

How to test:
1. `cargo check`
2. `cargo test test_highlighted_text_precomputes_fragments_when_constructed -- --nocapture`
3. `cargo test test_plain_text_has_no_highlight_fragments -- --nocapture`

Verification run:
- `cargo check` passed.
- Targeted `cargo test` currently fails before running these tests due unrelated compile error in `src/ai/window.rs:8729` (`AiApp::message_body_content` missing), outside this scope.

Risks / known gaps:
- `TextContent::Custom`, `LeadingContent::Custom`, and `TrailingContent::Custom` render behavior remains unchanged (still not rendered).
- Render still allocates `Vec<Div>` each frame; this change removes the expensive repeated substring/string allocations but not all render allocations.

Commits:
- None.