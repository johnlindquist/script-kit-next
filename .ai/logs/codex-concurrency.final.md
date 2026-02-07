Created the concurrency audit report at `plans/concurrency-improvements.md`.

**Changed files**
- `plans/concurrency-improvements.md`

**How to test**
1. Open `plans/concurrency-improvements.md` and review the severity-ranked findings/remediation plan.
2. Re-run verification gate:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

**Verification results I got**
- `cargo check`: passed.
- `cargo clippy --all-targets -- -D warnings`: failed due existing repo issues, including:
  - duplicate `mod tests` (`src/render_prompts/arg.rs:620` vs `src/main.rs:3846`)
  - missing method `AiApp::message_body_content` (`src/ai/window.rs:8549`)
  - multiple pre-existing unused imports in generated validation test files.
- `cargo test`: failed with the same pre-existing compile errors.

**Risks / known gaps**
- This task was analysis/report only; no runtime concurrency behavior was changed.
- Findings are from static analysis (no new scenario script execution was added in this task).

**Commits**
- No commits were made.