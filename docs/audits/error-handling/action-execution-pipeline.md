# Action Execution Pipeline Error-Handling Audit

## Scope
- `src/app_impl.rs` (legacy monolith reference)
- `src/app_actions/*.rs`
- `src/app_impl/*.rs` execution modules currently included by `src/main.rs:268`
- `src/protocol/*.rs` message/type contracts for actions + scriptlet execution signaling

## Execution Entrypoints (Located)
| ID | Entrypoint | Trigger | Downstream execution |
|---|---|---|---|
| `AEP-ENTRY-001` | `src/app_actions/handle_action.rs:237` `handle_action()` | Actions dialog selection in most hosts | Built-in actions, clipboard/file actions, scriptlet actions, SDK actions |
| `AEP-ENTRY-002` | `src/app_impl/execution_paths.rs:4` `execute_path_action()` | Path prompt actions | file open/copy/trash operations |
| `AEP-ENTRY-003` | `src/app_impl/selection_fallback.rs:113` `execute_selected()` | Enter on ScriptList main list | script/scriptlet/builtin/app/window/fallback dispatch |
| `AEP-ENTRY-004` | `src/app_impl/selection_fallback.rs:253` `execute_selected_fallback()` | Enter while fallback mode active | fallback built-ins/scripts |
| `AEP-ENTRY-005` | `src/app_impl/selection_fallback.rs:287` `execute_builtin_fallback_inline()` | fallback built-in execution | Open URL, copy, calc, run terminal, execute builtin |
| `AEP-ENTRY-006` | `src/app_impl/execution_scripts.rs:39` `execute_scriptlet()` | scriptlet selection/shortcut/direct command dispatch | interactive/scriptlet executor/terminal run |
| `AEP-ENTRY-007` | `src/app_impl/execution_scripts.rs:268` `execute_script_by_path()` | direct path execution | scriptlet or script dispatch |
| `AEP-ENTRY-008` | `src/app_impl/execution_scripts.rs:341` `execute_by_command_id_or_path()` | command-id dispatch (`scriptlet/...`, `builtin/...`, `app/...`) | typed command routing then path fallback |
| `AEP-ENTRY-009` | `src/app_actions/sdk_actions.rs:2` `trigger_sdk_action_internal()` | SDK action trigger (`setActions`) | protocol send (`ActionTriggered`/`Submit`) |
| `AEP-ENTRY-010` | `src/app_impl/actions_dialog.rs:4` `route_key_to_actions_dialog()` + `src/app_impl/startup_new_actions.rs:217` | keyboard routing when actions popup open | returns `ActionsRoute::Execute { action_id }` then executes entrypoint-specific handler |
| `AEP-ENTRY-011` | `src/app_impl/startup_new_prelude.rs:257` | Enter on main filter input | `execute_selected()` or `execute_selected_fallback()` |

## Current Outcome/Propagation Patterns
| Pattern ID | Pattern | Example(s) | Propagation today |
|---|---|---|---|
| `AEP-PROP-001` | User-visible success/failure HUD | `execute_path_action()` copy/open/trash branches (`src/app_impl/execution_paths.rs:65`, `src/app_impl/execution_paths.rs:220`, `src/app_impl/execution_paths.rs:364`) | explicit HUD messages |
| `AEP-PROP-002` | User-visible toast errors | `execute_scriptlet()` temp write/executor failures (`src/app_impl/execution_scripts.rs:75`, `src/app_impl/execution_scripts.rs:238`) | toast + `cx.notify()` |
| `AEP-PROP-003` | Log-only failures | command-id lookups and app launch failures (`src/app_impl/execution_scripts.rs:367`, `src/app_impl/execution_scripts.rs:391`, `src/app_impl/execution_scripts.rs:407`) | logs only, no UI |
| `AEP-PROP-004` | Ignored result (`let _ = ...`) | clipboard copy/send confirmations/open URL (`src/app_actions/handle_action.rs:1293`, `src/app_actions/handle_action.rs:1639`, `src/app_impl/selection_fallback.rs:333`) | error dropped entirely |
| `AEP-PROP-005` | Silent cancel | confirm flows in trash/delete (`src/app_actions/handle_action.rs:1134`, `src/app_impl/execution_paths.rs:351`) | returns without user-visible `Cancelled` state |
| `AEP-PROP-006` | Partial success handled ad hoc | clipboard bulk delete (`src/app_actions/handle_action.rs:1681` to `src/app_actions/handle_action.rs:1716`) | HUD with `Deleted X, failed Y`; no typed status |
| `AEP-PROP-007` | Async fire-and-forget with optimistic HUD | `create_script` spawns `open`, then immediately shows success/hides window (`src/app_actions/handle_action.rs:548` to `src/app_actions/handle_action.rs:560`) | success shown before operation outcome known |
| `AEP-PROP-008` | Transport drop not surfaced to user | SDK action send channel full/disconnected (`src/app_actions/sdk_actions.rs:55` to `src/app_actions/sdk_actions.rs:66`) | logged only |

## Action -> Outcome States -> Current UI -> Normalized Outcomes
| Action / Group | Observed outcome states now | Current UI feedback | Recommended normalized outcomes (unified status layer) |
|---|---|---|---|
| `SDK action trigger` (`AEP-ENTRY-009`) | `sent`, `channel_full_drop`, `channel_disconnected_drop`, `unknown_action`, `no_sender` | log-only | `queued`, `dispatched`, `dropped_retryable(backpressure)`, `dropped_terminal(disconnected)`, `rejected_unknown`, with visible non-blocking toast/HUD on drops |
| `Path copy actions` (`copy_path`, `copy_filename` in `AEP-ENTRY-002`) | `success`, `spawn_fail`, `write_fail`, `stdin_missing(silent)` | mixed HUD + silent branches (`src/app_impl/execution_paths.rs:140` to `src/app_impl/execution_paths.rs:160`) | `succeeded`, `failed_retryable(clipboard_io)`, `failed_terminal(validation)`; eliminate silent branches |
| `Path destructive action` (`move_to_trash` in `AEP-ENTRY-002`) | `confirm_open_failed`, `cancelled`, `trash_success`, `trash_failed` | HUD on open/success/failure; cancel silent (`src/app_impl/execution_paths.rs:351`) | `cancelled_user`, `failed_retryable(confirm_window_open)`, `failed_terminal(permission/not_found)`, `succeeded` |
| `Script list create/open actions` (`AEP-ENTRY-001`) | `spawn_success`, `spawn_fail_async` but immediate optimistic success | always success HUD in `create_script` (`src/app_actions/handle_action.rs:559`) | `running_external_process`, then terminal `succeeded`/`failed_retryable(spawn)`; defer success HUD until confirmed |
| `Scriptlet action via H3` (`scriptlet_action:*` in `AEP-ENTRY-001`) | `success`, `executor_nonzero`, `executor_err`, `action_not_found`, `file_read_fail` | HUD success/error exists; window hides even on failure (`src/app_actions/handle_action.rs:2557`) | keep context on failure and emit `failed_retryable(executor_io)` or `failed_terminal(not_found/parse)`; hide only on `succeeded` |
| `Scriptlet execution core` (`AEP-ENTRY-006`) | `temp_write_fail`, `executor_success`, `executor_nonzero`, `executor_err` | toast errors + hide on success | keep existing signal but add typed class/retryability and correlation ID |
| `Direct script dispatch` (`AEP-ENTRY-007`,`AEP-ENTRY-008`) | `found_and_exec`, `not_found`, `builtin_not_found`, `app_not_found`, `app_launch_failed` | mostly log-only (`src/app_impl/execution_scripts.rs:326`, `src/app_impl/execution_scripts.rs:391`, `src/app_impl/execution_scripts.rs:410`) | `rejected_not_found`, `failed_retryable(app_launch)`, `failed_terminal(invalid_command)`, visible feedback required |
| `Fallback builtins` (`AEP-ENTRY-005`) | `copy_success`, `calc_success/fail`, `open_url_fail_ignored`, `open_file_fail_ignored`, `notes_open_fail_log_only`, `builtin_missing` | mixed HUD + log-only + ignored results (`src/app_impl/selection_fallback.rs:321`, `src/app_impl/selection_fallback.rs:333`, `src/app_impl/selection_fallback.rs:366`, `src/app_impl/selection_fallback.rs:388`) | `succeeded`, `failed_retryable(open_external)`, `failed_terminal(builtin_missing)`, `failed_terminal(calc_invalid_expression)` with standardized feedback |
| `Bulk delete clipboard` (`AEP-ENTRY-001`) | `all_deleted`, `partial_deleted`, `none_deleted`, `cancelled` | HUD for success/partial; cancel silent | `succeeded`, `partially_succeeded {success_count, failure_count}`, `failed_terminal`, `cancelled_user` |
| `Main-list selection dispatch` (`AEP-ENTRY-003`) | `executed`, `history_save_failed`, `frecency_save_ignored` | action still runs; warn/log only (`src/app_impl/selection_fallback.rs:118`, `src/app_impl/selection_fallback.rs:165`) | add `succeeded_with_warnings` for non-fatal telemetry/storage failures |

## Retryable vs Terminal Failures (Normalization Targets)
| Class ID | Error class | Retryable? | Typical sources |
|---|---|---|---|
| `AEP-ERR-001` | `transport_backpressure` | Yes | `TrySendError::Full` (`src/app_actions/sdk_actions.rs:55`) |
| `AEP-ERR-002` | `transport_disconnected` | Usually No (until reconnect) | `TrySendError::Disconnected` (`src/app_actions/sdk_actions.rs:64`) |
| `AEP-ERR-003` | `external_spawn_failed` | Yes | `Command::spawn` failures (`src/app_actions/handle_action.rs:550`, `src/app_impl/execution_paths.rs:237`) |
| `AEP-ERR-004` | `clipboard_io_failed` | Yes | `pbcopy`/clipboard write failures (`src/app_impl/execution_paths.rs:83`, `src/app_impl/execution_paths.rs:109`) |
| `AEP-ERR-005` | `not_found` | No (without changed input/state) | missing script/app/builtin/action (`src/app_impl/execution_scripts.rs:326`, `src/app_impl/selection_fallback.rs:388`) |
| `AEP-ERR-006` | `user_cancelled` | Not an error | confirm cancel flows (`src/app_actions/handle_action.rs:1134`, `src/app_impl/execution_paths.rs:351`) |
| `AEP-ERR-007` | `partial_batch_failure` | Mixed | bulk delete (`src/app_actions/handle_action.rs:1681`) |

## Proposed Normalized Outcome Contract
Use one status envelope for all action entrypoints before any HUD/toast/log side effects.

```rust
enum UnifiedActionOutcomeState {
    Queued,
    Running,
    Succeeded,
    SucceededWithWarnings,
    PartiallySucceeded,
    CancelledUser,
    FailedRetryable,
    FailedTerminal,
    Dropped,
    Rejected,
}

struct UnifiedActionOutcome {
    action_id: String,
    entrypoint: String,       // e.g. "handle_action", "execute_path_action"
    state: UnifiedActionOutcomeState,
    message: Option<String>,  // user-facing summary
    error_class: Option<String>,
    retry_hint: Option<String>,
    success_count: Option<usize>,
    failure_count: Option<usize>,
    correlation_id: String,
}
```

## Gaps Blocking a Unified Status Layer
1. No explicit `Cancelled` terminal event in confirm flows (`AEP-PROP-005`).
2. Log-only and ignored-result branches create invisible failures (`AEP-PROP-003`, `AEP-PROP-004`).
3. No distinct `PartialSuccess` type outside one clipboard path (`AEP-PROP-006`).
4. Protocol action dispatch drops can be silent for users (`AEP-PROP-008`).
5. Some actions report success before async completion (`AEP-PROP-007`).

## Recommended Next Step for Implementation Phase
- Introduce a centralized `emit_action_outcome(...)` helper used by `AEP-ENTRY-001` through `AEP-ENTRY-009`, then map outcome states to HUD/toast/log policy in one place.
