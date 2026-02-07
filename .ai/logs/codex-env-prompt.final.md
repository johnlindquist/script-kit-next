Implemented the assigned audit and wrote the report at `plans/env-prompt-improvements.md`.

Changed files:
- `plans/env-prompt-improvements.md`

Verification run:
- Ran: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
- Result:
  1. `cargo check` passed.
  2. `cargo clippy --all-targets -- -D warnings` failed due pre-existing workspace issues (not introduced by this report), including:
     - duplicate `tests` module in `src/render_prompts/arg.rs:620` vs `src/main.rs:3846`
     - missing `AiApp::message_body_content` in `src/ai/window.rs:8549`
     - many unused-import warnings elevated to errors across `src/actions/dialog_builtin_action_validation_tests_*.rs`
  3. `cargo test` did not run because clippy failed first in the chained command.

How to test this deliverable:
1. Open and review `plans/env-prompt-improvements.md`.
2. Confirm it covers UX, masking, validation, and persistence findings with prioritized roadmap and test plan.

Risks / known gaps:
1. This task produced a report only; no runtime behavior changes were applied.
2. Repo-wide verification is currently blocked by unrelated existing compile/clippy failures.
3. Some recommendations require protocol/SDK compatibility decisions before implementation.

Commits made:
- None.