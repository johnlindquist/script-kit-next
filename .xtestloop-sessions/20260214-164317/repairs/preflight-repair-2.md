Here is the analysis and swarm task list:

---

## Diagnosis

The gate failure is **not** a simple dirty-tree issue. There are **two layers of breakage**:

1. **Committed HEAD** already has **183 test compile errors** — production code was overhauled (commit `deba8d98`) to use namespaced action IDs (`file:open_file`, `clip:clipboard_paste`, `chat:submit`, etc.) but ~46 numbered test files still reference old un-namespaced IDs (`open_file`, `clipboard_paste`, `submit`).

2. **Dirty working tree** from a previous xorloop session attempted to fix the compile errors but overcorrected — deleted inline `#[cfg(test)]` modules, changed `run_scriptlet_template()` quoting behavior, swapped `session.pid()` to `session.process_handle.pid`, used `const { assert!() }` patterns, and reduced `cfg(test)` export lists.

The result: all tests technically compile now but **1553 fail at runtime**, and multi-threaded runs crash with SIGABRT.

### Root Causes (ranked by failure count)
| Cause | Failures | Location |
|-------|----------|----------|
| Namespaced IDs in production, old IDs in tests | ~1500 | `src/actions/dialog_builtin_action_validation_tests_*.rs`, `tests/builders.rs`, `dialog_*_tests.rs` |
| Template quoting regression (`'World'` vs `World`) | ~10 | `src/executor/scriptlet.rs` |
| Deleted inline test modules | ~20 | `src/executor/scriptlet.rs`, `src/app_impl/execution_scripts.rs` |
| Reduced cfg(test) export list | ~10 | `src/config/mod.rs` |
| `const { assert!() }` pattern + misc | ~5 | `src/actions/window.rs`, `src/window_resize/mod.rs` |

---

## SWARM TASK LIST

### Task 1: worker-action-ids
**Scope:** `src/actions/dialog_builtin_action_validation_tests_*.rs`, `src/actions/tests/builders.rs`, `src/actions/tests/core.rs`, `src/actions/tests/main_tests.rs`, `src/actions/dialog_*_tests.rs`, `src/actions/builders_tests.rs`, `src/actions_button_visibility_tests.rs`, `src/clipboard_actions_focus_routing_tests.rs`, `src/webcam_actions_consistency_tests.rs`, `src/panel/tests.rs`, `src/components/*/tests.rs`
**Task:**
Update all test assertions that reference un-namespaced action IDs to use the new namespaced format from the committed production code. The mapping is:
- `"open_file"` → `"file:open_file"`, `"open_directory"` → `"file:open_directory"`, `"reveal_in_finder"` → `"file:reveal_in_finder"`, `"quick_look"` → `"file:quick_look"`, `"open_with"` → `"file:open_with"`, `"show_info"` → `"file:show_info"`, `"copy_path"` → `"file:copy_path"`, `"copy_filename"` → `"file:copy_filename"`, `"open_in_finder"` → `"file:open_in_finder"`, `"open_in_editor"` → `"file:open_in_editor"`, `"open_in_terminal"` → `"file:open_in_terminal"`, `"move_to_trash"` → `"file:move_to_trash"`, `"select_file"` → `"file:select_file"`
- `"clipboard_paste"` → `"clip:clipboard_paste"`, etc. (check `src/actions/builders/clipboard.rs` for exact mappings)
- `"submit"` → `"chat:submit"`, etc. (check `src/actions/builders/chat.rs` for exact mappings)
- Check `src/actions/builders/notes.rs` and `src/actions/builders/script_context.rs` for additional namespaced IDs

Build the full mapping by grepping each builder file for the new `"prefix:id"` patterns and systematically find-replace in test files. Verify with `cargo test -- --test-threads=1 2>&1 | grep FAILED | wc -l` after each batch.

### Task 2: worker-revert-semantics
**Scope:** `src/executor/scriptlet.rs`, `src/execute_script/mod.rs`, `src/actions/window.rs`, `src/window_resize/mod.rs`, `src/config/mod.rs`, `src/prompts/markdown/mod.rs`, `src/app_impl/execution_scripts.rs`
**Task:**
Revert the following dirty-tree behavior changes while **keeping** the good `#[cfg(test)]` re-export additions:
1. `src/executor/scriptlet.rs`: Revert `run_scriptlet_template()` to NOT quote template values (test expects `"Hello World!"`, not `"Hello 'World'!"`). Also restore the deleted `#[cfg(test)] mod secure_tempfile_tests` and `#[cfg(test)] mod scriptlet_environment_allowlist_tests` inline modules.
2. `src/execute_script/mod.rs`: Revert `session.process_handle.pid` back to `session.pid()`.
3. `src/actions/window.rs`: Revert `const { assert!(!ACTIONS_WINDOW_RESIZE_ANIMATE) };` back to `assert!(!ACTIONS_WINDOW_RESIZE_ANIMATE, "Actions window resize must stay instant with animation disabled");`.
4. `src/window_resize/mod.rs`: Revert `const { assert!(!WINDOW_RESIZE_ANIMATE) };` back to `assert!(!WINDOW_RESIZE_ANIMATE, "Window resize must stay instant with animation disabled");`.
5. `src/config/mod.rs`: Restore the full `#[cfg(test)] pub use defaults::{...}` list including `DEFAULT_LAYOUT_MAX_HEIGHT`, `DEFAULT_LAYOUT_STANDARD_HEIGHT`, `DEFAULT_WATCHER_*` constants.
6. `src/prompts/markdown/mod.rs`: Revert `use test_support::*` back to `pub(super) use test_support::*`.
7. `src/app_impl/execution_scripts.rs`: Restore the deleted `#[cfg(test)] mod builtin_command_window_visibility_tests` inline module.

**Keep these good changes from the dirty tree:**
- `src/scripts/input_detection.rs`: `#[cfg(test)] pub use detection::is_code_snippet;`
- `src/scripts/scriptlet_loader.rs`: `#[cfg(test)] pub(crate) use loading::build_scriptlet_file_path;` etc.
- `src/scripts/search.rs`: `#[cfg(test)] pub use unified::{...}` and `#[cfg(test)] pub(crate) use ascii::{...}`

### Task 3: worker-verify-gate
**Scope:** (whole project)
**Task:**
After Tasks 1 and 2 complete, run the full verification gate: `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`. If failures remain, triage and fix. The SIGABRT should disappear once the 1553 panicking tests are fixed (panics on background threads during parallel test execution cause the abort signal).

---

NEXT_AREA: After gate is green, delete the numbered `dialog_builtin_action_validation_tests_*.rs` files and consolidate into semantic test modules per CLAUDE.md consistency rules.
