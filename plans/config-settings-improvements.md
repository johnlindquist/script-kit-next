# Configuration & Settings Improvements Audit

Date: 2026-02-07
Agent: `codex-config-settings`
Scope: `src/**/*.rs`, `Cargo.toml`

## Executive Summary

The project already has a solid configuration foundation (`Config` types with defaults, safe config writes in `config::editor`, theme/config watchers, and atomic window-state writes). The biggest gaps are consistency and resiliency:

1. Path resolution is fragmented across modules, so `SK_PATH` support is incomplete in practice.
2. Config parsing is all-or-nothing, so one bad field drops all user settings.
3. Several runtime-critical values are hardcoded (watcher tuning, window sizing/layout, built-in hotkeys behavior).
4. Settings persistence exists for a few flows, but there is no unified settings update path.

## What’s Working Well

- Default-backed config getters in `Config` reduce call-site complexity (`src/config/types.rs:548`).
- `write_config_safely` provides validation, backup, and atomic writes (`src/config/editor.rs:406`).
- Window-state persistence uses atomic write semantics (`src/window_state.rs:227`).

## Findings (Priority Ordered)

### 1) Path Resolution Is Not Centralized (High)

Evidence:
- `setup::get_kit_path()` exists and supports `SK_PATH` (`src/setup.rs:787`).
- Multiple modules still hardcode `~/.scriptkit/...`:
  - Config loader: `src/config/loader.rs:23`
  - Watchers: `src/watcher.rs:126`, `src/watcher.rs:374`, `src/watcher.rs:650`
  - Theme loader: `src/theme/types.rs:1351`
  - Settings/open-config actions: `src/main.rs:3746`, `src/app_impl.rs:4747`, `src/app_actions.rs:1296`
  - Claude code config writes: `src/app_execute.rs:1268`, `src/ai/window.rs:3467`
  - Script creation paths: `src/script_creation.rs:31`, `src/script_creation.rs:34`

Impact:
- `SK_PATH` only works for some flows, causing split-brain behavior (different modules reading/writing different directories).

Recommendation:
- Add a single `settings_paths` module that derives all paths from `setup::get_kit_path()`.
- Migrate all read/write/watch call sites to `settings_paths::*` helpers.
- Keep legacy-path compatibility only inside migration helpers, not business logic.

---

### 2) Config Loading Is All-or-Nothing (High)

Evidence:
- `load_config()` returns `Config::default()` for any parse error (`src/config/loader.rs:89`, `src/config/loader.rs:123`).
- `Config.hotkey` is required and lacks `#[serde(default)]` (`src/config/types.rs:447`), making parse failure easy.
- Error hints are based on string matching on serde error text (`src/config/loader.rs:96`).

Impact:
- A single malformed/new field can silently drop all user custom settings.
- Error recovery is fragile and tied to error text format.

Recommendation:
- Make deserialization resilient:
  - Add `#[serde(default)]` to `hotkey` or deserialize via a compatibility struct with optional fields.
  - Merge parsed values into `Config::default()` (field-level fallback instead of whole-file fallback).
- Replace string-matching hint logic with typed validation errors per field.
- Log and surface which field failed while preserving valid settings.

---

### 3) Window State Persistence Path Is Legacy and Inconsistent (High)

Evidence:
- Stored at `~/.sk/kit/window-state.json` (`src/window_state.rs:190`).
- The rest of the app uses `~/.scriptkit/kit/...` and/or `SK_PATH` conventions.
- Tests encode the same legacy path (`src/window_state_persistence_tests.rs:15`).

Impact:
- Window settings are not colocated with other Script Kit config/theme data.
- `SK_PATH` override does not apply to window-state persistence.

Recommendation:
- Move window state under Script Kit root (for example: `<kit_root>/kit/window-state.json`).
- Implement transparent migration: read legacy file if new file missing, then rewrite to new location.
- Update tests to assert new path behavior and migration behavior.

---

### 4) Built-In Hotkeys Cannot Be Explicitly Disabled (Medium)

Evidence:
- AI and logs hotkeys always default if unset (`src/config/types.rs:617`, `src/config/types.rs:625`).
- Notes hotkey is optional (`src/config/types.rs:611`), showing a different and more flexible model.

Impact:
- Users cannot opt out of built-in AI/log hotkeys without patching source.

Recommendation:
- Support explicit disable semantics:
  - `aiHotkey: null` / `logsHotkey: null` means disabled, or
  - add `enabled` flags per built-in hotkey.
- Preserve current defaults for backwards compatibility when fields are absent.

---

### 5) Watcher Behavior Is Hardcoded Instead of Configurable (Medium)

Evidence:
- Debounce/storm/backoff/error thresholds are constants (`src/watcher.rs:21` to `src/watcher.rs:29`).

Impact:
- No tuning path for slower disks/network-mounted kits/very large script collections.

Recommendation:
- Introduce `watcher` config section in `config.ts`:
  - `debounceMs`, `stormThreshold`, `initialBackoffMs`, `maxBackoffMs`, `maxNotifyErrors`.
- Keep current constants as defaults through config getters.

---

### 6) Core Window/Layout Dimensions Are Hardcoded and Duplicated (Medium)

Evidence:
- Window heights are fixed in code (`src/window_resize.rs:47`, `src/window_resize.rs:50`).
- Header/input constants are fixed in panel module (`src/panel.rs:54` to `src/panel.rs:77`, `src/panel.rs:92`).
- Debug layout code duplicates constants and hardcodes default window size (`src/app_layout.rs:25` to `src/app_layout.rs:29`, `src/app_layout.rs:502`, `src/app_layout.rs:503`, `src/app_layout.rs:534` to `src/app_layout.rs:538`).

Impact:
- Users cannot tune launcher density/height behavior.
- Duplicate constants increase drift risk between runtime layout and debug-layout introspection.

Recommendation:
- Add `layout` settings in config (at minimum: `standardHeight`, `maxHeight`, header/input sizes).
- Use one shared source of truth for both runtime and debug layout calculations.

---

### 7) Safe Config Writer Exists but Is Not a General Settings Persistence API (Medium)

Evidence:
- Robust writer exists (`src/config/editor.rs:406`).
- It is currently used mainly for Claude Code toggles (`src/app_execute.rs:1271`, `src/ai/window.rs:3471`).

Impact:
- Runtime settings changes outside this path can diverge in safety/validation guarantees.
- No generic way to persist UI-driven settings changes with schema-aware updates.

Recommendation:
- Build a small `ConfigUpdateService` on top of `write_config_safely`:
  - typed patch operations,
  - centralized validation,
  - backup/recovery path reuse,
  - consistent logging/telemetry.

---

### 8) Cargo Comment Drift (Low)

Evidence:
- `Cargo.toml` build-dependency comment says SDK copy is for `~/.kit/lib/` (`Cargo.toml:173`), but repository docs/code use `~/.scriptkit/sdk/`.

Impact:
- Minor contributor confusion.

Recommendation:
- Update comment to current SDK deployment path.

## Suggested Rollout Plan

### Phase 1 (Safety + Consistency)
1. Centralize path helpers and migrate config/theme/window-state read/write paths.
2. Implement window-state path migration with tests.
3. Make config loading resilient (field-level fallback) with targeted parser tests.

### Phase 2 (User-Controlled Behavior)
1. Add optional disable support for AI/log hotkeys.
2. Add watcher tuning settings + defaults.
3. Add core layout/window size settings (start with heights only).

### Phase 3 (Persistence Platform)
1. Introduce `ConfigUpdateService` using existing safe writer primitives.
2. Route all runtime settings writes through that service.
3. Add structured setting-change logs (old/new value, source action, correlation id).

## Recommended Tests to Add with Implementation

- `config_loader_preserves_valid_fields_when_one_field_invalid`
- `config_loader_uses_default_hotkey_when_hotkey_missing_or_invalid`
- `window_state_migrates_from_legacy_sk_path_to_scriptkit_path`
- `watcher_uses_configured_debounce_and_backoff_values`
- `hotkeys_do_not_register_ai_or_logs_when_disabled`
- `layout_uses_configured_standard_and_max_height`

## Risk Notes

- Path migration changes can break existing setups if legacy fallback is not implemented.
- Relaxing config parsing should not hide severe config corruption; logging and user-facing diagnostics are required.
- Making hotkeys optional must avoid orphaned registrations during runtime reloads.
