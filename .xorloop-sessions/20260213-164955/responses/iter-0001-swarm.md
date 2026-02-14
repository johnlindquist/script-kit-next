Worker 1's changes to `src/executor/scriptlet.rs` are uncommitted. The `cargo check` and `cargo clippy --lib` both pass. The `--all-targets` clippy failures are pre-existing test compilation errors unrelated to our changes.

## Swarm Results Summary

### Task 1: `codex-scriptlet-result` — Completed (uncommitted)
- Added `ScriptletResult::from_output(output: std::process::Output) -> Self` method
- Replaced all 8 duplicated `Output → ScriptletResult` struct literals with `Ok(ScriptletResult::from_output(output))` in: `execute_shell_scriptlet`, `execute_with_interpreter`, `execute_applescript`, `execute_typescript`, `execute_open`, `execute_edit`, `execute_type`, `execute_submit`
- Left non-`Output` paths unchanged (template arm, `execute_paste`)
- Changes are uncommitted in working tree

### Task 2: `codex-truncate-centralize` — Completed (committed as `2407d4ca`)
- Added canonical `pub(crate) fn truncate_str_chars` to `crate::utils`
- Deleted all 3 local copies from `executor/errors.rs`, `prompt_handler/mod.rs`, `render_script_list/mod.rs`
- Updated all call sites

### Verification
- `cargo check` — passes
- `cargo clippy --lib` — passes (no warnings)
- `cargo clippy --all-targets` — fails due to **190 pre-existing** test compilation errors (unrelated to our changes)

The scriptlet refactor changes still need to be committed. Would you like me to spawn a worker to commit them, or would you prefer to handle that?
