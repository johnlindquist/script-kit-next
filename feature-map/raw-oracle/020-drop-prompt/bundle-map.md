# 020 Drop Prompt Bundle Map

Slug: `drop-prompt-atlas`

Feature: Drop Prompt / `drop()` / drag-and-drop file input / dropped file metadata submission.

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
- `lat.md/verification.md`
- `lat.md/design.md`
- `lat.md/permissions.md`
- `lat.md/builtins.md`
- `scripts/kit-sdk.ts`
- `src/prompt_handler/mod.rs`
- `src/main_sections/app_view_state.rs`
- `src/main_sections/render_impl.rs`
- `src/main_sections/prompt_messages.rs`
- `src/prompts/drop.rs`
- `src/render_prompts/other.rs`
- `src/app_impl/ui_window.rs`
- `src/app_impl/theme_focus.rs`
- `src/app_impl/window_orchestrator_bridge.rs`
- `src/focus_coordinator/mod.rs`
- `src/app_layout/collect_elements.rs`
- `src/app_layout/build_layout_info.rs`
- `src/components/prompt_layout_shell.rs`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `src/protocol/message/constructors/general.rs`
- `src/platform/permiso/drag_source.rs`
- `src/ai/window/images.rs`
- `src/ai/window/render_main_panel.rs`
- `src/prompts/chat/state.rs`
- `src/prompts/chat/render_core.rs`
- `src/mcp_resources/mod.rs`
- `src/setup/embedded_agents_part_000.md`
- `src/ai/script_generation.rs`
- `tests/sdk/test-drop.ts`
- `tests/smoke/test-protocol-submit.ts`
- `tests/tab_ai_input_coverage.rs`
- `tests/minimal_chrome_audit.rs`
- `tests/file_search_drag_and_verbs.rs`
- `tests/file_search_drag_and_verbs/mod.rs`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md .agents/skills/file-search-portals/SKILL.md lat.md/scripting.md lat.md/protocol.md lat.md/surfaces.md lat.md/verification.md lat.md/design.md lat.md/permissions.md lat.md/builtins.md scripts/kit-sdk.ts src/prompt_handler/mod.rs src/main_sections/app_view_state.rs src/main_sections/render_impl.rs src/main_sections/prompt_messages.rs src/prompts/drop.rs src/render_prompts/other.rs src/app_impl/ui_window.rs src/app_impl/theme_focus.rs src/app_impl/window_orchestrator_bridge.rs src/focus_coordinator/mod.rs src/app_layout/collect_elements.rs src/app_layout/build_layout_info.rs src/components/prompt_layout_shell.rs src/main_entry/runtime_stdin_match_simulate_key.rs src/protocol/message/constructors/general.rs src/platform/permiso/drag_source.rs src/ai/window/images.rs src/ai/window/render_main_panel.rs src/prompts/chat/state.rs src/prompts/chat/render_core.rs src/mcp_resources/mod.rs src/setup/embedded_agents_part_000.md src/ai/script_generation.rs tests/sdk/test-drop.ts tests/smoke/test-protocol-submit.ts tests/tab_ai_input_coverage.rs tests/minimal_chrome_audit.rs tests/file_search_drag_and_verbs.rs tests/file_search_drag_and_verbs/mod.rs -s "globalThis.drop" -s "drop()" -s "DropPrompt" -s "AppView::DropPrompt" -s "render_drop_prompt" -s "PromptMessage::ShowDrop" -s "DroppedFile" -s "FileInfo" -s "FocusTarget::DropPrompt" -s "drop_prompt" -s "actionDisabled" -s "no_files" -s "DropPrompt::submit" -s "handle_drop" -s "on_drop" -s "ExternalPaths" -s "DroppedItem" -s "await drop" -s "ViewType::DivPrompt" -l 24 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/drop-prompt-atlas.txt
```

Final bundle size on disk: 148,971 bytes. Packx reported 41,979 exact tokens and 146,697 total chars.
