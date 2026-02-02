# Actions Dialog Focus Consistency Research

## 1) Files investigated
- `src/app_impl.rs` (main menu toggle + FocusCoordinator integration, focus restore)
  - Toggle open uses overlay focus request at `src/app_impl.rs:3392`.
  - Toggle close restores focus via overlay pop at `src/app_impl.rs:3376`.
  - Focus application for ActionsDialog at `src/app_impl.rs:1503`.
  - Central close helper uses overlay pop at `src/app_impl.rs:3971`.
- `src/render_builtins.rs` (clipboard actions toggle + focus handling)
  - Clipboard open branch begins at `src/render_builtins.rs:165`.
  - Clipboard close branch uses manual focus reset at `src/render_builtins.rs:140`.
  - Clipboard on_close callback sets pending focus at `src/render_builtins.rs:206`.
- `src/focus_coordinator.rs` (overlay stack behavior)
  - Overlay pop fallback to MainFilter at `src/focus_coordinator.rs:365`.
- `src/actions/window.rs` (actions window does not take focus)
  - Notes on focus routing at `src/actions/window.rs:237`.

## 2) Current behavior: main menu vs clipboard

### Main menu (script list)
- Cmd+K triggers `toggle_actions()`.
- On open, it pushes a FocusCoordinator overlay request for ActionsDialog
  (`self.push_focus_overlay(FocusRequest::actions_dialog())`) so pending focus
  is set to `FocusTarget::ActionsDialog` and cursor owner becomes ActionsSearch
  (`src/app_impl.rs:3392`).
- It moves GPUI focus to the app focus handle and disables the input focus
  (`src/app_impl.rs:3395`), relying on the shared ActionsDialog entity for
  keyboard routing.
- On close (toggle or escape), it pops the overlay, then immediately focuses
  the main filter for feedback (`src/app_impl.rs:3376`, `src/app_impl.rs:3380`).
- When actions close via key routing, `close_actions_popup()` also uses
  `pop_focus_overlay()` so the restore stack decides where focus returns
  (`src/app_impl.rs:3971`).

### Clipboard history
- Cmd+K in clipboard view uses `toggle_clipboard_actions()`.
- On open, it sets `show_actions_popup = true`, focuses the app focus handle,
  sets `focused_input = ActionsSearch`, and clears `gpui_input_focused`
  (`src/render_builtins.rs:165` through `src/render_builtins.rs:173`).
- It does NOT push a FocusCoordinator overlay (no call to
  `push_focus_overlay(FocusRequest::actions_dialog())`).
- On close via toggle, it manually sets `focused_input = MainFilter`,
  restores `gpui_input_focused = true`, and calls `focus_main_filter()`
  (`src/render_builtins.rs:140` to `src/render_builtins.rs:159`).
- On close via Escape (dialog on_close callback), it sets
  `pending_focus = Some(FocusTarget::AppRoot)` and `focused_input = MainFilter`
  (`src/render_builtins.rs:206`), which relies on the legacy pending-focus
  path instead of the FocusCoordinator overlay stack.

## 3) Root cause analysis of inconsistencies
- The main menu path is migrated to the FocusCoordinator overlay API, but the
  clipboard path still uses legacy `focused_input` + `pending_focus` updates.
  That means the coordinator state can be out of sync with the actual UI focus.
- When clipboard actions close via key routing, `route_key_to_actions_dialog()`
  calls `close_actions_popup()` which pops the FocusCoordinator overlay
  (`src/app_impl.rs:3838`, `src/app_impl.rs:3971`). If no overlay was pushed in
  the clipboard path, the coordinator falls back to MainFilter
  (`src/focus_coordinator.rs:365`) regardless of clipboard-specific intent.
- Actions windows do not take focus by design (`src/actions/window.rs:237`),
  so the only consistent way to manage cursor ownership and focus restoration
  is the shared FocusCoordinator flow. Clipboard bypasses that flow.

## 4) Expected Behavior (Verification Criteria)

### Both dialogs should:
1. **Receive focus when opened**: Keyboard events should be routed to actions dialog
2. **Handle keyboard events identically**:
   - Up/Down: Navigate action items
   - Enter: Execute selected action
   - Escape: Close dialog and restore focus
   - Backspace: Clear search filter character
   - Printable chars: Filter actions by typing
3. **Restore focus correctly when closing**: Focus returns to the main input/filter

### Implementation Consistency:
- **ActionsDialog component**: Used in both places (`ActionsDialog::with_script` for main menu, `ActionsDialog::with_clipboard_entry` for clipboard)
- **Keyboard routing**: Both should use `route_key_to_actions_dialog()` for centralized handling
- **Focus restoration**: Both should use `pop_focus_overlay()` or equivalent coordinator mechanism

## 5) Proposed solution (if fixes are needed)
- Align clipboard actions with the main menu focus flow:
  - On open, call `push_focus_overlay(FocusRequest::actions_dialog())` and
    remove manual `focused_input`/`gpui_input_focused` edits.
  - On close (toggle or Escape), call `pop_focus_overlay()` and then
    `focus_main_filter()` (or rely on the coordinator restore stack).
- Remove the `pending_focus = AppRoot` path in the clipboard on_close callback
  and use the same overlay-based restore used by the main menu.
- Optionally, apply the same change to file search actions (same legacy pattern
  as clipboard) to keep all built-in actions dialogs consistent.

## 6) Verification Status

### Code Analysis Findings:
- Both implementations use the shared `ActionsDialog` component
- Both route keyboard events through `route_key_to_actions_dialog()`
- Focus management differs (FocusCoordinator overlay vs legacy fields)
- Functionality is consistent but implementation symmetry could be improved

### Tests to Run (when build system is available):
```bash
cargo check && cargo clippy --all-targets -- -D warnings && cargo test
```

### Manual Test Scenarios:
1. Main menu: Launch app > Select item > Press Cmd+K > Verify arrows navigate actions > Press Escape > Verify focus returns to filter
2. Clipboard: Open clipboard history > Select item > Press Cmd+K > Verify arrows navigate actions (not clipboard entries) > Press Escape > Verify focus returns to clipboard filter
