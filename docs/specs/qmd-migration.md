# Spec: QMD Cutover — Markdown Files As Memory Substrate

Status: accepted, pre-implementation. Owning decision:
[ADR 0003](../adr/0003-markdown-files-as-memory-substrate.md). Product framing:
`VISION.md` → "The Memory Layer". Task breakdown: `.notes/brain-time.md`.

The app has not shipped. There is no migration path and no backwards
compatibility: existing local `notes.sqlite` content and `todos.jsonl` are
abandoned, and `todos.jsonl` read/write paths are deleted outright.

## Goal

Invert the current storage model: markdown files on disk become canonical for
all memory content (notes, day pages, captures, fragments); SQLite becomes a
derived, rebuildable index (FTS, embeddings, signals, links). This is the
original QMD intent (github.com/tobi/qmd) — the retrieval recipe was built
natively (`src/brain/search.rs`), but the substrate was not.

## Current State (verified June 2026)

| Store | Location | Canonical? | Notes |
| --- | --- | --- | --- |
| Notes | `~/.scriptkit/db/notes.sqlite` (`notes`, `note_tags`, `note_aliases`, `note_links`, `note_cart_items`) | Yes (problem) | Markdown *content* in sqlite rows; frontmatter parsed by `src/notes/metadata.rs` |
| Todos | `~/.scriptkit/menu-syntax/todos.jsonl` | Yes (problem) | DELETED by this cutover; `;todo` becomes a day-page task line |
| Links | `~/.scriptkit/plugins/main/scriptlets/links.md` | Yes (already files) | Section-based |
| Snippets | `~/.scriptkit/plugins/main/scriptlets/snippets.md` | Yes (already files) | Section-based |
| Brain index | `~/.scriptkit/db/brain.sqlite` (docs, embeddings, signals, inbox) | Derived (correct) | Mirrors sources via `src/brain/indexer.rs` |
| Clipboard | `~/.scriptkit/db/clipboard.sqlite` | Yes | Pinned entries mirror to brain |
| Agent Chat | `~/.scriptkit/agent_chat-history.jsonl` + `agent_chat-conversations/*.json` | Yes | Stays as-is; day pages get one-line traces |

## Target Layout (confirmed)

```text
~/.scriptkit/brain/
  days/
    2026-06-11.md          # one day page per day; human-readable diary
  fragments/
    2026-06-11-0942-slack.md   # long captures; provenance frontmatter
  notes/
    <slug>.md              # migrated notes; frontmatter: id, tags, aliases,
                           # pinned, created/updated, source
  trash/
    ...                    # soft-delete = move, recovery = move back
```

Conventions:

- Frontmatter carries identity and metadata: `id` (existing NoteId preserved),
  `tags`, `aliases`, `pinned`, `created`, `updated`, `source` (provenance URI,
  e.g. `scriptkit://agent-chat/{thread}`).
- Day pages reference fragments by relative link plus a `>`-quoted excerpt
  (~40 words), keeping the diary readable.
- `;todo` captures append task-list lines (`- [ ] body #tag due:...`) to
  today's day page. No separate todo store exists.
- Filenames are stable after creation; renames go through the app so the link
  index can follow.

## Cutover Plan

Pre-ship: no mirror-out phase, no reversibility ceremony, no data import.
Work lands in three steps (parallelizable per `.notes/brain-time.md`):

1. **Substrate module** — `~/.scriptkit/brain/` paths, day-page append API,
   fragment writer with provenance frontmatter, excerpt generation, atomic
   writes.
2. **Notes flip** — `src/notes/storage.rs` persists notes as files under
   `brain/notes/`; SQLite becomes index-only (FTS, tags, aliases, links),
   rebuilt from files; soft delete = move to `trash/`. `todos.jsonl` read/write
   paths are deleted (`src/menu_syntax/templates.rs` target mapping, capture
   payload routing, `src/brain/indexer.rs` `sync_capture_stores` todo branch).
3. **Day pages indexed** — `src/brain/indexer.rs` adds `days/` and
   `fragments/` sources; the full brain index is rebuildable from files alone.

## Risks

- **Concurrent edits:** app + external editor writing the same file.
  Mitigation: hash-guarded last-write-wins, conflict copies
  (`<name>.conflict-<ts>.md`) rather than silent loss.
- **Performance:** notes list and search currently hit sqlite. Mitigation:
  FTS/index stays in sqlite (derived); files are only read on open/edit and at
  index time.
- **Sync tools:** users will point iCloud/Dropbox/git at `~/.scriptkit/brain/`.
  That is a feature, but the watcher must tolerate partial writes and atomic
  renames from sync clients.

## Verification

- Rebuild contract: index rebuilt from files alone reproduces search results,
  tags, pins, and backlinks (golden-set comparison).
- Behavior tests on the day-page append API (ordering, fragment threshold,
  excerpt generation, atomic writes).
- No source-audit tests for this work unless a genuinely load-bearing
  invariant emerges (per `CLAUDE.md` enforcement ladder).
