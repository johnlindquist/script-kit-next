Implemented the label-consistency task across `src/actions/*` and updated dialog rendering for action descriptions.

**Changed files**
- `src/actions/builders.rs`
- `src/actions/dialog.rs`
- `src/actions/builders_tests.rs`
- `src/actions/dialog_builtin_action_validation_tests_3.rs`
- `src/actions/dialog_builtin_action_validation_tests_25.rs`
- `src/actions/dialog_builtin_action_validation_tests_31.rs`
- `src/actions/dialog_builtin_action_validation_tests_37.rs`
- `src/actions/dialog_builtin_action_validation_tests_42.rs`
- `src/actions/dialog_builtin_action_validation_tests_43.rs`
- `src/actions/dialog_window_tests.rs`

**What changed**
- Normalized Finder labels to **`Reveal in Finder`** in file/path/script/scriptlet/agent action builders.
- Aligned `show_info` title to **`Show Info`**.
- Standardized shortcut/alias descriptions to consistent verb phrases (e.g. “Change/Remove/Set … for this item”).
- Updated `ActionsDialog` row UI to render optional `action.description` as a second line under title.
- Updated affected tests to match new labels and added coverage for file/path Finder label normalization and description consistency.

**Validation run**
- `cargo check` ✅ passed
- `cargo clippy --all-targets -- -D warnings` ❌ failed (pre-existing repo issues, mainly unused imports/dead code in many `src/actions/dialog_builtin_action_validation_tests_*.rs` files and one `clippy::collapsible_if` in `src/ai/window.rs`)
- `cargo test` ✅ passed (`9562 passed, 0 failed, 35 ignored` + doc tests passed)
- UI protocol check (stdin JSON) ✅ verified:
  - command parsed as `Show { request_id: None }`
  - app processed show command via stdin JSON protocol

**How to test**
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings` (currently expected to fail for pre-existing warnings noted above)
3. `cargo test`
4. `timeout 8 bash -c 'printf '\''{"type":"show"}\n'\'' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`
5. Optional label sanity check: `rg -n "Show in Finder|Open in Finder|Get Info" src/actions` (should return no matches)

**Risks / known gaps**
- Clippy gate is still red due existing lint debt outside this task’s functional scope.
- Any external automation keyed on old action titles (`Get Info`, `Show/Open in Finder`) will need to use the new normalized labels.