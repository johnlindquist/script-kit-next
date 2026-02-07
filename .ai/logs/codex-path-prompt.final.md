Created the requested analysis report at `plans/path-prompt-improvements.md`.

**Changed Files**
- `plans/path-prompt-improvements.md`

**What I Ran**
- `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

Result:
- Verification did **not** pass due pre-existing workspace issues unrelated to this report-only change, including:
  - Duplicate `tests` module (`src/render_prompts/arg.rs:620` vs `src/main.rs:3846`)
  - Missing `AiApp::message_body_content` (`src/ai/window.rs:8549`)
  - Multiple unused imports in existing test modules under `src/actions/dialog_builtin_action_validation_tests_*.rs`
  - Clippy `needless_return` in `src/render_prompts/other.rs:313`

**How To Test**
1. Open the report and review recommendations:
   - `sed -n '1,260p' plans/path-prompt-improvements.md`
2. (Optional) Re-run the repo verification gate:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

**Risks / Known Gaps**
- The deliverable is analysis-only; no path prompt runtime behavior was changed yet.
- Full verification is currently blocked by existing compile/clippy/test failures elsewhere in the repo.

**Commits**
- None.