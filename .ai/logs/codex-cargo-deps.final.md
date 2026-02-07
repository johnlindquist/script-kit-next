Created the dependency audit report at `plans/dependency-improvements.md`.

**Changed Files**
- `plans/dependency-improvements.md`

**What To Run (re-verify)**
1. `cargo audit`
2. `cargo update --workspace --dry-run --verbose`
3. `cargo machete --with-metadata`
4. `cargo tree -d`
5. `cargo check`
6. `cargo clippy --all-targets -- -D warnings`
7. `cargo test`

**Validation I ran**
- `cargo check` passed.
- `cargo clippy --all-targets -- -D warnings` failed due existing unrelated workspace issues (for example `src/render_prompts/arg.rs:620`, `src/main.rs:3846`, `src/ai/window.rs:8549`, and multiple unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`).
- `cargo test` failed on the same existing issues.

**Risks / Known Gaps**
- Report is analysis-only; no `Cargo.toml`/`Cargo.lock` changes were applied.
- `cargo outdated --root-deps-only` was not usable due a `cocoa` resolution conflict, so outdated analysis used `cargo update --dry-run` instead.
- Full verification gate is currently blocked by pre-existing compile/clippy/test failures unrelated to this report.

**Commits**
- No commits were made.