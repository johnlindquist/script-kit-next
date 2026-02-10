Implemented the split for all assigned `app_*` targets so every file in this scope is now under 500 lines.

**Changed files**
- Wrappers: `src/app_actions.rs`, `src/app_execute.rs`, `src/app_render.rs`, `src/app_layout.rs`, `src/app_navigation.rs`, `src/app_launcher.rs`
- `src/app_actions/*`: `src/app_actions/helpers.rs`, `src/app_actions/handle_action.rs`, `src/app_actions/sdk_actions.rs`, plus new grouped action-arm files in `src/app_actions/handle_action/`
- `src/app_execute/*`: `src/app_execute/builtin_execution.rs`, `src/app_execute/execution_helpers.rs`, `src/app_execute/utility_views.rs`, `src/app_execute/builtin_confirmation.rs`, plus new feature-arm files in `src/app_execute/builtin_execution/`
- `src/app_render/*`: `src/app_render/preview_panel.rs`, `src/app_render/focused_info.rs`, `src/app_render/preview_metadata.rs`, `src/app_render/group_header_item.rs`, `src/app_render/group_header_preview.rs`, `src/app_render/tests.rs`, plus new result-arm files in `src/app_render/preview_panel/`
- `src/app_layout/*`: `src/app_layout/build_component_bounds.rs`, `src/app_layout/build_layout_info.rs`
- `src/app_navigation/*`: `src/app_navigation/helpers.rs`, `src/app_navigation/impl_movement.rs`, `src/app_navigation/impl_scroll.rs`, `src/app_navigation/tests.rs`
- `src/app_launcher/*`: `src/app_launcher/core_types.rs`, `src/app_launcher/db_cache.rs`, `src/app_launcher/icon_cache.rs`, `src/app_launcher/scanning.rs`, `src/app_launcher/launch.rs`, `src/app_launcher/tests.rs`

**How to test**
1. Run the required gate:
```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

**Verification run**
- I ran the required gate command.
- It fails at `cargo check` due unrelated repo-wide errors from other parallel changes (examples: `src/camera/part_000.rs`, `src/editor/part_001.rs`, `src/focus_coordinator/part_000.rs`, `src/watcher/part_000.rs`).
- I also ran `cargo check` with log filtering for `src/app_actions`, `src/app_execute`, and `src/app_render`; there were no diagnostics referencing these files from my split.

**Risks / known gaps**
- Full gate is currently blocked by unrelated workspace compile failures, so clippy/tests were not reached.
- I did not commit or push because the mandatory verification gate is not passing in the shared tree.

**Commits**
- No commits were made.