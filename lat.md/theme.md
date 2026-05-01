# Theme

Theme rules define how shared tokens map to visible hierarchy so interactive surfaces stay consistent across launcher rows, popups, and built-in browsers.

## Row state opacity hierarchy

Focused rows must remain visually stronger than hovered rows so keyboard focus stays obvious even when the pointer is moving through the same list.

Shared `BackgroundOpacity` defaults should keep `hover < selected` in every appearance mode, with dark mode using ghost-tier hover (`0.06`) against a stronger selected state (`0.23`). Light mode keeps row chrome lighter on pale/vibrant surfaces: selected rows default to `0.08`, and hovered rows default to `0.04`.

Theme validation should warn when a theme config sets `hover >= selected`, because equal values collapse hover and focus into the same visual state and make hovered rows compete with the active row.

## Current sources

This page documents the shared row-state token contract and the guardrails that keep custom themes from erasing it.

- [src/theme/types.rs](../src/theme/types.rs)
- [src/theme/validation.rs](../src/theme/validation.rs)
- [src/theme/gpui_integration.rs](../src/theme/gpui_integration.rs)
- [src/theme/chrome.rs](../src/theme/chrome.rs)
- [src/theme/user_themes.rs](../src/theme/user_themes.rs)
- [src/list_item/mod.rs](../src/list_item/mod.rs)

## User themes directory

User-authored themes live at `~/.scriptkit/themes/<slug>.json`.

The directory is seeded at startup via [[src/setup/mod.rs]] and read through [[src/theme/user_themes.rs#list_user_themes]]. `save_user_theme` slugifies the display name into the file stem, writes the payload atomically (tmp file + rename), and refuses the save if `hover >= selected` so the row-state opacity contract in this page's top sections cannot be broken by a user file.

## Theme chooser key handling

The theme chooser owns its handled key events so selection, undo, and preview shortcuts cannot fall through into the focused main filter.

`[[src/render_builtins/theme_chooser.rs#ScriptListApp#render_theme_chooser]]` calls `cx.stop_propagation()` after handling actions routing, Escape, Cmd+W, preset mutation shortcuts, keyboard preview navigation, preset row clicks, and customizer control clicks. This prevents handled view keys from falling through to parent handlers. `[[tests/theme_chooser_key_propagation_contract.rs#handled_theme_chooser_keys_stop_propagation]]` and `[[tests/theme_chooser_key_propagation_contract.rs#handled_theme_chooser_clicks_stop_propagation]]` pin that handled-input contract.

Theme Chooser has a dedicated `ActionsDialogHost::ThemeChooser` catalog instead of the generic `BuiltinList` fallback. Cmd+K opens theme-specific rows such as Done, Surprise Me, Reset to Defaults, accent cycling, opacity, vibrancy, material, and font-size changes, all dispatched through `[[src/render_builtins/theme_chooser.rs#ScriptListApp#execute_theme_chooser_action]]`.

Theme selection can transiently churn native focus as AppKit updates window appearance, vibrancy, or activation state. `[[src/main_sections/app_view_state.rs#AppView#surface_contract]]` therefore gives `ThemeChooserView` the explicit dismiss policy instead of the standard blur-dismiss policy, so window blur cannot reset to the launcher during a theme click while Escape and Cmd+W remain explicit exits. `[[tests/app_view_policy_contract.rs#theme_chooser_ignores_window_blur_dismissal]]` pins this policy.

The shared input component owns semantic Enter submission for views that use the main filter. `InputEvent::PressEnter` dispatches to ThemeChooser while `ThemeChooserView` is active, so committing a theme and returning to ScriptList cannot reinterpret the same input event as a launcher submit. `[[tests/theme_chooser_key_propagation_contract.rs#theme_chooser_enter_is_owned_by_shared_input_press_enter]]` pins that ownership model.

## Related Pages

This page extends the visual contract described in the broader design notes.

- [design](./design.md)
