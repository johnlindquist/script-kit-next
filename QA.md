# Script Kit — QA Test Stories

Hand-off stories for manual QA. Each story is a real, end-to-end action a user
would take. Work top to bottom.

This file covers the launcher, built-ins, Notes, Dictation, Quick Terminal, the
Actions menu, the main-input sigils, and Agent Chat.

**Conventions**
- "Open the launcher" = press your global Script Kit hotkey (default brings up
  the main search window).
- "Main input" = the search box at the top of the launcher.
- The **footer** at the bottom shows the active shortcuts. Left chips:
  `~/.scriptkit` (cwd, opens with **Tab**) and `Codex · GPT-5.5` (agent · model,
  opens with **Shift+Tab**). Right: the primary action (**↵**) and **Actions
  (⌘K)**.
- **Agent Chat** = the AI chat surface. Open it from the main input with
  **Cmd+Return**.
- Report: what you did, what you expected, what actually happened, and a
  screenshot for anything visual.

---

## A. Launcher basics

### 1. Open and dismiss the launcher
1. Press the global hotkey to open the launcher.
2. Confirm the main input is focused and the script list is visible.
3. Press **Escape**.
- Expected: the window hides instantly with no flicker; pressing the hotkey
  again reopens it in the same place.

### 2. Filter the script list
1. Open the launcher.
2. Type part of a command name (e.g. `theme`).
3. Use the arrow keys to move the selection.
- Expected: the list narrows as you type, the top match is auto-selected, and
  the footer's primary action label updates to match the selected row.

### 3. Run a script with Enter
1. Open the launcher and select any script.
2. Press **Return**.
- Expected: the script runs; if it prompts for input, the appropriate prompt
  appears; otherwise the launcher closes/acts and you see the result.

### 4. Run a script by clicking
1. Open the launcher and type to filter.
2. Click a row with the mouse.
- Expected: clicking selects and runs the row (or opens its prompt) — same as
  pressing Return on it.

### 5. Recent / frequently used ordering
1. Run a couple of different commands.
2. Reopen the launcher with an empty query.
- Expected: items you just used surface near the top (frecency ordering).

### 6. Mini vs Full window sizing
1. Open the launcher (single-column list = Mini).
2. Select an item that shows a preview/detail pane.
- Expected: surfaces that need a preview/detail use the wider Full layout;
  simple lists stay in the compact Mini layout. No clipped content.

---

## B. Built-in tools

### 7. Clipboard History
1. Copy a few different things (text, then a URL, then an image).
2. Open the launcher, type `Clipboard History`, press Return.
3. Search within the history and select an entry; press Return.
- Expected: recent clips are listed newest-first; selecting one pastes/copies it
  back. Images show a thumbnail/preview.

### 8. Emoji picker
1. Open the launcher, type `Emoji`, press Return.
2. Search `rocket` and select an emoji.
- Expected: the emoji is inserted/copied so you can paste it into the frontmost
  app.

### 9. App Launcher
1. Open the launcher, run `App Launcher`.
2. Type an installed app name and press Return.
- Expected: the app launches and comes to the foreground.

### 10. Window Switcher
1. Have 2–3 apps open with windows.
2. Run `Window Switcher`, filter to a window, press Return.
- Expected: focus switches to that window.

### 11. Process Manager
1. Run `Process Manager` (a.k.a. Processes).
2. Search for a process by name.
3. Use **Actions (⌘K)** to kill a process you own (pick something safe).
- Expected: the list is searchable; the kill action ends the selected process
  and the list updates.

### 12. File Search
1. Run `File Search`.
2. Type part of a filename you know exists.
3. Select a file and open it.
- Expected: matching files appear; selecting opens the file (or its actions).

### 13. Kit Store
1. Run `Kit Store`.
2. Browse the community scripts and open one's detail.
- Expected: scripts are browsable with descriptions; an install action is
  available.

### 14. Theme Designer
1. Run `Theme Designer`.
2. Change a color/accent and observe the launcher chrome.
- Expected: the UI updates live to reflect the theme.

---

## C. Notes

### 15. Create a note
1. Open the launcher, run `Notes` (Open Notes).
2. Type a few lines of Markdown including a `# Heading` and a `- list`.
3. Close the Notes window.
- Expected: the Notes window is a floating editor; content autosaves; reopening
  shows what you typed.

### 16. Browse and reopen notes
1. Create 2–3 notes (Story 15).
2. From the launcher, run `Notes` and browse the list of existing notes.
3. Open an older one.
- Expected: notes are listed and searchable; opening restores the note's content
  and cursor.

### 17. Markdown rendering in Notes
1. In a note, add `**bold**`, `inline code`, and a `## Subheading`.
- Expected: Markdown renders/with appropriate styling in the editor/preview.

### 18. Notes-hosted Agent Chat
1. Open a note.
2. Trigger the in-Notes Agent Chat (via the Notes **Actions ⌘K** menu).
3. Ask it to summarize the note.
- Expected: an embedded chat answers using the note as context without leaving
  the Notes window.

---

## D. Dictation

### 19. Dictation setup
1. Run `Dictation Setup`.
2. Choose a microphone and model; grant mic permission if asked.
- Expected: device + model selection is shown; readiness is reflected without
  errors.

### 20. Dictate into the frontmost field
1. Focus a text field in another app (e.g. a notes app or browser).
2. Trigger Dictation, speak a sentence, then stop.
- Expected: a compact recording overlay appears with status; the transcript is
  delivered into the focused field.

### 21. Dictation history
1. After dictating at least once, run `Dictation History`.
2. Re-deliver or copy a past transcript.
- Expected: previous transcripts are listed and reusable.

---

## E. Quick Terminal

### 22. Open and use the Quick Terminal
1. Run `Quick Terminal`.
2. Run a shell command, e.g. `ls -la`.
- Expected: a working embedded terminal; output appears; theming matches the
  app. **Escape**/⌘W closes it.

### 23. Terminal apply-back (when available)
1. Open the Quick Terminal from a flow that supports apply-back.
2. Produce output and use the **Apply (⌘↩)** footer action.
- Expected: the Apply action only appears when an apply-back target exists, and
  it returns output to the originating surface.

---

## F. Actions menu (Cmd+K) across surfaces

### 24. Actions on the main menu
1. Open the launcher, select any script.
2. Press **Cmd+K**.
3. Filter the actions and run one (e.g. copy path / edit script).
- Expected: a searchable popover of contextual actions opens, scoped to the
  selected item; running one performs it and closes the popover.

### 25. Actions inside a built-in surface
1. Open `Clipboard History` (or `File Search`).
2. Press **Cmd+K** and run an action on the selected row.
- Expected: actions are scoped to that surface's selection.

### 26. Actions inside Agent Chat
1. Open Agent Chat (Story 30).
2. Press **Cmd+K**.
- Expected: chat-specific actions appear (e.g. change model, copy response,
  open history) and run correctly.

### 27. Actions inside Notes
1. Open a note.
2. Press **Cmd+K** and explore the available actions.
- Expected: Notes actions (new note, host chat, etc.) are listed and work.

### 28. Escape closes the popup first
1. Open any surface, press **Cmd+K** to open the actions popover.
2. Press **Escape** once.
- Expected: the first Escape closes only the popover (not the whole window); a
  second Escape backs out of the surface.

---

## G. Main-input sigils

### 29. Capture sigil `;`
1. Open the launcher.
2. Type `;` at the start of the input.
- Expected: a trigger popup appears suggesting capture targets/handlers; typing
  more filters the suggestions.

### 30. Project / cwd sigil `>` and the Tab cwd picker
1. Open the launcher.
2. Press **Tab** (or type `>`) to open the working-directory picker.
3. Pick a project directory (e.g. `~/dev/your-project`).
- Expected: the footer's left cwd chip updates from `~/.scriptkit` to the chosen
  directory, and it persists across surface switches and app restarts.

---

## Reporting template (per story)

```
Story #:
Result: PASS / FAIL / BLOCKED
What I did:
What I expected:
What happened:
Screenshot/notes:
```
