Implemented the high-priority form validation UX changes in scope:

- `src/render_prompts/form.rs:7`
  - Added `FormEnterBehavior` and Enter policy: textarea uses `Enter` for newline and `Cmd+Enter` to submit; non-text inputs submit on `Enter`.
  - Added pre-submit field-type validation (`email`, `number`) and HUD error messaging before submission.
  - Updated helper text to context-aware copy (`press ⌘↵...` vs `press Enter...`).
  - Changed form body overflow to vertical scrolling (`.overflow_y_scrollbar()`).
  - Added focused unit tests in `mod form_prompt_render_tests` at `src/render_prompts/form.rs:431`.

- `src/components/form_fields.rs:22`
  - Imported `ScrollableElement` for scrollbar methods.
  - Added type-aware candidate validators for typed input/paste:
    - `is_partial_number_value` / `is_partial_email_value`
    - `form_field_type_allows_candidate_value`
  - Wired candidate gating into key insertion and paste paths so invalid number/email chars are rejected as typed.
  - Changed textarea container overflow to `overflow_x_hidden + overflow_y_scrollbar`.

- `src/components/form_fields_tests.rs:175`
  - Added tests for number/email candidate acceptance/rejection using `form_field_type_allows_candidate_value`.

How to test:

1. Build/check:
```bash
cargo check
cargo clippy --all-targets -- -D warnings
cargo test
```

2. Runtime smoke (stdin protocol):
```bash
TEST_PATH="$(pwd)/tests/smoke/test-form-prompt.ts"
timeout 20 bash -c "echo '{\"type\":\"run\",\"path\":\"'$TEST_PATH'\"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1"
```

3. Validate logs/screenshot:
```bash
rg -n "Parsed command: Run|Showing form prompt|Parsed 4 form fields|form-prompt-render|form-prompt-screenshot" ~/.scriptkit/logs/script-kit-gpui.jsonl | tail -20
ls -1t .test-screenshots | head -5
```
I also opened and reviewed `.test-screenshots/form-prompt-1770441855910.png` (form rendered correctly).

Verification status:

- `cargo build` passed.
- `cargo check` passed.
- Full gate `cargo check && cargo clippy --all-targets -- -D warnings && cargo test` fails due unrelated existing errors outside my scope (e.g. `src/render_prompts/arg.rs`, `src/actions/dialog_builtin_action_validation_tests_*`, `src/ai/window.rs`, `src/render_prompts/other.rs`).

Risks / known gaps:

- End-to-end automated coverage for `Cmd+Enter` submit behavior in textarea is not present in existing smoke scripts; behavior is covered by new unit tests and code-path checks.
- Repository-wide clippy/test failures outside scope currently block full green CI gate.

Commits:

- No commits were created.