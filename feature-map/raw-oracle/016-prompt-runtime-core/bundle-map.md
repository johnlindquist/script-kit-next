# 016 Prompt Runtime Core Bundle Map

Slug: `prompt-runtime-core-atlas`

Feature: Prompt Runtime Core / `arg()` / `select()` / `div()` / `md()` / prompt handler routing.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/prompt-runtime/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `lat.md/design.md`
- `lat.md/surfaces.md`
- `lat.md/protocol.md`
- `lat.md/scripting.md`
- `lat.md/verification.md`
- `scripts/kit-sdk.ts`
- `src/prompt_handler/mod.rs`
- `src/main_sections/app_view_state.rs`
- `src/main_sections/render_impl.rs`
- `src/render_prompts/arg.rs`
- `src/render_prompts/arg/render.rs`
- `src/render_prompts/arg/helpers.rs`
- `src/render_prompts/div.rs`
- `src/render_prompts/other.rs`
- `src/prompts/select/prompt.rs`
- `src/prompts/select/render.rs`
- `src/prompts/select/search.rs`
- `src/prompts/select/types.rs`
- `src/prompts/div/prompt.rs`
- `src/prompts/div/render.rs`
- `src/prompts/div/render_html.rs`
- `src/prompts/markdown/mod.rs`
- `src/prompts/markdown/render_blocks.rs`
- `src/app_layout/collect_elements.rs`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `tests/sdk/test-arg.ts`
- `tests/sdk/test-select.ts`
- `tests/sdk/test-div.ts`
- `tests/sdk/test-md.ts`
- `tests/sdk/test-prompt-flow.ts`
- `tests/smoke/test-arg-actions-cmdk.ts`
- `tests/smoke/test-arg-text-submit.ts`
- `tests/smoke/test-div-submit-links.ts`
- `tests/smoke/test-select-actions-cmdk.ts`
- `tests/smoke/test-md-div-integration.ts`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md lat.md/design.md lat.md/surfaces.md lat.md/protocol.md lat.md/scripting.md lat.md/verification.md scripts/kit-sdk.ts src/prompt_handler/mod.rs src/main_sections/app_view_state.rs src/main_sections/render_impl.rs src/render_prompts/arg.rs src/render_prompts/arg/render.rs src/render_prompts/arg/helpers.rs src/render_prompts/div.rs src/render_prompts/other.rs src/prompts/select/prompt.rs src/prompts/select/render.rs src/prompts/select/search.rs src/prompts/select/types.rs src/prompts/div/prompt.rs src/prompts/div/render.rs src/prompts/div/render_html.rs src/prompts/markdown/mod.rs src/prompts/markdown/render_blocks.rs src/app_layout/collect_elements.rs src/main_entry/runtime_stdin_match_simulate_key.rs tests/sdk/test-arg.ts tests/sdk/test-select.ts tests/sdk/test-div.ts tests/sdk/test-md.ts tests/sdk/test-prompt-flow.ts tests/smoke/test-arg-actions-cmdk.ts tests/smoke/test-arg-text-submit.ts tests/smoke/test-div-submit-links.ts tests/smoke/test-select-actions-cmdk.ts tests/smoke/test-md-div-integration.ts -s "globalThis.arg" -s "globalThis.select" -s "globalThis.div" -s "globalThis.md" -s "type: 'arg'" -s "type: 'select'" -s "type: 'div'" -s "AppView::ArgPrompt" -s "AppView::DivPrompt" -s "AppView::SelectPrompt" -s "FocusedInput::ArgPrompt" -s "make_submit_callback" -s "render_arg_prompt" -s "render_div_prompt" -s "render_select_prompt" -s "ActionsDialogHost::ArgPrompt" -s "ActionsDialogHost::DivPrompt" -s "ActionsDialogHost::SelectPrompt" -s "collect_term_prompt_elements" -s "submit_arg_prompt_from_current_state" -s "toggle_selection" -l 4 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/prompt-runtime-core-atlas.txt
```

Final bundle size on disk: 29,516 bytes. Packx reported about 8.5k tokens and 27,476 total chars.

A richer context attempt was made first, but broad `actions`/`choices` matches expanded to about 168k tokens. The final bundle keeps focused owner symbols and exact prompt surface paths.
