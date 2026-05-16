# 021 Env Prompt Bundle Map

Slug: `env-prompt-atlas`

Feature: Env Prompt / `env()` / environment variable and secret prompt / keyring storage.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/prompt-runtime/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/storage-cache-security/SKILL.md`
- `lat.md/scripting.md`
- `lat.md/protocol.md`
- `lat.md/surfaces.md`
- `lat.md/verification.md`
- `lat.md/design.md`
- `lat.md/builtins.md`
- `lat.md/logging.md`
- `lat.md/workspace.md`
- `scripts/kit-sdk.ts`
- `src/prompt_handler/mod.rs`
- `src/execute_script/mod.rs`
- `src/app_execute/execution_helpers.rs`
- `src/main_sections/app_view_state.rs`
- `src/main_sections/render_impl.rs`
- `src/main_sections/prompt_messages.rs`
- `src/prompts/env/mod.rs`
- `src/prompts/env/prompt.rs`
- `src/prompts/env/render.rs`
- `src/prompts/env/helpers.rs`
- `src/prompts/env/tests.rs`
- `src/secrets.rs`
- `src/render_prompts/other.rs`
- `src/app_impl/ui_window.rs`
- `src/app_impl/theme_focus.rs`
- `src/app_impl/window_orchestrator_bridge.rs`
- `src/focus_coordinator/mod.rs`
- `src/app_layout/collect_elements.rs`
- `src/app_layout/build_layout_info.rs`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `src/protocol/message/constructors/general.rs`
- `tests/sdk/test-env.ts`
- `tests/smoke/test-env-prompt.ts`
- `tests/smoke/test-env-visual.ts`
- `tests/smoke/test-env-keychain.ts`
- `tests/smoke/test-env-prompt-existing.ts`
- `tests/smoke/test-env-prompt-with-title.ts`
- `tests/smoke/test-env-prompt-overflow.ts`
- `tests/smoke/test-protocol-submit.ts`
- `tests/tab_ai_input_coverage.rs`
- `tests/minimal_chrome_audit.rs`
- `tests/source_audits/execution_helpers.rs`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md .agents/skills/storage-cache-security/SKILL.md lat.md/scripting.md lat.md/protocol.md lat.md/surfaces.md lat.md/verification.md lat.md/design.md lat.md/builtins.md lat.md/logging.md lat.md/workspace.md scripts/kit-sdk.ts src/prompt_handler/mod.rs src/execute_script/mod.rs src/app_execute/execution_helpers.rs src/main_sections/app_view_state.rs src/main_sections/render_impl.rs src/main_sections/prompt_messages.rs src/prompts/env/mod.rs src/prompts/env/prompt.rs src/prompts/env/render.rs src/prompts/env/helpers.rs src/prompts/env/tests.rs src/secrets.rs src/render_prompts/other.rs src/app_impl/ui_window.rs src/app_impl/theme_focus.rs src/app_impl/window_orchestrator_bridge.rs src/focus_coordinator/mod.rs src/app_layout/collect_elements.rs src/app_layout/build_layout_info.rs src/main_entry/runtime_stdin_match_simulate_key.rs src/protocol/message/constructors/general.rs tests/sdk/test-env.ts tests/smoke/test-env-prompt.ts tests/smoke/test-env-visual.ts tests/smoke/test-env-keychain.ts tests/smoke/test-env-prompt-existing.ts tests/smoke/test-env-prompt-with-title.ts tests/smoke/test-env-prompt-overflow.ts tests/smoke/test-protocol-submit.ts tests/tab_ai_input_coverage.rs tests/minimal_chrome_audit.rs tests/source_audits/execution_helpers.rs -s "globalThis.env" -s "env(" -s "EnvPrompt" -s "AppView::EnvPrompt" -s "render_env_prompt" -s "PromptMessage::ShowEnv" -s "FocusTarget::EnvPrompt" -s "env_prompt" -s "secret" -s "keyring" -s "exists_in_keyring" -s "modified_at" -s "auto-submit" -s "show_api_key_prompt" -s "ViewType::DivPrompt" -l 16 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/env-prompt-atlas.txt
```

Final bundle size on disk: 199,030 bytes. Packx reported 58,239 exact tokens and 196,676 total chars.
