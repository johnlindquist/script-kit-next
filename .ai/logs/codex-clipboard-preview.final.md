**Changed files**
- `src/render_builtins/clipboard_preview.rs:40`  
Added `Link`, `File`, and `Color` to the content-type label match, mapped to `"Text"`.
- `src/render_builtins/clipboard_preview.rs:120`  
Added `Link`, `File`, and `Color` to the preview-rendering match, reusing the existing text-preview branch.

**Verification run**
- `cargo check -p script-kit-gpui` (ran as requested): fails due unrelated existing compile errors in other files (`src/app_actions/handle_action.rs`, `src/execute_script/part_001_body/execute_interactive_merged.rs`, `src/app_execute/builtin_execution.rs`, `src/app_render/group_header_item.rs`).
- `cargo check -p script-kit-gpui --lib`: passes, confirming this file’s change compiles.

**How to test**
1. Run `cargo check -p script-kit-gpui --lib`.
2. Optionally run `cargo check -p script-kit-gpui` after unrelated compile errors in other files are resolved.

**Risks / known gaps**
- `Link`, `File`, and `Color` are intentionally treated as `Text` in this view (label + preview). If distinct UI treatment is needed later, this will need follow-up.

**Commits**
- None.