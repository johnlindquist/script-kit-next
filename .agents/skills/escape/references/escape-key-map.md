# Escape Key Map

This reference collects the high-risk Escape ownership paths. Use it after loading `$escape` when the task spans more than one surface or entry path.

## Mental Model

Escape should be handled by the most local meaningful owner:

1. Child popups and modal routes: actions dialog route stack, confirm popup, shortcut recorder, ACP slash/model/attach/history popups, prompt child popups.
2. Active surface cancellation: prompt cancel, ACP streaming cancel, file/path/editor surface-owned cancel.
3. Launch-origin resolution: launcher-entered surfaces may return to ScriptList; direct shortcut/deeplink/stdin launches should close/reset.
4. Main window lifecycle: hide/close/reset and automation re-key.

Do not flatten these layers into a single `DismissPolicy` edit. `DismissPolicy` is per `SurfaceKind`; launch origin is per entry path.

## Primary Source Anchors

- `src/app_impl/lifecycle_reset.rs`: `close_and_reset_window`, `go_back_or_close`, prompt cancellation, reset-to-list behavior.
- `src/main_sections/window_visibility.rs`: `hide_main_window_helper`, non-RPC hide reset and automation re-key.
- `src/main_sections/app_state.rs`: `opened_from_main_menu` state currently tracks launcher-return behavior.
- `src/main_sections/app_view_state.rs`: `AppView`, `SurfaceKind`, `DismissPolicy`, `semanticSurface`, surface contracts.
- `src/app_impl/startup.rs` and `src/app_impl/startup_new_actions.rs`: physical key interceptors, main-window global/action intents, ACP Escape routing.
- `src/main_entry/runtime_stdin_match_simulate_key.rs` and `src/main_entry/app_run_setup.rs`: stdin `simulateKey` Escape behavior.
- `src/main_entry/runtime_tray_hotkeys.rs`: global/tray shortcut entry paths that can skip ScriptList.
- `src/actions/dialog.rs`: actions dialog Escape route-stack vs close behavior.
- `src/actions/window.rs`: attached actions popup parent automation surface preservation.
- `src/ai/acp/view.rs`: ACP local Escape handling, streaming cancellation, picker/model/attach popup dismissal.

## Stable Contracts

- `lat.md/surfaces.md`: surface registry, popup-first key routing, `DismissPolicy`, and generated surface contracts.
- `lat.md/automation.md`: `semanticSurface` re-keying, hide/RPC reset parity, actions dialog Escape filter-agnostic contract, popup close routing.
- `lat.md/protocol.md`: `simulateKey` is fire-and-forget; use follow-up state inspection.
- `tests/app_view_policy_contract.rs`: no wildcard/default escape hatches in surface/dismiss policy.
- `tests/hide_rpc_surface_reset_contract.rs`: reset before automation re-key for hide/RPC paths.
- `tests/actions_dialog_escape_filter_agnostic_contract.rs`: actions Escape pops/closes without clearing search text.
- `tests/actions_dialog_route_stack_contract.rs`: route-stack Escape restores parent route state.
- `tests/acp_shortcut_contracts.rs`: ACP Escape cancels streaming before idle close/return handling.
- `tests/stdin_show_hide_simulatekey_no_response_envelope_contract.rs`: `simulateKey` has no response envelope.

## Shortcut/Direct Launch Scenario

Oracle session `shortcut-escape-window-close-2` found the likely bug class: `go_back_or_close` already has the desired branch, but direct entry paths can inherit stale `opened_from_main_menu` state or bypass the branch.

Preferred implementation shape:

- Wrap `opened_from_main_menu` behind helpers such as `mark_escape_returns_to_main_menu`, `mark_escape_closes_window`, and `should_escape_return_to_main_menu`.
- Consider renaming the field to `return_to_main_menu_on_escape` if the edit is broad enough.
- Mark direct hotkey/deeplink/stdin command entry paths as close-on-Escape before command execution.
- Mark launcher-selected surfaces as return-to-main only at the ScriptList selection owner.
- Keep `go_back_or_close` as the ordinary close/return owner.
- Make `close_and_reset_window` clear stale return state and re-key automation to `scriptList` after reset/cancel.

## Edge Decisions To Make Explicit

- Direct filterable surface with typed filter: close immediately, or clear filter first? Current launcher-style behavior often clears first; shortcut-origin behavior may need close-first.
- Dirty prompt/editor input: current prompt cancellation usually cancels without confirmation. Add dirty confirmation only as a prompt rule, not as launch-origin logic.
- ACP direct hotkey vs return-preserving ACP: direct AI hotkeys should close when idle; context-capture or return-preserving ACP paths should restore their captured origin.
- Main menu Escape: keep ScriptList-specific filter/fallback/hide behavior out of `go_back_or_close`.

## Verification Recipes

Start with source contracts:

```bash
cargo test --test app_view_policy_contract
cargo test --test hide_rpc_surface_reset_contract
cargo test --test actions_dialog_escape_filter_agnostic_contract
cargo test --test actions_dialog_route_stack_contract
cargo test --test acp_shortcut_contracts
```

When behavior changes, add state-first proof for at least:

- Physical Escape from a launcher-entered prompt returns to ScriptList.
- Physical Escape from a direct shortcut prompt closes/hides the main window.
- `simulateKey` Escape from the same two entry paths matches physical behavior.
- ACP Escape cancels streaming first, closes popups first, and only then applies return/close policy.
- `listAutomationWindows` or `getState` after close reports a hidden/reset main window with `semanticSurface = scriptList`.
