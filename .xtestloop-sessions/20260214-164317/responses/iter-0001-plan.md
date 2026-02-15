Got the full Oracle response. Here's the swarm task list from GPT-5.2 Pro:

---

## Swarm Task List: `src/theme` Characterization Tests + Observability

### Task 1: worker-snapshots
**Scope:** `src/theme/{mod.rs,tests.rs}` + `tests/theme/snapshots/*.json`
**Task:**
Update `src/theme/mod.rs` to add the approved `src/theme/tests.rs` home without colliding with existing test modules. Rename the current `#[cfg(test)] #[path = "theme_tests/mod.rs"] mod tests;` to `mod legacy_theme_tests;` (same file, just a different module name), and similarly rename `lightweight_colors_test` to `legacy_lightweight_colors_test`. Then add: `#[cfg(test)] #[path = "tests.rs"] mod tests;` so the new approved test file is unambiguous.

Create `src/theme/tests.rs` with snapshot-characterization tests comparing serialized outputs to committed goldens under `tests/theme/snapshots/`. Use `include_str!` for fixtures and `serde_json::to_string_pretty(...)` for actuals. Add these tests:

- `snapshot_theme_dark_default_json()` / `snapshot_theme_light_default_json()` — serialize `Theme::dark_default()`/`light_default()` to JSON, compare against golden files. Locks in all default fields (colors, opacity, shadow, vibrancy, fonts, appearance).
- `snapshot_preset_preview_colors()` — iterate `presets::all_presets()`, snapshot a vector of `{ id, name, is_dark, bg, accent, text, secondary, border }` as `#RRGGBB` strings.
- `snapshot_color_string_parse_matrix()` — snapshot `hex_color::parse_color_string` for a fixed matrix of valid+invalid inputs, capturing exact error messages.

Create 4 fixture files: `tests/theme/snapshots/{theme_dark_default,theme_light_default,preset_preview_colors,color_string_parse_matrix}.json`

```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

---

### Task 2: worker-tracing-load-gpui
**Scope:** `src/theme/{types.rs,gpui_integration.rs}`
**Task:**
Add structured tracing to `types.rs` in `load_theme()` return paths with fields: `source`, `appearance`, `has_dark_colors`, `vibrancy_enabled`, `focus_aware_present`. Tighten error logging with structured fields (e.g., `io_error_kind = ?e.kind()`) and replace `content = %contents` with `content_len = contents.len()` to prevent leakage.

Refactor `log_theme_config()` to stop allocating formatted strings — log raw numeric fields with a tiny `struct Hex(u32)` Display wrapper for readability.

Add structured tracing in `gpui_integration.rs`: `debug!` at entry/exit of `map_scriptkit_to_gpui_theme` with `is_dark`, `vibrancy_enabled`, `opacity_main`, and mapped component fields. Add structured `debug!` alongside existing `crate::logging::log` calls in vibrancy branches.

```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

---

### Task 3: worker-tracing-resolver-semantic
**Scope:** `src/theme/{color_resolver.rs,semantic.rs}`
**Task:**
Add a single structured `debug!` in `ColorResolver::new()` with fields: `variant`, `source`, and key resolved values (background, text_primary, accent, border, success, warning, error). Construct into a local `let resolver = ...;`, log, then return unchanged.

Add minimal tracing to `semantic.rs`: `debug!` in `SemanticColors::dark()`/`light()` with mode and a few representative HSL fields, and in `dimmed()` with scale factors. Keep lightweight — no per-field diffs, no string allocations.

```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

---

**NEXT_AREA:** Burn down remaining production `unwrap/expect` + non-`tracing` logging in `src/theme/` (e.g., `gpui_integration::theme_style`'s `.expect(...)` and `crate::logging::log(&format!(...))`) by converting to fallible paths and emitting structured `error!/warn!` with correlation IDs.
