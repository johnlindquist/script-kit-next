**Changed Files**
- `src/actions/types/script_info.rs:95`  
  - `Default` now uses `DEFAULT_ACTION_VERB` (single source of truth for `"Run"`).
- `src/actions/types/script_info.rs:261`  
  - Added `ScriptInfo::agent(...)` constructor so agent metadata can be represented without manual flag mutation.
- `src/actions/types/script_info.rs:344`  
  - Replaced narrow `From` impls with a generic `impl<Name, Path> From<(Name, Path)>` (`Into<String>` for both), covering mixed tuple forms like `(String, &str)` and `(&str, String)`.
- `src/actions/types/script_info.rs:354`  
  - Added tests:
    - `test_script_info_agent_sets_expected_flags_when_constructed`
    - `test_script_info_from_converts_mixed_tuple_when_name_owned_path_borrowed`
    - `test_script_info_from_converts_mixed_tuple_when_name_borrowed_path_owned`

I audited `src/actions/types/action_model.rs` for this task but did not make additional edits there.

**How To Test**
1. `cargo check`
2. `cargo clippy --all-targets -- -D warnings`
3. (Scoped) `cargo test --lib script_info_completeness_tests::`

I ran (1) and (2). Both currently fail due unrelated workspace issues outside this task (for example `src/protocol/message/mod.rs:87`, `src/actions/tests/part_01.rs:7`, and multiple `clipboard_history` errors like `src/clipboard_history/database.rs:454`).

**Risks / Known Gaps**
- Full verification is currently blocked by unrelated in-flight changes in the shared tree.
- New `ScriptInfo::agent(...)` is available, but existing call sites still using manual `is_agent` mutation were not changed in this scoped task.