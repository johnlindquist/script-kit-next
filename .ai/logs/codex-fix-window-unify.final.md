Changed files:
- `src/actions/window/part_02.rs:63`  
  Added shared `compute_popup_height(dialog: &ActionsDialog) -> f32` and kept all popup-size math centralized in `actions_window_dynamic_height(...)`, including a single `POPUP_BORDER_HEIGHT` constant.
- `src/actions/window/part_02.rs:120`  
  `open_actions_window(...)` now uses `compute_popup_height(&dialog)` instead of inline/per-path height assembly.
- `src/actions/window/part_03.rs:23`  
  `resize_actions_window_direct(...)` now uses `compute_popup_height(&dialog)`.
- `src/actions/window/part_03.rs:144`  
  `resize_actions_window(...)` now uses `compute_popup_height(&dialog)`.

How to test:
1. Run `cargo check -p script-kit-gpui`
2. Run `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
3. Once build is green, open actions popup with empty results and resize/filter changes; initial open and resize should now match height behavior.

Verification results:
- `cargo check -p script-kit-gpui` failed due unrelated pre-existing errors outside this scope:
  - `src/clipboard_history/database.rs`, `src/clipboard_history/db_worker/db_impl.rs` (`E0063`, missing `ClipboardEntry` fields)
  - `src/clipboard_history/clipboard.rs`, `src/clipboard_history/database.rs`, `src/clipboard_history/quick_look.rs`, `src/clipboard_history/temp_file.rs` (`E0004`, non-exhaustive `ContentType` matches)
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings` also failed due unrelated repository-wide issues (including the above, plus `E0761` duplicate test module files and many unrelated warnings/errors in tests and prompt modules).

Risks / known gaps:
- I could not complete a clean check/clippy run because of unrelated parallel changes in other files.
- Runtime/UI verification of the popup behavior is blocked until those compile errors are resolved.