# 018 Editor And Template Prompt Bundle Map

Slug: `editor-template-prompt-atlas`

Feature: Editor and Template Prompt / `editor()` / `template()` / snippet tabstops / full-height editing.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/prompt-runtime/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `lat.md/scripting.md`
- `lat.md/protocol.md`
- `lat.md/surfaces.md`
- `lat.md/verification.md`
- `lat.md/acp-chat.md`
- `lat.md/design.md`
- `scripts/kit-sdk.ts`
- `src/prompt_handler/mod.rs`
- `src/main_sections/app_view_state.rs`
- `src/main_sections/render_impl.rs`
- `src/main_sections/prompt_messages.rs`
- `src/editor/mod.rs`
- `src/render_prompts/editor.rs`
- `src/prompts/template/mod.rs`
- `src/prompts/template/prompt.rs`
- `src/prompts/template/render.rs`
- `src/prompts/template/types.rs`
- `src/prompts/template/tests.rs`
- `src/app_impl/actions_dialog.rs`
- `src/app_impl/actions_toggle.rs`
- `src/app_impl/theme_focus.rs`
- `src/app_impl/ui_window.rs`
- `src/focus_coordinator/mod.rs`
- `src/app_layout/collect_elements.rs`
- `src/app_layout/build_layout_info.rs`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `src/protocol/message/constructors/general.rs`
- `src/window_resize/mod.rs`
- `src/window_resize/tests.rs`
- `tests/sdk/test-editor.ts`
- `tests/sdk/test-template.ts`
- `tests/smoke/audit-editor.ts`
- `tests/smoke/audit-template-prompt.ts`
- `tests/smoke/test-protocol-submit.ts`
- `tests/smoke/test-editor-actions-keys.ts`
- `tests/smoke/test-editor-height.ts`
- `tests/smoke/test-editor-visual-fill.ts`
- `tests/smoke/test-editor-v2-with-actions.ts`
- `tests/smoke/test-editor-v2-visual.ts`
- `tests/smoke/test-editor-template.ts`
- `tests/smoke/test-template.ts`
- `tests/smoke/test-template-tab-nav.ts`
- `tests/smoke/test-template-simple.ts`
- `tests/smoke/test-template-choices.ts`
- `tests/smoke/test-template-offset-tracking.ts`
- `tests/tab_ai_input_coverage.rs`
- `tests/minimal_chrome_audit.rs`
- `tests/source_audits/resize_presentation_contract.rs`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md lat.md/scripting.md lat.md/protocol.md lat.md/surfaces.md lat.md/verification.md lat.md/acp-chat.md lat.md/design.md scripts/kit-sdk.ts src/prompt_handler/mod.rs src/main_sections/app_view_state.rs src/main_sections/render_impl.rs src/main_sections/prompt_messages.rs src/editor/mod.rs src/render_prompts/editor.rs src/prompts/template/mod.rs src/prompts/template/prompt.rs src/prompts/template/render.rs src/prompts/template/types.rs src/prompts/template/tests.rs src/app_impl/actions_dialog.rs src/app_impl/actions_toggle.rs src/app_impl/theme_focus.rs src/app_impl/ui_window.rs src/focus_coordinator/mod.rs src/app_layout/collect_elements.rs src/app_layout/build_layout_info.rs src/main_entry/runtime_stdin_match_simulate_key.rs src/protocol/message/constructors/general.rs src/window_resize/mod.rs src/window_resize/tests.rs tests/sdk/test-editor.ts tests/sdk/test-template.ts tests/smoke/audit-editor.ts tests/smoke/audit-template-prompt.ts tests/smoke/test-protocol-submit.ts tests/smoke/test-editor-actions-keys.ts tests/smoke/test-editor-height.ts tests/smoke/test-editor-visual-fill.ts tests/smoke/test-editor-v2-with-actions.ts tests/smoke/test-editor-v2-visual.ts tests/smoke/test-editor-template.ts tests/smoke/test-template.ts tests/smoke/test-template-tab-nav.ts tests/smoke/test-template-simple.ts tests/smoke/test-template-choices.ts tests/smoke/test-template-offset-tracking.ts tests/tab_ai_input_coverage.rs tests/minimal_chrome_audit.rs tests/source_audits/resize_presentation_contract.rs -s "globalThis.editor" -s "globalThis.template" -s "EditorPrompt" -s "TemplatePrompt" -s "AppView::EditorPrompt" -s "AppView::TemplatePrompt" -s "render_editor_prompt" -s "EditorPrompt::with_height" -s "EditorPrompt::with_template" -s "TemplatePrompt::new" -s "FocusTarget::EditorPrompt" -s "FocusTarget::TemplatePrompt" -s "ActionsDialogHost::EditorPrompt" -s "editor_prompt" -s "template_prompt" -s "ViewType::EditorPrompt" -s "simulateKey" -s "Cmd+Enter" -s "next_tabstop" -l 10 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/editor-template-prompt-atlas.txt
```

Final bundle size on disk: 197,071 bytes. Packx reported 54,047 exact tokens and 194,430 total chars.
