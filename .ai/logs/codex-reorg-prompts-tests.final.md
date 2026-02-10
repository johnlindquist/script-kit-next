**Changes**
- Moved the loose prelude test into `src/prompts/prelude.rs:12` as an inline `#[cfg(test)] mod tests`.
- Removed the old test module declaration from `src/prompts/mod.rs` (deleted `mod prelude_tests;` near end of file).
- Deleted `src/prompts/prelude_tests.rs`.

**Verification Run**
- Passed: `cargo check -p script-kit-gpui --lib`
- Passed: `cargo clippy -p script-kit-gpui --lib -- -D warnings`
- Blocked by unrelated parallel-agent changes:  
  `cargo test -p script-kit-gpui --lib prompts::prelude::tests::test_prompts_prelude_exports_core_prompt_types` failed due compile errors outside this scope:
  - `src/actions/tests/dialog_builtin_validation/dialog_builtin_action_validation_tests_11/part_01.rs:1`
  - `src/components/form_fields/tests.rs:6`

**How to Test**
1. `cargo check -p script-kit-gpui --lib`
2. `cargo clippy -p script-kit-gpui --lib -- -D warnings`
3. After unrelated test-compilation issues are resolved, run:  
   `cargo test -p script-kit-gpui --lib prompts::prelude::tests::test_prompts_prelude_exports_core_prompt_types`

**Risks / Known Gaps**
- The moved prelude test could not be executed end-to-end in this shared tree because unrelated in-progress files currently break `lib test` compilation.

**Commits**
- None made.