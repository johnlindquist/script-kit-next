# Anyhow + Thiserror Audit

## Scope
- Repository: `script-kit-gpui`
- Audit date: 2026-02-07
- Requested crates: `anyhow` (`1.0`), `thiserror` (`2.0`)
- Files reviewed: `Cargo.toml` and targeted modules under `src/**/*.rs` related to error types, propagation, and swallowing patterns.

## Dependency Baseline
- Declared:
  - `anyhow = "1.0"` (`Cargo.toml:41`)
  - `thiserror = "2.0"` (`Cargo.toml:42`)

## Direct Answers

### 1) Is `anyhow` used appropriately (application-level) vs `thiserror` (library/domain-level)?
**Mostly yes, with a few boundary leaks.**

What is good:
- `anyhow` is used broadly at app/integration boundaries with contextual propagation (`.context`/`.with_context`) in modules like frecency, alias/shortcut persistence wrappers, and MCP logging (`src/frecency.rs:255`, `src/frecency.rs:300`, `src/aliases/persistence.rs:89`, `src/mcp_streaming.rs:175`).
- `thiserror` is used for domain-specific typed errors where variant matching/user messaging matters:
  - `WebcamStartError` (`src/camera.rs:23`)
  - `KeyboardMonitorError` (`src/keyboard_monitor.rs:118`)
  - `AgentParseError` (`src/agents/parser.rs:26`)
  - `ShortcutParseError` (`src/shortcuts/types.rs:18`)
  - `MenuExecutorError` (`src/menu_executor.rs:45`)

Gaps:
- `menu_executor` defines typed `MenuExecutorError`, but public API returns `anyhow::Result<()>` (`src/menu_executor.rs:32`, `src/menu_executor.rs:466`) while docs imply direct typed return (`src/menu_executor.rs:450`). This is workable via downcast, but blurs API intent.
- `src/error.rs` defines top-level `ScriptKitError` and alias `error::Result<T>` (`src/error.rs:15`, `src/error.rs:76`), but no in-repo usages were found outside that file. This indicates a fragmented/unfinished error architecture.

### 2) Are error types well-defined?
**Partially.**

Strong areas:
- Typed variants are descriptive and test-backed in shortcuts/menu executor (`src/menu_executor_tests.rs:22`, `tests/shortcut_error_messages.rs:4`).
- `PersistenceError` includes `source()` and `From` conversions (`src/shortcuts/persistence.rs:128`, `src/shortcuts/persistence.rs:138`, `src/shortcuts/persistence.rs:144`).

Weak areas:
- Some custom errors remain stringly/manual instead of `thiserror`:
  - `ConfigWriteError` (`src/config/editor.rs:45`) has `Display` but no `std::error::Error` impl and no structured source chaining.
  - `SpaceError` (`src/window_control_enhanced/spaces.rs:31`) is manual and all payloads are strings/ids, limiting source composition.
- Duplicate modeling in shortcuts persistence:
  - Rich typed path (`PersistenceError`) exists, but convenience APIs use `anyhow::Result` (`src/shortcuts/persistence.rs:269`, `src/shortcuts/persistence.rs:307`) and are what callers typically consume.

### 3) Are we using `.context()` for better error messages?
**Yes, extensively, and generally correctly.**

Examples:
- Frecency load/save are well-contextualized for file I/O and serialization (`src/frecency.rs:255`, `src/frecency.rs:306`, `src/frecency.rs:314`).
- Audit logging in MCP has clear stage-specific context (`src/mcp_streaming.rs:175`, `src/mcp_streaming.rs:179`, `src/mcp_streaming.rs:188`).
- Alias/shortcut persistence include path-specific context (`src/aliases/persistence.rs:89`, `src/shortcuts/persistence.rs:286`).

Observed gap pattern:
- Several code paths convert errors into `Option` (`.ok()`, `.ok()?`, `unwrap_or_default`) before they can carry context to logs/callers.

### 4) Any places where errors are swallowed silently?
**Yes. Multiple production paths suppress errors with no telemetry.**

Highest-impact occurrences:
1. Frecency startup/save errors dropped:
- `frecency_store.load().ok()` (`src/app_impl.rs:67`)
- `self.frecency_store.save().ok()` (`src/app_impl.rs:2526`)

2. Alias overrides load failures silently replaced by defaults:
- Cache path: `load_alias_overrides().unwrap_or_default()` (`src/aliases/persistence.rs:46`)
- Save path: `load_alias_overrides().unwrap_or_default()` (`src/aliases/persistence.rs:114`)

3. Context mention file reads silently ignored:
- `std::fs::read_to_string(path).ok()` (`src/prompts/context.rs:193`, `src/prompts/context.rs:215`)

4. Embedded SDK extraction failures are fully suppressed:
- `.ok()?` on create/write/rename (`src/executor/runner.rs:151`, `src/executor/runner.rs:156`, `src/executor/runner.rs:157`)

5. Cache lock poisoning hidden as cache miss:
- `get_cache().lock().ok()?` (`src/window_control.rs:494`)

6. SSE serialization fallback masks encode failures:
- `serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string())` (`src/mcp_streaming.rs:88`)

### 5) Are error conversions (`From` impls) correct?
**Yes for the conversions present; coverage is uneven.**

Correct conversions observed:
- `ScriptKitError` uses `#[from]` for JSON/IO variants (`src/error.rs:23`, `src/error.rs:26`).
- `AgentParseError` uses `#[from] serde_yaml::Error` (`src/agents/parser.rs:31`).
- `PersistenceError` implements `From<std::io::Error>` and `From<serde_json::Error>` correctly (`src/shortcuts/persistence.rs:138`, `src/shortcuts/persistence.rs:144`).

Gap:
- Several manual error enums rely on ad-hoc string mapping instead of source-preserving conversions (`src/config/editor.rs:45`, `src/window_control_enhanced/spaces.rs:31`).

## Priority Recommendations
1. Align typed API boundaries in `menu_executor`.
- Either return `Result<(), MenuExecutorError>` publicly or explicitly document/standardize downcast usage when keeping `anyhow::Result<()>`.

2. Remove silent fallback in alias persistence.
- Replace `unwrap_or_default()` with warn+context behavior (similar to shortcuts: `src/shortcuts/persistence.rs:311`-`src/shortcuts/persistence.rs:316`) to avoid silent data loss/corruption masking.

3. Replace critical `.ok()`/`.ok()?` suppressions with logging + intent.
- For explicitly best-effort paths, log the error and current state before continuing.

4. Normalize manual error enums to `thiserror` where variant handling matters.
- Start with `ConfigWriteError` and `SpaceError` to improve source chaining and consistency.

5. Decide on one top-level error model.
- Either adopt `ScriptKitError` broadly or remove/archive it to avoid dead, conflicting error patterns.

6. Add regression tests for swallowed-error paths.
- `aliases`: ensure invalid JSON load failure emits warning and does not silently overwrite.
- `frecency`: verify load/save failures are observable in logs.
- `executor/runner`: ensure SDK extraction failure path is diagnosable.

## Bottom Line
- The codebase uses `anyhow` and `.context()` effectively in many app-level paths.
- `thiserror` is used in several important domain modules, but error architecture is inconsistent at boundaries.
- The biggest current reliability gap is silent error suppression (`.ok()`, `.ok()?`, `unwrap_or_default`) in production flows that should at least emit structured diagnostics.
