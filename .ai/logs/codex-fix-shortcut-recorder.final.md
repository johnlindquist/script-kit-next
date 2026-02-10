Implemented the dead-end fix by choosing option (a): recording no longer ends when a non-modifier key is pressed with no modifiers held.

Changed file:
- `src/components/shortcut_recorder/component.rs:14`
- `src/components/shortcut_recorder/component.rs:207`
- `src/components/shortcut_recorder/component.rs:308`

What changed:
- Added `should_finish_recording(...)` logic so recording only finalizes for non-modifier keys when at least one modifier is active.
- Bare-key presses without modifiers are now ignored (recording stays active).
- Added edge-case tests:
  - `test_should_finish_recording_does_not_complete_without_modifiers`
  - `test_should_finish_recording_completes_with_modifier_and_non_modifier_key`

How to test:
1. `cargo test shortcut_recorder::component::tests::test_should_finish_recording`
2. `cargo check`
3. `cargo clippy --all-targets -- -D warnings`

Verification results:
- `cargo test shortcut_recorder::component::tests::test_should_finish_recording`: pass
- `cargo check`: pass
- `cargo clippy --all-targets -- -D warnings`: fails due pre-existing unrelated clippy errors in `src/prompts/div/render_html.rs:54`, `src/prompts/div/render_html.rs:64`, `src/prompts/div/render_html.rs:203`

Risks / known gaps:
- Clippy gate is currently blocked by unrelated existing lint issues outside this task scope.
- The new test covers the decision logic directly; it does not spin a full GPUI `Context` integration path.

Commit:
- `0897f3c`