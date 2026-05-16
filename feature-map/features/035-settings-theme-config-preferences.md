# 035 Settings, Theme, Config, and Preferences

This chapter maps Script Kit GPUI's Settings Hub, Theme Chooser, theme storage, config surfaces, and runtime preference boundaries.

Raw Oracle reference: [answer](../raw-oracle/035-settings-theme-config-preferences/answer.md), [prompt](../raw-oracle/035-settings-theme-config-preferences/prompt.md), [bundle map](../raw-oracle/035-settings-theme-config-preferences/bundle-map.md), [full log](../raw-oracle/035-settings-theme-config-preferences/output.log), [session metadata](../raw-oracle/035-settings-theme-config-preferences/session.json).

## Executive Summary

This feature has three related but separate surfaces:

- Settings Hub: a mini built-in list from `src/render_builtins/settings.rs` for operational rows such as Theme Designer, Dictation Setup, Select Microphone, permission helpers, suggested-item reset, snap-mode controls, and window-position reset.
- Theme Chooser: a dedicated `ThemeChooserView` from `src/render_builtins/theme_chooser.rs` for preset search, live preview, customization controls, theme-specific actions, native footer ownership, and explicit exit behavior.
- Config/theme storage: `~/.scriptkit/config.ts` stores operational preferences; `~/.scriptkit/theme.json` stores the active theme payload; `~/.scriptkit/themes/<slug>.json` stores user-authored saved themes.

The biggest regression risks are Settings render/state/element row drift, Theme Chooser handled inputs leaking back into the launcher, double footer rendering, config TypeScript/Rust drift, theme contrast regressions, stale config-cache behavior, and confusing preset selection with the full theme payload.

## What Users Can Do

| Capability | Entry | Result |
|---|---|---|
| Open Theme Designer | Settings Hub row | Enters `ThemeChooserView` with native `theme_chooser` footer. |
| Filter Settings rows | Settings Hub input | Rendered rows, `getState`, and `getElements` use the same visible-row projection. |
| Run Dictation Setup | Settings Hub row | Executes `builtin/dictation-setup`. |
| Select Microphone | Settings Hub row | Executes `builtin/select-microphone`; durable preference is `dictation.selectedDeviceId`. |
| Clear suggested history | Settings Hub row | Executes `builtin/clear-suggested`. |
| Open permission helpers | Settings Hub rows | Runs Check Permissions, Allow Accessibility, Allow Screen Recording, Request Accessibility, or Open Accessibility Settings built-ins. |
| Change snap mode | Conditional Settings rows | Executes snap-mode built-ins when available. |
| Reset window positions | Conditional Settings row | Calls the reset-window-position flow when custom positions exist. |
| Search and preview themes | Theme Chooser | Filters presets/user themes and live-previews selected theme. |
| Commit a theme | Theme Chooser Enter or Done action | Applies theme without reusing the same Enter as launcher submit. |
| Customize theme | Theme Chooser controls/actions | Changes accent, opacity, vibrancy/material, and UI font size in the theme payload. |
| Persist config | `config.ts` / config CLI / SDK types | Stores hotkeys, built-in enablement, layout, theme preset, dictation, AI, and window preferences. |
| Verify config reload | `getConfigFingerprint` | Reads config file metadata, not file contents. |

Do not claim every TypeScript schema field is runtime-wired. A field can exist in SDK/config CLI types, be parsed by Rust, have a getter, and still lack a visible application site.

## Core Concepts

Settings Hub is a filterable built-in list. `SettingsItem` holds row name, description, icon, and action. `SettingsAction` enumerates rows such as `ChooseTheme`, `DictationSetup`, `SelectMicrophone`, `ClearSuggested`, permission actions, snap-mode actions, and `ResetWindowPositions`.

The visible-row helpers are the contract:

- `settings_visible_row_names`
- `settings_selected_visible_row`
- `settings_selected_visible_row_name`

Render, selection, `getState`, and `getElements` should all read through the same projection. Tests should not index the unfiltered backing item list.

Theme Chooser is a dedicated view, not a Settings row renderer. It is represented by:

```rust
AppView::ThemeChooserView { filter, selected_index }
```

It owns preset filtering, keyboard navigation, click selection, live preview, persistent apply, customization controls, `ActionsDialogHost::ThemeChooser`, native footer surface `theme_chooser`, and explicit dismissal policy.

Theme storage has three layers:

| Storage | Owns | Notes |
|---|---|---|
| `~/.scriptkit/config.ts` | Operational preferences | Includes `theme.presetId`, dictation, AI, layout, command shortcuts, built-in config, and window-management preferences. |
| `~/.scriptkit/theme.json` | Active theme payload | Stores colors, fonts, opacity, vibrancy/material, and overrides written by theme code. |
| `~/.scriptkit/themes/<slug>.json` | Saved user themes | Listed and saved by `src/theme/user_themes.rs`; writes are atomic and validated. |

`config.ts.theme.presetId` selects a preset. It is not the same thing as the current `theme.json` color/font/vibrancy payload.

## Entry Points

| Entry | Owner | Behavior |
|---|---|---|
| Settings built-in | `src/render_builtins/settings.rs` | Enters `AppView::SettingsView { filter, selected_index }`. |
| Settings surface identity | `src/main_sections/app_view_state.rs` | Maps Settings to `SurfaceKind::Settings` and native footer `settings`. |
| Theme Designer row | `SettingsAction::ChooseTheme` | Calls `open_theme_chooser_view(cx)`. |
| Theme Chooser view | `src/render_builtins/theme_chooser.rs` | Dedicated theme surface with preview, apply, actions, and customization. |
| Theme Chooser footer | `AppView::native_footer_surface` | Publishes native footer surface `theme_chooser`. |
| Config loader | `src/config/loader.rs` | Loads/parses/cache-checks `config.ts`. |
| Config types/defaults | `src/config/types.rs`, `src/config/defaults.rs` | Runtime config model and getter defaults. |
| Config editor | `src/config/editor.rs` | Config write/edit API, re-exported by `src/config/mod.rs`. |
| Config CLI | `scripts/config-cli.ts` | Agent/user CLI for reading, writing, validating, and resetting config values. |
| SDK config types | `scripts/kit-sdk.ts`, `scripts/kit-sdk-config.ts` | TypeScript-facing config shape. |
| Theme service | `src/theme/service.rs`, `src/theme/gpui_integration.rs` | Loads/syncs theme into GPUI and gpui-component theme state. |
| User themes | `src/theme/user_themes.rs` | Lists and saves `~/.scriptkit/themes/<slug>.json`. |
| Config fingerprint | stdin protocol / `lat.md/protocol.md` | Returns metadata for config reload proof. |

## User Workflows

### Open Theme Designer From Settings

1. User opens Settings.
2. Settings builds rows from `get_settings_items()`.
3. User activates **Theme Designer**.
4. `execute_settings_action(SettingsAction::ChooseTheme, ...)` logs `settings.action_executed`.
5. `open_theme_chooser_view(cx)` switches to `ThemeChooserView`.
6. Theme Chooser owns native footer `theme_chooser` and explicit dismissal.

This flow is Settings-owned until the view transition. Once Theme Chooser is active, keys, clicks, footer, actions, and blur policy belong to Theme Chooser.

### Filter Settings Rows

User types into the Settings filter. Settings applies the filter to the visible-row helpers, and automation should see the same rows the UI renders. If the filter matches no rows, automation should expect an empty visible row list and no selected row; the exact empty-copy string is not pinned by the Oracle bundle.

### Run Operational Settings

Settings actions often create a `BuiltInEntry` and call `execute_builtin(&entry, cx)`. Examples:

| Row | Canonical id | Command type |
|---|---|---|
| Dictation Setup | `builtin/dictation-setup` | `SettingsCommandType::DictationSetup` |
| Select Microphone | `builtin/select-microphone` | `SettingsCommandType::SelectMicrophone` |
| Clear Suggested Items | `builtin/clear-suggested` | `FrecencyCommandType::ClearSuggested` |
| Check Permissions | `builtin/check-permissions` | `PermissionCommandType::CheckPermissions` |
| Allow Accessibility | `builtin/allow-accessibility` | `PermissionCommandType::AllowAccessibility` |
| Allow Screen Recording | `builtin/allow-screen-recording` | `PermissionCommandType::AllowScreenRecording` |
| Request Accessibility Permission | `builtin/request-accessibility` | `PermissionCommandType::RequestAccessibility` |
| Open Accessibility Settings | `builtin/accessibility-settings` | `PermissionCommandType::OpenAccessibilitySettings` |

### Change Snap Mode

Settings checks `crate::window_control::current_snap_mode()` and conditionally appends rows. It hides the row for the current mode and shows other possible modes. Activation searches configured built-ins and executes:

- `builtin/disable-window-snapping`
- `builtin/snap-mode-simple`
- `builtin/snap-mode-expanded`
- `builtin/snap-mode-precision`

If a configured built-in is unavailable, Settings shows an unavailable toast such as `Snap Mode: Simple is unavailable`.

### Reset Window Positions

Settings appends **Reset Window Positions** only when `crate::window_state::has_custom_positions()` is true. Activation calls `reset_window_positions_to_default_main_menu(cx)`.

### Search And Preview Themes

Theme Chooser uses `ThemeChooserView { filter, selected_index }`. `theme_chooser_filtered_indices` filters the preset/user-theme list, and `THEME_LIST_PAGE_SIZE` is 5 in the Oracle bundle. Keyboard or click selection previews the theme, syncing GPUI component colors/font state and native vibrancy/material through `sync_theme_chooser_preview` and gpui integration helpers.

### Commit Or Exit Theme Chooser

Theme Chooser owns Enter through the shared input `InputEvent::PressEnter` branch. That branch submits through `submit_theme_chooser_from_input_enter(window, cx)` and returns so the same Enter event cannot commit a theme, transition to ScriptList, and then launch a script.

Escape and Cmd+W are explicit exits. Theme Chooser uses an explicit dismiss policy because theme changes can temporarily churn native focus/blur while AppKit updates appearance or vibrancy.

### Use Theme Chooser Actions

Cmd+K opens a dedicated `ActionsDialogHost::ThemeChooser` catalog rather than the generic built-in list fallback. Known action ids from the bundle include:

- `theme_chooser_done`
- `theme_chooser_undo_close`
- `theme_chooser_opacity_decrease`
- `theme_chooser_opacity_increase`
- `theme_chooser_vibrancy_toggle`
- `theme_chooser_material_cycle`
- `theme_chooser_font_size_decrease`
- `theme_chooser_font_size_increase`

The full action list should be verified from `execute_theme_chooser_action` before adding protocol tests for every action string.

## Interaction Matrix

| User intent | Entry point | UI state | Key/click | Code path | Result | Proof |
|---|---|---|---|---|---|---|
| Open Settings | Settings built-in | Launcher/main window | Trigger builtin | `AppView::SettingsView` | Mini settings list with `settings` native footer. | `tests/settings_surface_contract.rs` |
| Filter Settings | Settings input | Settings visible | Type text | `settings_visible_row_names` | UI/state/elements agree on visible rows. | `tests/settings_visible_rows_contract.rs` |
| Open Theme Designer | Settings row | Settings selected row | Enter/click | `SettingsAction::ChooseTheme -> open_theme_chooser_view` | Theme Chooser opens. | `src/render_builtins/settings.rs`, `lat.md/builtins.md` |
| Run dictation setup | Settings row | Settings selected row | Enter/click | `builtin/dictation-setup -> execute_builtin` | Dictation setup owns next view. | `src/render_builtins/settings.rs` |
| Select microphone | Settings row | Settings selected row | Enter/click | `builtin/select-microphone -> execute_builtin` | Microphone picker/setup flow owns next view. | `src/config/types.rs#DictationPreferences` |
| Change snap mode | Conditional Settings row | Settings selected row | Enter/click | Find configured snap builtin -> execute | Snap preference/action flow runs or unavailable toast appears. | `src/render_builtins/settings.rs` |
| Reset window positions | Conditional Settings row | Settings selected row | Enter/click | `reset_window_positions_to_default_main_menu` | Saved positions reset. | `src/render_builtins/settings.rs` |
| Filter themes | Theme Chooser input | ThemeChooserView | Type text | `theme_chooser_filtered_indices` | Preset/user-theme list narrows. | `src/render_builtins/theme_chooser.rs` |
| Preview theme | Theme row | ThemeChooserView | Arrow/click | `sync_theme_chooser_preview` | GPUI theme and native vibrancy/material preview update. | `src/theme/gpui_integration.rs` |
| Commit theme | Theme Chooser | ThemeChooserView | Enter / Done | `submit_theme_chooser_from_input_enter` or `theme_chooser_done` | Theme applies; Enter does not leak to launcher. | `tests/theme_chooser_key_propagation_contract.rs` |
| Exit theme chooser | Theme Chooser | ThemeChooserView | Escape/Cmd+W | Theme chooser handled key path | Explicit exit without blur-dismiss regression. | `tests/app_view_policy_contract.rs` |
| Open theme actions | Theme Chooser | ThemeChooserView | Cmd+K | `ActionsDialogHost::ThemeChooser` | Theme-specific actions appear. | `lat.md/theme.md` |
| Prove config changed | Protocol | Automation | `getConfigFingerprint` | Config fingerprint handler | Metadata changes without exposing config contents. | `tests/get_config_fingerprint_contract.rs` |

## State Machine

Settings to Theme Chooser:

```text
Launcher/Settings trigger
  -> AppView::SettingsView { filter, selected_index }
  -> user filters/selects Theme Designer
  -> SettingsAction::ChooseTheme
  -> open_theme_chooser_view
  -> AppView::ThemeChooserView { filter, selected_index }
  -> native footer surface theme_chooser
```

Theme preview/apply:

```text
ThemeChooserView
  -> filter preset list
  -> select preset/user theme
  -> sync preview into GPUI + native appearance state
  -> user customizes controls/actions
  -> save/apply theme payload
  -> Enter/Done exits explicitly
```

Config read proof:

```text
caller requests getConfigFingerprint
  -> inspect config file metadata
  -> return path/len/modified/status/hash fields
  -> no config content returned
```

## Visual And Focus States

Settings is a mini built-in list with a native footer surface named `settings`. It should not render as generic `scriptList` in automation. Its rows are filterable, and selected row state follows the filtered projection.

Theme Chooser is a theme-design surface with native footer surface `theme_chooser`. It may change native appearance, vibrancy, material, and font sizing during preview. It uses explicit dismissal so a transient blur during those native updates does not close the view.

Rem sizing follows the gpui-component `Root` wrapper and the current theme font size. Theme UI font changes must reach `GpuiTheme::global_mut(cx).font_size`; otherwise controls can display a new size while rem-based components remain stale.

## Keystrokes And Commands

| Gesture | Surface | Behavior |
|---|---|---|
| Type | Settings | Filters Settings rows through visible-row helpers. |
| Enter/click row | Settings | Activates the selected visible row. |
| Cmd+K | Settings | Opens Settings actions popup routing. |
| Type | Theme Chooser | Filters theme presets/user themes. |
| Up/Down/Page keys | Theme Chooser | Navigate theme list; handled keys stop propagation even on empty filter results. |
| Click preset | Theme Chooser | Selects/previews preset and stops propagation. |
| Enter | Theme Chooser | Commits/applies theme through Theme Chooser branch, not launcher submit. |
| Escape/Cmd+W | Theme Chooser | Explicitly exits/undo-closes Theme Chooser. |
| Cmd+K | Theme Chooser | Opens theme-specific actions catalog. |
| Cmd+J | Theme Chooser | Remix/surprise behavior, per Oracle answer; verify exact action id before testing. |
| Cmd+R | Theme Chooser | Reset customizations to preset defaults, per Oracle answer; verify exact action id before testing. |

## Data, Storage, And Privacy Boundaries

`config.ts` is executable TypeScript-shaped configuration. Agents should use supported config writers/CLI paths and preserve formatting where possible. Do not parse it with casual regex when the config CLI or Rust config editor can express the operation.

`getConfigFingerprint` intentionally returns metadata, not the contents of `config.ts`. It is useful for proving reload/read visibility without leaking user config values.

Theme payloads are local visual preferences. User-authored theme files live under `~/.scriptkit/themes/` and should be validated before save. The row-state opacity guardrail rejects `hover >= selected` because that would make hover compete with keyboard-selected focus.

Config and theme boundaries:

- `config.ts.theme.presetId` selects a preset.
- `theme.json` stores active theme colors/fonts/vibrancy/material.
- user theme files store named theme payloads.
- SDK/config CLI types are not proof that a field is runtime-applied.

## Error, Empty, Loading, And Disabled States

Settings empty filter state should expose zero visible rows and no selected row. Do not rely on a specific empty-state string unless source/runtime proof pins it.

Settings snap rows are conditional. Tests should not assume a fixed row count because current snap mode and custom window-position state alter available rows.

Theme Chooser empty filter state still handles navigation keys and stops propagation so keys do not leak to the parent launcher.

Unknown Theme Chooser action ids log a warning rather than defining a crash path in the Oracle bundle.

User theme save validation can fail if the payload breaks row opacity hierarchy. Treat that as a product guardrail, not an incidental validation error.

Missing config files produce structured `getConfigFingerprint` failure with `config_file_missing`.

Absent optional config groups use defaults, including theme, dictation, AI, window management, editor font size, and terminal font size.

## Code Ownership

| Area | Primary files |
|---|---|
| Settings Hub | `src/render_builtins/settings.rs`, `tests/settings_surface_contract.rs`, `tests/settings_visible_rows_contract.rs`, `lat.md/builtins.md` |
| Theme Chooser | `src/render_builtins/theme_chooser.rs`, `src/render_builtins/theme_chooser_*`, `src/app_impl/theme_focus.rs`, `tests/theme_chooser_key_propagation_contract.rs`, `tests/app_view_policy_contract.rs` |
| Theme system | `src/theme/mod.rs`, `src/theme/types.rs`, `src/theme/service.rs`, `src/theme/user_themes.rs`, `src/theme/validation.rs`, `src/theme/presets.rs`, `src/theme/audit.rs`, `src/theme/gpui_integration.rs`, `src/theme/chrome.rs`, `src/theme/opacity.rs` |
| Config runtime | `src/config/mod.rs`, `src/config/loader.rs`, `src/config/defaults.rs`, `src/config/types.rs`, `src/config/editor.rs`, `src/config/command_ids.rs` |
| Config TS/CLI | `scripts/kit-sdk.ts`, `scripts/config-cli.ts`, `scripts/config-schema.ts`, `scripts/kit-sdk-config.ts`, `scripts/config-cli.test.ts` |
| Protocol proof | `tests/get_config_fingerprint_contract.rs`, `lat.md/protocol.md` |
| Visual/footer contracts | `src/main_sections/app_view_state.rs`, `src/footer_popup.rs`, `lat.md/design.md`, `lat.md/windowing.md` |

## Invariants And Regression Risks

- Settings render, `getState`, and `getElements` must share filtered rows.
- Settings must report `SurfaceKind::Settings` and automation semantic surface `settings`.
- Theme Chooser handled keys and clicks must call `cx.stop_propagation()`.
- Theme Chooser Enter is owned by shared input while `ThemeChooserView` is active.
- Theme Chooser must not blur-dismiss during native focus churn.
- Settings and Theme Chooser must each own exactly one native footer path, with fallback only when the native host is unavailable.
- Preset id, active theme payload, and user theme file are different storage concepts.
- User-authored themes must preserve `hover < selected` opacity hierarchy.
- Rust config types, loader parsing, SDK types, and config CLI schema should remain aligned.
- `getConfigFingerprint` stays read-only and request-id-only.
- Theme font-size sync must reach gpui-component theme state.
- Light-mode vibrancy should remain readable; Oracle notes a minimum opacity clamp in integration code, but exact thresholds require source inspection before editing.

## Verification Recipes

Source checks:

```bash
cargo test --test settings_surface_contract
cargo test --test settings_visible_rows_contract
cargo test --test theme_chooser_key_propagation_contract
cargo test --test app_view_policy_contract
cargo test --test theme_contrast_audit
cargo test --test config_contract_alignment
cargo test --test get_config_fingerprint_contract
cargo test --test config_reload_during_streaming_contract
bun test scripts/config-cli.test.ts
lat check
```

Use the repo's actual Bun test convention if `bun test scripts/config-cli.test.ts` is not the local standard.

Runtime Settings proof:

1. Open Settings.
2. Verify active semantic surface is `settings`.
3. Verify native footer has one owner row.
4. Type `theme` and assert visible rows and selected row in `getState`/`getElements`.
5. Activate Theme Designer and verify `ThemeChooserView`.
6. Return to Settings, type `snap`, and confirm conditional rows match current snap mode.

Runtime Theme Chooser proof:

1. Open Theme Chooser.
2. Type a filter and verify visible presets/user themes.
3. Navigate with keyboard and click a preset.
4. Use Cmd+K and run a Theme Chooser action.
5. Change UI font size and verify rem-scaled UI responds.
6. Toggle vibrancy/material and confirm no blur-dismiss.
7. Press Enter and confirm no launcher submit happens from the same event.
8. Reopen and confirm Escape/Cmd+W exits explicitly.

Config fingerprint proof:

1. Request `getConfigFingerprint`.
2. Modify `~/.scriptkit/config.ts` through a supported writer or CLI.
3. Request `getConfigFingerprint` again.
4. Assert `len`, `modified_ms`, or hash metadata changed.
5. In isolated tests, missing config should return `config_file_missing`.

## Agent Notes

Start from `.agents/skills/theme-config-preferences/SKILL.md` for this feature. Use `lat.md/theme.md`, `lat.md/builtins.md`, `lat.md/workspace.md`, `lat.md/protocol.md`, `lat.md/design.md`, and `lat.md/windowing.md` before editing behavior.

Do not treat Settings Hub and Theme Chooser as one surface. Settings launches Theme Chooser; Theme Chooser owns its own state, footer, action host, key handling, and dismissal rules.

Do not claim a config field is runtime-wired just because it appears in SDK or CLI types. Verify Rust parsing, getter/default behavior, and application site.

Do not claim theme customization writes `config.ts` unless the code path does so. Theme color/font/vibrancy payloads belong to `theme.json` or user theme files.

Do not add Settings rows without updating visible-row contracts and automation expectations.

Do not add Theme Chooser controls without auditing key/click propagation tests.

Do not change `getConfigFingerprint` casually. It is intentionally metadata-only and read-only.

Use screenshots only for visual footer/chrome/vibrancy/font regressions that state receipts cannot prove.

## Related Features

- `launcher-surface-contracts`: `AppView`, `SurfaceKind`, transitions, native footer surfaces.
- `builtin-filterable-surfaces`: mini built-in list visible-row behavior.
- `keyboard-focus-routing`: handled key propagation and focus restoration.
- `actions-popups`: Cmd+K action host behavior.
- `gpui-ui-foundation`: GPUI layout, focus, theme use, component lifecycle.
- `window-resizing` / `platform-windowing-macos`: vibrancy, native window behavior, rem sizing, footer host.
- `protocol-automation`: `getState`, `getElements`, and config fingerprint proof.
- `testing-quality-gates`: choosing narrow source/runtime checks.

## Open Questions And Gaps

- Exact Settings key map is not fully pinned by the Oracle bundle. Verify the full key router before documenting every shortcut.
- The complete Theme Chooser action id list should be read from `execute_theme_chooser_action` before protocol fixtures assert every id.
- Some split Theme Chooser files may be legacy or partial; confirm which are compiled before editing them.
- `uiScale` is parsed and test-visible in the bundle, but runtime UI-scale application is not established here.
- Config CLI includes fields beyond the visible Rust snippets; verify full Rust config before calling any field schema-only.
- Exact theme contrast thresholds require direct inspection of `src/theme/audit.rs` and `tests/theme_contrast_audit.rs`.
- Settings back-stack/exit behavior likely follows shared built-in navigation, but the exact return target is outside this focused bundle.
