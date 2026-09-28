# Script Kit — QA Test Stories

Hand-off stories for manual QA. Each story is a real, end-to-end action a user
would take. Work top to bottom.

Script Kit is one window you tell what should happen. This file covers the
launcher, saying things to it, the line above the input, built-ins, Notes,
voice, Quick Terminal, ⌘K, the main-input symbols, and Agent Chat.

**Conventions**
- "Open the launcher" = press your global Script Kit hotkey.
- "Main input" = the input at the top of the window. It should never move, in
  any story. If it jumps even a pixel, that's a FAIL.
- "The line" = the one-line strip directly above the main input. It says what's
  staged ("Acting on …"), what's waiting ("Runs in 8s"), or what just ran
  ("Last · … just now").
- The **capsule** floats at the bottom right: **↵** (its label names exactly
  what Enter will run), **⌘K**, and **⌘↵**. Left chips: `~/.scriptkit` (cwd,
  opens with **Tab** on an empty input) and `Codex · GPT-5.6 SOL` (agent ·
  model, opens with **Shift+Tab**).
- **Agent Chat** = the model's chat. It opens under the same input with
  **Cmd+Return**.
- Nothing should ever pop up over the window: no popovers, dialogs, toasts, or
  second windows. If one appears, that's a FAIL.
- Report: what you did, what you expected, what actually happened, and a
  screenshot for anything visual.

---

## A. Launcher basics

### 1. Open and dismiss the launcher
1. Press the global hotkey to open the launcher.
2. Confirm the main input is focused, the line is visible above it, and the
   list starts with a section header.
3. Press **Escape**.
- Expected: the window fades out instantly with no flicker; pressing the
  hotkey again fades it back in at the same place and size.

### 2. Filter the script list
1. Open the launcher.
2. Type part of a command name (e.g. `theme`).
3. Use the arrow keys to move the selection.
- Expected: the list narrows as you type, the top match is auto-selected, the
  focused row's icon tile turns gold (no row highlight or side bar), and the ↵
  label updates to match the selected row. The first section header never
  disappears.

### 3. Run a script with Enter
1. Open the launcher and select any script.
2. Press **Return**.
- Expected: the script runs; if it prompts for input, the prompt opens under
  the same input; otherwise the launcher acts and the line reads
  "Last · <script> just now".

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
  simple lists stay in the compact Mini layout. No clipped content. The input
  stays where it was.

---

## B. Say what should happen

### 7. Say a sentence
1. Open the launcher.
2. Type `remind me to call mom tomorrow at 9`.
- Expected: a "Jev · ⌘1–3" header appears with up to three readings of the
  sentence; rows below it do not move when the readings arrive. Parts of your
  sentence get colored underlines (the text itself stays white). ⌘1 runs the
  first reading.

### 8. The ↵ label stays steady while you type
1. Open the launcher.
2. Type `when does LeBron James play next?` one word at a time, pausing a
   beat between words.
- Expected: the ↵ label only changes when what Enter would run truly changes,
  never goes back to a label it already showed, and never flashes a generic
  "Run" in between. The input never moves.

### 9. A missing integration is a row, not an error
1. Make sure no calendar is connected.
2. Type `what's on my calendar tomorrow`.
- Expected: a row says "Needs Google Calendar"; ↵ reads "Connect Google
  Calendar" and opens the connect view under the same input. After
  connecting, your sentence runs without retyping it.

### 10. Symbols say it exactly
1. Select some text in another app, then open the launcher.
2. Type `/rewrite @selection .professional`.
- Expected: each symbol lists its completions in the main list as you type it
  and gets an underline in its own color: `/rewrite` (action), `@selection`
  (subject), `.professional` (details). Try `todo;`, `due:friday`, `>:dev`,
  and `|creative` too.

### 11. It learns
1. Say a sentence and run it with Enter.
2. Clear the input and type its first few letters again.
- Expected: the sentence is listed as a learned match and ↵ offers it right
  away.

---

## C. The line: hold, undo, and staging

### 12. Irreversible actions wait instead of asking
1. Run `Empty Trash`.
- Expected: no dialog. The line reads "Runs in 8s". Typing pauses it
  ("Paused while you type"); **⌘Z** on an empty input stops it and the line
  says "Didn't run: …". Left alone, it runs when the wait ends.

### 13. Reversible actions undo with ⌘Z
1. On a file row, press **⌘K** and run `Move to Trash`.
- Expected: it happens at once; the line reads "Last · … just now" with a
  ⌘Z hint. **⌘Z** on an empty input restores it exactly once.

### 14. Act on the last run
1. Run anything, then open the launcher with an empty input.
2. Press **↑**.
- Expected: the last run is staged in the line and the list offers Undo it /
  Run it again / Share it.

### 15. Act on the clipboard
1. Copy some text in another app.
2. Open the launcher and paste into the empty input.
- Expected: the text is staged in the line (not typed into the input), and the
  list becomes what you can do with it (e.g. "Add it to today's note").

---

## D. Built-in tools

### 16. Clipboard History
1. Copy a few different things (text, then a URL, then an image).
2. Open the launcher, type `Clipboard History`, press Return.
3. Search within the history and select an entry; press Return.
- Expected: recent clips are listed newest-first; selecting one pastes/copies it
  back. Images show a thumbnail/preview.

### 17. Emoji picker
1. Open the launcher, type `Emoji`, press Return.
2. Search `rocket` and select an emoji.
- Expected: the emoji is inserted/copied so you can paste it into the frontmost
  app.

### 18. App Launcher
1. Open the launcher, run `App Launcher`.
2. Type an installed app name and press Return.
- Expected: the app launches and comes to the foreground.

### 19. Window Switcher
1. Have 2–3 apps open with windows.
2. Run `Window Switcher`, filter to a window, press Return.
- Expected: focus switches to that window.

### 20. Process Manager
1. Run `Process Manager` (a.k.a. Processes).
2. Search for a process by name.
3. Press **⌘K** on a process you own (pick something safe) and kill it.
- Expected: the list is searchable; the kill action ends the selected process
  and the list updates.

### 21. File Search
1. Run `File Search`.
2. Type part of a filename you know exists.
3. Select a file and open it.
- Expected: matching files appear; selecting opens the file (or its actions).

### 22. Kit Store
1. Run `Kit Store`.
2. Browse the community scripts and open one's detail.
- Expected: scripts are browsable with descriptions; an install action is
  available.

### 23. Theme Designer
1. Run `Theme Designer`.
2. Change a color/accent and observe the launcher chrome.
- Expected: the UI updates live to reflect the theme.

---

## E. Notes

### 24. Add to a note
1. Open the launcher, run `Open Notes` (or press **Cmd+Ctrl+N**).
2. Type a line and press Return; add two more.
- Expected: the note opens under the same input with a `NOTE · <name>` header;
  each line you add becomes a row. There is no separate Notes window. Reopening
  shows what you typed.

### 25. Edit and delete a line
1. In a note, select a line and press Return: its words move into the input.
2. Change them and press Return again.
3. Select another line and press **⌘⌫**.
- Expected: the edited line keeps its time and bullet. The delete waits 8 s in
  the line with ⌘Z to stop it — no confirm dialog.

### 26. Browse and reopen notes
1. Create 2–3 notes.
2. In a note, press **⌘P**.
3. Open an older one.
- Expected: notes are listed by name under a `NOTES` header and filter as you
  type; opening restores the note.

### 27. Ask about a note
1. Open a note.
2. Press **⌘↵**.
- Expected: the model's chat opens under the same input with the note staged
  in the line ("Asking about …"), and the first answer uses it as context.

---

## F. Voice

### 28. Microphone
1. Run `Microphone`.
2. Pick an input with Return; grant mic permission if asked.
- Expected: the inputs are rows under a header; the chosen one is used next
  time you talk.

### 29. Talk to type
1. Focus a text field in another app (e.g. a notes app or browser).
2. Hold the voice key (or press the dictation hotkey, default **⌘⇧;**) and
   speak a sentence, then let go.
- Expected: the main window fades in without taking the other app's cursor;
  your words fill the input; the rows are "Type it into <app>" (⌘1) and the
  request (⌘2). For plain dictation, letting go types the words at the cursor.

### 30. Talk to ask
1. Hold the voice key and say a request, e.g. "summarize what I copied".
2. Let go.
- Expected: Jev reads it as a request and, if it's sure, runs it; if it
  isn't, the window asks once ("Pick one · ⌘1 or ⌘2"). No overlay appears.

### 31. Dictation history
1. After dictating at least once, run `Dictation History`.
2. Paste or copy a past transcript.
- Expected: previous transcripts are rows, newest first, under the same input;
  ↵ pastes, ⌘K offers the rest.

---

## G. Quick Terminal

### 32. Open and use the Quick Terminal
1. Run `Quick Terminal`.
2. Run a shell command, e.g. `ls -la`.
- Expected: a working embedded terminal; output appears; theming matches the
  app. **Escape**/⌘W closes it.

### 33. Terminal apply-back (when available)
1. Open the Quick Terminal from a flow that supports apply-back.
2. Produce output and use the **Apply (⌘↩)** capsule action.
- Expected: the Apply action only appears when an apply-back target exists, and
  it returns output to the originating surface.

---

## H. ⌘K acts in place

### 34. ⌘K on the main list
1. Open the launcher, select any script.
2. Press **Cmd+K**.
3. Filter the actions and run one (e.g. copy path / edit script).
- Expected: the script is staged in the line ("Acting on …") and the list under
  the same input becomes its actions. The first action does what the row does.
  Nothing pops up.

### 35. ⌘K inside a built-in
1. Open `Clipboard History` (or `File Search`).
2. Press **Cmd+K** and run an action on the selected row.
- Expected: actions are scoped to that row, listed under the same input.

### 36. ⌘K inside Agent Chat
1. Open Agent Chat (Story 38).
2. Press **Cmd+K**.
- Expected: chat-specific actions appear (e.g. change model, copy response,
  past chats) and run correctly.

### 37. Escape lets go first
1. Open any surface and press **Cmd+K**.
2. Press **Escape** once, then again.
- Expected: the first Escape lets go of the staged row and the surface comes
  back exactly as it was; the next Escape goes back one step (a view, then the
  typed words, then the window).

---

## I. Agent Chat

### 38. Ask a quick question
1. Open the launcher with nothing typed and press **Cmd+Return**.
2. Ask something.
- Expected: the model's chat opens under the same input with nothing attached;
  the answer streams in. Tool calls and questions from the model are rows,
  never dialogs.

### 39. Past chats
1. Have at least two chats.
2. In a chat, pick "See past chats" (or type `my last chat` in the launcher).
- Expected: past chats are rows under a "Past chats" header, newest first;
  typing filters; Return reopens the thread under the same input.

---

## J. Main-input symbols

### 40. Capture target `;todo`
1. Open the launcher.
2. Type `buy milk ;todo` (or `buy milk todo;`).
- Expected: a bare `;` marks nothing; once the list name is complete, `;todo`
  gets the target underline, and the ↵ label names where the words will go.
  Nothing pops up.

### 41. The Tab cwd picker
1. Open the launcher.
2. Press **Tab** on the empty input to open the working-directory picker.
   (Typing `>` does not pick a cwd.)
3. Pick a project directory (e.g. `~/dev/your-project`).
- Expected: the left cwd chip updates from `~/.scriptkit` to the chosen
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
