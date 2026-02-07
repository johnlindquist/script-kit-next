Updated `plans/startup-performance.md` with a full startup audit and prioritized optimization plan.

**Changed files**
- `plans/startup-performance.md`

**What’s in the report**
- Measured startup timeline (launch to first render) from runtime logs.
- Bottleneck analysis of the scoped startup path:
  - `src/main.rs` pre-render init path.
  - `src/app_impl.rs` constructor/background init behavior.
  - `src/lib.rs` and `src/app_actions.rs` startup relevance notes.
- Concrete P0/P1/P2 opportunities for lazy-loading, deferral, reduced allocations, and parallelization.
- Ordered implementation plan, verification commands, and risk notes.

**Verification run**
- `cargo check` passed.
- `cargo clippy --all-targets -- -D warnings` failed due existing repo issues unrelated to this report change (examples: duplicate `tests` module in `src/main.rs`/`src/render_prompts/arg.rs`, missing `AiApp::message_body_content` in `src/ai/window.rs`, many unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`).
- `cargo test` failed on the same existing `AiApp::message_body_content` compile error.

**How to test**
1. Open `plans/startup-performance.md`.
2. Run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`
3. For startup timing reproduction, use the commands included in the report’s “Verification Plan”.

**Risks / known gaps**
- Suggested startup deferrals (tray/services/watchers) may change availability timing and require lifecycle revalidation.
- Shared startup snapshots need careful invalidation on watcher updates.
- First-show lazy hydration can shift latency unless prefetched.

**Commits**
- None (no commit created).