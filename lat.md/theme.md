# Theme

Theme rules define how shared tokens map to visible hierarchy so interactive surfaces stay consistent across launcher rows, popups, and built-in browsers.

## Row state opacity hierarchy

Focused rows must remain visually stronger than hovered rows so keyboard focus stays obvious even when the pointer is moving through the same list.

Shared `BackgroundOpacity` defaults should keep `hover < selected` in every appearance mode, with dark mode using ghost-tier hover (`0.06`) against a stronger selected state (`0.23`). Light mode keeps row chrome lighter on pale/vibrant surfaces: selected rows default to `0.08`, and hovered rows default to `0.04`.

Theme validation should warn when a theme config sets `hover >= selected`, because equal values collapse hover and focus into the same visual state and make hovered rows compete with the active row.

## Preset contrast guardrail

Script Kit presets must keep text bases readable before semantic opacity tiers are applied; pre-dimming `secondary`, `tertiary`, `muted`, or `dimmed` text slots is forbidden.

[[src/theme/presets.rs#theme_script_kit_dark]] and [[src/theme/presets.rs#theme_script_kit_light]] reuse the readable primary base color and let `BackgroundOpacity` text tiers create hierarchy. [[src/theme/audit.rs#audit_theme_contrast]] samples window, input, chrome, selection, surface, prompt, and status contrast. The `script_kit_dark_preset_passes_all_contrast_checks` and `script_kit_light_preset_passes_all_contrast_checks` tests in [[src/theme/audit.rs]] pin the stock presets so future token edits cannot reintroduce double-dimming.

A companion `double_dim_audit_across_all_presets_is_informational` test in [[src/theme/audit.rs]] walks every preset and prints `[double-dim] <preset>: <slots>` lines for any third-party preset that still ships pre-dimmed text bases. It fails only when a Script Kit preset regresses; third-party retunes are tracked separately in `.goals/third-party-preset-contrast.md` because preset identities need design sign-off before their text tokens are rewritten.

### Third-party preset disposition

Failures originally surfaced by [[src/theme/tests.rs#audit_selection_visibility_across_presets]] and [[src/theme/tests.rs#audit_app_chrome_surface_contrast]] are resolved per preset using one of three paths: **A** normalize automatically, **B** retune the preset hex (requires design sign-off), or **C** accept and document via per-preset opt-out.

[[src/theme/presets.rs#passes_interactive_visibility]] now enforces both `selected` (≥1.10:1) and `hover` (≥1.03:1) visibility, so [[src/theme/presets.rs#normalize_dark_interactive_tokens]] and [[src/theme/presets.rs#normalize_light_interactive_tokens]] nudge `selected_subtle` toward the visibility minimum whenever either threshold would otherwise fail. The shared binary search [[src/theme/presets.rs#find_min_visible_selected_subtle_toward]] blends `selected_subtle` toward white (dark presets) or black (light presets) until both opacities clear their thresholds.

| Preset | Failing surface | Path | Rationale |
| --- | --- | --- | --- |
| one-dark, one-dark-pro, github-dark, ayu-dark, material-ocean, vitesse-dark, darcula, oxocarbon-dark, flexoki-dark, kanagawa-dragon, aura-dark, vesper, midnight-blue, ember, arctic | `hover` visibility (was <1.03:1 at `DARK_ROW_HOVER_OPACITY` = 0.06) | **A** | Dark presets ship a `selected_subtle` tuned only for the stronger `selected` opacity (0.23); at the hover opacity the same color collapses into the bg. Normalize now brightens `selected_subtle` toward white until both thresholds clear. |
| fairy-floss | `primary/selection_surface` (4.06:1, needed 4.5:1) | **C** | `bg.main = #5A5475` (L≈0.10) is unusually bright for a "dark" preset; white text composited at `selected` (0.23) yields `#807B94`, capped at 4.06:1. The pastel identity does not survive a darker bg, so [[src/theme/tests.rs#is_chrome_contrast_exempt]] grants a per-pair opt-out instead of weakening the global 4.5:1 floor. |

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
