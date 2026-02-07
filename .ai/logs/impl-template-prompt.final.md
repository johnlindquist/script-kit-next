Implemented the highest-priority `TemplatePrompt` fixes in `src/prompts/template.rs`:

1. `parse_template_inputs` now uses shared extraction (`template_variables::extract_variable_names`) for consistent parsing semantics (`src/prompts/template.rs:108`).
2. Added single-pass placeholder rendering helpers (`src/prompts/template.rs:121`, `src/prompts/template.rs:133`, `src/prompts/template.rs:161`) to avoid recursive multi-pass replacement issues.
3. Updated `filled_template`, `preview_template`, and `submit` to use the single-pass renderer (`src/prompts/template.rs:333`, `src/prompts/template.rs:350`, `src/prompts/template.rs:391`).
4. Added regression tests for `${...}` parsing, whitespace/control-tag handling, JS-expression skipping, and single-pass substitution behavior (`src/prompts/template.rs:756`, `src/prompts/template.rs:766`, `src/prompts/template.rs:776`, `src/prompts/template.rs:804`, `src/prompts/template.rs:818`).

Changed files:
- `src/prompts/template.rs`

How to test:
1. `cargo check`
2. After the unrelated test-compile blocker is fixed, run:
   1. `cargo test test_parse_dollar_brace_placeholders -- --nocapture`
   2. `cargo test test_parse_brace_placeholders_with_whitespace_and_skip_control_tags -- --nocapture`
   3. `cargo test test_template_prompt_substitute_single_pass_does_not_rewrite_user_literal_placeholders -- --nocapture`

Verification run:
- `cargo check` passed.
- `cargo test ...` is currently blocked by an unrelated existing lib-test compile error: `AiApp::message_body_content` missing in `src/ai/window.rs:8549`.

Risks / known gaps:
- Field editing is still manual key handling (cursor movement/selection/IME parity with shared input primitives is not part of this scoped change).
- Protocol/schema improvements for explicit template field metadata were not in scope.

Commits made:
- None.