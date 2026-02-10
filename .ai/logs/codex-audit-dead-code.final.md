Implemented a dead-code cleanup in `src/actions` and completed the requested audit report.

### Findings (Markdown)

#### 1. Dead modules declared in test registry
`src/actions/tests/part_01.rs` was still declaring five modules that only contained “Purged batch” comments and no tests.

Removed declarations:
- `dialog_builtin_action_validation_tests_25`
- `dialog_builtin_action_validation_tests_31`
- `dialog_builtin_action_validation_tests_37`
- `dialog_builtin_action_validation_tests_42`
- `dialog_builtin_action_validation_tests_43`

Deleted stale files:
- `src/actions/dialog_builtin_action_validation_tests_25.rs`
- `src/actions/dialog_builtin_action_validation_tests_31.rs`
- `src/actions/dialog_builtin_action_validation_tests_37.rs`
- `src/actions/dialog_builtin_action_validation_tests_42.rs`
- `src/actions/dialog_builtin_action_validation_tests_43.rs`

#### 2. TODO/FIXME/HACK audit
No `TODO`, `FIXME`, or `HACK` markers found in `src/actions/**/*.rs`.

#### 3. Feature-gated code audit
No clearly stale feature-gated branches in `src/actions` required changes in this pass.

#### 4. Additional stale-code candidate (not changed in this task)
`src/actions/builders.rs` appears unwired from the active module graph (active builder wiring uses `src/actions/builders_rewire.rs` via `src/actions/mod.rs:17`), but it is still referenced textually by `src/clipboard_actions_menu_coverage_tests.rs` outside this scope.

---

### Changed Files

- `src/actions/tests/part_01.rs`
- `src/actions/dialog_builtin_action_validation_tests_25.rs` (deleted)
- `src/actions/dialog_builtin_action_validation_tests_31.rs` (deleted)
- `src/actions/dialog_builtin_action_validation_tests_37.rs` (deleted)
- `src/actions/dialog_builtin_action_validation_tests_42.rs` (deleted)
- `src/actions/dialog_builtin_action_validation_tests_43.rs` (deleted)

---

### How To Test

I ran these scoped checks:

1. `cargo test -p script-kit-gpui test_actions_prelude_exports_core_types -- --nocapture`
2. `cargo test -p script-kit-gpui test_builtin_vs_sdk_actions -- --nocapture`
3. `rustfmt --check src/actions/tests/part_01.rs`
4. `rg -n "TODO|FIXME|HACK" src/actions || true`

All passed (with no TODO/FIXME/HACK matches).

---

### Risks / Known Gaps

- I did not run full workspace gates (`cargo check/clippy/test`) because parallel agents are actively modifying unrelated files and scoped verification is required for this swarm run.
- `src/actions/builders.rs` remains a potential dead/stale artifact but touches cross-scope behavior/tests and was not changed here.

Commits made: none.