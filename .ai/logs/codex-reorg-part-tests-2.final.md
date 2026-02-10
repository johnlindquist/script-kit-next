Implemented the part-test reorg for the assigned modules by moving test include files to `tests/` and updating include callsites.

Changed files (task scope):
- `src/hud_manager/part_003.rs`
- `src/hud_manager/part_003_tests/tests_part_000.rs` -> `src/hud_manager/tests/part_000.rs`
- `src/hud_manager/part_003_tests/tests_part_001.rs` -> `src/hud_manager/tests/part_001.rs`
- `src/extension_types/part_002.rs`
- `src/extension_types/part_002_tests/tests_part_000.rs` -> `src/extension_types/tests/part_000.rs`
- `src/extension_types/part_002_tests/tests_part_001.rs` -> `src/extension_types/tests/part_001.rs`
- `src/file_search/part_004.rs`
- `src/file_search/part_004_tests/tests_part_000.rs` -> `src/file_search/tests/part_000.rs`
- `src/file_search/part_004_tests/tests_part_001.rs` -> `src/file_search/tests/part_001.rs`
- `src/term_prompt/part_003.rs`
- `src/term_prompt/part_003_tests/tests_part_000.rs` -> `src/term_prompt/tests/part_000.rs`
- `src/term_prompt/part_003_tests/tests_part_001.rs` -> `src/term_prompt/tests/part_001.rs`

How to test:
1. `cargo check -p script-kit-gpui`
2. `cargo test -p script-kit-gpui hud_manager::`
3. `cargo test -p script-kit-gpui extension_types::`
4. `cargo test -p script-kit-gpui file_search::`
5. `cargo test -p script-kit-gpui term_prompt::`

Verification run:
- Passed: `cargo check -p script-kit-gpui`
- Blocked for scoped tests: `cargo test -p script-kit-gpui hud_manager::` fails due unrelated missing file `src/theme/validation_tests.rs` in concurrent repo state.

Risks / known gaps:
- Scoped test execution is currently blocked by unrelated in-progress changes outside this task.
- No parent `mod.rs` updates were needed for these modules because test include declarations live in `part_002.rs`/`part_003.rs`/`part_004.rs`.

Commits made:
- `0172b03` `refactor(test-layout): move part test files into tests dirs` (this task’s scoped change)
- `66f72a3` `refactor(test-layout): normalize part test directories` (accidentally captured unrelated staged files from other agents)