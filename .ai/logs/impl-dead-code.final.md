Implemented the compiler-confirmed dead-code cleanup from `plans/dead-code-cleanup.md` by removing unused imports in the targeted validation test modules.

**Changed files**
- `src/actions/dialog_builtin_action_validation_tests_7.rs`
- `src/actions/dialog_builtin_action_validation_tests_8.rs`
- `src/actions/dialog_builtin_action_validation_tests_15.rs`
- `src/actions/dialog_builtin_action_validation_tests_19.rs`
- `src/actions/dialog_builtin_action_validation_tests_27.rs`
- `src/actions/dialog_builtin_action_validation_tests_30.rs`
- `src/actions/dialog_builtin_action_validation_tests_33.rs`
- `src/actions/dialog_builtin_action_validation_tests_34.rs`
- `src/actions/dialog_builtin_action_validation_tests_35.rs`
- `src/actions/dialog_builtin_action_validation_tests_38.rs`
- `src/actions/dialog_builtin_action_validation_tests_39.rs`
- `src/actions/dialog_builtin_action_validation_tests_41.rs`
- `src/actions/dialog_builtin_action_validation_tests_44.rs`
- `src/actions/dialog_builtin_action_validation_tests_45.rs`
- `src/actions/dialog_builtin_action_validation_tests_46.rs`

**What was removed**
- Unused `use` items only (no behavior changes), matching the report’s compiler-confirmed list for these files.

**How to test**
1. `cargo check`
2. Optional deeper check: `cargo check --all-targets --message-format short`

**Verification run**
- `cargo check`: passed.
- `cargo check --lib`: passed.
- `cargo check --all-targets`: still fails due pre-existing unrelated errors outside this change set (`src/execute_script.rs`, `src/render_prompts/arg.rs`, `src/ai/window.rs`), but the prior unused-import diagnostics for the edited test files are gone.

**Risks / known gaps**
- I limited this pass to low-risk, compiler-confirmed unused imports.
- Broader dead-code cleanup items from the plan (unused functions/fields/commented-out blocks) remain and should be done in follow-up passes, especially where files are currently owned by other agents.

**Commits**
- No commits were made.