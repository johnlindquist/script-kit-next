# Filesystem Crate Audit

Date: 2026-02-07  
Scope: `notify` (8.0), `dirs` (5.0), `shellexpand` (2.1), `glob` (0.3.3), `filetime` (0.2), `tempfile` (3), `which` (8.0)

## Executive Summary

- `notify`: Uses `recommended_watcher` everywhere (`src/watcher.rs:247`, `src/watcher.rs:489`, `src/watcher.rs:867`, `src/watcher.rs:1491`). With current dependency config, macOS backend is **FSEvents**, not kqueue.
- `dirs`: Used only via `dirs::home_dir()`; platform-specific directory APIs like `dirs::config_dir()`/`dirs::cache_dir()` are not used.
- `shellexpand`: Mostly tilde-only expansion. I added env+tilde expansion for `SK_PATH` in `get_kit_path()` (`src/setup.rs:790`) with a new test (`src/setup.rs:2128`).
- `glob`: Used correctly for kit/script discovery, with warning logs on pattern errors. Two callsites drop per-entry glob errors silently.
- `filetime`: Used narrowly and safely to sync icon cache mtime.
- `tempfile`: Mostly RAII-safe usage; one helper intentionally persists temp files (`.keep()`), and one separate flow uses manual temp paths without cleanup.
- `which`: Used for CLI presence checks (`mdflow`, backend CLIs); behavior is straightforward and safe.

## Dependency Presence

- `Cargo.toml:29` → `notify = "8.0"`
- `Cargo.toml:38` → `dirs = "5.0"`
- `Cargo.toml:33` → `shellexpand = "2.1"`
- `Cargo.toml:89` → `glob = "0.3.3"`
- `Cargo.toml:88` → `filetime = "0.2"`
- `Cargo.toml:135` → `tempfile = "3"`
- `Cargo.toml:131` → `which = "8.0.0"`

## 1) `notify` audit

### Current configuration

- Watchers instantiate via `recommended_watcher(...)` in all watcher loops:
  - `src/watcher.rs:247`
  - `src/watcher.rs:489`
  - `src/watcher.rs:867`
  - `src/watcher.rs:1491`
- Cargo feature graph (`cargo tree -e features -i notify@8.2.0`) shows `notify` default features only, including `macos_fsevent`, not `macos_kqueue`.
- `notify` source confirms backend mapping:
  - On macOS without `macos_kqueue`: `RecommendedWatcher = FsEventWatcher`
  - With `macos_kqueue`: `RecommendedWatcher = KqueueWatcher`

### Answer: “Is notify configured with optimal backend (kqueue on macOS)?”

- **No, it is not configured for kqueue.** It is configured for macOS FSEvents (default).
- If the requirement is specifically “use kqueue on macOS,” current config does not satisfy it.

### Recommendation

- If you want kqueue explicitly, switch to:
  - `notify = { version = "8.0", default-features = false, features = ["macos_kqueue"] }`
- Otherwise, keep default FSEvents (current behavior), which is the crate’s default macOS path.

## 2) `dirs` audit

### Current usage

- `dirs::home_dir()` is used across setup/path derivation (`src/setup.rs:797`, `src/setup.rs:814`, `src/process_manager.rs:53`, `src/logging.rs:1007`, etc.).
- No usage found for platform directories:
  - `dirs::config_dir()`
  - `dirs::cache_dir()`
  - `dirs::data_dir()`
  - `dirs::state_dir()`

### Answer: “Is dirs used for all platform paths?”

- **No.** The codebase relies heavily on hardcoded `~/.scriptkit/...` plus `shellexpand::tilde(...)` (for example `src/app_launcher.rs:514`, `src/watcher.rs:162`) rather than `dirs` platform APIs.

### Recommendation

- Introduce a centralized path service (or reuse `get_kit_path()` everywhere) for all Script Kit paths.
- Consider `dirs` platform APIs for non-`~/.scriptkit` caches/logs when true OS-native locations are desired.

## 3) `shellexpand` audit

### Current usage

- Most callsites use `shellexpand::tilde(...)` only (e.g. `src/stdin_commands.rs:183`, `src/main.rs:711`, `src/watcher.rs:162`).
- Before this change, `SK_PATH` was tilde-only.

### Code change made

- Updated `get_kit_path()` to prefer full expansion (tilde + env vars):
  - `src/setup.rs:790` now uses `shellexpand::full(&sk_path)`, with `tilde(...)` fallback.
- Added regression test:
  - `src/setup.rs:2128` `test_get_kit_path_with_env_var_expansion`.

### Answer: “Is shellexpand handling all tilde/env expansions?”

- **Partially.** `SK_PATH` now handles env+tilde expansion; most other callsites are still tilde-only.

### Recommendation

- For user-provided path inputs (stdin commands, config-defined paths), prefer `shellexpand::full(...)` where env var support is expected.

## 4) `glob` audit

### Current usage

- Script loaders:
  - `src/scripts/loader.rs:34`
  - `src/scripts/scheduling.rs:33`
  - `src/scripts/scriptlet_loader.rs:308`
  - `src/agents/loader.rs:53`
- Pattern errors are logged via `warn!` at each callsite.

### Notes

- `src/scripts/loader.rs:35` and `src/scripts/scheduling.rs:34` use `filter_map(|p| p.ok())`, which drops entry-level errors silently.
- `src/scripts/scriptlet_loader.rs:364` and `src/agents/loader.rs:77` log entry errors explicitly.

### Verdict

- `glob` usage is generally sound, with a minor observability gap on silently dropped entries in two modules.

## 5) `filetime` audit

### Current usage

- App icon cache mtime sync:
  - `src/app_launcher.rs:583`
  - `src/app_launcher.rs:584`

### Verdict

- Narrow and appropriate use; error path is logged and non-fatal.

## 6) `tempfile` audit

### Safe RAII usages

- Config transpilation temp JS: `NamedTempFile` without `.keep()` (`src/config/loader.rs:161`).
- TS validation temp file: `tempfile::Builder::tempfile()` without `.keep()` (`src/config/editor.rs:328`).
- Tests widely use `TempDir`/`NamedTempFile` RAII.

### Cleanup risk areas

- Clipboard temp helper persists files intentionally using `.keep()`:
  - `src/clipboard_history/temp_file.rs:31`
  - `src/clipboard_history/temp_file.rs:53`
- A separate CleanShot flow uses manual `std::env::temp_dir()` file creation (`src/app_actions.rs:1619`) without explicit cleanup after launching `open`.

### Answer: “Are temp files cleaned up?”

- **Mostly yes** for `tempfile` RAII callsites.
- **Not always** for persisted temp artifacts (`.keep()`) and manual temp file writes outside `tempfile`.

### Recommendation

- Define explicit retention/cleanup policy for clipboard-exported temp files.
- Track and prune stale `script-kit-clipboard-*` files on startup or periodic maintenance.

## 7) `which` audit

### Current usage

- `src/agents/executor.rs:31`, `src/agents/executor.rs:36`, `src/agents/executor.rs:38`
- `src/agents/types.rs:192`

### Verdict

- Usage is correct for command-availability checks and backend gating.
- Minor optimization opportunity: avoid duplicate PATH scans by caching command resolution per request/session.

## Direct Answers to Assignment Questions

- Is notify file watcher configured with optimal backend (kqueue on macOS)?
  - **No**. Current config resolves to **FSEvents** on macOS.

- Is dirs used for all platform paths?
  - **No**. Mostly `home_dir()` plus hardcoded `~/.scriptkit` paths.

- Is shellexpand handling all tilde/env expansions?
  - **Partially**. `SK_PATH` now handles env+tilde; many other callsites remain tilde-only.

- Are temp files cleaned up?
  - **Mostly**, via RAII. Exceptions exist where files are intentionally persisted or manually written.
