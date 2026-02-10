**Changed files**
- `src/actions/builders/notes.rs:46` Added trash-mode command-bar entries when `has_selection && is_trash_view`:
`restore_note` (`Restore Note`, `⌘Z`, `IconName::Refresh`, section `Trash`) and `permanently_delete_note` (`Delete Permanently`, `IconName::Trash`, section `Trash`).
- `src/actions/builders/notes.rs:242` Added tests covering trash-action presence with selection and absence without selection.

**How to test**
1. Open Notes, switch to Trash view, select a trashed note, press `Cmd+K`, confirm `Restore Note` and `Delete Permanently` are listed.
2. In Trash view with no selected note, press `Cmd+K`, confirm those two actions are not listed.
3. Run:
`cargo check`
`cargo clippy --all-targets -- -D warnings`

**Verification run**
- Ran `cargo check && cargo clippy --all-targets -- -D warnings`: failed due pre-existing workspace errors (not from `notes.rs`), including duplicate `mod tests` files and existing `clipboard_history` compile issues.
- Ran `cargo test --lib trash_actions`: also blocked by unrelated existing compile/test failures.

**Risks / known gaps**
- `src/notes/window/panels.rs:240` does not currently map `restore_note` / `permanently_delete_note` IDs in `execute_action`, so selecting these new Cmd+K items may no-op until that mapping is added.