# Notes Window: New Note + Switcher Plan

## Goals
- Single persistent notes window for all note interactions.
- Creating a new note always loads in the existing notes window.
- Switching notes replaces the current note content in the same window.
- Command+P opens a note switcher with search and filtering.

## Constraints
- Only one note is open at a time.
- Notes window uses the same global keyboard shortcut each time.
- Creating a new note should feel like clearing the editor into a blank slate.

## Existing Code Map (Relevant Files)
- `src/notes/window.rs` — NotesApp state, editor/input wiring, create/select note, actions/browse panel toggles, singleton window handle.
- `src/notes/actions_panel.rs` — Cmd+K actions list (New Note, Browse Notes, etc.) and keycap display.
- `src/notes/browse_panel.rs` — Cmd+P overlay UI with search input, list rendering, selection callbacks.
- `src/notes/storage.rs` — SQLite persistence + FTS search for notes.
- `src/notes/model.rs` — Note/NoteId types, title extraction, preview helpers.
- `src/notes/mod.rs` — module exports and public API.
- `src/main.rs` — notes hotkey listener and tray commands that open Notes / New Note.
- `src/hotkeys.rs` — registration of global Notes hotkey.
- `src/hotkey_pollers.rs` — notes hotkey poller (alternate path).
- `tests/smoke/test-notes-actions-panel.ts` — actions panel list/shortcuts.
- `tests/smoke/test-notes-browse-panel.ts` — browse panel presence and behavior.
- `tests/smoke/test-notes-hotkey.ts` — notes hotkey opens notes window.
- `tests/smoke/test-notes-single-view.ts` — single-note view behavior.

## Libraries / Dependencies
- `gpui` — windowing, input events, rendering primitives.
- `gpui_component` — Input, Button, themes, overlay panels.
- `rusqlite` + SQLite FTS5 — storage and full-text search.
- `chrono` — timestamps.
- `uuid` — note identifiers.
- `serde` — data serialization.

## Current Behavior (from code)
- Notes window is tracked as a singleton via `NOTES_WINDOW` and toggled by `open_notes_window` in `src/notes/window.rs`.
- Notes are persisted on editor changes via `on_editor_change` in `src/notes/window.rs` using `storage::save_note`.
- `NotesAction::NewNote` uses `create_note` -> `storage::save_note` -> `select_note` in `src/notes/window.rs`.
- Cmd+P toggles `show_browse_panel` in `NotesApp::render`, and `open_browse_panel` creates a `BrowsePanel` entity in `src/notes/window.rs`.
- Browse panel search currently filters by title only in `src/notes/browse_panel.rs`.

## Target Behavior (Requirements)
- Single notes window instance only; global shortcut always focuses that window.
- New note creation always loads into the existing window and clears editor content.
- Command+P opens a switcher with search + filtering across existing notes.
- Selecting a note replaces editor content in the same window.

## User Flows (with File Touchpoints)
### New Note (Menu or Shortcut)
1. Trigger from actions menu (Cmd+K → New Note) or direct shortcut.
2. Create new note record (`Note::new`, `storage::save_note`) in `src/notes/window.rs`.
3. Select the new note (`select_note`) and reset editor content.
4. Focus editor input (`editor_state.focus`) so typing starts immediately.

### Switch Notes (Command+P)
1. User hits Cmd+P while focused in the notes window (`NotesApp::render` key handler in `src/notes/window.rs`).
2. Show `BrowsePanel` overlay (`open_browse_panel` in `src/notes/window.rs`).
3. Search input receives focus (`BrowsePanel` in `src/notes/browse_panel.rs`).
4. Filter note list based on query (title + content or FTS).
5. On selection, call back into `NotesApp::handle_browse_select` to load note.
6. Close panel and focus editor.

## Data + State Handling
- `NotesApp` state lives in `src/notes/window.rs`: `selected_note_id`, `notes`, `deleted_notes`, `editor_state`.
- New note creation uses `Note::new` in `src/notes/model.rs` and persists via `storage::save_note` in `src/notes/storage.rs`.
- Switching notes uses `select_note` in `src/notes/window.rs` to load content into `editor_state`.
- Search can use `storage::search_notes` (FTS) in `src/notes/storage.rs` or in-memory filtering in `src/notes/browse_panel.rs`.

## Window Lifecycle + Hotkeys
- Global notes hotkey registered in `src/hotkeys.rs` and listened for in `src/main.rs`.
- `open_notes_window` in `src/notes/window.rs` owns singleton lifecycle and focus behavior.
- Tray “New Note” uses `notes::quick_capture` in `src/notes/window.rs` (currently TODO).

## Switcher Details (Cmd+P)
- UI component: `BrowsePanel` in `src/notes/browse_panel.rs`.
- Data source: `NotesApp.notes` from `src/notes/window.rs` (backed by `storage::get_all_notes`).
- Sorting: use current `storage::get_all_notes` ordering (pinned first, updated desc).
- Filtering: update to search title + content preview or FTS query.
- Selection should call `NotesApp::handle_browse_select` and then focus editor.

## Edge Cases + Decisions
- Hotkey behavior: decide whether `open_notes_window` toggles closed or always focuses if open.
- New note while switcher open: close switcher first, then create note.
- Switching notes with unsaved changes: current behavior auto-saves on change (no explicit prompt).
- Switching to the same note: no-op.
- Empty DB: browse panel shows “No notes found”.

## Implementation Steps (with File Targets)
1. Confirm singleton behavior + focus strategy in `src/notes/window.rs` (`open_notes_window`).
2. Ensure in-window shortcut (Cmd+N) creates a new note in `NotesApp::render` key handler.
3. Wire browse panel callbacks in `open_browse_panel` to `handle_browse_select`, `handle_browse_action`, and close behavior.
4. Decide search strategy for Cmd+P (use `storage::search_notes` or expand `BrowsePanel` filtering).
5. Update `notes::quick_capture` in `src/notes/window.rs` to create a new note in the same window.
6. Update/extend tests for hotkeys/actions/browse panel.

## Test Plan
- `tests/smoke/test-notes-hotkey.ts` — global notes hotkey still opens/focuses the same window.
- `tests/smoke/test-notes-actions-panel.ts` — “New Note” action present and triggers note creation.
- `tests/smoke/test-notes-browse-panel.ts` — Cmd+P opens browse panel and filters results.
- `tests/smoke/test-notes-single-view.ts` — only one note visible at a time.
- Add/extend any browse panel selection tests once callbacks are wired.

## Diagrams
- Canonical DDD diagrams live in `ai/diagrams/`.
- `ai/diagrams/features/feature-notes-window-new-note-switcher.md` — single-window new note + switcher flow.
- `ai/diagrams/journeys/sequence-notes-new-note-and-switcher.md` — step-by-step journey for creation and switching.
