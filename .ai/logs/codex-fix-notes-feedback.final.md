Implemented the stale/missing selected-note feedback fix in the scoped notes window paths.

**Changed Files**
- `src/notes/window/notes_actions.rs`
  - Added shared selection resolver + feedback helpers:
    - `resolve_selected_note(...)`
    - `show_selected_note_missing_feedback(...)`
    - `selected_note_for_action(...)`
  - Updated:
    - `copy_note_as_markdown` to mutable receiver
    - `copy_note_deeplink` to validate selected note and show feedback when stale/missing
    - `create_note_quicklink` to validate selected note and show feedback when stale/missing
    - `duplicate_selected_note` to use shared stale/missing feedback path
- `src/notes/window/notes.rs`
  - Updated `export_note` to mutable receiver and to use shared selected-note validation/feedback instead of silent return.
- `src/notes/window/panels.rs`
  - In `execute_note_switcher_action`, stale `note_{uuid}` actions now show missing-note feedback and close browse panel instead of silent close/warn-only.
- `src/notes/window/tests.rs`
  - Added unit tests for selected-note resolution:
    - `test_resolve_selected_note_returns_none_when_selection_is_missing`
    - `test_resolve_selected_note_returns_none_when_selection_is_stale`
    - `test_resolve_selected_note_returns_note_when_selection_exists`

**Verification Run**
- Ran: `cargo check -p script-kit-gpui`
  - Fails due unrelated concurrent workspace issues (not in notes scope), e.g.:
    - `src/clipboard_history/database.rs` missing fields in `ClipboardEntry`
    - non-exhaustive `ContentType` matches in clipboard modules
- Ran: `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
  - Fails due broad unrelated errors/warnings in other modules/tests (duplicate test modules, missing imports/types in actions test files, clipboard type changes, etc.).
- Ran scoped test attempt: `cargo test -p script-kit-gpui notes::window::tests -- --nocapture`
  - Blocked by the same unrelated repo-wide compile failures.

**How To Test (once repo-wide compile blockers are cleared)**
1. Open Notes window, trigger `Duplicate`, `Copy Deeplink`, `Create Quicklink`, and `Export` with no valid selected note (or stale selection state).
2. Confirm feedback appears: `Selected note could not be found`.
3. For note switcher (`Cmd+P`), trigger a stale `note_{uuid}` action and confirm the same feedback appears and switcher closes.
4. Re-run:
   - `cargo check -p script-kit-gpui`
   - `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
   - `cargo test -p script-kit-gpui notes::window::tests`

**Risks / Known Gaps**
- Verification is currently blocked by unrelated parallel changes outside this task’s scope, so full green check/clippy could not be completed in this workspace state.