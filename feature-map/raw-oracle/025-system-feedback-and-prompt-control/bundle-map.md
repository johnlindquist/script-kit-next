# 025 System Feedback And Prompt Control APIs Bundle Map

Slug: `system-feedback-prompt-control-atlas`

Feature: System Feedback and Prompt Control APIs / `beep()` / `say()` / `notify()` / `hud()` / `setStatus()` / `menu()` / `setActions()` / `setInput()`.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/prompt-runtime/SKILL.md`
- `.agents/skills/actions-popups/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/platform-windowing-macos/SKILL.md`
- `.agents/skills/theme-config-preferences/SKILL.md`
- `lat.md/protocol.md`
- `lat.md/verification.md`
- `lat.md/surfaces.md`
- `lat.md/design.md`
- `lat.md/windowing.md`
- `lat.md/tray-menu.md`
- `lat.md/acp-chat.md`
- `scripts/kit-sdk.ts`
- `src/protocol/message/variants/prompts_media.rs`
- `src/protocol/message/variants/query_ops.rs`
- `src/protocol/message/variants/system_control.rs`
- `src/protocol/message/constructors/general.rs`
- `src/protocol/message/constructors/final_sections.rs`
- `src/protocol/types/batch_wait.rs`
- `src/main_sections/prompt_messages.rs`
- `src/prompt_handler/mod.rs`
- `src/hud_manager/mod.rs`
- `src/app_impl/actions_dialog.rs`
- `src/actions/`
- `src/actions_dialog_items.rs`
- `src/actions_button_visibility_tests.rs`
- `src/app_actions/`
- `src/tray/mod.rs`
- `src/main_entry/runtime_tray_hotkeys.rs`
- `tests/hud_visibility_decoupled_contract.rs`
- `tests/smoke/test-hud.ts`
- `tests/smoke/test-hud-multiple.ts`
- `tests/smoke/test-hud-auto-dismiss.ts`
- `tests/smoke/test-sdk-actions.ts`
- `tests/actions_dialog_batch_setinput_resize_parity_contract.rs`
- `tests/actions_dialog_enter_routing_contract.rs`
- `tests/actions_dialog_arrow_nav_skips_section_headers_contract.rs`
- `tests/protocol/types/batch.rs`
- `tests/sdk_automation_runtime/mod.rs`
- `tests/sdk_automation_contracts/mod.rs`
- `scripts/agentic/index.ts`
- `scripts/agentic/macos-input.ts`
- `scripts/generate-api-tests.ts`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/actions-popups/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md .agents/skills/platform-windowing-macos/SKILL.md .agents/skills/theme-config-preferences/SKILL.md lat.md/protocol.md lat.md/verification.md lat.md/surfaces.md lat.md/design.md lat.md/windowing.md lat.md/tray-menu.md lat.md/acp-chat.md scripts/kit-sdk.ts src/protocol/message/variants/prompts_media.rs src/protocol/message/variants/query_ops.rs src/protocol/message/variants/system_control.rs src/protocol/message/constructors/general.rs src/protocol/message/constructors/final_sections.rs src/protocol/types/batch_wait.rs src/main_sections/prompt_messages.rs src/prompt_handler/mod.rs src/hud_manager/mod.rs src/app_impl/actions_dialog.rs src/actions src/actions_dialog_items.rs src/actions_button_visibility_tests.rs src/app_actions src/tray/mod.rs src/main_entry/runtime_tray_hotkeys.rs tests/hud_visibility_decoupled_contract.rs tests/smoke/test-hud.ts tests/smoke/test-hud-multiple.ts tests/smoke/test-hud-auto-dismiss.ts tests/smoke/test-sdk-actions.ts tests/actions_dialog_batch_setinput_resize_parity_contract.rs tests/actions_dialog_enter_routing_contract.rs tests/actions_dialog_arrow_nav_skips_section_headers_contract.rs tests/protocol/types/batch.rs tests/sdk_automation_runtime/mod.rs tests/sdk_automation_contracts/mod.rs scripts/agentic/index.ts scripts/agentic/macos-input.ts scripts/generate-api-tests.ts -s "globalThis.beep" -s "globalThis.say" -s "globalThis.notify" -s "globalThis.hud" -s "globalThis.setStatus" -s "globalThis.menu" -s "globalThis.setActions" -s "globalThis.setInput" -s "PromptMessage::ShowHud" -s "PromptMessage::SetStatus" -s "PromptMessage::SetInput" -s "PromptMessage::SetActions" -s "Message::Notify" -s "Message::Beep" -s "Message::Say" -s "Message::Hud" -s "Message::SetStatus" -s "Message::Menu" -s "SetActions" -s "ActionTriggered" -s "set_sdk_actions_and_shortcuts" -s "set_prompt_input" -s "BatchCommand::SetInput" -s "show_hud" -s "hud_manager" -s "script_requested_hide" -s "setActions" -s "setInput" -l 14 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/system-feedback-prompt-control-atlas.txt
```

Final bundle size on disk: 103,773 bytes. Packx reported 29,614 exact tokens and 101,394 total chars.
