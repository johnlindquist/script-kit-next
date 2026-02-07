# Error Handling Improvements Report

Date: 2026-02-07  
Agent: `codex-error-handling`

## Scope and Method

- Scope analyzed: `src/**/*.rs`
- Patterns scanned: `unwrap(`, `expect(`, `panic!(`, `.ok()`, `let _ =`, `is_err()`
- Approach:
1. Full-text scan over all Rust sources.
2. Manual triage to separate runtime paths from test-only assertions.
3. Deep context review for high-risk runtime call sites.

## Severity Rubric

- **Critical**: Can panic/crash in normal runtime flow.
- **High**: Can panic or hide operational failure in common paths; strong reliability impact.
- **Medium**: Error is swallowed or downgraded in non-critical paths; debugging/observability impact.
- **Low**: Invariant-based unwrap/expect, mostly safe but brittle; maintenance risk.

## Executive Summary

- Raw occurrences across `src/**/*.rs`:
  - `unwrap(`: **3038**
  - `expect(`: **217**
  - `panic!(`: **196**
- Most raw hits are in test modules and generated validation test files.
- Runtime-relevant risks are concentrated in startup/window bootstrap, setup/config migration, hotkeys, native interop helpers, and async UI update sites where errors are frequently dropped.

## Findings (Prioritized)

## Critical

### EH-CRIT-001: Startup panics in main window bootstrap
- **Locations**:
  - `src/main.rs:2328`
  - `src/main.rs:2331`
  - `src/main.rs:2400`
- **Issue**:
  - `.unwrap()` and `.expect(...)` are used in app/window initialization path.
- **Impact**:
  - Any window creation/update failure can hard-crash app startup.
- **Suggested fix**:
1. Return `anyhow::Result<()>` from bootstrap section.
2. Replace `.unwrap()` with `?` and attach `.context("...")`.
3. Replace `.expect("App entity should be set")` with explicit error:
   - `ok_or_else(|| anyhow!("App entity missing after window creation"))?`

### EH-CRIT-002: `tsconfig.json` mutation can panic on malformed structure
- **Locations**:
  - `src/setup.rs:1493`
  - `src/setup.rs:1524`
  - `src/setup.rs:1526`
- **Issue**:
  - Assumes `compilerOptions` and `paths` are JSON objects via chained unwraps.
- **Impact**:
  - Invalid user `tsconfig.json` can panic setup.
- **Suggested fix**:
1. Validate object types before mutation.
2. Coerce invalid shapes safely (replace with `{}`) while logging warning.
3. Convert function to return `Result<(), SetupError>` and propagate to caller.

### EH-CRIT-003: Script session unwrap during execution handoff
- **Location**:
  - `src/execute_script.rs:157`
- **Issue**:
  - `self.script_session.lock().take().unwrap()` assumes session always present.
- **Impact**:
  - Race or unexpected state causes panic during script launch path.
- **Suggested fix**:
1. Use `let Some(session) = ... else { return Err(anyhow!(...)); }`.
2. Include attempted operation + state details in error.
3. Surface via existing script error pipeline (`PromptMessage::ScriptError`).

## High

### EH-HIGH-001: Secrets cache/home path uses `expect` in runtime code
- **Locations**:
  - `src/secrets.rs:77`
  - `src/secrets.rs:90`
  - `src/secrets.rs:117`
- **Issue**:
  - Lock acquisition and `home_dir` resolution panic on failure.
- **Impact**:
  - Secrets subsystem can crash process instead of returning recoverable failure.
- **Suggested fix**:
1. Use `unwrap_or_else(|e| e.into_inner())` for poisoned mutex if acceptable, else typed error.
2. Change `secrets_path()` to `Result<PathBuf>` and propagate `dirs::home_dir` failure.

### EH-HIGH-002: Confirm window creation assumes dialog entity exists
- **Location**:
  - `src/confirm/window.rs:285`
- **Issue**:
  - `.expect("Dialog entity should have been created")` on optional entity.
- **Impact**:
  - Unexpected window lifecycle ordering panics confirmation flow.
- **Suggested fix**:
1. Replace with `ok_or_else` and return `Result` from constructor path.
2. Add structured error logs with correlation id and window state.

### EH-HIGH-003: Hotkey path unwraps mutex/global initialization
- **Locations**:
  - `src/hotkeys.rs:858`
  - `src/hotkeys.rs:890`
  - `src/hotkeys.rs:1108`
- **Issue**:
  - Unwrap on handler mutex and `MAIN_MANAGER.get()`.
- **Impact**:
  - Initialization races/poisoning can panic event thread.
- **Suggested fix**:
1. Replace with `if let Ok(...)`/`match` + fallback logging.
2. Handle missing `MAIN_MANAGER` as explicit startup error, not panic.

### EH-HIGH-004: Logging fallback can panic when opening `/dev/null`
- **Locations**:
  - `src/logging.rs:817`
  - `src/logging.rs:831`
- **Issue**:
  - `expect("Failed to open /dev/null")` inside error fallback path.
- **Impact**:
  - Logging setup failure can cascade into panic.
- **Suggested fix**:
1. Use in-memory sink fallback (`std::io::sink()` via custom writer) instead of `expect`.
2. Return startup error if mandatory logging sinks cannot initialize.

### EH-HIGH-005: Runtime image buffer construction expects internal invariant
- **Location**:
  - `src/list_item.rs:1385`
- **Issue**:
  - `RgbaImage::from_raw(...).expect(...)` in icon decode pipeline.
- **Impact**:
  - Corrupt/transformed image bytes can panic render path.
- **Suggested fix**:
1. Convert to `ok_or_else(|| image::ImageError::...)` and propagate.
2. Keep caller behavior (`Option`) but log decode reason before drop.

### EH-HIGH-006: Objective-C class lookups/string conversion unwrap in runtime
- **Locations**:
  - `src/frontmost_app_tracker.rs:211`
  - `src/frontmost_app_tracker.rs:365`
  - `src/frontmost_app_tracker.rs:384`
  - `src/frontmost_app_tracker.rs:471`
  - `src/frontmost_app_tracker.rs:472`
- **Issue**:
  - `Class::get(...).unwrap()` and `CString::new(...).unwrap()` in native observer setup.
- **Impact**:
  - Missing runtime class or interior NUL string causes panic.
- **Suggested fix**:
1. Wrap class lookup with explicit `Option -> Result` mapping.
2. Replace `CString::new` unwrap with fallible conversion and contextual error.

## Medium

### EH-MED-001: FFI helper string conversion unwrap pattern repeated in multiple modules
- **Locations**:
  - `src/window_control.rs:242`
  - `src/window_control_enhanced/capabilities.rs:139`
  - `src/menu_executor.rs:165`
  - `src/menu_bar.rs:277`
- **Issue**:
  - `CString::new(s).unwrap()` used in helper conversion functions.
- **Impact**:
  - Panic on interior-NUL input; often low probability but avoidable.
- **Suggested fix**:
1. Return `Result<CFStringRef>`/`Option<CFStringRef>` from helper.
2. Bubble error up with context about source string.

### EH-MED-002: `menu_item.unwrap()` after guard should be simplified to total match
- **Location**:
  - `src/menu_executor.rs:525`
- **Issue**:
  - unwrap used after explicit `is_none()` branch.
- **Impact**:
  - Currently safe by control flow, but brittle if refactored.
- **Suggested fix**:
1. Replace with `if let Some(menu_item) = ... { ... } else { ... }`.

### EH-MED-003: Scriptlet parser unwrap cluster depends on subtle invariants
- **Locations**:
  - `src/scriptlets.rs:512`
  - `src/scriptlets.rs:631`
  - `src/scriptlets.rs:721`
  - `src/scriptlets.rs:975`
  - `src/scriptlets.rs:1212`
  - `src/scriptlets.rs:1340`
  - `src/scriptlets.rs:1352`
  - `src/scriptlets.rs:1353`
  - `src/scriptlets.rs:1432`
  - `src/scriptlets.rs:1448`
  - `src/scriptlets.rs:1462`
  - `src/scriptlets.rs:1473`
  - `src/scriptlets.rs:1483`
- **Issue**:
  - Unwraps are often guarded but rely on parser state-machine invariants.
- **Impact**:
  - Malformed edge-case content may panic parser instead of returning parse failure.
- **Suggested fix**:
1. Convert unwraps to guarded branches returning `None`/error with line context.
2. Add regression tests for malformed/nested fence and conditional edge cases.

### EH-MED-004: Theme service drops specific error details
- **Locations**:
  - `src/theme/service.rs:92`
  - `src/theme/service.rs:134`
- **Issue**:
  - Uses `is_err()` and generic message, losing root cause.
- **Impact**:
  - Reduced observability during watcher/app-context failures.
- **Suggested fix**:
1. Capture and log concrete error value with structured fields.
2. Include `correlation_id` and operation tags (`watcher_start`, `theme_sync_update`).

### EH-MED-005: Clipboard DB query paths collapse all errors into `None`
- **Locations**:
  - `src/clipboard_history/db_worker/db_impl.rs:29`
  - `src/clipboard_history/db_worker/db_impl.rs:59`
- **Issue**:
  - `.ok()` on `query_row` hides DB corruption/query errors as cache miss/not-found.
- **Impact**:
  - Data issues become silent functional misses.
- **Suggested fix**:
1. Distinguish `rusqlite::Error::QueryReturnedNoRows` from actual errors.
2. Log non-`NoRows` failures and return `Result<Option<T>>` where possible.

### EH-MED-006: Clipboard image decode utilities suppress decode/encode errors
- **Locations**:
  - `src/clipboard_history/image.rs:124`
  - `src/clipboard_history/image.rs:129`
  - `src/clipboard_history/image.rs:139`
  - `src/clipboard_history/image.rs:141`
- **Issue**:
  - `.ok()?` converts failures to `None` with no diagnostics in critical conversions.
- **Impact**:
  - Hard to debug malformed clipboard content and format drift.
- **Suggested fix**:
1. Add tracing on decode/encode failures before returning `None`.
2. Consider `Result` variants on internal helpers where call sites can surface errors.

### EH-MED-007: Async UI update errors frequently swallowed (`cx.update(...).ok()` / `let _ =`)
- **Representative locations**:
  - `src/main.rs:2439`
  - `src/main.rs:2466`
  - `src/main.rs:2471`
  - `src/main.rs:2488`
  - `src/main.rs:2504`
  - `src/app_execute.rs:113`
  - `src/app_execute.rs:119`
  - `src/prompt_handler.rs:1591`
  - `src/render_builtins.rs:75`
  - `src/render_builtins.rs:172`
  - `src/render_builtins.rs:198`
  - `src/render_builtins.rs:296`
  - `src/app_actions.rs:1233`
  - `src/app_actions.rs:1282`
  - `src/app_actions.rs:1784`
  - `src/app_actions.rs:1838`
  - `src/app_actions.rs:1970`
  - `src/app_actions.rs:2024`
- **Issue**:
  - App-context update failures are intentionally ignored in many places.
- **Impact**:
  - Legitimate state-transition failures become invisible; race/shutdown bugs hard to diagnose.
- **Suggested fix**:
1. Replace bare `.ok()`/`let _ =` with helper:
   - `log_update_error("operation_name", update_result, correlation_id)`.
2. Keep best-effort semantics where needed, but always log failure with operation/state.

## Low

### EH-LOW-001: Frecency load/save errors intentionally discarded
- **Locations**:
  - `src/app_impl.rs:25`
  - `src/app_impl.rs:2487`
- **Issue**:
  - Errors discarded with `.ok()`.
- **Impact**:
  - Non-fatal, but silently loses ranking persistence.
- **Suggested fix**:
1. Maintain best-effort behavior.
2. Add warn-level structured log when load/save fails.

### EH-LOW-002: Invariant-based unwraps in parser/helpers are safe but brittle
- **Representative locations**:
  - `src/prompts/context.rs:106`
  - `src/prompts/context.rs:116`
  - `src/theme/gpui_integration.rs:298`
- **Issue**:
  - Unwrap/expect behind local invariants.
- **Impact**:
  - Low runtime risk; future refactors may violate assumptions.
- **Suggested fix**:
1. Prefer total matches in hot paths.
2. Keep unwraps only where invariant is proven and tested in same function.

## Test-Only / Non-Production Notes

Large unwrap/panic volumes are in test-only blocks and generated validation tests. Representative files:
- `src/actions/dialog_builtin_action_validation_tests_*.rs`
- `src/mcp_protocol.rs` (assertion-heavy tests)
- `src/stdin_commands.rs` (parser tests)
- `src/protocol/io.rs` (parser tests)

These are not immediate production stability issues unless test patterns are copied into runtime code.

## Recommended Fix Plan

### Phase 1 (Immediate safety)
1. Fix all **Critical** items (EH-CRIT-001..003).
2. Add explicit error propagation + context in startup/setup/script execution boundaries.
3. Add targeted regression tests for malformed `tsconfig.json` and missing script session.

### Phase 2 (Reliability)
1. Address **High** panic sources in hotkeys, confirm dialog, secrets, logging fallback, native tracker.
2. Introduce shared fallible FFI string/class helpers used by window/menu/frontmost modules.

### Phase 3 (Observability + maintenance)
1. Replace swallowed update patterns with centralized logging helper.
2. Differentiate recoverable miss (`None`) vs real error in DB/image helpers.
3. Harden parser invariants in `scriptlets.rs` with malformed-input test matrix.

## Verification Commands Used

```bash
rg -n "unwrap\(|expect\(|panic!\(" src --glob '*.rs'
rg -n "\.ok\(\)|let _ =|is_err\(\)" src --glob '*.rs'
nl -ba <file> | sed -n '<start>,<end>p'
```

## Notes / Gaps

- This report focuses on runtime-risk triage. Exhaustive listing of every test-only unwrap/panic (thousands of lines) is intentionally omitted.
- Some `.ok()` sites are deliberate best-effort paths; recommendation is to keep behavior but improve structured logging and state context.
