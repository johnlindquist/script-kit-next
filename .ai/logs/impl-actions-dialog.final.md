Implemented the high-priority actions-dialog system updates from `plans/actions-dialog-system.md` within scope.

**Changed Files**
- `src/actions/dialog.rs`
- `src/actions/builders.rs`
- `src/actions/dialog_tests.rs`
- `src/actions/builders_tests.rs`

**What Changed**
- `src/actions/dialog.rs:131` added shared helpers:
  - `initial_selection_index(...)`
  - `should_rebuild_grouped_items_for_config_change(...)`
  - `resolve_selected_protocol_action_index(...)`
- `src/actions/dialog.rs:263` added `sdk_action_indices` to preserve visible-row → protocol-index mapping.
- `src/actions/dialog.rs:308` introduced `from_actions_with_context(...)` and routed constructors through it for consistent initialization/selection.
- `src/actions/dialog.rs:556` updated `set_config(...)` to rebuild grouped rows when section style changes and coerce selection safely.
- `src/actions/dialog.rs:668` updated `set_sdk_actions(...)` to build stable index mapping and warn on duplicate SDK names.
- `src/actions/dialog.rs:1129` updated `get_selected_protocol_action(...)` to use index mapping (not duplicate-prone name matching).
- `src/actions/dialog.rs:732` now delegates shortcut formatting to shared builder formatter.
- `src/actions/builders.rs:277` promoted formatter as shared `pub(crate)` and aligned symbol/key normalization with dialog behavior.
- `src/actions/builders.rs:577` added action ID dedupe in `get_script_context_actions(...)` with warning on duplicates.
- `src/actions/dialog_tests.rs:445` added regression tests for header-safe initial selection, config-rebuild predicate, and SDK index mapping.
- `src/actions/builders_tests.rs:1304` added regression test ensuring mixed flags do not produce duplicate action IDs.

**Verification Run**
- Passed: `cargo check`
- Attempted targeted test: `cargo test --lib initial_selection_index_skips_header_row -- --nocapture`
- Blocked by unrelated existing compile error outside scope:
  - `src/ai/window.rs` `E0599`: missing `AiApp::message_body_content`

**How To Test**
1. `cargo check`
2. After the unrelated `src/ai/window.rs` test compile error is fixed:
   - `cargo test --lib initial_selection_index_skips_header_row -- --nocapture`
   - `cargo test --lib selected_protocol_action_uses_visible_index_mapping -- --nocapture`
   - `cargo test --lib mixed_script_and_agent_flags_do_not_create_duplicate_action_ids -- --nocapture`
   - `cargo test --lib config_change_requires_rebuild_when_section_style_changes -- --nocapture`

**Risks / Known Gaps**
- Full `cargo test`/`cargo test --lib` verification is currently blocked by unrelated `src/ai/window.rs` failures.
- Script-context ID dedupe is intentional; if any future behavior depends on duplicate IDs, those duplicates will now be filtered and only first occurrence kept.

**Swarm Coordination**
- Status updated to `done` for `impl-actions-dialog`.
- Released all claims for scoped files.

**Commits**
- No commits made.