# Feature Tour

Script Kit GPUI looks like a launcher, but it is also a script runtime, notes system, clipboard memory, dictation system, and a local MCP server for AI agents. This guide maps the features people usually miss.

## One Hotkey, Three Gestures

The main hotkey (default `⌘;`) is gesture-aware:

| Gesture | Result |
| --- | --- |
| Tap | Toggle the launcher open/closed |
| Hold (~250 ms) | Open the **Day Page** — today's markdown diary |
| Double-tap | Open **Agent Chat** |

## Global Hotkeys

All of these are on by default and configurable in `~/.scriptkit/config.ts` (each has a matching `...Enabled` flag):

| Default | Feature | Config key |
| --- | --- | --- |
| `⌘⇧Space` | Agent Chat window | `aiHotkey` |
| `⌘⌃N` | Notes window | `notesHotkey` |
| `⌘⇧;` | Dictation (push-to-talk by default) | `dictationHotkey` |
| `⌘⌃I` | Inline AI edit of the focused text field | `inlineAiHotkey` |
| `⌘⌃R` | Instant Rewrite — streams rewrite variations of the focused text | `rewriteHotkey` |
| `⌘⇧L` | Toggle log capture | `logsHotkey` |

```typescript
export default {
  dictationHotkey: { modifiers: ["meta", "shift"], key: "Semicolon" },
  rewriteHotkeyEnabled: false, // e.g. free ⌘⌃R for Xcode
};
```

## The Main Input Is a Grammar

The launcher input accepts sigils that switch modes in place:

| Sigil | Meaning |
| --- | --- |
| `@` | Attach context (`@selection`, `@file:readme`, `@clipboard`) |
| `/` | Slash commands |
| `.` | Writing styles (`.professional`, `.concise`) |
| `\|` | AI profile selection |
| `;` / `todo; ...` | Structured capture (todo, note, link, snippet, cal, social) — `+` is a legacy alias |
| `:` | List filters (`:type:script`, `:tag:work`, `:source:main`) |
| `>` | Working-directory picker / command invocation |
| `~` `!` `?` | Exit to File Search, Quick Terminal, Actions Help |

Two keyboard moves worth learning:

- `Tab` — **Quick AI** on typed text (sends your query to AI), or the cwd picker on an empty input.
- `Shift+Tab` — open the agent **model/profile picker**.

Full details with examples: [Main Menu Input](./main-menu-input.md).

## Instant Answers and Fallbacks

- Type a math expression and an **inline calculator** row appears — `Enter` copies the result.
- When no script matches, fallback rows appear: **Do in Current App** (search the frontmost app's menu commands), **Ask AI**, **Search Files**, **Open URL**, **Open File**, and **Calculate**.

## Built-ins Worth Trying

Search these in the launcher:

| Search | What it opens |
| --- | --- |
| `clipboard` | Clipboard History with previews and power actions |
| `apps` | App Launcher |
| `window` | Window Switcher |
| `tabs` | Browser Tabs |
| `notes` | Notes window / note search |
| `emoji` | Emoji Picker |
| `files` | File Search |
| `process` | Process Manager (view/kill script processes) |
| `template` | New Script from Template |
| `sdk` | SDK Reference (generated from the live API catalog) |
| `dictation` | Dictation setup, history, and start-dictation variants |
| `permissions` | Permissions wizard (Accessibility, Screen Recording, Microphone, ...) |
| `dark mode`, `volume`, `sleep` | macOS system actions |

## Clipboard Sediment: Copies That Keep Themselves

Clipboard history quietly feeds your Day Page:

- Copy a **URL** once and it is auto-kept as a link on today's Day Page.
- Copy the **same text twice** and it is promoted to your brain (re-copy = signal it matters).
- **Secret-looking content is rejected** before it is ever stored in history.

No popups involved — see the [clipboard sediment ADR](../adr/0004-clipboard-sediment.md).

## Your Brain Is Markdown

Everything captured lands as plain markdown under `~/.scriptkit/brain/`:

```
~/.scriptkit/brain/
├── days/        # one file per day (the Day Page)
├── fragments/   # long captures (>200 words) with provenance frontmatter
├── notes/       # notes
└── trash/       # deleted notes (restorable)
```

## Notes Window Power Features

Open with `⌘⌃N`:

- `⌘P` — note switcher (search and jump between notes, including day notes)
- `⌘⇧P` — toggle the MD/TXT badge: markdown preview vs. plain-text editing
- `⌘N` — new note
- Hover footer — word/char counts, and a `trash (n)` badge that opens the trash view for restore/empty

## Clipboard History Power Actions

With an entry selected, `⌘K` (or the shortcut directly):

| Shortcut | Action |
| --- | --- |
| `⇧⌘P` | Pin / Unpin (pinned entries survive cleanup) |
| `⇧⌘C` | Copy Text from Image (OCR) — image entries |
| `⇧⌘S` | Save Text as Snippet (creates a scriptlet) |
| `⌃X` / `⇧⌘X` / `⌃⇧X` | Delete entry / delete matching search / delete all unpinned |
| `Space` | Quick Look preview |
| `⌃⌘A` | Attach to Agent Chat |

## Hidden Main-List Keys

With a script selected in the launcher:

| Shortcut | Action |
| --- | --- |
| `⌘K` | Actions menu |
| `⌘I` | Toggle info panel |
| `⌘E` | Edit script in `$EDITOR` |
| `⌘L` | Show script logs |
| `⌘N` | Create a new script (advertised in the empty state; also available as the **New Script** built-in) |

## Where to Go Next

- [Main Menu Input](./main-menu-input.md) — the full input grammar
- [SDK Scripting](./sdk-scripting.md) — write your own tools
- [MCP and Agent Context](./mcp-and-agent-context.md) — agents and context
- [Dictation](./dictation.md) — voice input
- [Computer Use](./computer-use.md) — window observation for agents
