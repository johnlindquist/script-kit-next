Implemented deliverable: created `plans/platform-compatibility.md` with a prioritized cross-platform compatibility audit and remediation plan.

Changed files:
- `plans/platform-compatibility.md`

Verification run:
1. `cargo check` passed.
2. `cargo clippy --all-targets -- -D warnings` failed due existing workspace issues unrelated to this report file, including:
- duplicate `tests` module (`src/render_prompts/arg.rs:620` vs `src/main.rs:3846`)
- missing `AiApp::message_body_content` (`src/ai/window.rs:8549`)
- many existing unused imports in action-dialog validation test files (for example `src/actions/dialog_builtin_action_validation_tests_7.rs:51`)
3. `cargo test` failed on the same existing `AiApp::message_body_content` issue (`src/ai/window.rs:8549`) and emitted existing unused-import warnings.

How to test:
1. Open the report: `cat plans/platform-compatibility.md`
2. Re-run validation gate:
- `cargo check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

Risks / known gaps:
- This change is a static-analysis report only; no runtime behavior was modified.
- Non-mac target compilation (`--target ...`) was recommended in the report but not executed in this pass.
- Workspace currently has pre-existing build/test issues that prevent clean clippy/test completion.

Commits made:
- None.