# 026 Clipboard, Selected Text, And Accessibility APIs Bundle Map

Slug: `clipboard-selected-text-accessibility-atlas`

Feature: Clipboard, Selected Text, and Accessibility APIs / `copy()` / `paste()` / `clipboard.*` / `getSelectedText()` / `setSelectedText()` / accessibility permission helpers.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/platform-windowing-macos/SKILL.md`
- `.agents/skills/storage-cache-security/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `lat.md/protocol.md`
- `lat.md/builtins.md`
- `lat.md/sharing.md`
- `lat.md/permissions.md`
- `lat.md/verification.md`
- `scripts/kit-sdk.ts`
- `src/protocol/message/variants/system_control.rs`
- `src/protocol/message/constructors/general.rs`
- `src/protocol/message/constructors/prompts.rs`
- `src/protocol/types/primitives.rs`
- `src/execute_script/mod.rs`
- `src/prompt_handler/mod.rs`
- `src/selected_text.rs`
- `src/executor/selected_text.rs`
- `src/permissions_wizard.rs`
- `src/clipboard_history/macos_paste.rs`
- `src/app_actions/handle_action/paste.rs`
- `src/app_actions/handle_action/emoji.rs`
- `src/app_actions/handle_action/clipboard.rs`
- `tests/source_audits/stdin_check_accessibility_wired.rs`
- `tests/source_audits/stdin_request_accessibility_wired.rs`
- `tests/source_audits/stdin_get_selected_text_wired.rs`
- `tests/source_audits/stdin_set_selected_text_wired.rs`
- `tests/config_contract_alignment.rs`
- `tests/source_audits/clipboard_actions.rs`
- `tests/source_audits/action_file_clipboard_tools.rs`
- `tests/smoke/test-clipboard-newlines.ts`
- `scripts/generate-api-tests.ts`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/platform-windowing-macos/SKILL.md .agents/skills/storage-cache-security/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md lat.md/protocol.md lat.md/builtins.md lat.md/sharing.md lat.md/permissions.md lat.md/verification.md scripts/kit-sdk.ts src/protocol/message/variants/system_control.rs src/protocol/message/constructors/general.rs src/protocol/message/constructors/prompts.rs src/protocol/types/primitives.rs src/execute_script/mod.rs src/prompt_handler/mod.rs src/selected_text.rs src/executor/selected_text.rs src/permissions_wizard.rs src/clipboard_history/macos_paste.rs src/app_actions/handle_action/paste.rs src/app_actions/handle_action/emoji.rs src/app_actions/handle_action/clipboard.rs tests/source_audits/stdin_check_accessibility_wired.rs tests/source_audits/stdin_request_accessibility_wired.rs tests/source_audits/stdin_get_selected_text_wired.rs tests/source_audits/stdin_set_selected_text_wired.rs tests/config_contract_alignment.rs tests/source_audits/clipboard_actions.rs tests/source_audits/action_file_clipboard_tools.rs tests/smoke/test-clipboard-newlines.ts scripts/generate-api-tests.ts -s "globalThis.clipboard" -s "globalThis.copy" -s "globalThis.paste" -s "globalThis.setSelectedText" -s "globalThis.getSelectedText" -s "globalThis.hasAccessibilityPermission" -s "globalThis.requestAccessibilityPermission" -s "ClipboardMessage" -s "SetSelectedTextMessage" -s "GetSelectedTextMessage" -s "CheckAccessibilityMessage" -s "RequestAccessibilityMessage" -s "Message::Clipboard" -s "ClipboardAction::Read" -s "ClipboardAction::Write" -s "ClipboardFormat::Text" -s "ClipboardFormat::Image" -s "Message::GetSelectedText" -s "Message::SetSelectedText" -s "Message::CheckAccessibility" -s "Message::RequestAccessibility" -s "selected_text_response" -s "text_set_success" -s "text_set_error" -s "accessibility_status" -s "get_selected_text" -s "set_selected_text" -s "has_accessibility_permission" -s "request_accessibility_permission" -s "simulate_paste_with_cg" -s "text_len" -s "clipboard" -l 14 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/clipboard-selected-text-accessibility-atlas.txt
```

Final bundle size on disk: 295,937 bytes. Packx reported 78,239 exact tokens and 293,715 total chars.
