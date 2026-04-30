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

## Related Pages

This page extends the visual contract described in the broader design notes.

- [design](./design.md)
