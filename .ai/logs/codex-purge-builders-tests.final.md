Purged redundant tests from `src/actions/builders_tests.rs`.

Changed files:
- `src/actions/builders_tests.rs`

What I removed:
- `clipboard_text_has_all_destructive_actions`
- `clipboard_text_has_save_actions`
- `clipboard_image_has_ocr_action` (duplicate of stronger earlier OCR coverage)
- `clipboard_action_count_text_vs_image` (weak/brittle count-based assertion)

Why:
- These were duplicate/low-value checks already covered earlier in the same file by stronger assertions (`clipboard_text_entry_has_core_actions`, `clipboard_image_entry_has_ocr_action`, `clipboard_text_entry_lacks_image_specific_actions`).

How to test:
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. Optional focused run: `cargo test actions::builders::builders_extended_tests`

Validation status I observed:
- `cargo check`: passed (standalone run).
- `cargo clippy --all-targets -- -D warnings`: fails due unrelated repo-wide issues (unused imports/dead code and other lints in `src/actions/dialog_builtin_action_validation_tests_*.rs`, plus clippy findings in `src/prompts/select.rs`).
- `cargo test`: failing due unrelated in-progress changes (e.g. failures in `src/actions/dialog_builtin_action_validation_tests_39.rs`, `src/actions/dialog_builtin_action_validation_tests_41.rs`, `src/actions/dialog_builtin_action_validation_tests_46.rs`; and in another run compile errors in `src/prompts/chat.rs`).

Risks / known gaps:
- Full green gate is currently blocked by concurrent unrelated agent changes outside this task’s scope.
- I only removed duplicate/low-signal tests; no behavior tests were added or rewritten.