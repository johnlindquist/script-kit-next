**Changes made**
- Split oversized test roots into include-based aggregators:
  - `src/scripts_tests.rs` now includes 18 chunks in `src/scripts_tests/chunk_01.rs` … `src/scripts_tests/chunk_18.rs`
  - `src/scriptlet_tests.rs` now includes 9 chunks in `src/scriptlet_tests/chunk_01.rs` … `src/scriptlet_tests/chunk_09.rs`
  - `src/executor_tests.rs` now includes 12 chunks in `src/executor_tests/chunk_01.rs` … `src/executor_tests/chunk_12.rs`
- Existing test-module registration remains valid (no wiring changes needed):
  - `src/scripts/mod.rs:83`
  - `src/scriptlets.rs:1760`
  - `src/executor/mod.rs:62`

All generated chunk files are under 500 lines.

**Verification run**
- Attempted required gate in workspace:
  - `cargo check` (failed due unrelated in-progress refactors in other agent scopes)
  - Could not proceed to successful `cargo clippy --all-targets -- -D warnings` / `cargo test` because compile is currently globally broken.
- Also validated split correctness directly:
  - Reconstructed each original by concatenating chunks and compared to `HEAD` with `diff` for:
    - `src/scripts_tests.rs`
    - `src/scriptlet_tests.rs`
    - `src/executor_tests.rs`
  - All three matched exactly (no content drift, only file-structure split).

**How to test (once shared tree is stable)**
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`

**Risks / known gaps**
- Full gate is currently blocked by unrelated active-agent compilation errors outside this task’s scope, so end-to-end crate verification could not complete in the shared working tree during this run.

**Commits**
- No commits were made (global verification gate could not be completed due unrelated workspace breakage).