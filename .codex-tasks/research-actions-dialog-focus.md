# Actions Dialog Focus Handling Research

## Focus Transfer Mechanism
- `FocusCoordinator` in `src/focus_coordinator.rs` manages focus state for the app.
- `FocusRequest` contains a `target` and `cursor_owner`.
- `FocusRequest::actions_dialog()` creates a request targeting `ActionsDialog` with `CursorOwner::ActionsSearch`.
- `push_overlay()` saves current focus state and switches focus to the actions dialog.

## Key Files
- `src/focus_coordinator.rs` - `FocusCoordinator`, `FocusRequest`, `FocusTarget::ActionsDialog`
- `src/app_impl.rs` - `toggle_actions` uses `focus_coordinator.push_overlay()`
- `src/actions/dialog.rs` - `ActionsDialog` struct

## Focus Flow
- Main menu: `Cmd+K` triggers `toggle_actions` -> `push_overlay(FocusRequest::actions_dialog())` -> `FocusTarget::ActionsDialog` + `CursorOwner::ActionsSearch`.
- This sets pending focus which `apply_pending_focus()` consumes to actually focus the element.
- When the dialog closes, `restore_stack` pops the previous focus state.

## Clipboard History Comparison
- Clipboard history in `render_builtins.rs` manually calls `focus_handle.focus` without using `FocusCoordinator`.
- This bypasses the pending focus system.
- Opening actions dialog from clipboard does not properly transfer focus because it does not use `push_overlay()`.

## Fix Pattern
- Any component that opens the actions dialog should call:
  `focus_coordinator.push_overlay(FocusRequest::actions_dialog())`
- This saves current focus state and sets pending focus to `ActionsDialog`.
