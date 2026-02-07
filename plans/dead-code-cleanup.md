# Dead Code Cleanup Report

Generated: 2026-02-07
Agent: `codex-dead-code`
Scope: `src/**/*.rs`

## Summary

- Compiler-confirmed unused imports: **26** (all in `src/actions/dialog_builtin_action_validation_tests_*.rs`).
- Compiler-confirmed dead-style warning: **1** (`needless_return` in `src/render_prompts/other.rs:313`).
- `todo!()` / `unimplemented!()` stubs: **none found**.
- Explicit `unreachable!()` macro: **1** (`src/schema_parser.rs:414`).
- `TODO`/`FIXME`/`HACK` comments: **26**.
- `#[allow(dead_code)]` suppressions: **505** occurrences (high dead-code masking risk).
- `#[allow(unused_imports)]` suppressions: **75** occurrences.
- Feature flags in `Cargo.toml`: `system-tests`, `slow-tests`, `ocr`, `perf` are all referenced in Rust code (no unused feature flag found).

## Methodology

Commands used:

- `cargo check --all-targets --message-format short`
- `cargo clippy --all-targets -- -W dead_code -W unused -W unreachable_code -W unreachable_patterns`
- `cargo clippy --bin script-kit-gpui -- -W dead_code -W unused -W unreachable_code -W unreachable_patterns`
- `cargo test`
- `rg` scans for: `TODO|FIXME|HACK`, `unimplemented!|todo!`, `unreachable!`, `#[allow(dead_code)]`, `#[allow(unused_imports)]`, `#[cfg(feature = "...")]`

Limitations:

- Full dead-code linting is partially masked by `#[allow(dead_code)]` across the codebase.
- Test-target compilation currently fails for unrelated issues, reducing automatic dead-code discovery depth:
  - `src/render_prompts/arg.rs:620` (`tests` module redefined; collides with `src/main.rs:3846`)
  - `src/ai/window.rs:8549` (`AiApp::message_body_content` not found)

## 1) Unused Imports (Compiler-Confirmed)

From `cargo check --all-targets`:

1. `src/actions/dialog_builtin_action_validation_tests_7.rs:51` unused import `ActionsDialogConfig`
2. `src/actions/dialog_builtin_action_validation_tests_8.rs:51` unused import `ActionsDialogConfig`
3. `src/actions/dialog_builtin_action_validation_tests_15.rs:25` unused imports `ActionsDialog`, `GroupedActionItem`, `build_grouped_items_static`, `coerce_action_selection`
4. `src/actions/dialog_builtin_action_validation_tests_15.rs:32` unused imports `ScriptletAction`, `Scriptlet`
5. `src/actions/dialog_builtin_action_validation_tests_19.rs:23` unused imports `ScriptletAction`, `Scriptlet`
6. `src/actions/dialog_builtin_action_validation_tests_27.rs:30` unused imports `ScriptletAction`, `Scriptlet`
7. `src/actions/dialog_builtin_action_validation_tests_30.rs:16` unused import `ActionsDialogConfig`
8. `src/actions/dialog_builtin_action_validation_tests_33.rs:16` unused import `ActionsDialogConfig`
9. `src/actions/dialog_builtin_action_validation_tests_34.rs:14` unused imports `build_grouped_items_static`, `coerce_action_selection`
10. `src/actions/dialog_builtin_action_validation_tests_34.rs:16` unused imports `ActionsDialogConfig`, `AnchorPosition`
11. `src/actions/dialog_builtin_action_validation_tests_35.rs:15` unused imports `ACTION_ITEM_HEIGHT`, `POPUP_MAX_HEIGHT`, `POPUP_WIDTH`
12. `src/actions/dialog_builtin_action_validation_tests_35.rs:21` unused imports `ActionsDialogConfig`, `AnchorPosition`, `SearchPosition`
13. `src/actions/dialog_builtin_action_validation_tests_38.rs:21` unused import `build_grouped_items_static`
14. `src/actions/dialog_builtin_action_validation_tests_38.rs:29` unused import `crate::scriptlets::Scriptlet`
15. `src/actions/dialog_builtin_action_validation_tests_39.rs:38` unused import `crate::actions::command_bar::CommandBarConfig`
16. `src/actions/dialog_builtin_action_validation_tests_39.rs:40` unused import `AnchorPosition`
17. `src/actions/dialog_builtin_action_validation_tests_39.rs:41` unused imports `WindowPosition`, `count_section_headers`
18. `src/actions/dialog_builtin_action_validation_tests_39.rs:46` unused import `crate::protocol::ProtocolAction`
19. `src/actions/dialog_builtin_action_validation_tests_41.rs:45` unused import `crate::protocol::ProtocolAction`
20. `src/actions/dialog_builtin_action_validation_tests_44.rs:8` unused imports `GroupedActionItem`, `build_grouped_items_static`, `coerce_action_selection`
21. `src/actions/dialog_builtin_action_validation_tests_44.rs:11` unused imports `ActionsDialogConfig`, `SectionStyle`
22. `src/actions/dialog_builtin_action_validation_tests_45.rs:8` unused imports `GroupedActionItem`, `build_grouped_items_static`, `coerce_action_selection`
23. `src/actions/dialog_builtin_action_validation_tests_45.rs:11` unused imports `ActionsDialogConfig`, `SectionStyle`
24. `src/actions/dialog_builtin_action_validation_tests_46.rs:8` unused import `ActionsDialog`
25. `src/actions/dialog_builtin_action_validation_tests_46.rs:11` unused import `ActionsDialogConfig`
26. `src/actions/dialog_builtin_action_validation_tests_46.rs:17` unused imports `ScriptletAction`, `Scriptlet`

## 2) Unused Functions / Dead Code (Suppressed or Highly Likely)

Because `#[allow(dead_code)]` is widely used, these are high-confidence *masked* dead-code candidates.

### 2.1 Suppression Hotspots (by file count)

- `src/main.rs` (31)
- `src/scriptlets.rs` (25)
- `src/platform.rs` (21)
- `src/focus_coordinator.rs` (21)
- `src/hotkeys.rs` (20)
- `src/file_search.rs` (20)
- `src/actions/types.rs` (19)
- `src/hud_manager.rs` (17)
- `src/config/types.rs` (17)

### 2.2 Representative masked function/const candidates

- `src/scriptlets.rs:91` `parse_bundle_frontmatter`
- `src/scriptlets.rs:116` `tool_type_to_icon`
- `src/scriptlets.rs:142` `resolve_scriptlet_icon`
- `src/scriptlets.rs:292` `Scriptlet::action_id`
- `src/scriptlets.rs:710` `parse_actions_file`
- `src/scriptlets.rs:762` `get_actions_file_path`
- `src/scriptlets.rs:773` `load_shared_actions`
- `src/scriptlets.rs:809` `merge_shared_actions`
- `src/scriptlets.rs:1020` `parse_scriptlets_with_validation`
- `src/scriptlets.rs:1188` `split_by_headers_with_line_numbers`
- `src/scriptlets.rs:1526` `INTERPRETER_TOOLS`
- `src/scriptlets.rs:1536` `get_interpreter_command`
- `src/scriptlets.rs:1555` `interpreter_not_found_message`
- `src/scriptlets.rs:1581` `get_platform_install_instructions`
- `src/scriptlets.rs:1606` `get_macos_install_instructions`
- `src/scriptlets.rs:1673` `is_interpreter_tool`
- `src/scriptlets.rs:1685` `get_interpreter_extension`
- `src/scriptlets.rs:1704` `validate_interpreter_tool`
- `src/app_impl.rs:1542` `request_focus`
- `src/app_impl.rs:1556` `focus_via_coordinator`
- `src/app_impl.rs:1597` `clear_focus_overlays`
- `src/app_impl.rs:1606` `current_cursor_owner`
- `src/app_impl.rs:2130` `invalidate_filter_cache`
- `src/app_impl.rs:2339` `invalidate_preview_cache`
- `src/app_impl.rs:2345` `filtered_scripts`
- `src/app_impl.rs:3902` `toggle_terminal_commands`
- `src/app_impl.rs:4720` `edit_script`
- `src/app_impl.rs:4746` `open_config_for_shortcut`
- `src/app_impl.rs:4804` `create_config_template`
- `src/app_impl.rs:5213` `update_alias_text`
- `src/app_impl.rs:6088` `execute_script_by_path`
- `src/app_impl.rs:6692` `is_dismissable_view`
- `src/platform.rs:266` `ensure_move_to_active_space`
- `src/platform.rs:738` `is_app_active` (one cfg branch)
- `src/platform.rs:753` `is_app_active` (other cfg branch)
- `src/platform.rs:832` `invalidate_focus_cache`
- `src/platform.rs:844` `NS_FLOATING_WINDOW_LEVEL`
- `src/platform.rs:850` `NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE`
- `src/platform.rs:856` `NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY`
- `src/platform.rs:862` `NS_WINDOW_COLLECTION_BEHAVIOR_IGNORES_CYCLE`
- `src/platform.rs:868` `NS_WINDOW_COLLECTION_BEHAVIOR_PARTICIPATES_IN_CYCLE`
- `src/platform.rs:1191` `get_swizzle_diagnostics` (non-mac cfg)
- `src/platform.rs:1201` `get_swizzle_diagnostics` (mac cfg)
- `src/platform.rs:1207` `log_swizzle_diagnostics`
- `src/platform.rs:1345` `configure_window_vibrancy_material`
- `src/platform.rs:1607` `toggle_blending_mode` (non-mac cfg)
- `src/platform.rs:1663` `toggle_blending_mode` (mac cfg)
- `src/platform.rs:2019` `update_all_secondary_windows_appearance`
- `src/platform.rs:2194` `flip_y`
- `src/platform.rs:2927` `open_path_with_system_default`
- `src/secrets.rs:396` `has_secret`
- `src/secrets.rs:413` `list_secret_keys`

## 3) Unused Struct Fields (Suppressed or Highly Likely)

These fields are marked with `#[allow(dead_code)]`, indicating likely unused state or staged scaffolding:

- `src/window_control_enhanced/capabilities.rs:59` `ax_element`
- `src/form_prompt.rs:23` `html`
- `src/executor/runner.rs:241` `script_path`
- `src/main.rs:1255` `focused_clipboard_entry_id`
- `src/main.rs:1267` `gpui_input_subscriptions`
- `src/main.rs:1270` `bounds_subscription`
- `src/main.rs:1273` `appearance_subscription`
- `src/main.rs:1286` `config`
- `src/main.rs:1320` `theme_chooser_scroll_handle`
- `src/main.rs:1424` `pending_path_action_result`
- `src/main.rs:1440` `nav_coalescer`
- `src/main.rs:1466` `last_scrolled_main`
- `src/main.rs:1468` `last_scrolled_arg`
- `src/main.rs:1470` `last_scrolled_clipboard`
- `src/main.rs:1472` `last_scrolled_window`
- `src/main.rs:1474` `last_scrolled_design_gallery`
- `src/builtins.rs:269` `group`
- `src/components/button.rs:40` `text_hover`
- `src/editor.rs:136` `config`
- `src/editor.rs:143` `subscriptions`
- `src/actions/builders.rs:61` `image_dimensions`
- `src/file_search.rs:52` `file_type`
- `src/scheduler.rs:43` `source`
- `src/storybook/browser.rs:24` `current_theme`
- `src/actions/types.rs:338` `description`
- `src/actions/types.rs:350` `has_action`
- `src/actions/types.rs:355` `value`
- `src/terminal/alacritty.rs:351` `theme`
- `src/clipboard_history/types.rs:81` `ocr_text`
- `src/clipboard_history/types.rs:103` `byte_size`
- `src/clipboard_history/types.rs:106` `ocr_text`
- `src/keyword_manager.rs:54` `restart_delay_ms`
- `src/keyword_manager.rs:101` `injector`
- `src/tray.rs:195` `tray_icon`
- `src/hud_manager.rs:44` `accent_active`
- `src/hud_manager.rs:203` `created_at`
- `src/hud_manager.rs:206` `action_label`
- `src/hud_manager.rs:209` `action`
- `src/hud_manager.rs:224` `action_label`
- `src/hud_manager.rs:226` `action`
- `src/hud_manager.rs:352` `slot`

## 4) Unreachable Code / Match Arms

- Explicit unreachable arm: `src/schema_parser.rs:414` (`FieldType::Any => unreachable!("handled above")`).
- No additional compiler-reported `unreachable_patterns` from clippy in current builds.

## 5) Commented-Out Code Blocks / Stale Scaffolding

- `src/main.rs:186` commented-out import: `// use crate::hotkey_pollers::start_hotkey_event_handler;`
- `src/app_execute.rs:1598` blocking scaffold describing a deferred `FileSearchView` integration with commented code snippets.
- `src/ai/window.rs:6934` deprecated dead function kept with suppression marker (`render_command_bar_overlay`).

## 6) TODO / FIXME / HACK Inventory

1. `src/prompt_handler.rs:90` TODO render placeholder in header
2. `src/prompt_handler.rs:91` TODO render hint
3. `src/prompt_handler.rs:92` TODO render footer
4. `src/prompt_handler.rs:1117` TODO implement full UI for new prompt scaffolding
5. `src/app_layout.rs:500` TODO get actual window size from context
6. `src/app_impl.rs:2520` TODO implement agent execution via mdflow
7. `src/app_impl.rs:5985` TODO parse inputs from code
8. `src/platform.rs:754` TODO implement for other platforms
9. `src/platform.rs:825` TODO implement for other platforms
10. `src/platform.rs:2018` TODO use in appearance change handler
11. `src/platform.rs:2145` TODO implement for other platforms
12. `src/platform.rs:2377` TODO implement for other platforms
13. `src/app_execute.rs:568` TODO implement clear conversation
14. `src/ai/window.rs:1264` TODO handle input changes
15. `src/ai/window.rs:2095` TODO proper image clipboard support
16. `src/ai/sdk_handlers.rs:43` TODO get active chat ID from window state
17. `src/ai/sdk_handlers.rs:57` TODO get active chat ID from window state
18. `src/ai/sdk_handlers.rs:265` TODO get streaming status from window state
19. `src/main.rs:51` TODO re-enable hotkey pollers after Root wrapper update
20. `src/main.rs:185` TODO re-enable hotkey poller import
21. `src/main.rs:425` HACK swizzle GPUI BlurredView to preserve tint
22. `src/main.rs:2407` HACK swizzle GPUI BlurredView immediately after window creation
23. `src/config/editor.rs:88` TODO update value if different
24. `src/hud_manager.rs:334` TODO get editor from config
25. `src/storybook/browser.rs:54` TODO implement theme loading from registry
26. `src/storybook/browser.rs:124` TODO focus search input and enable text input

## 7) Stub Macros (`todo!`, `unimplemented!`)

- No matches in `src/**/*.rs`.

## 8) Feature Flags (Unused Flag Audit)

Features declared in `Cargo.toml`:

- `Cargo.toml:151` `default = ["ocr"]`
- `Cargo.toml:155` `system-tests = []`
- `Cargo.toml:159` `slow-tests = []`
- `Cargo.toml:161` `ocr = []`
- `Cargo.toml:165` `perf = []`

Rust-side feature references:

- `src/lib.rs:167` `ocr`
- `src/login_item.rs:199` `system-tests`
- `src/login_item.rs:223` `system-tests`
- `src/login_item.rs:248` `system-tests`
- `src/config/editor.rs:1046` `system-tests`
- `src/app_launcher.rs:1159` `slow-tests`
- `src/app_launcher.rs:1192` `slow-tests`
- `src/app_launcher.rs:1213` `slow-tests`
- `src/app_launcher.rs:1268` `slow-tests`
- `src/app_launcher.rs:1313` `slow-tests`
- `src/app_launcher.rs:1340` `slow-tests`
- `src/executor_tests.rs:253` `system-tests`
- `src/executor_tests.rs:275` `system-tests`
- `src/executor_tests.rs:297` `system-tests`
- `src/executor_tests.rs:323` `system-tests`
- `src/executor_tests.rs:1930` `system-tests`
- `src/executor_tests.rs:1996` `system-tests`
- `src/perf.rs:377` `perf`
- `src/perf.rs:394` `perf`
- `src/perf.rs:434` `perf`

Conclusion: no unused feature flags detected.

## 9) Cleanup Order (Recommended)

1. Remove the 26 compiler-confirmed unused imports in `src/actions/dialog_builtin_action_validation_tests_*.rs`.
2. Resolve test-target compile blockers (`src/render_prompts/arg.rs:620`, `src/ai/window.rs:8549`) to unblock full dead-code linting.
3. Replace broad `#[allow(dead_code)]` with narrower gating where possible (`cfg(test)`, feature-gated modules, or deletion).
4. Prune stale fields/functions in `src/main.rs`, `src/scriptlets.rs`, `src/platform.rs`, `src/hud_manager.rs` after call-site verification.
5. Convert or close TODO/HACK items with tracked issues; remove resolved comments.
6. Re-run full gate:
   - `cargo check --all-targets`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`
