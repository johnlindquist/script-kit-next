Implemented the `src/notes/window.rs` split into `src/notes/window/*.rs`, with each file under 500 lines, and wired module exports so `notes` API remains available.

Changed files:
- `src/notes/window.rs`
- `src/notes/window/clipboard_ops.rs`
- `src/notes/window/editor_formatting.rs`
- `src/notes/window/editor_ops_a.rs`
- `src/notes/window/editor_ops_b.rs`
- `src/notes/window/init.rs`
- `src/notes/window/keyboard.rs`
- `src/notes/window/navigation.rs`
- `src/notes/window/notes.rs`
- `src/notes/window/notes_actions.rs`
- `src/notes/window/panels.rs`
- `src/notes/window/render.rs`
- `src/notes/window/render_editor.rs`
- `src/notes/window/render_editor_body.rs`
- `src/notes/window/render_editor_footer.rs`
- `src/notes/window/render_editor_titlebar.rs`
- `src/notes/window/render_overlays.rs`
- `src/notes/window/render_shortcuts.rs`
- `src/notes/window/render_ui.rs`
- `src/notes/window/tests.rs`
- `src/notes/window/traits.rs`
- `src/notes/window/vibrancy.rs`
- `src/notes/window/window_ops.rs`

How to test:
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`

Verification results from this run:
- Ran all 3 commands.
- Gate is currently blocked by unrelated parallel-agent breakages outside `src/notes/*`, e.g.:
  - `src/prompts/chat/actions.rs` (doc comment placement)
  - `src/prompts/chat/render_core.rs` (unexpected `}`)
  - Earlier runs also hit unrelated in-progress split issues in `src/main_sections/*` and `src/app_impl/*`.
- Notes-specific compile errors from this split were resolved (visibility/wiring/lifetime/type issues).

Risks / known gaps:
- Full repo verification could not complete due unrelated compile failures in other active agent scopes.
- UI runtime/screenshot verification could not be executed because the project currently does not build end-to-end.
- I widened many extracted `NotesApp` methods to `pub(super)` for sibling-module access; behavior should be unchanged, but visibility is broader than file-private.

Commits:
- None (no commit made because full verification gate is not passing in the shared tree).