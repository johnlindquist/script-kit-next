---
name: manage-notes
description: Manage notes — creating, searching, editing, and automating notes, which open in the main window under the same input, via the MCP notes tools and the automation protocol.
---

# Manage Notes

Create and automate notes. A note opens in Script Kit's main window, under the same input; there is no separate Notes window.

## Where Notes Live

Day notes are markdown files at `~/.scriptkit/brain/days/<date>.md`. Other notes are stored in SQLite at `~/.scriptkit/db/notes.sqlite`. Scripts and agents must use the runtime write ports (`kit/notes_create`, `kit/notes_update`, `kit/notes_delete`); do not raw-write the database.

Every active note is also mirrored as a read-only markdown file at `~/.scriptkit/notes/<slug>-<id8>.md`, so external tools (grep, editors, git) can read note content as plain files. The mirror is write-through from the app; treat the files as read-only — edits to them are not synced back and are overwritten on the next save.

## Opening a Note

- **Hotkey**: `Cmd+Ctrl+N` opens today's note by default — override with `notesHotkey` or disable with `notesHotkeyEnabled: false` in `config.ts`
- **From the launcher**: Search for "Notes" in the main menu
- **From automation**: the `openNotes` protocol command

### Configuring the Hotkey

```typescript
// ~/.scriptkit/config.ts
notesHotkey: {
  modifiers: ["meta", "shift"],
  key: "KeyN"
}
```

## Working in a Note

A note's lines are rows under a `NOTE · <name>` header; the input never moves.

- Type and press `Enter` — add a line
- `Enter` on a line — put its words in the input; `Enter` again saves them in place (its time and bullet are kept)
- `Cmd+Backspace` — delete a line; it waits 8 s in the line above the input, and `Cmd+Z` stops it
- `Cmd+P` — list notes by name under `NOTES`
- `Cmd+Enter` — ask the model about the note; the chat opens under the same input with the note attached
- Tags from frontmatter or `#tag` markdown, and wiki-style links and backlinks from `[[Note Title]]`

## Creating and Organizing Notes

Use the MCP notes tools when creating or organizing notes from an agent. They route through the app runtime, update the durable metadata index, and optionally open the changed note under the main input.

```json
{
  "name": "kit/notes_create",
  "arguments": {
    "title": "Project Plan",
    "body": "# Project Plan\n\n#planning [[Research Notes]]",
    "tags": ["planning", "projects/script-kit"],
    "aliases": ["Plan"],
    "open": true,
    "select": true
  }
}
```

```json
{
  "name": "kit/notes_update",
  "arguments": {
    "id": "NOTE_UUID",
    "content": "# Project Plan\n\nUpdated body with [[Decision Log]].",
    "tags": ["planning", "decisions"],
    "aliases": ["Plan", "Project Plan"],
    "open": true,
    "select": true
  }
}
```

Tags and aliases passed to the mutation tools are written into visible YAML frontmatter so users can edit them directly. Markdown `#tags` and `[[Wiki Links]]` are indexed from the note body. Backlinks are derived from the normalized link index; they are not copied into note content.

Read `kit://notes` to inspect organized notes. The resource returns each note's tags, aliases, outbound link count, and backlink count, and supports `?tag=...`, `?alias=...`, `?link=...`, or `?q=...` filters. Add `&full=true` to get complete note bodies instead of 240-char previews (bounded to 20 notes by default). Read `kit://notes/{id}` for full note content plus the same metadata.

### Instruction Notes

Notes tagged `#instructions` (or `tags: [instructions]` in frontmatter) are standing agent guidance: new Agent Chat threads automatically stage `kit://notes?tag=instructions&full=true` as context, so the agent reads them at thread start. Use this for durable user preferences, style rules, or project conventions that should apply to every conversation.

### Provenance

`kit/notes_create` accepts an optional `source` string (for example `scriptkit://agent-chat/{thread_id}`), written to frontmatter so a note links back to where it came from.

## Automation

An open note is a view of the main window. Open one with `openNotes`, then inspect it like any other main-window view: `getState` reports the open view, and `getElements` on the main window returns the note's rows under its `NOTE · <name>` header.

```json
{
  "type": "getElements",
  "requestId": "elm-note",
  "target": { "type": "main" },
  "limit": 20
}
```

## Common Pitfalls

- **No raw DB access**: Do not read/write `notes.sqlite` directly from scripts. Use the MCP notes tools for creation/update/delete.
- **No Notes window**: Do not target a separate Notes window; a note is a view of the main window.
- **No invented JS globals**: The current public Notes script surface is the MCP notes tools and the automation protocol. Do not document or rely on `notesOpen()`, `notesCreate()`, or similar JavaScript globals unless they are added to `scripts/kit-sdk.ts`.

## Related Examples

- **Canonical**: `~/.scriptkit/plugins/examples/scriptlets/notes/main.md` — copy-ready Notes automation payloads
- **Compatibility mirror**: `~/.scriptkit/plugins/examples/scriptlets/notes.md` — auto-generated flat copy of the canonical source

## Related Skills

- [start-chat](../start-chat/SKILL.md) — ask the model about a note and continue there
- [add-actions](../add-actions/SKILL.md) — expose note-related actions under ⌘K
- [new-scriptlet](../new-scriptlet/SKILL.md) — package Notes automation helpers as scriptlet bundles

## Done When

- [ ] `openNotes` opens the note under the main input
- [ ] `kit/notes_create` can create a tagged/linked note and open it
- [ ] `getElements` on the main window returns the note's rows under its `NOTE · <name>` header
- [ ] `kit/notes_update` changes the note and the view shows the new text
