**Implementation Plan**

Patch this as a narrow focus and routing fix, not as a new Day Page popup.

1. **Primary owners to inspect and patch**
- [day_page_switcher.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_switcher.rs): `open_note_switcher`, `wire_note_switcher_activation`, `handle_day_switcher_key`, close/restore helpers.
- [day_page_view.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/day_page_view.rs): Day Page key entry and Cmd+P dispatch.
- [actions_dialog.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/actions_dialog.rs): Actions Escape focus restore.
- [command_bar.rs](/Users/johnlindquist/dev/script-kit-gpui/src/actions/command_bar.rs): only if source inspection proves the local Day wrapper cannot safely preserve Notes parity.

2. **Cmd+P root-cause split**
- **Focus-ownership hypothesis:** Day opens the shared `CommandBar`, but focus may remain on the Day editor or otherwise fail to move to the key owner that routes switcher keys. This is the most compact explanation for typing, arrows, and Escape all failing, but it is not proven until current source and devtools confirm key ownership.
- **Key-routing-parity hypothesis:** even after focus is fixed, Day’s hand-written `handle_day_switcher_key` may still drift from Notes for `key_char`, composed input, row shortcuts, Home/End/PageUp/PageDown, and stale popup state.

Fix focus ownership first if `open_note_switcher` still does not move focus off the editor. Do not assume shared routing alone fixes the bug; if the Day handler is never reached, routing changes will not matter.

3. **CommandBar reuse**
Day should keep using the existing shared `CommandBar` with `CommandBarConfig::notes_recent_style()` and `get_note_switcher_actions()`. No Day-specific popup UI.

Keep a thin Day wrapper for Day-specific execution and restore semantics. Consider exposing/reusing `command_bar_key_intent` only if, after the focus fix, the current local wrapper is reached and still fails parity. That avoids turning this dirty-tree fix into a broader CommandBar API refactor.

4. **Focus restoration**
- **Day Cmd+P:** every close path should restore the Day editor. Preserve the existing `on_close` style if it already marks the switcher closed externally and focuses the editor. Add stale-state reconciliation only if current source shows external dismissal can leave `note_switcher.is_open()` logically stuck.
- **Day Actions Escape:** when Actions is opened from `ActionsDialogHost::MainList` while `current_view` is `AppView::DayPage`, restore focus to the Day editor, not the main filter. Prefer the smallest existing path, likely the current `EditorPrompt`/Day editor focus route, before adding a new `FocusTarget::DayPage`.

5. **Verification**
Use the smallest checks that can fail for the changed behavior:
- Focused Rust check/test through `./scripts/agentic/agent-cargo.sh`, never bare `cargo`.
- Add a small behavior test for Actions focus mapping if the focus function is testable without UI.
- Avoid broad source-audit tests or exact source-line count assertions.
- Required `script-kit-devtools` proof:
  - Day Page Cmd+P opens the Notes-style switcher.
  - Typing filters results.
  - Down and Up move selection.
  - Escape closes the switcher.
  - After close, typing lands in the Day editor.
  - Day Page Actions opens, Escape closes it, and focus returns to the Day editor rather than the main filter.

6. **Risks**
- Detached-window key ownership is not proven by the panel outputs. Devtools must confirm whether the detached `CommandBar` window, Day root, editor, or app handler owns keys after Cmd+P.
- The Day router can drift from Notes if it remains a duplicated copy of private `CommandBar` intent.
- Adding a new explicit Day Page focus target may be cleaner, but it is higher scope than necessary unless current source already has a natural extension point.
- Vestigial Day switcher inline state may be design debt, but removing it is out of scope for this bug unless it directly blocks the fix.


