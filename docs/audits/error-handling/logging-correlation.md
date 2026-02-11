# Logging Correlation Audit (logging + app_impl)

## Scope

- Requested paths: `src/logging.rs`, `src/app_impl.rs`
- Actual module paths in this repo:
  - `src/logging/mod.rs`
  - `src/app_impl/mod.rs` + referenced `src/app_impl/*.rs`
- Search terms used: `tracing::error`, `tracing::warn`, `log_debug`, `correlation_id`

## Match Inventory

- `src/logging/mod.rs`
  - `tracing::error`: 5
  - `tracing::warn`: 6
  - `log_debug`: 2 (definition only)
  - `correlation_id`: 35
- `src/app_impl/*`
  - `tracing::error`: 0
  - `tracing::warn`: 3
  - `log_debug`: 4
  - `correlation_id`: 0

## Findings

### F1: Input-history load failures are logged but not surfaced to users

- Evidence:
  - `src/app_impl/startup.rs:468`
  - `src/app_impl/startup_new_state.rs:182`
- Current behavior:
  - `history.load()` failure emits `tracing::warn!("Failed to load input history: {}", e)`.
  - No HUD/toast/inline status is shown.
- Impact:
  - Startup silently degrades behavior (history unavailable) with no user explanation.

### F2: Input-history save failures during execute flow are logged but not surfaced

- Evidence:
  - `src/app_impl/selection_fallback.rs:118`
- Current behavior:
  - `input_history.save()` failure emits `tracing::warn!("Failed to save input history: {}", e)`.
  - Selection execution continues with no user feedback.
- Impact:
  - Users can repeatedly lose history persistence with no visible signal.

### F3: Log-capture start failure logs `error` without guaranteed UI feedback

- Evidence:
  - `src/logging/mod.rs:706` (toggle flow)
  - `src/logging/mod.rs:719` (`tracing::error!` on `start_capture` failure)
- Current behavior:
  - On failure, `toggle_capture()` returns `(false, None)`.
  - No structured status payload is emitted to indicate whether the user saw failure.
- Impact:
  - Operational failure is in logs, but user-facing status handling is ambiguous and not auditable from logs alone.

### F4: App status surfaces are not structurally linked to correlation IDs

- Evidence:
  - `src/app_impl/*` has zero `correlation_id` usage.
  - HUD entrypoint does not emit status logs: `src/app_impl/shortcuts_hud_grid.rs:95`.
- Current behavior:
  - Correlation IDs are injected by the logging formatter (`current_correlation_id()` fallback), but app-level HUD/toast calls are not required to emit a paired, structured status event.
- Impact:
  - Hard to trace a user-visible message back to the exact action/run when status is rendered without a matching status log record.

### F5: `log_debug` usage in `app_impl` is diagnostic-only (not a user-feedback gap)

- Evidence:
  - `src/app_impl/filtering_cache.rs:13`, `src/app_impl/filtering_cache.rs:18`, `src/app_impl/filtering_cache.rs:90`, `src/app_impl/filtering_cache.rs:199`
- Current behavior:
  - Cache HIT/MISS/INVALIDATED debug logs; no user messaging.
- Assessment:
  - This is expected telemetry, not a UX gap.

## Proposed Standard: `user_status.v1`

Goal: Every user-visible status event is emitted as a structured log payload with the same `correlation_id` used by the action flow.

### Required fields

- `event_type`: `"user_status"`
- `correlation_id`: non-empty string
- `status_id`: stable machine key (example: `input_history.load_failed`)
- `status_level`: `"success" | "info" | "warn" | "error"`
- `status_surface`: `"hud" | "toast" | "inline" | "dialog"`
- `user_message`: exact user-visible text (or canonical variant key)
- `action`: operation name (example: `startup.load_input_history`)
- `outcome`: `"started" | "succeeded" | "failed" | "fallback" | "cancelled"`

### Optional fields

- `error_kind` (typed classification)
- `error_detail` (sanitized)
- `retryable` (bool)
- `suppression_reason` (required when a failure is intentionally not shown)
- `duration_ms`

### Emission rules

1. Any HUD/toast/inline status shown to users must have a matching `user_status.v1` log entry.
2. Any `warn`/`error` emitted inside a user action flow must either:
   - emit a user-visible status event, or
   - emit a `user_status.v1` event with `status_surface="dialog"|"inline"|"none"` plus `suppression_reason`.
3. On thread/task boundaries, propagate correlation context explicitly:
   - capture `let cid = logging::current_correlation_id();`
   - in spawned task/thread: `let _guard = logging::set_correlation_id(cid);`

## Recommended initial remediations (from this audit)

1. Add explicit status handling for input-history load/save failures (visible or explicitly suppressed with reason).
2. Add explicit failed-state user status for `toggle_capture()` start failures.
3. Add a single app-level helper that emits `user_status.v1` and then renders HUD/toast, so correlation and payload are always paired.
