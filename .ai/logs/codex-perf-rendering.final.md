Created the requested comprehensive performance audit report at:

- `plans/rendering-performance.md`

It includes prioritized findings and specific recommendations for:

- unnecessary rerenders
- missing/fragile `cx.notify()` usage
- expensive clone/allocation hot paths
- render-time side effects and avoidable rebuilds

**Changed files**

- `plans/rendering-performance.md`

**Verification run**

- `cargo check` ✅ passed
- `cargo clippy --all-targets -- -D warnings` ❌ failed due existing repo issues unrelated to this doc change (examples: duplicate `tests` module between `src/render_prompts/arg.rs` and `src/main.rs`, missing `AiApp::message_body_content` in `src/ai/window.rs`, many unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`)
- `cargo test` ❌ failed due existing repo issue (`AiApp::message_body_content` missing in `src/ai/window.rs`) plus warnings in action test files

**How to test**

1. Open `plans/rendering-performance.md` and review the prioritized recommendations.
2. Re-run validation gate:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

**Risks / known gaps**

- This task delivered analysis/report only; no runtime fixes were implemented yet.
- Findings are static-analysis based (no added profiling instrumentation in this pass).
- Full lint/test gate is currently blocked by pre-existing repository failures above.

**Commits**

- No commits were made.