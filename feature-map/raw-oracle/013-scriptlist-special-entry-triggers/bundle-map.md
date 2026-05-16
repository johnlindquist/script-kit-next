# 013 ScriptList Special Entry Triggers Bundle Map




- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/main-menu-search-selection/SKILL.md`
- `.agents/skills/acp-context-composer/SKILL.md`
- `.agents/skills/quick-terminal-pty/SKILL.md`
- `.agents/skills/file-search-portals/SKILL.md`
- `.agents/skills/actions-popups/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `removed-docs`
- `removed-docs`
- `removed-docs`
- `removed-docs`
- `removed-docs`
- `removed-docs`
- `removed-docs`
- `src/app_impl/filter_input_core.rs`
- `src/app_impl/filter_input_change.rs`
- `src/render_script_list/mod.rs`
- `src/main_sections/app_view_state.rs`
- `src/app_impl/tab_ai_mode/acp_entry.rs`
- `src/app_impl/tab_ai_mode/mod.rs`
- `src/app_execute/utility_views.rs`
- `src/app_execute/builtin_execution.rs`
- `src/app_impl/actions_toggle.rs`
- `src/app_impl/actions_dialog.rs`
- `src/ai/window/context_picker/mod.rs`
- `src/ai/window/context_picker/types.rs`
- `src/main_window_preflight/types.rs`
- `src/main_window_preflight/build.rs`
- `tests/file_search_tilde_entry.rs`
- `tests/acp_main_menu_skill_launch_contract.rs`
- `tests/acp_popup_automation_parity_contract.rs`
- `tests/acp_mention_popup_registry_lifecycle_contract.rs`
- `tests/acp_portal_contract.rs`
- `tests/tab_ai_routing.rs`
- `tests/quick_terminal_contracts.rs`
- `scripts/agentic/tx_wait_for_acp_runtime_semantics.ts`


```bash
packx AGENTS.md CLAUDE.md .agents/skills/main-menu-search-selection/SKILL.md .agents/skills/acp-context-composer/SKILL.md .agents/skills/quick-terminal-pty/SKILL.md .agents/skills/file-search-portals/SKILL.md .agents/skills/actions-popups/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md removed-docs removed-docs removed-docs removed-docs removed-docs removed-docs removed-docs src/app_impl/filter_input_core.rs src/app_impl/filter_input_change.rs src/render_script_list/mod.rs src/main_sections/app_view_state.rs src/app_impl/tab_ai_mode/acp_entry.rs src/app_impl/tab_ai_mode/mod.rs src/app_execute/utility_views.rs src/app_execute/builtin_execution.rs src/app_impl/actions_toggle.rs src/app_impl/actions_dialog.rs src/ai/window/context_picker/mod.rs src/ai/window/context_picker/types.rs src/main_window_preflight/types.rs src/main_window_preflight/build.rs tests/file_search_tilde_entry.rs tests/acp_main_menu_skill_launch_contract.rs tests/acp_popup_automation_parity_contract.rs tests/acp_mention_popup_registry_lifecycle_contract.rs tests/acp_portal_contract.rs tests/tab_ai_routing.rs tests/quick_terminal_contracts.rs scripts/agentic/tx_wait_for_acp_runtime_semantics.ts -s "ScriptListSpecialEntry" -s "special_entry_from_script_list_filter" -s "script_list_special_entry_routed" -s "open_tab_ai_acp_with_slash_picker" -s "open_tab_ai_acp_with_mention_picker" -s "open_quick_terminal" -s "ActionsHelp" -s "AcpMentionPicker" -s "AcpSlashPicker" -s "QuickTerminalView" -s "AcpEntryRequest" -s "context_picker" -s "triggerBuiltin tab-ai" -l 10 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/scriptlist-special-triggers-atlas.txt
```
