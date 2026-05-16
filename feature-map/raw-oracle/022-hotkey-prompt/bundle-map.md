# 022 Hotkey Prompt Bundle Map

Slug: `hotkey-prompt-atlas`

Feature: Hotkey Prompt / `hotkey()` / keyboard shortcut capture and adjacent shortcut recorder.

Included context:

- `AGENTS.md`, `CLAUDE.md`
- `.agents/skills/prompt-runtime/SKILL.md`
- `.agents/skills/sdk-script-execution/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/agentic-testing/SKILL.md`
- `.agents/skills/keyboard-focus-routing/SKILL.md`
- `.agents/skills/theme-config-preferences/SKILL.md`
- `lat.md/scripting.md`
- `lat.md/protocol.md`
- `lat.md/surfaces.md`
- `lat.md/verification.md`
- `lat.md/design.md`
- `lat.md/builtins.md`
- `scripts/kit-sdk.ts`
- `src/prompt_handler/mod.rs`
- `src/main_sections/prompt_messages.rs`
- `src/protocol/message/constructors/general.rs`
- `src/components/shortcut_recorder.rs`
- `src/components/shortcut_recorder/component.rs`
- `src/components/shortcut_recorder/render.rs`
- `src/components/shortcut_recorder/render_helpers.rs`
- `src/components/shortcut_recorder/types.rs`
- `src/components/shortcut_recorder/tests.rs`
- `src/app_impl/shortcut_recorder.rs`
- `src/app_impl/refresh_scriptlets.rs`
- `src/app_actions/handle_action/shortcuts.rs`
- `src/hotkeys/mod.rs`
- `src/shortcuts/mod.rs`
- `src/shortcuts/types.rs`
- `src/shortcuts/hotkey_compat.rs`
- `scripts/update-config-shortcut.ts`
- `scripts/remove-config-shortcut.ts`
- `scripts/agentic/menu-shortcut-transitions.ts`
- `src/main_entry/runtime_stdin_match_simulate_key.rs`
- `tests/sdk/test-hotkey.ts`
- `tests/smoke/test-shortcut-recorder.ts`
- `tests/smoke/test-shortcut-recorder-modal.ts`
- `tests/smoke/test-shortcut-recorder-focus.ts`
- `tests/source_audits/shortcut_config_source.rs`
- `tests/source_audits/shortcut_alias_file_actions.rs`
- `tests/source_audits/shortcut_lookup_exports.rs`
- `tests/source_audits/action_shortcut_alias.rs`
- `tests/shortcut_error_messages.rs`
- `tests/shortcut_recorder_popup_window_contract.rs`

Packx command:

```bash
packx AGENTS.md CLAUDE.md .agents/skills/prompt-runtime/SKILL.md .agents/skills/sdk-script-execution/SKILL.md .agents/skills/protocol-automation/SKILL.md .agents/skills/agentic-testing/SKILL.md .agents/skills/keyboard-focus-routing/SKILL.md .agents/skills/theme-config-preferences/SKILL.md lat.md/scripting.md lat.md/protocol.md lat.md/surfaces.md lat.md/verification.md lat.md/design.md lat.md/builtins.md scripts/kit-sdk.ts src/prompt_handler/mod.rs src/main_sections/prompt_messages.rs src/protocol/message/constructors/general.rs src/components/shortcut_recorder.rs src/components/shortcut_recorder/component.rs src/components/shortcut_recorder/render.rs src/components/shortcut_recorder/render_helpers.rs src/components/shortcut_recorder/types.rs src/components/shortcut_recorder/tests.rs src/app_impl/shortcut_recorder.rs src/app_impl/refresh_scriptlets.rs src/app_actions/handle_action/shortcuts.rs src/hotkeys/mod.rs src/shortcuts/mod.rs src/shortcuts/types.rs src/shortcuts/hotkey_compat.rs scripts/update-config-shortcut.ts scripts/remove-config-shortcut.ts scripts/agentic/menu-shortcut-transitions.ts src/main_entry/runtime_stdin_match_simulate_key.rs tests/sdk/test-hotkey.ts tests/smoke/test-shortcut-recorder.ts tests/smoke/test-shortcut-recorder-modal.ts tests/smoke/test-shortcut-recorder-focus.ts tests/source_audits/shortcut_config_source.rs tests/source_audits/shortcut_alias_file_actions.rs tests/source_audits/shortcut_lookup_exports.rs tests/source_audits/action_shortcut_alias.rs tests/shortcut_error_messages.rs tests/shortcut_recorder_popup_window_contract.rs -s "globalThis.hotkey" -s "hotkey()" -s "HotkeyInfo" -s "PromptMessage::HotkeyComingSoon" -s "hotkey() is not yet implemented" -s "shortcut recorder" -s "ShortcutRecorder" -s "record shortcut" -s "update-config-shortcut" -s "remove-config-shortcut" -s "update_script_hotkey" -s "register_script_hotkey" -s "config.ts" -s "refresh_scriptlets" -s "HotkeyConfig" -s "simulateKey" -l 12 --strip-comments --minify -f markdown --no-interactive --stdout > ~/.oracle/bundles/hotkey-prompt-atlas.txt
```

Final bundle size on disk: 227,353 bytes. Packx reported 58,464 exact tokens and 224,794 total chars.
