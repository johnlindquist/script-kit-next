# 023 Mini And Micro Prompts Bundle Map

Slug: `mini-micro-prompts-atlas`

Feature: Mini and Micro Prompts / `mini()` / `micro()` / compact arg-like choice prompts.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/prompt-runtime/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/window-resizing/SKILL.md`
- `.agents/skills/keyboard-focus-routing/SKILL.md`
- `lat.md/scripting.md`
- `lat.md/protocol.md`
- `lat.md/surfaces.md`
- `lat.md/verification.md`
- `lat.md/design.md`
- `lat.md/windowing.md`
- `lat.md/tests/mini-window-contract.md`
- `scripts/kit-sdk.ts`
- `src/prompt_handler/mod.rs`
- `src/execute_script/mod.rs`
- `src/main_sections/app_view_state.rs`
- `src/main_sections/render_impl.rs`
- `src/main_sections/prompt_messages.rs`
- `src/render_prompts/mini.rs`
- `src/render_prompts/micro.rs`
- `src/render_prompts/arg/helpers.rs`
- `src/render_prompts/arg/render.rs`
- `src/components/prompt_layout_shell.rs`
- `src/app_impl/ui_window.rs`
- `src/app_impl/prompt_ai.rs`
- `src/app_layout/collect_elements.rs`
- `src/app_layout/build_layout_info.rs`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `src/window_resize/mod.rs`
- `src/window_resize/tests.rs`
- `tests/sdk/test-editor.ts`
- `tests/autonomous/test-core-prompts.ts`
- `tests/autonomous/test-prompt-transitions.ts`
- `tests/mini_window_sizing_contract.rs`
- `tests/dictation_setup_nux_contract.rs`
- `tests/minimal_chrome_audit.rs`
- `tests/tab_ai_input_coverage.rs`
- `tests/smoke/test-sdk-warnings.ts`
- `tests/smoke/test-protocol-submit.ts`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md .agents/skills/window-resizing/SKILL.md .agents/skills/keyboard-focus-routing/SKILL.md lat.md/scripting.md lat.md/protocol.md lat.md/surfaces.md lat.md/verification.md lat.md/design.md lat.md/windowing.md lat.md/tests/mini-window-contract.md scripts/kit-sdk.ts src/prompt_handler/mod.rs src/execute_script/mod.rs src/main_sections/app_view_state.rs src/main_sections/render_impl.rs src/main_sections/prompt_messages.rs src/render_prompts/mini.rs src/render_prompts/micro.rs src/render_prompts/arg/helpers.rs src/render_prompts/arg/render.rs src/components/prompt_layout_shell.rs src/app_impl/ui_window.rs src/app_impl/prompt_ai.rs src/app_layout/collect_elements.rs src/app_layout/build_layout_info.rs src/main_entry/runtime_stdin_match_simulate_key.rs src/window_resize/mod.rs src/window_resize/tests.rs tests/sdk/test-editor.ts tests/autonomous/test-core-prompts.ts tests/autonomous/test-prompt-transitions.ts tests/mini_window_sizing_contract.rs tests/dictation_setup_nux_contract.rs tests/minimal_chrome_audit.rs tests/tab_ai_input_coverage.rs tests/smoke/test-sdk-warnings.ts tests/smoke/test-protocol-submit.ts -s "globalThis.mini" -s "globalThis.micro" -s "mini() is not yet implemented" -s "micro() is not yet implemented" -s "PromptMessage::ShowMini" -s "PromptMessage::ShowMicro" -s "AppView::MiniPrompt" -s "AppView::MicroPrompt" -s "render_mini_prompt" -s "render_micro_prompt" -s "ViewType::MiniPrompt" -s "ArgPromptNoChoices" -s "mini_prompt" -s "MicroPrompt" -s "footerless" -s "visibleChoiceCount" -s "selectByValue" -s "simulateKey" -l 12 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/mini-micro-prompts-atlas.txt
```

Final bundle size on disk: 167,123 bytes. Packx reported 46,207 exact tokens and 164,907 total chars.
