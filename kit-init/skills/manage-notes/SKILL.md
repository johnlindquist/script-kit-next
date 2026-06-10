---
name: manage-notes
description: Manage the Notes window — creating, searching, editing, and automating notes via the SDK and automation protocol.
---

# Manage Notes

Create and automate notes in Script Kit's floating Notes window.

## Where Notes Live

Notes are stored in SQLite at `~/.scriptkit/db/notes.sqlite`. The Notes window manages them directly. Scripts and agents must use the runtime write ports (`kit/notes_create`, `kit/notes_update`, `kit/notes_delete`) or the Notes automation target; do not raw-write the database.

Every active note is also mirrored as a read-only markdown file at `~/.scriptkit/notes/<slug>-<id8>.md`, so external tools (grep, editors, git) can read note content as plain files. The mirror is write-through from the app; treat the files as read-only — edits to them are not synced back and are overwritten on the next save.

## Opening the Notes Window

- **Hotkey**: `Cmd+Ctrl+N` by default — override with `notesHotkey` or disable with `notesHotkeyEnabled: false` in `config.ts`
- **From the launcher**: Search for "Notes" in the main menu

### Configuring the Hotkey

```typescript
// ~/.scriptkit/config.ts
notesHotkey: {
  modifiers: ["meta", "shift"],
  key: "KeyN"
}
```

## Notes Window Features

- Markdown editing with formatting toolbar
- Multiple notes with a Cmd+P note switcher
- Full-text search across all notes
- Tags from frontmatter or `#tag` markdown
- Wiki-style links and backlinks from `[[Note Title]]`
- Soft delete with trash and restore
- Export to plain text, Markdown, or HTML
- Character count in footer

## Creating and Organizing Notes

Use the MCP notes tools when creating or organizing notes from an agent. They route through the app runtime, save any dirty open Notes editor before mutation, update the durable metadata index, and optionally open/select the changed note in the Notes window.

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

Notes created from Agent Chat ("Save as Note") carry `source: scriptkit://agent-chat/{thread_id}` frontmatter linking back to the originating conversation. `kit/notes_create` accepts an optional `source` string for the same purpose.

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

## Agent Chat Handoffs

Use the Notes UI actions for cross-surface handoffs:

- **Send to Agent Chat** — opens or focuses Agent Chat with the active note content
- **Save as Note** — creates or updates a note from Agent Chat-generated content

These are UI actions, not JavaScript globals. The current public Notes script surface is the automation target (`kind: notes`) plus the runtime MCP notes tools above unless real Notes functions are added to `scripts/kit-sdk.ts`.

## Common Pitfalls

- **No raw DB access**: Do not read/write `notes.sqlite` directly from scripts. Use the MCP notes tools for creation/update/delete and the automation protocol for window/editor control.
- **Default hotkey**: The Notes window toggles with `Cmd+Ctrl+N` unless overridden via `notesHotkey` or disabled via `notesHotkeyEnabled: false`.
- **Automation target must be open**: `getElements` and `batch` commands targeting Notes require the Notes window to be open. Use `waitFor` with a timeout to handle the case where it is not yet visible.
- **No invented JS globals**: The current public Notes script surface is the automation target (`kind: notes`) and MCP notes tools. Do not document or rely on `notesOpen()`, `notesCreate()`, or similar JavaScript globals unless they are added to `scripts/kit-sdk.ts`.

## Related Examples

- **Canonical**: `~/.scriptkit/plugins/examples/scriptlets/notes/main.md` — copy-ready Notes automation payloads for `getElements`, `waitFor`, and `batch`
- **Compatibility mirror**: `~/.scriptkit/plugins/examples/scriptlets/notes.md` — auto-generated flat copy of the canonical source

## Related Skills

- [start-chat](../start-chat/SKILL.md) — send note content into Agent Chat and continue there
- [add-actions](../add-actions/SKILL.md) — expose note-related actions in the Actions Menu
- [new-scriptlet](../new-scriptlet/SKILL.md) — package Notes automation helpers as scriptlet bundles

## Done When

- [ ] Notes window opens and is addressable via automation target
- [ ] `kit/notes_create` can create a tagged/linked note and open/select it in Notes
- [ ] `getElements` returns elements with `panel:notes-window` and `input:notes-editor` semantic IDs
- [ ] `batch` commands successfully set editor content
