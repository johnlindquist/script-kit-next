Implemented the label/behavior mismatch fixes in the scoped files.

**Changed Files**
- `src/actions/builders/file_path.rs:70`
  - Renamed `open_with` action label/description to match current behavior:
    - Title: `Show Info`
    - Description: `Show file information in Finder`
  - Added note clarifying current behavior.
  - Added test: `test_get_file_context_actions_labels_open_with_as_show_info_when_macos` at `src/actions/builders/file_path.rs:119`.
- `src/actions/builders/notes.rs:101`
  - Renamed `copy_note_as` label/description:
    - Title: `Copy Note as Markdown`
    - Description: `Copy note as Markdown`
- `src/actions/builders/notes.rs:137`
  - Renamed `export` label/description:
    - Title: `Copy as HTML`
    - Description: `Copy note as HTML`
  - Added test: `test_get_notes_command_bar_actions_uses_markdown_and_html_copy_labels_when_selected` at `src/actions/builders/notes.rs:170`.

**Verification Run**
- Ran: `cargo check`  
  - Failed due unrelated existing branch errors (e.g. `src/clipboard_history/*` missing new `ClipboardEntry` fields, non-exhaustive `ContentType` matches).
- Ran: `cargo clippy --all-targets -- -D warnings`  
  - Failed due unrelated existing branch/test-tree errors (module collisions, unresolved imports across many pre-existing test files).
- Ran scoped test attempt:  
  - `cargo test --lib test_get_notes_command_bar_actions_uses_markdown_and_html_copy_labels_when_selected -- --exact`
  - Also blocked by unrelated existing compile errors in other modules/tests.

**How To Test (once branch compiles cleanly)**
1. `cargo test --lib test_get_file_context_actions_labels_open_with_as_show_info_when_macos -- --exact`
2. `cargo test --lib test_get_notes_command_bar_actions_uses_markdown_and_html_copy_labels_when_selected -- --exact`
3. `cargo check`
4. `cargo clippy --all-targets -- -D warnings`

**Risks / Known Gaps**
- `open_with` and `show_info` now both display “Show Info” (different IDs/shortcuts), which is consistent with current behavior but still reflects underlying runtime duplication.
- Full verification is currently blocked by unrelated, pre-existing compile/clippy failures outside scoped files.