# PathPrompt Actions Popup Error-Handling Audit

## Scope
- Entry point: `src/render_prompts/path.rs`
- Prompt entity and shared state: `src/prompts/path/prompt.rs`, `src/prompts/path/render.rs`, `src/prompts/path/types.rs`
- Action execution path reached from PathPrompt actions: `src/app_impl/execution_paths.rs`
- Prompt wiring (event subscription + submit callback side effects): `src/prompt_handler/mod.rs`

## Flow Summary
1. `Cmd+K` or header actions button calls `PathPrompt::toggle_actions()` (`src/prompts/path/prompt.rs:221`).
2. `toggle_actions()` checks shared mutex state, emits `ShowActions`/`CloseActions` (`src/prompts/path/prompt.rs:222`, `src/prompts/path/prompt.rs:198`, `src/prompts/path/prompt.rs:212`).
3. Parent subscription handles events and opens/closes dialog (`src/prompt_handler/mod.rs:1211`, `src/render_prompts/path.rs:138`, `src/render_prompts/path.rs:169`).
4. While open, outer key handler routes keys to dialog and executes selected action (`src/render_prompts/path.rs:273`, `src/render_prompts/path.rs:293`, `src/render_prompts/path.rs:313`).
5. Path actions dispatch to `execute_path_action()` (`src/app_impl/execution_paths.rs:4`).

## Error/Failure Inventory (What User Sees vs Logs)
| ID | Location | Failure condition | User currently sees | Logs currently show |
|---|---|---|---|---|
| `PATH-ACT-LOCK-001` | `src/render_prompts/path.rs:24`, `src/render_prompts/path.rs:42` | `path_actions_showing`/`path_actions_search_text` lock fails in setter helpers | No direct UI error; state may stop syncing | `ERROR` via `logging::log(...)` |
| `PATH-ACT-LOCK-002` | `src/prompts/path/prompt.rs:222` | `actions_showing` mutex poisoned during toggle | Toggle still proceeds using poisoned inner value; no explicit warning in UI | `tracing::error!("path_prompt_actions_showing_mutex_poisoned_in_toggle")` |
| `PATH-ACT-LOCK-003` | `src/prompts/path/render.rs:70`, `src/prompts/path/render.rs:79`, `src/prompts/path/render.rs:154`, `src/prompts/path/render.rs:185` | Shared mutex poisoned while rendering header or key handling | Header/actions state may be stale or inconsistent; no inline error | `tracing::error!(...)` with specific poison markers |
| `PATH-ACT-LOCK-004` | `src/prompt_handler/mod.rs:1241` | Reset lock for `path_actions_showing` fails on `ShowPath` | No UI signal | No log (silent) |
| `PATH-ACT-DLG-001` | `src/render_prompts/path.rs:274` | `show_actions_popup == true` but `actions_dialog == None` | Actions popup appears non-functional; keypresses do nothing visible | `WARN` log (`actions popup open without dialog entity`) |
| `PATH-ACT-DLG-002` | `src/render_prompts/path.rs:104` | Search sync reads missing dialog and falls back to empty search text | Header search text clears/reset with no explanation | No explicit warning; only downstream state update logs |
| `PATH-ACT-DLG-003` | `src/prompts/path/prompt.rs:187` | `show_actions()` when no selected row (`filtered_entries.get(...)` is `None`) | `Cmd+K`/actions button appears to do nothing | No log for this branch |
| `PATH-ACT-EXEC-001` | `src/render_prompts/path.rs:293` | Enter pressed but no selected action ID | No action executes; popup remains, no explanation | No log |
| `PATH-ACT-EXEC-002` | `src/render_prompts/path.rs:304`, `src/render_prompts/path.rs:370` | Path info missing before execution | Popup may close; action does not run | No log |
| `PATH-ACT-EXEC-003` | `src/render_prompts/path.rs:306`, `src/render_prompts/path.rs:371` | Close-before-execute ordering: popup closes first, then action runs | On failure, user loses context immediately and must reopen actions | Only action execution logs/HUD after close, depending action branch |
| `PATH-ACT-EXEC-004` | `src/app_impl/execution_paths.rs:390` | Unknown action ID | Popup closes; no user-facing failure banner | `UI` log (`Unknown path action`) |
| `PATH-ACT-EXEC-005` | `src/app_impl/execution_paths.rs:138`, `src/app_impl/execution_paths.rs:141` | macOS `copy_filename`: `stdin` missing or `write_all` fails | No success/failure HUD in those branches | Spawn failure is logged; write failure is not logged |
| `PATH-ACT-EXEC-006` | `src/app_impl/execution_paths.rs:52`, `src/app_impl/execution_paths.rs:55` | macOS `copy_path`: `stdin` missing branch not handled | No success/failure HUD for missing-stdin branch | Spawn/write failures logged; missing-stdin not logged |
| `PATH-ACT-EXEC-007` | `src/prompt_handler/mod.rs:1169` | Path submit callback `try_send` fails (`Full` / `Disconnected`) after `select_file` action | Selection appears accepted, but script may not receive result | `WARN` for full, `UI` log for disconnected |
| `PATH-ACT-FOCUS-001` | `src/render_prompts/path.rs:113` | Focus restore only if `current_view` still `AppView::PathPrompt` | If view changed/raced, focus restore may not happen and no fallback cue | No log for failed/missed restoration |

## Silent Failure States and Required UX/Recovery
| State ID | Silent symptom today | UI signal that should exist | Recovery action that should exist |
|---|---|---|---|
| `PATH-SILENT-001` (`PATH-ACT-LOCK-004`) | Shared state reset lock failure is invisible | Non-blocking HUD: `Actions state unavailable` | Auto-reset local prompt state + show `Press Cmd+K to retry` |
| `PATH-SILENT-002` (`PATH-ACT-DLG-003`) | `Cmd+K` does nothing when no selection exists | Inline hint in footer/header: `No item selected` | Auto-select first row when list non-empty; otherwise disable actions button |
| `PATH-SILENT-003` (`PATH-ACT-EXEC-001`) | Enter with no selected action yields no response | Small toast: `No action selected` | Keep popup open and move selection to first actionable item |
| `PATH-SILENT-004` (`PATH-ACT-EXEC-002`) | Missing path context drops action silently | Blocking error toast: `Item no longer available` | Refresh entries and reopen actions anchored to new selection |
| `PATH-SILENT-005` (`PATH-ACT-EXEC-005`/`006`) | Clipboard write/missing-stdin branches can do nothing visible | Error HUD with exact reason (`Clipboard unavailable`) | Keep popup open and expose `Retry` shortcut (`Enter`) |
| `PATH-SILENT-006` (`PATH-ACT-FOCUS-001`) | Lost focus after close has no cue | Visual focus ring on restored control, else toast `Focus restored to prompt` | Explicit fallback to prompt focus handle or app root with log + HUD |

## Ambiguous Success States and Required UX/Recovery
| State ID | Ambiguity today | UI signal that should exist | Recovery action that should exist |
|---|---|---|---|
| `PATH-AMBIG-001` (`PATH-ACT-EXEC-003`) | Popup closes before action result; failure appears detached from initiating action | Pending HUD state: `Running <action>...` followed by success/failure | Reopen actions with prior selection on failure |
| `PATH-AMBIG-002` (`PATH-ACT-EXEC-004`) | Unknown action logs only; user sees closed popup | Error HUD: `Action not supported in this context` | Keep dialog open and select nearest valid action |
| `PATH-AMBIG-003` (`PATH-ACT-EXEC-007`) | Selection appears successful even when response channel drops message | Error toast with queue/disconnect reason | Offer `Retry submit` action in-place without leaving prompt |
| `PATH-AMBIG-004` (`PATH-ACT-DLG-002`) | Header search text clears when dialog missing | Inline badge: `Actions disconnected` | Auto-rebuild dialog entity and restore search term if available |

## Needs-Retry States and Required UX/Recovery
| State ID | Why retry is needed | UI signal that should exist | Recovery action that should exist |
|---|---|---|---|
| `PATH-RETRY-001` (`PATH-ACT-DLG-001`) | Popup state says open but entity missing | Error HUD: `Actions panel failed to open` | One-key retry (`Cmd+K`) should force re-create dialog state |
| `PATH-RETRY-002` (`PATH-ACT-LOCK-001/002/003`) | Poisoned/failed locks can leave stale state | Warning indicator in header actions pill | Soft reset shared state mutex values and retry toggle automatically once |
| `PATH-RETRY-003` (`PATH-ACT-EXEC-005/006`) | Clipboard integration can fail transiently | Failure HUD with retry countdown | Auto-retry once, then expose manual retry |
| `PATH-RETRY-004` (`PATH-ACT-EXEC-007`) | Channel backpressure/disconnect can be transient during script lifecycle | Toast: `Could not deliver selection` | Retry queueing once; if still failing, keep prompt open and ask user to rerun action |

## Highest-Risk Gaps
1. Close-before-execute (`src/render_prompts/path.rs:306`, `src/render_prompts/path.rs:371`) removes action context before success/failure is known.
2. Multiple silent branches around missing selection/path/action IDs (`src/prompts/path/prompt.rs:187`, `src/render_prompts/path.rs:293`, `src/render_prompts/path.rs:304`) make `Cmd+K` look flaky.
3. Focus restore is best-effort only and has no fallback logging/UI when `AppView` is no longer `PathPrompt` (`src/render_prompts/path.rs:113`).
4. macOS clipboard copy paths have missing failure branches in `copy_filename` and partial coverage in `copy_path` (`src/app_impl/execution_paths.rs:138`, `src/app_impl/execution_paths.rs:52`).

## Suggested Acceptance Criteria for Follow-up Fixes
- Every action invocation emits exactly one user-visible terminal state: success, failure, or canceled.
- Any failed action keeps or restores enough UI context to retry immediately.
- `Cmd+K` never results in a no-op without an inline reason.
- Focus restoration path logs both success and fallback cases with correlation ID.
