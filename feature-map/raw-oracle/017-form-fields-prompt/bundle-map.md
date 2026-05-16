# 017 Form And Fields Prompt Bundle Map

Slug: `form-fields-prompt-atlas`

Feature: Form and Fields Prompt / `form()` / `fields()` / specialized field types / SDK form runtime.

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
- `src/main_sections/prompt_messages.rs`
- `src/render_prompts/form.rs`
- `src/render_prompts/form/render.rs`
- `src/render_prompts/form/helpers.rs`
- `src/render_prompts/form/tests.rs`
- `src/prompts/mod.rs`
- `src/prompts/prelude.rs`
- `src/app_layout/collect_elements.rs`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `tests/sdk/test-form-all-types.ts`
- `tests/sdk/test-form-specialized.ts`
- `tests/sdk/test-fields.ts`
- `tests/sdk/test-fields-basic.ts`
- `tests/sdk/test-fields-datetime.ts`
- `tests/sdk/FORM_FIELDS_PARITY_REPORT.md`
- `tests/smoke/test-form-prompt.ts`
- `tests/smoke/test-form-typing.ts`
- `tests/smoke/test-protocol-submit.ts`
- `tests/smoke/audit-fields.ts`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md lat.md/design.md lat.md/surfaces.md lat.md/protocol.md lat.md/scripting.md lat.md/verification.md scripts/kit-sdk.ts src/prompt_handler/mod.rs src/main_sections/app_view_state.rs src/main_sections/render_impl.rs src/main_sections/prompt_messages.rs src/render_prompts/form.rs src/render_prompts/form/render.rs src/render_prompts/form/helpers.rs src/render_prompts/form/tests.rs src/prompts/mod.rs src/prompts/prelude.rs src/app_layout/collect_elements.rs src/main_entry/runtime_stdin_match_simulate_key.rs tests/sdk/test-form-all-types.ts tests/sdk/test-form-specialized.ts tests/sdk/test-fields.ts tests/sdk/test-fields-basic.ts tests/sdk/test-fields-datetime.ts tests/sdk/FORM_FIELDS_PARITY_REPORT.md tests/smoke/test-form-prompt.ts tests/smoke/test-form-typing.ts tests/smoke/test-protocol-submit.ts tests/smoke/audit-fields.ts -s "globalThis.form" -s "globalThis.fields" -s "type: 'form'" -s "type: 'fields'" -s "FieldsMessage" -s "FormPrompt" -s "AppView::FormPrompt" -s "render_form_prompt" -s "FormPromptState::new" -s "FocusTarget::FormPrompt" -s "ActionsDialogHost::FormPrompt" -s "FormEnterBehavior" -s "collect_form_submit_validation_errors" -s "FieldsComingSoon" -s "fields() prompt coming soon" -s "Unhandled message type: Fields" -s "form()" -s "fields()" -l 10 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/form-fields-prompt-atlas.txt
```

Final bundle size on disk: 65,442 bytes. Packx reported about 18.8k tokens and 63,548 total chars.
