# 019 Path Prompt Bundle Map

Slug: `path-prompt-atlas`

Feature: Path Prompt / `path()` / file and directory selection / path actions.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/prompt-runtime/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/file-search-portals/SKILL.md`
- `lat.md/scripting.md`
- `lat.md/protocol.md`
- `lat.md/surfaces.md`
- `lat.md/builtins.md`
- `lat.md/verification.md`
- `lat.md/design.md`
- `lat.md/acp-chat.md`
- `scripts/kit-sdk.ts`
- `src/prompt_handler/mod.rs`
- `src/main_sections/app_view_state.rs`
- `src/main_sections/render_impl.rs`
- `src/main_sections/prompt_messages.rs`
- `src/prompts/path/mod.rs`
- `src/prompts/path/prompt.rs`
- `src/prompts/path/render.rs`
- `src/prompts/path/types.rs`
- `src/render_prompts/path.rs`
- `src/file_search/mod.rs`
- `src/file_search/directory.rs`
- `src/render_builtins/file_search.rs`
- `src/render_builtins/file_search_list.rs`
- `src/render_builtins/file_search_layout.rs`
- `src/app_impl/path_action.rs`
- `src/app_impl/root_file_search.rs`
- `src/app_impl/ui_window.rs`
- `src/focus_coordinator/mod.rs`
- `src/app_layout/collect_elements.rs`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `tests/sdk/test-path.ts`
- `tests/smoke/test-path-key-events.ts`
- `tests/smoke/test-path-actions-visual.ts`
- `tests/smoke/test-path-visual-consistency.ts`
- `tests/smoke/test-protocol-submit.ts`
- `tests/source_audits/root_file_search_contract.rs`
- `tests/file_search_drag_and_verbs.rs`
- `tests/file_search_mutation_refresh.rs`
- `tests/file_search_tilde_entry.rs`
- `tests/file_search_ai_routing.rs`
- `tests/hide_path_actions_dialog_registry_teardown_contract.rs`
- `tests/hide_path_confirm_popup_registry_teardown_contract.rs`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md .agents/skills/file-search-portals/SKILL.md lat.md/scripting.md lat.md/protocol.md lat.md/surfaces.md lat.md/builtins.md lat.md/verification.md lat.md/design.md lat.md/acp-chat.md scripts/kit-sdk.ts src/prompt_handler/mod.rs src/main_sections/app_view_state.rs src/main_sections/render_impl.rs src/main_sections/prompt_messages.rs src/prompts/path/mod.rs src/prompts/path/prompt.rs src/prompts/path/render.rs src/prompts/path/types.rs src/render_prompts/path.rs src/file_search/mod.rs src/file_search/directory.rs src/render_builtins/file_search.rs src/render_builtins/file_search_list.rs src/render_builtins/file_search_layout.rs src/app_impl/path_action.rs src/app_impl/root_file_search.rs src/app_impl/ui_window.rs src/focus_coordinator/mod.rs src/app_layout/collect_elements.rs src/main_entry/runtime_stdin_match_simulate_key.rs tests/sdk/test-path.ts tests/smoke/test-path-key-events.ts tests/smoke/test-path-actions-visual.ts tests/smoke/test-path-visual-consistency.ts tests/smoke/test-protocol-submit.ts tests/source_audits/root_file_search_contract.rs tests/file_search_drag_and_verbs.rs tests/file_search_mutation_refresh.rs tests/file_search_tilde_entry.rs tests/file_search_ai_routing.rs tests/hide_path_actions_dialog_registry_teardown_contract.rs tests/hide_path_confirm_popup_registry_teardown_contract.rs -s "globalThis.path" -s "PathPrompt" -s "AppView::PathPrompt" -s "render_path_prompt" -s "PromptMessage::ShowPath" -s "PathOptions" -s "FocusTarget::PathPrompt" -s "path_prompt" -s "PathAction" -s "select_file" -s "copy_path" -s "move_to_trash" -s "ViewType::PathPrompt" -s "handle_enter" -s "FileSearch" -s "path prompt" -l 8 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/path-prompt-atlas.txt
```

Final bundle size on disk: 137,206 bytes. Packx reported 39,492 exact tokens and 134,820 total chars.
