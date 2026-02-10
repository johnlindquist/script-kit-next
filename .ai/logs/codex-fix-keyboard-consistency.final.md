Changed files:
- `src/actions/command_bar/part_01.rs:282`
  - Added `test_command_bar_key_intent_maps_required_key_variants_to_same_intents` to assert:
    - `up` + `arrowup` -> `MoveUp`
    - `down` + `arrowdown` -> `MoveDown`
    - `enter` + `Enter` -> `ExecuteSelected`
    - `escape` + `Escape` -> `Close`
- `src/actions/window/part_01.rs:544`
  - Added `test_actions_window_key_intent_maps_required_navigation_key_variants` for `up/arrowup` and `down/arrowdown`.
- `src/actions/window/part_01.rs:567`
  - Added `test_actions_window_key_intent_maps_required_confirm_and_cancel_key_variants` for `enter/Enter` and `escape/Escape`.

What I found in the requested audit:
- `src/actions/dialog/part_01.rs`, `src/actions/command_bar/part_02.rs`, and `src/actions/command_bar/part_03.rs` did not have direct single-variant key matches needing code changes.
- Existing key routing already goes through shared helpers (`is_key_up`, `is_key_down`, `is_key_enter`, `is_key_escape`) that handle variants case-insensitively.

Verification run:
- `cargo check -p script-kit-gpui` -> failed due unrelated in-flight repo errors (examples: `src/actions/dialog/part_02/part_03.rs`, multiple `clipboard_history/*` files, duplicate test module paths).
- `cargo clippy -p script-kit-gpui --all-targets -- -D warnings` -> failed due unrelated existing errors/warnings outside this task.
- `cargo test -p script-kit-gpui --lib key_intent_maps_required` -> failed for the same unrelated compile blockers.

How to test (once branch compiles cleanly again):
1. `cargo check -p script-kit-gpui`
2. `cargo clippy -p script-kit-gpui --all-targets -- -D warnings`
3. `cargo test -p script-kit-gpui --lib key_intent_maps_required`

Risks / known gaps:
- Full verification is currently blocked by unrelated compile failures in other active areas of the repo.
- This task adds regression tests and does not alter runtime key-handling behavior.

Commits:
- None (not committed because required verification commands are currently failing for unrelated reasons).