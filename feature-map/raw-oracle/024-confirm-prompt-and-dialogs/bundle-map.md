# 024 Confirm Prompt And Dialogs Bundle Map

Slug: `confirm-prompt-dialogs-atlas`

Feature: Confirm Prompt and Dialogs / SDK `confirm()` / in-window confirm state / parent confirm popup fallback.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/actions-popups/SKILL.md`
- `.agents/skills/prompt-runtime/SKILL.md`
- `.agents/skills/keyboard-focus-routing/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/launcher-surface-contracts/SKILL.md`
- `lat.md/design.md`
- `lat.md/surfaces.md`
- `lat.md/automation.md`
- `lat.md/protocol.md`
- `lat.md/builtins.md`
- `lat.md/sharing.md`
- `lat.md/storybook.md`
- `lat.md/verification.md`
- `scripts/kit-sdk.ts`
- `src/protocol/message/variants/prompts_media.rs`
- `src/prompt_handler/mod.rs`
- `src/confirm/`
- `src/app_impl/about_route.rs`
- `src/app_impl/ui_window.rs`
- `src/main_sections/app_view_state.rs`
- `src/main_sections/render_impl.rs`
- `src/main_entry/app_run_setup.rs`
- `src/main_entry/runtime_stdin.rs`
- `src/main_entry/runtime_stdin_match_core.rs`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `src/app_execute/builtin_confirmation.rs`
- `src/app_execute/builtin_execution.rs`
- `src/app_actions/handle_action/clipboard.rs`
- `src/app_actions/handle_action/files.rs`
- `src/app_actions/handle_action/scripts.rs`
- `src/app_actions/helpers.rs`
- `src/notes/window/`
- `src/ai/window/`
- `src/ai/acp/view.rs`
- `src/windows/automation_surface_collector.rs`
- `src/windows/automation_registry.rs`
- `src/stories/popup_component_states.rs`
- `tests/smoke/test-confirm-screenshot.ts`
- `tests/smoke/test-confirm-focus.ts`
- `tests/smoke/test-confirm-tab.ts`
- `tests/sdk_automation_semantic_ids/mod.rs`
- `tests/main_automation_surface_rekey_owner_contract.rs`
- `tests/main_window_footer_surface_owner_contract.rs`
- `tests/hide_path_confirm_popup_registry_teardown_contract.rs`
- `tests/surface_contract_matrix_artifact_contract.rs`
- `tests/actions_dialog_arrow_nav_skips_section_headers_contract.rs`
- `tests/actions/dialog_random_action_window_tests.rs`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/actions-popups/SKILL.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/keyboard-focus-routing/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md .agents/skills/launcher-surface-contracts/SKILL.md lat.md/design.md lat.md/surfaces.md lat.md/automation.md lat.md/protocol.md lat.md/builtins.md lat.md/sharing.md lat.md/storybook.md lat.md/verification.md scripts/kit-sdk.ts src/protocol/message/variants/prompts_media.rs src/prompt_handler/mod.rs src/confirm src/app_impl/about_route.rs src/app_impl/ui_window.rs src/main_sections/app_view_state.rs src/main_sections/render_impl.rs src/main_entry/app_run_setup.rs src/main_entry/runtime_stdin.rs src/main_entry/runtime_stdin_match_core.rs src/main_entry/runtime_stdin_match_simulate_key.rs src/app_execute/builtin_confirmation.rs src/app_execute/builtin_execution.rs src/app_actions/handle_action/clipboard.rs src/app_actions/handle_action/files.rs src/app_actions/handle_action/scripts.rs src/app_actions/helpers.rs src/notes/window src/ai/window src/ai/acp/view.rs src/windows/automation_surface_collector.rs src/windows/automation_registry.rs src/stories/popup_component_states.rs tests/smoke/test-confirm-screenshot.ts tests/smoke/test-confirm-focus.ts tests/smoke/test-confirm-tab.ts tests/sdk_automation_semantic_ids/mod.rs tests/main_automation_surface_rekey_owner_contract.rs tests/main_window_footer_surface_owner_contract.rs tests/hide_path_confirm_popup_registry_teardown_contract.rs tests/surface_contract_matrix_artifact_contract.rs tests/actions_dialog_arrow_nav_skips_section_headers_contract.rs tests/actions/dialog_random_action_window_tests.rs -s "confirm" -s "Confirm" -s "ShowConfirm" -s "ConfirmPrompt" -s "confirm_with_parent_dialog" -s "open_parent_confirm_dialog" -s "confirm-popup" -s "panel:confirm-dialog" -s "button:0:confirm" -s "button:1:cancel" -s "ParentConfirmOptions" -s "focused_button" -s "resolve_confirm_prompt" -s "FooterAction::Apply" -s "FooterAction::Close" -s "route_key_to_confirm_popup" -s "simulateKey" -s "TestConfirmation" -l 14 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/confirm-prompt-dialogs-atlas.txt
```

Final bundle size on disk: 294,580 bytes. Packx reported 81,704 exact tokens and 291,970 total chars.
