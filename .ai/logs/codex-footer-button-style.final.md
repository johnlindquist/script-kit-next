Implemented the footer style/placement change for script-generation mode.

Changed files
- `src/prompts/chat/render_core.rs:4`
  - `render_footer_button` now supports optional shortcut text (so footer buttons can be text-only label when needed).
  - Added `render_script_generation_footer_button` to keep script actions clickable while matching footer text style (accent text, no bg/hover pill).
  - Moved script-generation secondary actions (`Save`, `Run`) to the left side near the model name.
  - Right-side primary button now switches by mode:
    - normal: `Continue in Chat ⌘↵`
    - `script_generation_mode=true`: `Save and Run` (replaces Continue)
  - `Actions ⌘K` stays on the right.
  - Added focused tests at `src/prompts/chat/render_core.rs:606`.

- `src/prompts/chat/prompt.rs`
  - No edits made in this task.

Verification run
- `cargo check -p script-kit-gpui` passed.
- Attempted targeted test run: `cargo test -p script-kit-gpui render_core_footer_tests -- --nocapture`
  - Failed due unrelated concurrent-tree test compilation errors outside this task’s scope (duplicate test module files / unrelated action test compile errors).

How to test
1. Open chat in normal mode: footer should show `Continue in Chat ⌘↵` and `Actions ⌘K`.
2. Open chat with `script_generation_mode=true`:
   - Left side near model shows text-style `Save` and `Run`.
   - Right-side primary is `Save and Run` (Continue hidden).
3. Click `Save`, `Run`, and `Save and Run` to verify script-generation actions fire and status updates.

Risks / known gaps
- UI screenshot-based visual verification was not run.
- Scoped tests are currently blocked by unrelated in-progress test-tree errors from other files.
- No commits were made.