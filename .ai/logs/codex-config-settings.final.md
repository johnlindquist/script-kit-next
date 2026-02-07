Completed the config/settings audit and wrote the report to:

- `plans/config-settings-improvements.md`

The report includes prioritized findings and recommendations for:
- `SK_PATH`/path-centralization gaps
- all-or-nothing config parsing behavior
- window-state persistence path consistency/migration
- missing disable semantics for built-in hotkeys
- hardcoded watcher/layout/window constants that should be configurable
- settings persistence API consolidation

**Verification run**
- Executed: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
- Result: failed due pre-existing workspace issues unrelated to this report-only change, including:
  - duplicate `tests` module in `src/render_prompts/arg.rs:620` vs `src/main.rs:3846`
  - many unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`
  - missing method `AiApp::message_body_content` in `src/ai/window.rs:8549`
  - clippy `needless_return` in `src/render_prompts/other.rs:313`

**Risks / Known gaps**
- This task produced an analysis report only; no runtime behavior changed.
- Recommended fixes in the report are not yet implemented.

**Commits**
- None made.