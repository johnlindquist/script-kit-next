Split for all non-dialog oversized `src/actions` files is complete, with each resulting file under 500 lines.

**Changed Files**
- `src/actions/builders.rs`
- `src/actions/builders/chat.rs`
- `src/actions/builders/clipboard.rs`
- `src/actions/builders/file_path.rs`
- `src/actions/builders/notes.rs`
- `src/actions/builders/script_context.rs`
- `src/actions/builders/scriptlet.rs`
- `src/actions/builders/shared.rs`
- `src/actions/builders/tests.rs`
- `src/actions/builders_tests.rs`
- `src/actions/builders_tests/part_01.rs`
- `src/actions/builders_tests/part_02.rs`
- `src/actions/builders_tests/part_03.rs`
- `src/actions/builders_tests/part_04.rs`
- `src/actions/command_bar.rs`
- `src/actions/command_bar/part_01.rs`
- `src/actions/command_bar/part_02.rs`
- `src/actions/command_bar/part_03.rs`
- `src/actions/types.rs`
- `src/actions/types/script_info.rs`
- `src/actions/types/action_model.rs`
- `src/actions/types/tests.rs`
- `src/actions/window.rs`
- `src/actions/window/part_01.rs`
- `src/actions/window/part_02.rs`
- `src/actions/window/part_03.rs`
- `src/actions/tests.rs`
- `src/actions/tests/part_01.rs`
- `src/actions/tests/main_tests.rs`
- `src/actions/tests/core_part_01.rs`
- `src/actions/tests/core_part_02.rs`
- `src/actions/tests/core_part_03.rs`

**Verification**
- Ran required gate exactly:
  - `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`
- Result: **blocked before clippy/test** by unrelated tree errors outside this scope, including:
  - `src/protocol/io/parsing.rs:1`
  - multiple `src/stories/*/split/part_*.rs` doc-comment/split parse errors
- Also verified split target constraint in `src/actions` (non-dialog): no `.rs` file > 500 lines remains.

**Risks / Known Gaps**
- Functional verification is incomplete until shared-tree compile errors in `src/protocol/*` and `src/stories/*` are fixed by their owners.
- I converted top-of-file inner docs (`//!`) to regular comments in include-part files to keep `include!` compilation-safe; behavior is unchanged.

**How To Test**
1. Re-run: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test` after external compile blockers are resolved.
2. Optional size check: `find src/actions -name '*.rs' | sort | while read -r f; do case "$f" in *'/dialog'*|*/dialog*.rs) continue ;; esac; wc -l "$f"; done | sort -nr | head`

**Commits**
- No commits were made (verification gate is currently blocked by unrelated compile errors).