Implemented the shortcut conflict fix by changing Notes command-bar `Copy Deeplink` from `⇧⌘D` to `⇧⌘Y`, so `⇧⌘D` remains for `Insert Date/Time` in keyboard handling.

Changed files:
- `src/actions/builders/notes.rs:118` (`copy_deeplink` shortcut now `⇧⌘Y`)
- `src/actions/builders/notes.rs:200` (added regression test `test_get_notes_command_bar_actions_sets_non_conflicting_copy_deeplink_shortcut`)

Verification run:
- `cargo check` (failed due pre-existing unrelated errors in `clipboard_history/*`, e.g. missing `ClipboardEntry` fields and non-exhaustive `ContentType` matches)
- `cargo clippy --all-targets -- -D warnings` (failed due broad pre-existing unrelated errors, including duplicate `tests` module files and many existing unused-import/test-module issues)
- Also attempted scoped test:
  - `cargo test --lib test_get_notes_command_bar_actions_sets_non_conflicting_copy_deeplink_shortcut` (blocked by unrelated existing compile issues outside this task)

How to test (once workspace is green):
1. Run `cargo check`
2. Run `cargo clippy --all-targets -- -D warnings`
3. Open Notes with a selected note, open command bar, confirm `Copy Deeplink` shows `⇧⌘Y`
4. Press `⇧⌘D` in editor, confirm it still inserts date/time

Risks / known gaps:
- Other Notes UI surfaces outside assigned scope may still display `Copy Deeplink` as `⇧⌘D` (for example in actions-panel metadata), so there may be temporary shortcut-label inconsistency until those are aligned.