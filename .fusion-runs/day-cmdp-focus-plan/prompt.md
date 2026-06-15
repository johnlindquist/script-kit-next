User request:

Fix the main-window Day Page Cmd+P behavior so it aligns with the Notes window Cmd+P behavior: same components/features/UI/UX, typing works, up/down arrows work, Escape works, and focus returns to the Day view after the switcher closes. Also fix the related Day Page Actions menu bug where pressing Escape closes actions but does not focus back to the Day view. After planning, implementation will be verified with script-kit-devtools.

Current repo constraints:

- Repo: /Users/johnlindquist/dev/script-kit-gpui
- Use current source over stale docs.
- Cargo commands must use ./scripts/agentic/agent-cargo.sh.
- Shared UI/components should be reused; do not invent a separate Day Page popup UI.
- There is a dirty tree; fixes must be narrowly scoped.

Relevant source:

- GLOSSARY.md says Day Page is `src/main_sections/day_page_view.rs` and `src/main_sections/day_page_types.rs`, and Cmd+P should use the same Notes search container/result language as Notes window Cmd+P, but selections open locally in the Day Page editor unless explicit Notes-window action is used.
- Notes Cmd+P is in `src/notes/window/panels.rs`:
  - `open_browse_panel` builds `NoteSwitcherNoteInfo` rows, uses `get_note_switcher_actions`, calls `self.note_switcher.open_centered(window, cx)`, wires activation via `wire_command_bar_activation(NotesCommandBarRole::NoteSwitcher, ...)`, clears context title, then calls `request_focus_surface(NotesFocusSurface::BrowsePanel, window, cx)`.
  - `close_browse_panel` calls `self.note_switcher.close(cx)`, clears mention portal edit, then requests editor focus.
  - `wire_command_bar_activation` installs `dialog.set_on_close(...)` to call `handle_detached_popup_closed_externally`, which marks the CommandBar closed without re-entering close path and restores primary focus. This specifically covers Escape/Cmd+K/focus loss while the detached popup is key.
- Day Cmd+P is in `src/main_sections/day_page_switcher.rs`:
  - `DayPageView::new` creates `note_switcher: CommandBar::new(Vec::new(), CommandBarConfig::notes_recent_style(), ...)`.
  - `open_note_switcher` loads regular notes and day notes, calls `get_note_switcher_actions`, `self.note_switcher.set_actions(actions, cx)`, `self.note_switcher.open_centered(window, cx)`, `self.wire_note_switcher_activation(window, cx)`, and clears context title.
  - `wire_note_switcher_activation` sets `on_close` to mark closed externally and focus editor. It sets activation routing to `execute_note_switcher_action`.
  - `handle_day_switcher_key` manually maps Escape, Cmd+P, up/down, enter, backspace, Option+Backspace, Cmd+V, and characters into `self.note_switcher`.
- Day key entry is in `src/main_sections/day_page_view.rs`:
  - `handle_key_parts` checks `self.is_day_switcher_open()` and then delegates to `handle_day_switcher_key`.
  - Cmd+P calls `self.open_note_switcher(window, cx)`.
  - Escape otherwise returns from fragment/note/past day or closes the window.
- Main Actions dialog close is in `src/app_impl/actions_dialog.rs`:
  - `route_key_to_actions_dialog` closes on Escape via `self.close_actions_popup(host, window, cx)`.
  - `close_actions_popup` invokes any dialog on_close, marks actions closed, closes detached actions window, pops focus overlay, calls `request_focus_restore_for_actions_host(host)`, clears context, then applies pending focus unless closing from actions window. If no pending focus applies, it focuses the app root.
  - There are existing Day Page special cases in route/close to block Agent Chat handoff.
- `src/actions/command_bar.rs`:
  - `CommandBarConfig::notes_recent_style()` is the reusable Notes recent-note switcher config.
  - `CommandBar` has reusable handlers: `handle_char`, `handle_backspace`, `handle_backspace_word`, `handle_paste`, `select_prev`, `select_next`, `execute_selected_action`, `mark_closed_externally`, `is_open`.
  - It has internal `command_bar_key_intent`, but that is private.

Symptoms to explain:

- Day Page Cmd+P popup opens but typing in it is broken.
- Up/down arrow keys do not work.
- Escape does not work.
- After popup closes, focus does not return to the Day view.
- Related: Day Page Actions menu Escape close does not focus back to the Day view.

Please produce an implementation plan with:

1. The likely source-owner functions to patch.
2. Whether Day should reuse `CommandBar` routing more directly or keep a local wrapper, and why.
3. The focus-restoration path for Day Cmd+P and Day Actions Escape.
4. Small behavior/source tests or runtime proof steps.
5. Risks in the current design.
