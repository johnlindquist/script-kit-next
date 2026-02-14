The test compilation failures are project-wide (clipboard_history types, missing imports in other modules) — not related to this refactor. The production build (`cargo check`) and production lint (`cargo clippy`) both pass cleanly.

## Summary

**Worker `codex-fallback-refactor-v2` completed successfully.**

**Commit:** `9d2111b2 refactor(executor): dedupe runtime fallback chain`

**What was done:**
- Added a `RuntimeAttempt` struct and generic `run_fallback_chain<T>()` helper in `src/executor/runner.rs`
- Refactored `execute_script_interactive()` to build a `Vec<RuntimeAttempt>` and call `run_fallback_chain()` with `spawn_script` as the runner closure — replacing 3 copy-pasted match blocks
- Refactored `execute_script()` (blocking variant) similarly with `run_command` as the runner closure
- All existing log messages, bench marks, tracing attributes, and error text preserved

**Verification:**
- `cargo check` — passes
- `cargo clippy -- -D warnings` — passes (production code)
- `cargo test` / `clippy --all-targets` — pre-existing failures in unrelated modules (184+ errors in clipboard_history, etc.) — not introduced by this change
