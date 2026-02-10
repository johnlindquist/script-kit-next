Implemented both requested ChatPrompt UI fixes.

Changed files:
- `src/prompts/chat/render_core.rs:4`
  - Moved script-generation action buttons into footer via `render_script_generation_footer_actions(...)`.
  - Footer now renders actions inline next to existing footer items/model text.
  - Removed old standalone action-row render path from the main layout (`render_script_generation_actions` path is no longer used).
- `src/prompts/chat/streaming.rs:460`
  - Re-applied bottom-follow on built-in stream finalization by calling `chat.scroll_turns_to_bottom()` after cache refresh.
  - Keeps view pinned to newest content when stream completes.
- `src/prompts/chat/mod.rs:10`
  - Updated imports for new footer rendering/types usage.
- `src/prompts/chat/tests.rs:162`
  - Added regression checks for:
    - footer placement (no standalone script action row),
    - completion-path bottom-follow call.

Verification run:
- `cargo check -p script-kit-gpui` passed.
- Targeted test run attempted:
  - `cargo test -p script-kit-gpui script_generation_actions_render_in_footer_instead_of_standalone_row -- --nocapture`
  - Blocked by unrelated shared-tree test compile failures (outside this task), e.g. duplicate module files in `src/protocol/message/tests*` and `src/designs/core/tests*`, plus unrelated `src/actions/tests/*` errors.

How to test manually:
1. Open script-generation ChatPrompt and generate a script response.
2. Confirm `Save`, `Run`, `Save and Run` appear inside the footer next to model/footer controls, not as a separate row above footer.
3. Stream a longer response and confirm the message view continues following new content to the bottom.
4. Run `cargo check -p script-kit-gpui`.

Risks / known gaps:
- Could not execute scoped chat tests due unrelated global test-target compile breakages in the shared branch.
- No screenshot-based UI verification was run in this pass.

Commits:
- None.