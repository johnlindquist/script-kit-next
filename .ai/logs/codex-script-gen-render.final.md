Implemented the missing chat UI rendering for script-generation actions.

**What changed**
- Added `render_script_generation_actions` in `src/prompts/chat/render_core.rs:28`:
  - Shows `Save`, `Run`, and `Save and Run` buttons when `should_show_script_generation_actions()` is true.
  - Buttons call:
    - `handle_script_generation_action(ScriptGenerationAction::Save)`
    - `handle_script_generation_action(ScriptGenerationAction::Run)`
    - `handle_script_generation_action(ScriptGenerationAction::SaveAndRun)`
  - Shows `script_generation_status` text when present.
    - Error color: `theme.colors.ui.error` (red)
    - Success color: `theme.colors.ui.success` (green)
- Inserted this row into the main chat layout below messages and above the footer in `src/prompts/chat/render_core.rs:476`.
- Added `AnyElement` import needed for typed render composition in `src/prompts/chat/mod.rs:14`.
- Added focused logic tests in `src/prompts/chat/tests.rs`:
  - `should_show_script_generation_actions_only_when_draft_is_ready`
  - `script_generation_action_should_run_after_save_only_for_run_variants`

**Changed files**
- `src/prompts/chat/render_core.rs`
- `src/prompts/chat/mod.rs`
- `src/prompts/chat/tests.rs`

**Verification run**
- Passed: `cargo check -p script-kit-gpui`
- Attempted scoped test run, but blocked by pre-existing unrelated workspace test compilation issues (`E0761` duplicate module files, plus unrelated `actions` test compile failures), so `cargo test` is not currently runnable in this tree.

**How to test manually**
1. Open chat prompt in script-generation mode (`script_generation_mode=true`).
2. Generate a script response and wait for streaming to finish.
3. Confirm buttons appear below the last message: `Save`, `Run`, `Save and Run`.
4. Click each and confirm status text appears under buttons.
5. Confirm error statuses render red and success statuses render green.

**Risks / known gaps**
- No UI screenshot verification was run in this pass.
- Full/targeted `cargo test` remains blocked by unrelated existing test-tree issues.