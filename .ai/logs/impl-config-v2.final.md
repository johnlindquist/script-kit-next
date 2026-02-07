Implemented the 3 requested configuration improvements with minimal focused changes and verified with `cargo check` + targeted tests.

**Changed Files**
- `src/config/defaults.rs`: added layout defaults `DEFAULT_LAYOUT_STANDARD_HEIGHT` and `DEFAULT_LAYOUT_MAX_HEIGHT`.
- `src/config/types.rs`: added settings types `LayoutConfig`, `ThemeSelectionPreferences`, `ScriptKitUserPreferences`; added `Config.layout` + `get_layout()`.
- `src/config/loader.rs`: added `settings.json` loader (`load_user_preferences`) with resilient parsing/fallback.
- `src/config/mod.rs`: re-exported new settings types and `load_user_preferences`.
- `src/window_resize.rs`: replaced hardcoded runtime heights with settings-driven layout (`load_user_preferences().layout`) plus sanitization.
- `src/app_layout.rs`: replaced hardcoded debug layout window height with `window_resize::initial_window_height()`.
- `src/theme/types.rs`: theme loading now checks settings `theme.presetId` first (preset-based theme selection), then falls back to `theme.json`/system default.
- `src/config/config_tests.rs`: updated config fixture initializations for new optional config fields.
- `src/hotkey_pollers.rs`: includes correlation-id propagation on hotkey events (already in the agent’s claimed scope).

**How to Test**
1. `cargo check`
2. `cargo test window_resize::tests`
3. `cargo test theme_from_user_preferences`
4. `cargo test user_preferences_loader_parses_layout_and_theme_preset`
5. `cargo test user_preferences_loader_recovers_from_invalid_layout_field`

Optional runtime validation:
- Put this in `~/.scriptkit/kit/settings.json`:
```json
{
  "layout": { "standardHeight": 560, "maxHeight": 820 },
  "theme": { "presetId": "nord" }
}
```
- Launch via stdin protocol and inspect behavior/logs.

**Verification Run**
- `cargo check`: passed.
- All targeted tests above: passed.

**Risks / Known Gaps**
- `window_resize` uses a `OnceLock` cache for layout settings, so changing `settings.json` while the app is running requires restart to take effect.
- Unknown `theme.presetId` falls back gracefully (warns, then uses file/default theme).
- Full gate `cargo clippy --all-targets -- -D warnings && cargo test` was not run in this pass; only targeted tests plus `cargo check`.

**Commits**
- No commits were made.