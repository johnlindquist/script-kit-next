Created the requested report at `plans/missing-features.md`.

**Changed files**
- `plans/missing-features.md`

**What’s in the report**
- Source-backed capability matrix across built-ins, prompt types, protocol surface, and runtime modules.
- Status per requested area: implemented vs partial vs missing.
- Explicit gaps for snippet management, process management UI, notification actions API, rich file preview, quick math depth, color picker, emoji picker.
- Priority roadmap (`P0/P1/P2`) with concrete next feature targets.

**Verification run**
- Ran: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
- `cargo check` passed.
- `cargo clippy` failed due pre-existing repo issues unrelated to this doc-only change (examples: `src/render_prompts/arg.rs:620` duplicate `tests` module conflict with `src/main.rs:3846`; many unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`; missing `AiApp::message_body_content` in `src/ai/window.rs:8549`).
- Ran `cargo test` separately; it also fails on the same pre-existing `AiApp::message_body_content` error in `src/ai/window.rs:8549`.

**How to test**
1. Open `plans/missing-features.md` and review the matrix/recommendations.
2. Optional: re-run validation commands:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

**Risks / known gaps**
- Report scope is Rust-only (`src/**/*.rs`), so extension markdown/SDK-side features outside Rust may provide additional coverage not counted here.
- Some “partial/missing” calls are parity judgments against modern launcher expectations, not compile/runtime failures.

**Commits**
- No commits made.