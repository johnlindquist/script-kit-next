# AI Chat Keyboard Shortcuts Research

Date: 2026-02-01
Scope: AI chat window (Script Kit GPUI) with references from chat apps, AI chat tooling,
WAI-ARIA Authoring Practices (APG), and WCAG keyboard accessibility guidance.

## Common shortcuts across apps (chat + AI)

| Action | Common keys (Win/Linux / macOS) | Notes |
| --- | --- | --- |
| Show keyboard shortcuts | Ctrl+/ or Cmd+/; Teams uses Ctrl+. | Slack and Discord use Ctrl/Cmd+/. Teams exposes a shortcuts list with Ctrl+. |
| Quick switcher / jump to conversation | Ctrl+K or Cmd+K | Both Slack and Discord use this for quick switching. |
| Search in current conversation | Ctrl+F or Cmd+F | Slack, Discord, and Teams all provide in-thread search. |
| Edit last message | Up arrow or Shift+Up/Alt+Up | Slack: Up arrow from the message field edits last message; Discord: Shift+Up (Win) / Alt+Up (Mac). |
| Cycle prior chat prompts | Up/Down arrows | Gemini Code Assist chat and Claude Code history use Up/Down arrows. |

## Navigation patterns (APG)

- Tab/Shift+Tab move focus between UI components; arrow keys move within composite
  components like listboxes, menus, and toolbars.
- Listbox: Up/Down move focus; Home/End jump; type-ahead is recommended; focus and
  selection are distinct.
- Combobox: Down Arrow opens/moves into popup; Escape closes; Enter accepts selection;
  supports standard single-line text editing keys.
- Toolbar: a single tab stop for the toolbar; arrow keys move among controls; Home/End
  jump to first/last.
- Modal dialog: focus moves into dialog; Tab/Shift+Tab cycle within; Escape closes; focus
  returns to the invoker on close.

## Accessibility requirements (WCAG)

- 2.1.1 Keyboard: all functionality operable via keyboard (with limited path-based
  exceptions).
- 2.1.2 No Keyboard Trap: focus must be able to move away using only the keyboard; if
  a non-standard exit is required, users must be told how to exit.
- 2.1.4 Character Key Shortcuts: single-character shortcuts must be disabled, remapped,
  or only active when the relevant component has focus.

## Suggestions for the Script Kit AI chat window

### Baseline, discoverable shortcuts

- Add a shortcuts overlay: Ctrl/Cmd+/ (plus in-app menu access) to match Slack/Discord.
- Add quick switcher: Ctrl/Cmd+K to jump to chats/threads.
- Add in-thread search: Ctrl/Cmd+F (optional Ctrl/Cmd+Shift+F for global search).
- Provide a focus cycle shortcut (optional) like F6 to jump between major sections
  (sidebar, message list, composer), mirroring Slack's section navigation.

### Composer behavior

- Enter sends; Shift+Enter inserts newline (Option+Enter on macOS as alternate).
- Up/Down arrow cycles prompt history when input is empty or caret is at start
  (avoid hijacking cursor navigation in the middle of text).
- Up arrow from empty input edits the last user message (Slack pattern).
- Ctrl+C stops generation (aligns with Claude Code); consider Esc for cancel only
  when it does not conflict with dialog closing.

### Message list and actions

- Treat the message list as a listbox: Up/Down to move between messages; Home/End
  for first/last; type-ahead to jump by author or title when the list has focus.
- If per-message actions are exposed (copy, regenerate, delete), present them as a
  toolbar with a single tab stop and arrow-key navigation.

### Pickers, popovers, and dialogs

- Model picker, slash-command palette, and file/mention pickers should follow the
  combobox pattern (Down to open, Escape to close, Enter to accept).
- Confirm dialogs should follow modal dialog rules (focus trap, Escape closes,
  focus returns to the trigger).

### Character key shortcuts and remapping

- Avoid single-letter shortcuts globally. If added (e.g., "R" to regenerate), make
  them active only when the message list is focused or provide a remap/disable
  mechanism to satisfy WCAG 2.1.4.

## Sources

1. Slack keyboard shortcuts (shortcuts list, quick switcher, search, focus sections).
   https://slack.com/help/articles/201374536-Slack-keyboard-shortcuts
2. Discord Commands, Shortcuts, and Navigation Guide (shortcuts overlay, quick switcher,
   search, edit last message).
   https://support.discord.com/hc/en-us/articles/225977308-Commands-Shortcuts-and-Navigation-Guide
3. Microsoft Teams Free keyboard shortcuts (shortcuts list, search, new chat).
   https://support.microsoft.com/en-us/office/keyboard-shortcuts-for-microsoft-teams-free-191a7d11-696a-4989-83b5-a503cd035db3
4. Gemini Code Assist keyboard shortcuts (chat prompt history navigation).
   https://cloud.google.com/gemini/docs/codeassist/keyboard-shortcuts
5. Claude Code interactive mode (cancel generation, history navigation, multiline input).
   https://docs.anthropic.com/en/docs/claude-code/interactive-mode
6. WAI-ARIA APG: Developing a Keyboard Interface (Tab vs arrow conventions).
   https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/
7. WAI-ARIA APG: Listbox pattern (keyboard interaction).
   https://www.w3.org/WAI/ARIA/apg/patterns/listbox/
8. WAI-ARIA APG: Combobox pattern (keyboard interaction).
   https://www.w3.org/WAI/ARIA/apg/patterns/combobox/
9. WAI-ARIA APG: Toolbar pattern (keyboard interaction).
   https://www.w3.org/WAI/ARIA/apg/patterns/toolbar/
10. WAI-ARIA APG: Modal dialog pattern (focus trap, Escape).
    https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/
11. WCAG 2.2 Understanding 2.1.1 Keyboard.
    https://www.w3.org/WAI/WCAG22/Understanding/keyboard
12. WCAG 2.2 Understanding 2.1.2 No Keyboard Trap.
    https://www.w3.org/WAI/WCAG22/Understanding/no-keyboard-trap.html
13. WCAG 2.1 Understanding 2.1.4 Character Key Shortcuts.
    https://www.w3.org/WAI/WCAG21/Understanding/character-key-shortcuts.html
