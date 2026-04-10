---
name: notes
description: Working with the Notes window — creating, searching, editing, and automating notes via the SDK and automation protocol.
---

# Notes

Create and automate notes in Script Kit's floating Notes window.

## Where Notes Live

Notes are stored in SQLite at `~/.scriptkit/db/notes.sqlite`. The Notes window manages them directly — scripts interact via automation targets, not raw database access.

## Opening the Notes Window

- **Hotkey**: No default — set `notesHotkey` in `kit/config.ts` to enable
- **From the launcher**: Search for "Notes" in the main menu

### Configuring the Hotkey

```typescript
// ~/.scriptkit/kit/config.ts
notesHotkey: {
  modifiers: ["meta", "shift"],
  key: "KeyN"
}
```

## Notes Window Features

- Markdown editing with formatting toolbar
- Multiple notes with sidebar navigation
- Full-text search across all notes
- Soft delete with trash and restore
- Export to plain text, Markdown, or HTML
- Character count in footer

## Automation Targets

The Notes window is a first-class automation target. Use `target: { "type": "kind", "kind": "notes" }` to address it.

### Semantic IDs

| Semantic ID | Element |
|-------------|---------|
| `panel:notes-window` | The Notes window container |
| `input:notes-editor` | The active editor area |

### Query Elements

```json
{
  "type": "getElements",
  "requestId": "elm-notes",
  "target": { "type": "kind", "kind": "notes" },
  "limit": 10
}
```

### Wait for Notes Window

```json
{
  "type": "waitFor",
  "requestId": "w-notes",
  "target": { "type": "kind", "kind": "notes" },
  "condition": {
    "type": "elementExists",
    "semanticId": "input:notes-editor"
  },
  "timeout": 3000,
  "pollInterval": 25
}
```

### Batch Commands

```json
{
  "type": "batch",
  "requestId": "b-notes",
  "target": { "type": "kind", "kind": "notes" },
  "commands": [
    { "type": "setInput", "text": "Hello from automation" }
  ]
}
```

## ACP Handoffs

Use the Notes UI actions for cross-surface handoffs:

- **Send to ACP Chat** — opens or focuses ACP Chat with the active note content
- **Save as Note** — creates or updates a note from ACP-generated content

These are UI actions, not JavaScript globals. The current public Notes script surface is the automation target (`kind: notes`) unless real Notes functions are added to `scripts/kit-sdk.ts`.

## Common Pitfalls

- **No raw DB access**: Do not read/write `notes.sqlite` directly from scripts. Use the automation protocol.
- **Hotkey required**: The Notes window has no default hotkey. Users must set `notesHotkey` in config before it appears in the launcher shortcuts.
- **Automation target must be open**: `getElements` and `batch` commands targeting Notes require the Notes window to be open. Use `waitFor` with a timeout to handle the case where it is not yet visible.
- **No invented JS globals**: The current public Notes script surface is the automation target (`kind: notes`). Do not document or rely on `notesOpen()`, `notesCreate()`, or similar JavaScript globals unless they are added to `scripts/kit-sdk.ts`.

## Related Examples

- **Canonical**: `~/.scriptkit/kit/examples/scriptlets/notes/main.md` — copy-ready Notes automation payloads for `getElements`, `waitFor`, and `batch`
- **Compatibility mirror**: `~/.scriptkit/kit/examples/scriptlets/notes.md` — auto-generated flat copy of the canonical source

## Related Skills

- [acp-chat](../acp-chat/SKILL.md) — send note content into ACP Chat and continue there
- [custom-actions](../custom-actions/SKILL.md) — expose note-related actions in the Actions Menu
- [scriptlets](../scriptlets/SKILL.md) — package Notes automation helpers as scriptlet bundles

## Done When

- [ ] Notes window opens and is addressable via automation target
- [ ] `getElements` returns elements with `panel:notes-window` and `input:notes-editor` semantic IDs
- [ ] `batch` commands successfully set editor content
