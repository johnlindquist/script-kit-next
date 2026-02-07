Implemented the assignment as an audit/report deliverable.

**Changed files**
- `plans/logging-observability.md`

**What’s in the report**
- Severity-level inconsistencies (`logging::log` flattening to INFO)
- Correlation ID propagation gaps across async/channel boundaries
- Raw payload logging risks (stdin/protocol/MCP)
- `SCRIPT_KIT_AI_LOG=1` compact-category coverage gaps and doc/code drift
- Prioritized remediation plan + verification recommendations

**Verification run**
- `cargo check` ✅
- `cargo clippy --all-targets -- -D warnings` ❌ (pre-existing failures, e.g. duplicate `mod tests` and many unused imports in action test files, plus `E0599` at `src/ai/window.rs:8549`)
- `cargo test` ❌ (same pre-existing `E0599` at `src/ai/window.rs:8549`)

**How to test**
1. Open `plans/logging-observability.md`.
2. Re-run:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

**Risks / known gaps**
- This task produced analysis only; no source remediation was applied yet.
- Full green verification is currently blocked by unrelated existing repo failures above.

**Commits**
- No commits were made.