# The Spine: Main Menu as Universal Prompt Builder

## Core Principle

**The input bar is the spine of the app. Everything below it is a projection of the input string + cursor position.**

The main menu search is leveraged for _everything_. The unified search remains the default, but as soon as someone uses the grammar, the main input becomes a "prompt builder" and each sigil triggers its related items in the list.

## What This Replaces

1. **Mini text agent** — removed entirely. Text operations are handled through the grammar (`.` sigil).
2. **Auto-switching to agent view** — removed. `@`, `/`, `|` no longer navigate to separate ACP picker views.
3. **Trigger popup** — removed. Sigils drive the main list in-place, no overlay popups.
4. **Portals** — removed as a UI concept. Context browsing (files, clipboard history, browser history, etc.) happens in the main list via `@` sub-search.

## Sigil Roster

### Prompt-Builder Sigils (compose an AI prompt, send to chat view)

| Sigil | List shows | After selection |
|-------|-----------|-----------------|
| `@` | Context types (selection, clipboard, screenshot, etc.) + sub-searches (`@file:`, `@browser-history:`, etc.) | Appended to input; sub-search types swap list to filtered browsing |
| `/` | Skills / slash commands | `/rewrite` appended, returns to free text |
| `\|` | Profiles | `\|creative` appended, returns to free text (no teaching menu) |
| `.` | Pre-configured rewrite styles | `.professional` appended; sugar for `\|style /rewrite @selection` |

### Script Execution Sigil (executes scripts, not AI prompts)

| Sigil | List shows | After selection |
|-------|-----------|-----------------|
| `;` | Capture targets | Script args as placeholder hints in input; list shows recent invocations; `@` context attachments available |

### Mode Exit Sigils (leave the grammar entirely)

| Sigil | Exits to |
|-------|----------|
| `~` | Pure file browser |
| `>` | Quick terminal |
| `?` | Actions help |

### List Filter (orthogonal — filters the unified search)

| Sigil | Behavior |
|-------|----------|
| `:` | Advanced query / tag filter on the unified list |

### Deferred

| Sigil | Direction |
|-------|-----------|
| `#` | User-defined tags on files/scripts/items for quick retrieval (future) |

### Dropped

| Sigil | Reason |
|-------|--------|
| `!` | Removed — `!` means NOT/inverse across 18+ surveyed launcher/search systems |

## Input Model

- **Plain text input.** No atomic chips/tokens. The grammar is just a string.
- **Decorative hint chips.** Visual badges in the input bar show the parsed state (e.g., `; capture`, `: refine`). Read-only — not interactive.
- **List is a pure function of input string + cursor position.** No hidden state machine. Backspace deletes one character, and the list reactively updates based on what the text parses to.
- **Cursor-segment aware.** Arrowing the cursor into different segments of the input swaps the list to match. If the input is `@file:readme |creative /rewrite make it punchier` and the cursor is inside `readme`, the list shows file search results filtered by "readme."
- **Segment visual states:**
  - **Resolved** (selected from list or matches known entity) — accent color
  - **Unknown** (typed, doesn't match any known entity) — warning color. The AI agent receives a preflight instruction: "The user typed an unknown file/skill 'X', please ask them about their intent before continuing."
  - **Active** (cursor inside, list showing matches) — focus indicator
- **Auto-expanding input.** Starts single-line, grows up to ~6 lines via existing `auto_grow(1, 6)` on `InputState`. Shift+Enter for explicit newlines. Scrolls beyond the cap.

## Execution Flow

### Prompt Submission

1. User composes prompt in main menu: `|creative /rewrite @selection make it punchier`
2. User hits **Enter**.
3. All `@` context references are **snapshotted** at Enter-press (before any view transition).
4. Main menu **morphs to chat view in-place** — same window, view transition.
5. Window **expands** from launcher size to chat size.
6. Chat view streams the AI response.
7. **Cmd+Enter** from chat view → paste the response back to the frontmost app.

### The Chat View Is the Executor

The main menu is the _composer_. The chat view is the _executor_. The main menu never handles streaming, cancellation, or multi-turn conversation — it builds the prompt and hands it off.

## Chat View Grammar

When the user types a sigil (`@`, `/`, etc.) in the chat composer for a follow-up turn:

- The chat transcript **hides instantly** (no animation).
- The main list **appears** below the input, showing results for the active sigil.
- Once the segment is resolved, the list disappears and the chat transcript returns.
- **Single shared input bar** across both modes — there is one input, always visible at the top.

## Escape Behavior (Progressive)

1. **First Escape:** Cancel the AI stream (if active).
2. **Second Escape:** Return to the main menu launcher. Input clears. Window shrinks.
3. **Third Escape:** Dismiss the app (hide the panel, return focus to frontmost app).

## Window Sizing

- **Launcher mode (default):** Compact window — search bar + list.
- **Chat mode (after Enter):** Window expands to accommodate conversation transcript.
- **Escape from chat:** Window shrinks back to launcher size.

## Default State (Empty Input)

- Unified search list — unchanged from today (apps, scripts, recent items).
- **Placeholder text:** `Search or type @ / | . ; for commands` (terse, one-line).

## Free-Text Tail Behavior

When the cursor is in the free-text portion of the input (after all sigils have been resolved), the list shows:
- **Recent prompts** matching the current text.
- **Resumable past conversations** matching the current text.

## `.` Sigil Details

`.` is pure sugar. `.professional` is equivalent to `|professional /rewrite @selection`.

- Styles are just profiles — no separate concept.
- Users create custom styles the same way they create profiles.
- The `.` sigil is convenience for the most common operation: "rewrite my selected text with this style."

## `;` Script Execution Details

`;` is script execution, not AI prompt composition. It coexists with the grammar but doesn't compose into AI prompts.

- After selecting a capture target, the input shows **placeholder hints** (ghost text) for expected script arguments.
- The list shows **recent invocations** of the selected script.
- `@` context attachments work — the script receives attached context as **separate structured arguments**.

## `@` Context Details

`@` shows 18 built-in context types + sub-search entry points:

**Built-in context types** (immediate attachment):
`Current`, `Full`, `Selection`, `Browser`, `Window`, `Diagnostics`, `Screenshot`, `Clipboard`, `FrontmostApp`, `MenuBar`, `RecentScripts`, `GitStatus`, `GitDiff`, `Processes`, `System`, `Dictation`, `Calendar`, `Notifications`

**Sub-search types** (swap list to filtered browsing):
`@file:` → file search, `@browser-history:` → browser history, `@clipboard:` → clipboard history search, `@dictation:` → dictation history, `@scripts:` → script search, `@scriptlets:` → scriptlet search, `@skills:` → skill search, `@notes:` → notes browser, `@history:` → ACP conversation history

Selecting a sub-search type (e.g., `@file:`) appends it to the input and swaps the main list to that search domain. Characters typed after the prefix filter within that domain.

## `@` References as Backlinks

`@` references are **persistent, navigable backlinks** when stored in conversations and notes.

- At send time, content is **snapshotted** for the AI (stable input).
- The stored reference is a **live backlink** — clicking `@file:readme.md` in a past conversation opens the current file.
- **Tombstone-safe:** if the referenced file/resource is deleted, the backlink is inert. No errors, no broken state.

## UX Polish (Post-Step-25, tackle with Oracle + DevTools)

1. **Live context previews in `@` list.** Instead of generic labels like "Browser URL" or "Selection", show the actual browser URL, actual window name, truncated selection text in the row subtitle. This gives users confidence about what they're attaching.

2. **`.` style preview.** When the user types `.` to pick a style, preview the text that will be operated on (e.g., the current selection content) so they can see what the style will transform.

3. **Slash commands: user commands only in main menu.** In the main menu `/` list, only show the user's custom commands. Agent-provided commands like `/compact` don't make sense until an Agent Chat conversation is active; those should only appear in the ACP chat grammar overlay (Step 19).

4. **No sigil prefix in row titles.** When typing `|`, the profile list should show "creative" not "|creative". Same for other sigils. Also ensure the correct icons are displayed for built-in profiles (e.g., creative → palette, concise → scissors, etc.).

## Open Questions (Deferred)

1. **First-run discoverability.** Is the terse placeholder sufficient, or does the app need a first-run onboarding moment?
2. **GUI escape hatch.** Should there be a button/menu for users who won't memorize sigils — a structured form for building prompts without typing grammar?
3. **`#` tag system.** User-defined tags on files/scripts/items. Direction decided, implementation deferred.
