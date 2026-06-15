## Role Findings

The Day Page Cmd+P bug is most likely not a component mismatch. Day already uses the right shared pieces: `CommandBarConfig::notes_recent_style()` and `get_note_switcher_actions()` in [day_page_switcher.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_switcher.rs:113). The mismatch is routing and focus ownership.

Patch owners:

- [day_page_switcher.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_switcher.rs:113): `open_note_switcher`, `wire_note_switcher_activation`, `close_day_switcher`, `handle_day_switcher_key`.
- [day_page_view.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_view.rs:621): Day Page key entry and focus handoff after Cmd+P.
- [command_bar.rs](/Users/johnlindquist/dev/script-kit-gpui/src/actions/command_bar.rs:101): expose reusable key routing or a public key-intent adapter instead of duplicating Day-specific keyboard parsing.
- [actions_dialog.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/actions_dialog.rs:1190): fix Day Page Actions Escape focus restoration.

## Evidence And Assumptions

Notes Cmd+P has a complete lifecycle: open shared `CommandBar`, wire activation, wire `on_close`, clear context title, then request BrowsePanel focus. Close restores editor focus.

Day Cmd+P opens the same visual command bar, but then relies on `DayPageView::handle_key_parts` reaching `handle_day_switcher_key`. That is fragile because the Day editor can remain focused and consume printable keys, arrows, or Escape before the Day root wrapper sees them.

Actions Escape has a clearer focus bug: `AppView::DayPage` maps to `ActionsDialogHost::MainList`, and `request_focus_restore_for_actions_host(MainList)` maps to `FocusRequest::main_filter()`. But `apply_pending_focus(EditorPrompt)` already knows how to focus the Day editor when the current view is Day Page, so the missing piece is choosing `FocusRequest::editor_prompt()` for Day Page actions close.

## Failure Modes

- Keeping the current local Day key wrapper means future CommandBar features can keep drifting from Notes: Home/End/PageUp/PageDown, key-char handling, shortcut edge cases, and Escape route behavior.
- Focusing the editor while the switcher is open can cause text input to edit the Day page instead of filtering the switcher.
- Treating Day Page as `MainList` for focus restoration is semantically leaky. It works for action population, but not for focus.
- Adding a separate Day popup would violate the shared UI contract and create another inconsistent Notes-like selector.

## Recommendation

Reuse `CommandBar` routing more directly, but keep a thin Day-specific adapter for the parts that are truly Day-specific: execute selected note/day action, close, and restore Day editor focus.

Implementation shape:

1. Add a reusable public `CommandBar` key-routing method or key-intent API that covers the existing private `command_bar_key_intent` behavior. It should handle type, paste, backspace, arrows, Home/End/PageUp/PageDown, Enter, and Escape. Return an outcome like `Handled`, `Close`, or `Execute(action_id)` so hosts can own close/focus semantics.
2. Replace most of `DayPageView::handle_day_switcher_key` with that shared route. Keep only Day handling for `Cmd+P` toggle close and for executing selected actions through `execute_note_switcher_action`.
3. In `open_note_switcher`, after `open_centered`, activation wiring, and context-title clearing, move focus away from the Day editor to the Day Page/popup routing owner. The goal is: while the switcher is open, printable keys route to the switcher, not the editor.
4. Ensure every close path calls one Day-owned restoration helper, for example `restore_focus_after_note_switcher_close`, which closes or marks closed externally, clears any switcher state, then calls `focus_editor`.
5. Fix Actions Escape by changing focus restoration so `ActionsDialogHost::MainList` plus `current_view == AppView::DayPage` requests `FocusRequest::editor_prompt()`, not `main_filter()`. `apply_pending_focus` already routes `EditorPrompt` to `DayPageView::focus_editor`.

Verification:

- Run focused Rust tests through `./scripts/agentic/agent-cargo.sh`, not bare cargo.
- Add a pure behavior test for the extracted Actions focus mapping: ScriptList + MainList restores main filter, DayPage + MainList restores editor prompt.
- Add a focused test around the shared CommandBar key route if exposed: char filters, arrows move selection, Escape yields close, Enter yields selected action.
- Runtime proof with script-kit-devtools:
  - Open Day Page.
  - Press Cmd+P.
  - Type a note/day query and verify filter text/result list changes.
  - Press Down/Up and verify selected row changes.
  - Press Escape and verify popup closes.
  - Type into Day editor and verify focus returned.
  - Open Day Actions with Cmd+K, press Escape, then type and verify the Day editor receives input.

## Self Score

8/10. The design is strongly supported by current source, and the focus bug is concrete. I would want one quick implementation spike or devtools trace to confirm whether Day Cmd+P keys are lost to editor focus, parent routing, or detached `ActionsWindow` ownership in the exact failing path.


