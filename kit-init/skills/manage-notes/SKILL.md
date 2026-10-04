---
name: manage-notes
description: Manage notes — creating, searching, editing, and automating notes, which open in the main window under the same input, via the MCP notes tools and the automation protocol.
---

# Manage Notes

Create and automate notes. A note opens in Script Kit's main window, under the same input; there is no separate Notes window.

## Where Notes Live

Notes are a plain folder of Markdown files, laid out the way Obsidian and other Markdown apps expect. The folder is `~/.scriptkit/brain` unless `notesFolder` in `config.ts` names another (read when Script Kit starts):

- `days/YYYY-MM-DD.md` — one page per day. Captures are list items that start with their time: `- 14:32 called Sam`, tasks `- [ ] 14:32 book flights`. A note started that day is listed as `- 14:32 [[Trip ideas]]`.
- `notes/<Title>.md` — one file per note, named after its `# Heading`; renaming the heading renames the file, and `[[links]]` to it in other notes and day pages follow (links written as code stay as written). A taken name gets a number (`Trip ideas 2.md`). Notes may sit in folders (`notes/Travel/Lisbon.md`); a renamed note stays in its folder.
- `[[Title]]` links name files: the note with that file name first, then a title or alias. `[[Title|words]]` and `[[Title#Heading]]` work too. YAML front matter is kept (a block that isn't valid YAML is kept as typed); Script Kit adds `id`, `created` and `updated`. A copied note file gets its own `id`.
- Files can be edited elsewhere while open in Script Kit: saves merge line by line, and lines changed in both places are kept in both versions.

`~/.scriptkit/db/notes.sqlite` is a search index rebuilt from the files. Scripts and agents change notes through the runtime write ports (`kit/notes_create`, `kit/notes_update`, `kit/notes_delete`); never write the database.

## Finding a Note

Every notes search (the launcher, `@notes:`, the Notes list and the model's context search) matches the same way:

- The words of a title or body in any order, the last one half typed: `trip lisb`, `hotel castle`. Word forms count (`booking` finds `booked`), and accents don't matter.
- Request phrasing is skipped when the words alone find nothing: `find my note about the lisbon trip`.
- A slip in a title still finds it when nothing else matches: `lisbn` → "Lisbon trip".
- Day pages by date: `today`, `yesterday`, `tuesday`, `march 3`, `3 mar`, `2026-03-03`, `2026-03`, or with words on the page (`march sam invoice`). In the Notes list a date alone also lists the notes changed that day.
- `@notes:` is one word, so join words with `-`: `@notes:lisbon-trip`.
- `#tag`, `tag:`, `alias:` and `link:` search note metadata.

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

A note fills the main window as one Markdown editor; the line above it names what's open.

- `Cmd+N` — start a note titled with the time; type to rename it
- `Cmd+P` — switch notes in place; `Cmd+T` — today's page
- On a day page, `Enter` at the end of a timed line (`- 14:32 …`) starts the next one with the current time
- `[[Note Title]]` links notes; `Cmd+.` follows a link and `Esc` comes back
- `Cmd+K` — the note's actions (pin, copy link, duplicate, show in Finder, move to trash)
- Tags come from front matter or `#tag`

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
