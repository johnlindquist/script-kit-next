Implemented the three requested fixes in the script-generation ChatPrompt flow.

Changed files:
- `src/prompts/chat/prompt.rs`
  - Added `on_run_script` callback field and `with_run_script_callback(...)` builder.
- `src/prompts/chat/actions.rs`
  - Updated `handle_script_generation_action(...)` so `Run`/`Save and Run` dispatch to parent callback (app pipeline) instead of detached bun/kit process.
- `src/app_impl/prompt_ai.rs`
  - In `show_script_generation_chat(...)`, wired `with_run_script_callback(...)` to call `ScriptListApp::execute_script_by_path(...)` via app entity update.
- `src/prompts/chat/render_core.rs`
  - Added Cmd+Return behavior in key handler for script-generation mode:
    - `Cmd+Return` now triggers `ScriptGenerationAction::SaveAndRun`.
  - Updated script-generation footer buttons (`Save`, `Run`, `Save and Run`) to use PromptFooter-equivalent visual treatment.
  - Added/updated footer shortcut and key-behavior tests in this file.
- `src/components/prompt_footer.rs`
  - Exposed shared footer button style helpers:
    - `footer_button_font_size_px(...)`
    - `footer_button_hover_rgba(...)`
    - `footer_button_active_rgba(...)`
- `src/ai/script_generation.rs`
  - Removed `run_saved_script(...)` detached runner (no longer used by ChatPrompt run path).

Verification run:
- `cargo check -p script-kit-gpui`  
  - Passed.
- Attempted focused test:
  - `cargo test -p script-kit-gpui test_cmd_enter_runs_save_and_run_when_script_generation_mode_enabled -- --nocapture`
  - Blocked by unrelated pre-existing duplicate test module files:
    - `src/protocol/message/tests.rs` and `src/protocol/message/tests/mod.rs`
    - `src/designs/core/tests.rs` and `src/designs/core/tests/mod.rs`

How to test manually:
1. Launch app and open script-generation chat.
2. Generate a script response.
3. Click `Run` and confirm it executes via app flow (main window/script pipeline behavior, not detached background run).
4. Click `Save and Run` and confirm same.
5. Press `Cmd+Return` in script-generation mode and confirm it triggers `Save and Run`.
6. Confirm footer button visuals for `Save`, `Run`, `Save and Run` match standard PromptFooter button treatment (font sizing, padding, hover/active surface).

Risks / known gaps:
- Rust test execution is currently blocked by unrelated duplicate test-module files in the working tree, so targeted test runtime verification could not complete.
- I did not perform screenshot-based UI verification in this run.

Commits made:
- None.