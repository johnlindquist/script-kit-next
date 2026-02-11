# Unified Error/Status Layer Specification

## Purpose

Define one canonical status-event contract for all user-visible operation feedback. This spec standardizes:

- Event data model for `progress`, `success`, `warn`, `error`, `partial`
- Severity and terminal-state semantics
- Persistence and dedupe behavior
- Correlation-ID propagation
- Recovery actions (`retry`, `open_logs`, `copy_details`)
- UX acceptance criteria so failures are always visible, recovery is obvious, and success is unmistakable

## Scope

Applies to all operation-status signals emitted by app runtime, prompt handlers, and action execution paths.

Out of scope:

- Transport protocol changes outside status-event payloads
- Replacing existing logging backends

## Canonical Event Model

### Event Kind and Severity

`status_kind` and `severity` are separate.

- `status_kind` (required): `progress | success | warn | error | partial`
- `severity` (required): `info | warning | error | critical`

Required mapping:

- `progress` -> `info`
- `success` -> `info`
- `warn` -> `warning`
- `partial` -> `warning` or `error` (must reflect impact)
- `error` -> `error` or `critical`

### Status Event Schema (v1)

```ts
type StatusKind = "progress" | "success" | "warn" | "error" | "partial";
type Severity = "info" | "warning" | "error" | "critical";
type RecoveryActionId = "retry" | "open_logs" | "copy_details";

type StatusRecoveryAction = {
  id: RecoveryActionId;
  label: string;
  style: "primary" | "secondary";
  enabled: boolean;
  disabled_reason?: string;
  payload?: Record<string, unknown>;
};

type StatusEventV1 = {
  schema_version: "status.v1";

  // Identity
  event_id: string; // UUID/ULID; unique per event
  correlation_id: string; // Required for all events
  operation_id: string; // Stable ID for one user/system operation lifecycle
  source: string; // e.g. "path_prompt.actions.copy_path"
  code: string; // Stable machine code, e.g. "clipboard.write_failed"

  // Meaning
  status_kind: StatusKind;
  severity: Severity;
  title: string; // Short and user-facing
  message: string; // Specific, action-oriented

  // Timing
  created_at_ms: number; // Unix epoch ms

  // Optional context
  attempt?: number; // Starts at 1; increment on retry
  progress?: {
    current?: number;
    total?: number;
    percent?: number; // 0-100
    unit?: string; // files, bytes, steps
  };
  details?: {
    attempted?: string; // what was attempted
    failed_at?: string; // where it failed
    current_state?: string; // state snapshot at failure
    error_chain?: string[];
    metadata?: Record<string, unknown>;
  };

  // UX behavior hints
  dedupe_key?: string; // see dedupe rules
  persistence?: "ephemeral" | "sticky" | "feed";
  ttl_ms?: number; // only for ephemeral

  recovery_actions?: StatusRecoveryAction[];
};
```

### Required Field Guarantees

- `event_id`, `correlation_id`, `operation_id`, `source`, `code`, `status_kind`, `severity`, `title`, `message`, `created_at_ms` are required.
- `code` values are stable identifiers and must not be localized.
- `message` must describe impact and next-step intent in plain language.
- For `warn`, `error`, and `partial`, `details` must include `attempted` and `current_state`.
- For retryable failures, `attempt` is required and `recovery_actions` must include `retry`.

## Lifecycle and Terminal-State Rules

### Operation Lifecycle

- `operation_id` defines one lifecycle.
- `progress` is non-terminal.
- `success`, `warn`, `error`, and `partial` are terminal.
- Each `operation_id` must emit exactly one terminal event.
- Terminal events must never be dropped by dedupe.

### Visibility Timing

- First `progress` event should be visible within 100 ms of operation start.
- If operation exceeds 400 ms, a visible progress indicator is required.
- Terminal event must replace or follow the last progress event within 300 ms of completion/failure.

## Persistence Rules

| status_kind | Default persistence |                           TTL | Dismissal                             |
| ----------- | ------------------- | ----------------------------: | ------------------------------------- |
| `progress`  | `ephemeral`         | Until terminal or 30s timeout | Auto-clear on terminal                |
| `success`   | `ephemeral`         |                    2.5s to 4s | Auto-dismiss                          |
| `warn`      | `feed`              |                           N/A | Dismissible; retained in status feed  |
| `partial`   | `sticky`            |                           N/A | Must remain until user dismisses/acts |
| `error`     | `sticky`            |                           N/A | Must remain until user dismisses/acts |

Escalation rules:

- Repeated identical `warn` (>= 3 in 60s by dedupe key) escalates to `sticky`.
- Any `error` with `severity=critical` is non-auto-dismissible in primary surface until user acknowledges.

## Dedupe Rules

### Dedupe Key

If `dedupe_key` is omitted, compute:
`<source>|<code>|<operation_id>|<status_kind>`

### Dedupe Windows

- `progress`: coalesce within 250 ms; keep most recent payload.
- `success`/`warn`: merge identical events within 5 s into one visible item with `xN` count in feed metadata.
- `error`/`partial`: dedupe only if all are true:
  - same `source`
  - same `code`
  - same `operation_id`
  - same `attempt`
  - same `details.current_state`

### Never-Dedupe Cases

- Different `correlation_id`
- Different `attempt`
- Any terminal event that would hide a distinct failure root cause

## Correlation-ID Rules

- Every status event must include `correlation_id`.
- One top-level user action creates one correlation ID.
- Child operations may create new `operation_id` values but must retain the same `correlation_id`.
- `open_logs` must filter logs by `correlation_id` by default.
- `copy_details` payload must include `correlation_id`, `operation_id`, `event_id`, `code`, `source`, and `details`.

## Standardized Recovery Action Model

### Canonical Actions

Allowed standardized action IDs:

- `retry`
- `open_logs`
- `copy_details`

No aliases are allowed for these actions.

### Action Requirements by Kind

- `error`: must include `open_logs` and `copy_details`; include `retry` when retryable.
- `partial`: must include `open_logs` and `copy_details`; include `retry` when incomplete work can be resumed/retried.
- `warn`: include `open_logs` when warning implies degraded behavior or potential failure chain.

### Action Behavior

- `retry`
  - Re-executes the same operation contract with `attempt + 1`.
  - Must keep or restore user context needed to retry.
  - Disabled only when operation is non-retryable; include `disabled_reason`.
- `open_logs`
  - Opens log view pre-filtered to `correlation_id` and `operation_id` when available.
- `copy_details`
  - Copies structured JSON summary plus human-readable short summary.
  - Must include actionable fields (`code`, `message`, `attempted`, `failed_at`, `current_state`).

### Action Ordering in UI

1. `retry` (primary when enabled)
2. `open_logs` (secondary)
3. `copy_details` (secondary)

## UX Requirements (Non-Negotiable)

### Failure Visibility

- `error` and `partial` must appear in a persistent visible surface immediately.
- Failures cannot be represented only in logs.
- Silent failure states are forbidden: every failed operation must emit a terminal user-visible event.

### Recovery Clarity

- Every `error`/`partial` must present explicit next actions in the same UI element.
- Recovery actions must be keyboard accessible and discoverable without opening extra menus.
- If `retry` is unavailable, reason must be shown inline.

### Success Unmistakability

- `success` must use explicit affirmative copy and success iconography.
- Success message must contain the completed action and target (e.g. `Copied path to clipboard`).
- Success must be visually distinct from progress and warning states.

### Partial Outcome Clarity

- `partial` must include what succeeded and what failed (counts or named targets).
- `partial` cannot reuse the same styling as `success`.

## Acceptance Criteria

- `USL-AC-001`: Every operation lifecycle (`operation_id`) emits exactly one terminal event.
- `USL-AC-002`: 100% of `error` and `partial` events are visible in UI without opening logs.
- `USL-AC-003`: 100% of `error` events include `open_logs` and `copy_details`; retryable errors include `retry`.
- `USL-AC-004`: `open_logs` always opens pre-filtered by `correlation_id`.
- `USL-AC-005`: Dedupe never suppresses a distinct root-cause failure.
- `USL-AC-006`: Success messages are visually and textually distinct from progress/warn/partial/error.
- `USL-AC-007`: `partial` events expose both success and failure scope in user-visible text.
- `USL-AC-008`: For retry actions, next attempt increments `attempt` and preserves `correlation_id`.

## Event Examples

### Example A: Retryable Error

```json
{
  "schema_version": "status.v1",
  "event_id": "01K2ERROR8Q8W6V2M7T4Y2WZ6R5",
  "correlation_id": "01K2CORR9V0W1A2B3C4D5E6F7G",
  "operation_id": "path.copy_path",
  "source": "path_prompt.actions.copy_path",
  "code": "clipboard.write_failed",
  "status_kind": "error",
  "severity": "error",
  "title": "Could not copy path",
  "message": "Clipboard write failed. Retry or open logs for details.",
  "created_at_ms": 1760049000123,
  "attempt": 1,
  "details": {
    "attempted": "Write selected file path to clipboard",
    "failed_at": "execution_paths::copy_path",
    "current_state": "clipboard_provider_unavailable",
    "error_chain": ["spawn pbcopy failed: Broken pipe"]
  },
  "persistence": "sticky",
  "recovery_actions": [
    { "id": "retry", "label": "Retry", "style": "primary", "enabled": true },
    {
      "id": "open_logs",
      "label": "Open Logs",
      "style": "secondary",
      "enabled": true
    },
    {
      "id": "copy_details",
      "label": "Copy Details",
      "style": "secondary",
      "enabled": true
    }
  ]
}
```

### Example B: Partial Outcome

```json
{
  "schema_version": "status.v1",
  "event_id": "01K2PARTL8Q8W6V2M7T4Y2WZ6R5",
  "correlation_id": "01K2CORR9V0W1A2B3C4D5E6F7G",
  "operation_id": "batch.rename",
  "source": "path_prompt.actions.batch_rename",
  "code": "batch.rename_partial",
  "status_kind": "partial",
  "severity": "warning",
  "title": "Renamed 8 of 10 files",
  "message": "2 files were skipped due to permissions.",
  "created_at_ms": 1760049022450,
  "details": {
    "attempted": "Rename 10 selected files",
    "failed_at": "batch_rename::apply",
    "current_state": "2 permission_denied",
    "metadata": {
      "succeeded": 8,
      "failed": 2
    }
  },
  "persistence": "sticky",
  "recovery_actions": [
    {
      "id": "open_logs",
      "label": "Open Logs",
      "style": "secondary",
      "enabled": true
    },
    {
      "id": "copy_details",
      "label": "Copy Details",
      "style": "secondary",
      "enabled": true
    }
  ]
}
```

### Example C: Progress -> Success Sequence

1. Emit `progress` (`title: "Copying path..."`, `persistence: "ephemeral"`)
2. On completion, emit `success` within 300 ms (`title: "Path copied"`, `message: "Copied /Users/me/file.txt to clipboard"`)
3. Auto-dismiss success after 2.5s to 4s

## Implementation Notes

- Keep `code` and `source` values grep-friendly and stable.
- Prefer structured fields over interpolated strings in details.
- When adapting existing flows, add instrumentation where terminal events are currently missing.
