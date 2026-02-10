Implemented the empty-state copy split in the dialog render path.

**Changed files**
- `src/actions/dialog/part_04/body_part_02.rs:15`
  - Replaced hardcoded empty text with:
  - `actions_dialog_empty_state_message(&self.search_text)`
- `src/actions/dialog/part_01.rs:250`
  - Added helper:
  - `actions_dialog_empty_state_message(search_text: &str) -> &'static str`
  - Returns `"No actions available"` when `search_text.trim().is_empty()`, otherwise `"No actions match your search"`.
- `src/actions/dialog/part_01.rs:307`
  - Added focused unit tests:
  - `test_actions_dialog_empty_state_message_returns_available_when_search_is_empty`
  - `test_actions_dialog_empty_state_message_returns_no_match_when_search_has_text`

**Verification run**
- `cargo check` (failed due unrelated workspace errors)
- `cargo clippy --all-targets -- -D warnings` (failed due unrelated workspace errors)

Key unrelated blockers observed:
- Duplicate module file conflicts:
  - `src/protocol/message/tests.rs` vs `src/protocol/message/tests/mod.rs`
  - `src/designs/core/tests.rs` vs `src/designs/core/tests/mod.rs`
- Existing compile failures in clipboard/history code (missing fields, non-exhaustive matches), plus many other pre-existing errors from concurrent edits.

**How to test**
1. After workspace-level compile issues are resolved, run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
2. Run the dialog and verify behavior:
   - No search text + zero actions: shows `No actions available`
   - Non-empty search text + zero matches: shows `No actions match your search`

**Risks / known gaps**
- Because of unrelated repo breakage from concurrent changes, I could not complete a green verification run in this branch state.
- Current logic treats whitespace-only input as empty (`trim().is_empty()`).