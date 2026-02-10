Implemented the assigned fixes in the scoped window files.

Changed files:
- `src/actions/window/part_02.rs:50` added `test_actions_window_dynamic_height_matches_single_row_when_empty`.
- `src/actions/window/part_02.rs:63` added shared `actions_window_dynamic_height(...)` and used it in open-path sizing at `src/actions/window/part_02.rs:136`.
- `src/actions/window/part_02.rs:257` added warn/debug logging for failed popup configure update (position/focus context).
- `src/actions/window/part_02.rs:294` added warn/debug logging for failed close update.
- `src/actions/window/part_02.rs:337` added warn/debug logging for failed notify update.
- `src/actions/window/part_03.rs:30` and `src/actions/window/part_03.rs:159` switched resize paths to the shared height helper (unifies empty-state height with open path).
- `src/actions/window/part_03.rs:275` upgraded resize update failure logging to warn + debug context.

Verification run:
- `cargo check` failed due pre-existing unrelated workspace errors (examples: `src/clipboard_history/database.rs` missing new `ClipboardEntry` fields; non-exhaustive `ContentType` matches).
- `cargo clippy --all-targets -- -D warnings` failed due pre-existing unrelated workspace issues (examples: duplicate `tests` module files in `src/protocol/message` and `src/designs/core`, plus many existing test/import warnings/errors).
- `cargo test test_actions_window_dynamic_height_matches_single_row_when_empty -- --nocapture` also blocked by unrelated compile/test-tree errors.

How to test once workspace baseline is green:
1. `cargo test test_actions_window_dynamic_height_matches_single_row_when_empty -- --nocapture`
2. `cargo check`
3. `cargo clippy --all-targets -- -D warnings`
4. Optional runtime log check: `echo '{"type":"show"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`

Risks / known gaps:
- I could not complete a green verification pass because of unrelated existing compile/clippy failures outside `src/actions/window/`.
- Logging paths for update failures are now explicit, but runtime exercise of those failure branches wasn’t possible in this workspace state.

Commits:
- None (not committed because required verification is currently failing due unrelated repo issues).