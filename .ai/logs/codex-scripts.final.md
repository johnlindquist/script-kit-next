Implemented the `src/scripts/*` split so all Rust files in that scope are now under 500 lines.

**Changed Files**
- Modified:
  - `src/scripts/search.rs`
  - `src/scripts/scriptlet_loader.rs`
  - `src/scripts/input_detection.rs`
  - `src/scripts/grouping.rs`
- Added:
  - `src/scripts/search/ascii.rs`
  - `src/scripts/search/nucleo.rs`
  - `src/scripts/search/highlight.rs`
  - `src/scripts/search/paths.rs`
  - `src/scripts/search/prefix_filters.rs`
  - `src/scripts/search/scripts.rs`
  - `src/scripts/search/scriptlets.rs`
  - `src/scripts/search/builtins.rs`
  - `src/scripts/search/apps.rs`
  - `src/scripts/search/windows.rs`
  - `src/scripts/search/unified.rs`
  - `src/scripts/search/tests/mod.rs`
  - `src/scripts/search/tests/core_search.rs`
  - `src/scripts/search/tests/metadata_filters.rs`
  - `src/scripts/search/tests/prefix_syntax.rs`
  - `src/scripts/scriptlet_loader/parsing.rs`
  - `src/scripts/scriptlet_loader/loading.rs`
  - `src/scripts/scriptlet_loader/tests.rs`
  - `src/scripts/input_detection/detection.rs`
  - `src/scripts/input_detection/tests.rs`
  - `src/scripts/grouping/search_mode.rs`
  - `src/scripts/grouping/grouped_view.rs`

`src/scripts/mod.rs` did not need changes because top-level exports remained valid.

**Verification**
- Line-count check passed for scope (`src/scripts` has no `.rs` file > 500 lines).
- Ran required gate:
  - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
  - Fails before completion due unrelated files outside `src/scripts`, including:
    - `src/protocol/io/parsing.rs:1` (`E0753` doc comment placement)
    - `src/app_launcher/part_01.rs:379` (unclosed delimiter)
- Also ran separately:
  - `cargo clippy --all-targets -- -D warnings` (fails on unrelated `src/actions/*` and `src/designs/*`)
  - `cargo test` (same unrelated blockers)

**How To Test**
1. Confirm split constraint:
   - `find src/scripts -name '*.rs' -print0 | xargs -0 wc -l | grep -v ' total$' | awk '$1 > 500 {print}'`
2. Run full gate after unrelated compile issues are fixed:
   - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

**Risks / Known Gaps**
- Full verification is currently blocked by compile errors outside assigned scope.
- Runtime/log verification (`SCRIPT_KIT_AI_LOG=1`) was not possible since the mandatory gate does not pass globally.

**Commits**
- None (no verified commit possible while global gate is failing).