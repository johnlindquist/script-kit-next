Implemented the directory/file rename and include-path updates for the four requested modules.

**Commit**
- `d0e14c5f6c552b2d3ad0f44f5e480437ac4977e1`

**Changed Files**
- `src/builtins/part_003.rs`
- `src/builtins/tests/part_000.rs` (moved from `src/builtins/part_003_tests/tests_part_000.rs`)
- `src/builtins/tests/part_001.rs` (moved from `src/builtins/part_003_tests/tests_part_001.rs`)
- `src/scriptlet_cache/part_002.rs`
- `src/scriptlet_cache/tests/part_000.rs` (moved)
- `src/scriptlet_cache/tests/part_001.rs` (moved)
- `src/scriptlet_cache/tests/part_002.rs` (moved)
- `src/watcher/part_004.rs`
- `src/watcher/tests/part_000.rs` (moved)
- `src/watcher/tests/part_001.rs` (moved)
- `src/watcher/tests/part_002.rs` (moved)
- `src/frecency/part_002.rs`
- `src/frecency/tests/part_000.rs` (moved)
- `src/frecency/tests/part_001.rs` (moved)
- `src/frecency/tests/part_002.rs` (moved)

**What Changed**
- Replaced `include!("part_NNN_tests/tests_part_XXX.rs")` with `include!("tests/part_XXX.rs")` in:
  - `src/builtins/part_003.rs`
  - `src/scriptlet_cache/part_002.rs`
  - `src/watcher/part_004.rs`
  - `src/frecency/part_002.rs`
- Moved files from:
  - `module/part_NNN_tests/tests_part_XXX.rs`
  - to `module/tests/part_XXX.rs`

**How To Test**
1. `cargo check -p script-kit-gpui --lib`
2. `cargo test -p script-kit-gpui --lib builtins::part_003::tests`
3. `cargo test -p script-kit-gpui --lib scriptlet_cache::part_002::tests`
4. `cargo test -p script-kit-gpui --lib watcher::part_004::tests`
5. `cargo test -p script-kit-gpui --lib frecency::part_002::tests`

**Verification Run**
- Passed: `cargo check -p script-kit-gpui --lib`
- Scoped test commands for builtins/scriptlet_cache/watcher failed in this shared tree due unrelated missing file: `src/theme/validation_tests.rs` (outside this task’s scope).
- Frecency scoped test command was repeatedly blocked by Cargo lock contention from concurrent agents in the shared tree.

**Risks / Known Gaps**
- I did not modify `src/*/mod.rs` files because these test include declarations are located in the `part_*.rs` files for the four target modules.
- Full/clean test verification is currently blocked by unrelated concurrent tree changes (`src/theme/validation_tests.rs` missing).