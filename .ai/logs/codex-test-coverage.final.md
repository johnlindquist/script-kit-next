Implemented deliverable report at `plans/test-coverage-gaps.md`.

**Changed Files**
- `plans/test-coverage-gaps.md`

**Verification Run**
1. `cargo check`  
- Passed.

2. `cargo clippy --all-targets -- -D warnings`  
- Failed due pre-existing repo issues unrelated to this report, including:
  - many unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`
  - missing function `AiApp::message_body_content` in `src/ai/window.rs:8549`
  - duplicate `tests` module error involving `src/render_prompts/arg.rs:620` and `src/main.rs:3846`
  - `needless_return` in `src/render_prompts/other.rs:313`

3. `cargo test`  
- Failed on pre-existing compile error:
  - `AiApp::message_body_content` missing in `src/ai/window.rs:8549`

**How To Review**
1. Open `plans/test-coverage-gaps.md`
2. Optionally re-run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

**Risks / Known Gaps**
- Report is heuristic/static analysis, not instrumentation-based line coverage.
- Some cross-module indirect coverage may not be detected by file-local heuristics.
- No code-path behavior was changed; only planning/report artifact was added.

**Commits**
- None (not committed because repository-wide clippy/test gate is currently failing on pre-existing issues).