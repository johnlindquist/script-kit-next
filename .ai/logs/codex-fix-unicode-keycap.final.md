Implemented the Unicode-safety fix in `src/actions/dialog/part_02/part_01.rs`.

Changed files:
- `src/actions/dialog/part_02/part_01.rs:159`  
  Replaced byte-slice truncation with `Self::clipboard_context_title(&entry_info.preview)`.
- `src/actions/dialog/part_02/part_01.rs:185`  
  Added `clipboard_context_title(preview: &str)` that truncates using `.chars().take(...)` (Unicode-safe, no byte-boundary panic).
- `src/actions/dialog/part_02/part_01.rs:402`  
  Added focused tests:
  - `test_clipboard_context_title_does_not_panic_when_preview_contains_multibyte_unicode`
  - `test_parse_shortcut_keycaps_does_not_panic_when_shortcut_contains_multibyte_unicode`

How to test:
1. `cargo check -p script-kit-gpui`
2. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`

Verification status:
- Both commands were run and currently fail due unrelated workspace issues (outside this file), mainly in `src/clipboard_history/*`, `src/actions/tests/*`, and duplicate test module files under `src/protocol/message/*` / `src/designs/core/*`.
- No diagnostics were emitted for `src/actions/dialog/part_02/part_01.rs`.

Risks / known gaps:
- Because the workspace is currently broken by unrelated parallel changes, full green verification isn’t possible right now.
- Truncation is now Unicode scalar-safe (no panic), but still not grapheme-cluster aware for complex joined emoji sequences.