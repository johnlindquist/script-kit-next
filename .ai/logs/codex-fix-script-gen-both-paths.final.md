Implemented the GenerateScript builtin fix so it now opens ChatPrompt in script-generation mode instead of running background generation.

Changed files:
- `src/app_execute/builtin_execution.rs:952`
- `src/app_execute/builtin_execution.rs:1027`

What changed:
- Removed the `script_generation_prompt` capture variable and the GenerateScript-only empty prompt guard.
- Stopped applying global AI-command window hide/reset behavior for `AiCommandType::GenerateScript`.
- Replaced the `AiCommandType::GenerateScript` arm logic with:
  - `let query = self.filter_text.trim().to_string();`
  - `self.show_script_generation_chat(Some(query), cx);`

Verification run:
- `cargo check -p script-kit-gpui` passed (with existing warnings).
- `cargo test -p script-kit-gpui builtin_execution_ai_feedback_tests` failed due pre-existing unrelated test compile issues in other modules (not in this change scope).

How to test manually:
1. Launch app.
2. Type text in the main filter.
3. Run builtin `Generate Script with AI`.
4. Confirm the main window stays visible and ChatPrompt opens in script-generation mode with the typed text prefilled/submitted.
5. Repeat with empty filter and confirm chat still opens (empty handling is internal to chat flow).

Risks / known gaps:
- No new automated test added for this behavior in this change.
- Scoped test execution is currently blocked by unrelated pre-existing test compile errors elsewhere in the repo.