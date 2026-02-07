# Test Coverage Gap Analysis (Rust)

Date: 2026-02-07  
Agent: `codex-test-coverage`

## Scope
- Source audited: `src/**/*.rs`
- Tests audited: `src/**/*_tests.rs`, `src/**/tests.rs`, `tests/**/*.rs`
- Out of scope for this report: TypeScript smoke/sdk tests (`tests/**/*.ts`)

## Method
1. Inventory Rust source and Rust test files.
2. Heuristic coverage mapping:
   - file considered "covered" if it has in-file test markers (`#[cfg(test)]` / `#[test]`) or sibling test modules (`*_tests.rs`, `tests.rs`),
   - plus targeted grep for function/codepath references.
3. Quality audit for weak tests:
   - source-string/code-audit style tests,
   - ignored tests,
   - sleep/time-based test patterns,
   - skew toward generated micro-tests.

## Coverage Snapshot

- Rust source files: **381**
- Production-ish Rust files (excluding `*_tests.rs`, `/tests.rs`, `.orig`): **345**
- Likely untested files (heuristic): **116**
- Rust test attributes found (`#[test]`, `#[tokio::test]`, `#[gpui::test]`): **9,210**

Module-level heuristic (selected):

| Module | Total | Covered | Uncovered | Covered % |
|---|---:|---:|---:|---:|
| `(root)` | 82 | 70 | 12 | 85.4 |
| `actions` | 52 | 46 | 6 | 88.5 |
| `components` | 18 | 7 | 11 | 38.9 |
| `render_prompts` | 7 | 1 | 6 | 14.3 |
| `scripts` | 9 | 4 | 5 | 44.4 |
| `executor` | 7 | 3 | 4 | 42.9 |
| `confirm` | 4 | 0 | 4 | 0.0 |
| `theme` | 11 | 7 | 4 | 63.6 |
| `clipboard_history` | 16 | 12 | 4 | 75.0 |
| `stories` | 24 | 0 | 24 | 0.0 |
| `storybook` | 5 | 0 | 5 | 0.0 |

## Priority Findings

### P0: Core runtime paths with minimal direct behavioral coverage
High-LOC, high-impact files with no direct local test modules and weak/no direct references in Rust tests:

- `src/app_impl.rs` (7396 LOC)
- `src/render_builtins.rs` (4678 LOC)
- `src/app_execute.rs` (2174 LOC)
- `src/prompt_handler.rs` (2087 LOC)
- `src/execute_script.rs` (1599 LOC)
- `src/render_script_list.rs` (1291 LOC)
- `src/executor/runner.rs` (923 LOC)
- `src/app_layout.rs` (698 LOC)
- `src/app_navigation.rs` (581 LOC)

Critical codepaths currently under-covered:
- layout construction: `build_layout_info` in `src/app_layout.rs`
- navigation/scroll coalescing: `move_selection_*`, `apply_nav_delta`, `handle_scroll_wheel` in `src/app_navigation.rs`
- prompt message dispatch: `handle_prompt_message` in `src/prompt_handler.rs`
- script execution/session lifecycle: `execute_script_interactive`, `spawn_script`, `ScriptSession` kill/wait in `src/executor/runner.rs`
- builtin view/action rendering and action dispatch in `src/render_builtins.rs`

### P0: Confirm flow has effectively no dedicated tests
`confirm` module is fully untested by local/sibling test heuristics (`0/4` files):
- `src/confirm/mod.rs`
- `src/confirm/constants.rs`
- `src/confirm/dialog.rs`
- `src/confirm/window.rs`

Given wide usage (`open_confirm_window` / `dispatch_confirm_key`) from `src/app_impl.rs`, `src/app_execute.rs`, `src/app_actions.rs`, this is a high regression risk.

### P1: Test quality is skewed toward source-text audits, not behavior
Detected **29** source-file string-audit assertions across 9 files (examples):
- `tests/prompt_footer.rs`
- `src/keyboard_routing_tests.rs`
- `src/actions_button_visibility_tests.rs`
- `src/clipboard_actions_focus_routing_tests.rs`
- `src/list_state_init_tests.rs`
- `src/webcam_actions_consistency_tests.rs`

These tests catch textual regressions but can pass while runtime behavior is broken.

### P1: Heavy concentration in generated action micro-tests reduces marginal signal
- `actions` test attributes: **6,265**
- `src/actions/dialog_builtin_action_validation_tests_*.rs`: **5,480** (**87.5%** of actions test attributes)

Also, purged placeholders are still compiled as modules with zero tests:
- `src/actions/dialog_builtin_action_validation_tests_25.rs`
- `src/actions/dialog_builtin_action_validation_tests_31.rs`
- `src/actions/dialog_builtin_action_validation_tests_37.rs`
- `src/actions/dialog_builtin_action_validation_tests_42.rs`
- `src/actions/dialog_builtin_action_validation_tests_43.rs`

### P1: System-dependent tests are mostly ignored
Ignored tests found: **21** total.
- `src/system_actions.rs`: 10
- `src/text_injector.rs`: 3
- `src/selected_text.rs`: 3
- `src/login_item.rs`: 3
- `src/window_control.rs`: 2

Important system behavior remains unverified in standard CI paths.

### P2: Time-based flakiness risk in Rust tests
Sleep/poll timing patterns in Rust tests (34 hits), concentrated in:
- `src/executor_tests.rs` (20)
- `src/notification/tests.rs` (9)
- `src/notification/service_tests.rs` (3)

These may be stable locally but are sensitive to CI timing variance.

## Recommended New Tests (Prioritized)

### Immediate (P0)
1. `src/app_navigation_tests.rs`
- `test_move_selection_page_down_clamps_to_last_item`
- `test_handle_scroll_wheel_coalesces_rapid_deltas`
- `test_validate_selection_bounds_recovers_from_out_of_range_index`

2. `src/app_layout_tests.rs`
- `test_build_layout_info_reports_prompt_bounds_for_script_list`
- `test_build_layout_info_handles_empty_components_without_panic`

3. `src/confirm/confirm_window_tests.rs`
- `test_dispatch_confirm_key_submits_on_enter`
- `test_dispatch_confirm_key_cancels_on_escape`
- `test_dispatch_confirm_key_toggles_focus_on_tab`
- `test_dispatch_confirm_key_accepts_left_right_and_arrow_variants`

4. `src/executor/runner_integration_tests.rs`
- `test_spawn_script_returns_running_session_and_pid`
- `test_script_session_kill_is_idempotent`
- `test_wait_returns_exit_code_after_process_exit`
- `test_send_receive_message_round_trip_when_channel_open`

5. `src/prompt_handler_tests.rs`
- `test_handle_prompt_message_routes_confirm_request_to_confirm_window`
- `test_handle_prompt_message_ignores_unknown_message_without_state_corruption`

### Next wave (P1)
1. `src/render_builtins_tests.rs`
- `test_toggle_clipboard_actions_updates_state_and_notifies`
- `test_execute_window_action_handles_unknown_action_without_panic`
- `test_theme_chooser_filtered_indices_matches_case_insensitive_filter`

2. `src/execute_script_tests.rs`
- `test_protocol_tile_to_window_control_maps_all_variants`
- `test_get_displays_returns_visible_bounds_for_each_display`

3. `src/theme/theme_types_behavior_tests.rs`
- `test_background_opacity_clamped_within_expected_range`
- `test_get_colors_uses_focus_aware_palette_when_unfocused`
- `test_with_opacity_offset_preserves_invariants`

4. `src/scripts/scheduling_tests.rs`
- `test_register_scheduled_scripts_skips_invalid_schedule_metadata`
- `test_register_scheduled_scripts_counts_only_registered_jobs`

### Quality hardening (P1/P2)
1. Convert source-text regression tests into behavioral tests where possible.
- Keep a small number of code-audit tests for architectural invariants.
- Replace most string matching with API-level or integration assertions.

2. Reduce ignored-test debt.
- Move accessibility/system tests behind `--features system-tests` with deterministic fakes where possible.
- Add CI job for ignored/system suites on dedicated runners.

3. Reduce timing flakiness.
- Replace `thread::sleep` waits with event-driven synchronization (`recv_timeout`, explicit state signals).

4. Rebalance `actions` test suite.
- Keep high-signal generated coverage, but add fewer end-to-end behavior tests that validate key interaction flows across modules.

## Suggested Execution Plan

1. Land P0 tests for `confirm`, `app_navigation`, `app_layout`, `executor/runner` first.
2. Add integration harness for stdin protocol-driven scenarios touching `prompt_handler` + confirm flows.
3. Refactor low-signal source-audit tests in batches (avoid giant rewrite).
4. Track progress with a lightweight checklist in this file (or a dedicated issue).

## Known Limitations of This Analysis

- This is a static + heuristic gap analysis, not instrumentation-based line coverage.
- "Covered" means tests exist nearby or references were detected, not guaranteed branch/path coverage.
- Some cross-module tests may exercise code indirectly without explicit file-name references.
