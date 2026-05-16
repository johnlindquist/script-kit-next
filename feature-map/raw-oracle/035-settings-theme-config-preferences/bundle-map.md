# 035 Settings, Theme, Config, and Preferences Bundle Map




## Lat Context

```bash
source search "settings theme config preferences theme chooser font scale model provider config.ts user preferences runtime settings"
```


- `removed-docs`
- `removed-docs`
- `removed-docs themes directory`
- `removed-docs sizing`
- `removed-docs Hub`

## Skills

- `.agents/skills/theme-config-preferences/SKILL.md`
- `.agents/skills/builtin-filterable-surfaces/SKILL.md`
- `.agents/skills/protocol-automation/SKILL.md`
- `.agents/skills/testing-quality-gates/SKILL.md`

## Packx Command

```bash
packx --limit 90k -l 8 \
  -s "Settings Hub" \
  -s "SettingsAction" \
  -s "settings_visible_rows" \
  -s "ThemeChooserView" \
  -s "execute_theme_chooser_action" \
  -s "ThemeSelectionPreferences" \
  -s "AiPreferences" \
  -s "DictationPreferences" \
  -s "WindowManagementPreferences" \
  -s "theme.json" \
  -s "user themes" \
  -s "get_config_fingerprint" \
  -s "font_size" \
  -s "native footer" \
  -f markdown --no-interactive --stdout \
  AGENTS.md CLAUDE.md .goals/feature_map.md \
  .agents/skills/theme-config-preferences/SKILL.md \
  .agents/skills/builtin-filterable-surfaces/SKILL.md \
  .agents/skills/protocol-automation/SKILL.md \
  .agents/skills/testing-quality-gates/SKILL.md \
  removed-docs removed-docs removed-docs removed-docs removed-docs removed-docs removed-docs \
  src/render_builtins/settings.rs src/render_builtins/theme_chooser.rs src/render_builtins/theme_chooser_customize_controls.rs \
  src/render_builtins/theme_chooser_preview_panel.rs src/render_builtins/theme_chooser_list_header.rs src/render_builtins/theme_chooser_setup_key.rs \
  src/config/mod.rs src/config/loader.rs src/config/defaults.rs src/config/types.rs src/config/editor.rs src/config/command_ids.rs \
  src/theme/mod.rs src/theme/types.rs src/theme/service.rs src/theme/user_themes.rs src/theme/validation.rs src/theme/presets.rs src/theme/audit.rs src/theme/gpui_integration.rs src/theme/chrome.rs src/theme/opacity.rs \
  src/main_sections/fonts.rs src/app_impl/theme_focus.rs src/main_sections/app_view_state.rs src/footer_popup.rs \
  scripts/kit-sdk.ts scripts/config-schema.ts scripts/config-cli.ts scripts/kit-sdk-config.ts \
  tests/settings_surface_contract.rs tests/settings_visible_rows_contract.rs tests/theme_chooser_key_propagation_contract.rs tests/app_view_policy_contract.rs \
  tests/theme_contrast_audit.rs tests/config_contract_alignment.rs tests/get_config_fingerprint_contract.rs tests/config_reload_during_streaming_contract.rs tests/config-schema.test.ts scripts/config-cli.test.ts \
  > ~/.oracle/bundles/settings-theme-config-preferences-atlas.txt
```
