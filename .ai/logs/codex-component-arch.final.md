Created the architecture audit report at `plans/component-architecture.md` covering:

- unified list item API gaps (`Custom` variants/a11y fields not rendered),
- button/`FooterButton` duplication,
- form field + alias input editing-model duplication,
- prompt shell overlap (`PromptContainer` vs `prompt_layout_shell`),
- prioritized refactor plan and follow-up test plan.

Changed files:

- `plans/component-architecture.md`

How to test:

1. Read the report: `sed -n '1,260p' plans/component-architecture.md`
2. Verification gate I ran: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

Verification results:

- `cargo check`: passed.
- `cargo clippy` / `cargo test`: failed due pre-existing unrelated repo issues (examples: duplicate `mod tests` in `src/render_prompts/arg.rs`, missing `AiApp::message_body_content` in `src/ai/window.rs`, multiple unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`).

Risks / known gaps:

- This task delivered an audit report only; no runtime component refactor was implemented yet.
- Full verification is currently blocked by existing workspace failures outside this task’s scope.

Commits made:

- None.