# Notes Window Codebase Analysis

## Scope
This review focuses on the Notes window implementation in `src/notes/` and the key integration points in `src/` that open, manage, and style the window.

## File Map (src/notes)
- `src/notes/mod.rs` - Module entry point with feature overview and re-exports.
- `src/notes/window.rs` - Primary Notes window implementation (NotesApp, UI, input handling, autosave, autosizing, open/close/toggle, window configuration).
- `src/notes/storage.rs` - SQLite persistence with WAL, FTS5 search, triggers, and CRUD helpers.
- `src/notes/model.rs` - Core data model (`Note`, `NoteId`, `ExportFormat`) and title/preview helpers.
- `src/notes/actions_panel.rs` - Legacy modal actions panel (Cmd+K) with search/filtering and keyboard navigation.
- `src/notes/browse_panel.rs` - Legacy browse panel (Cmd+P) with search, list UI, and note actions (pin/delete).

## Integration Points in src/
- `src/main.rs` - Handles `openNotes` stdin command, notes hotkey triggers, and notes deeplinks (`scriptkit://notes/{id}`).
- `src/app_execute.rs` - Executes built-in Notes commands, hides main window, and opens Notes or quick capture.
- `src/hotkeys.rs` and `src/hotkey_pollers.rs` - Notes hotkey registration and dispatch channel.
- `src/actions/builders.rs` - Builds Notes command bar actions and note switcher actions.
- `src/actions/command_bar.rs` - Notes-specific CommandBar config (`notes_style`).
- `src/window_state.rs` - Persists Notes window bounds (including per-display positions).
- `src/builtins.rs` - Notes built-in entries and command types.
- `src/tray.rs` - Menu bar item for opening Notes.

## Architecture Overview
- **Separate window**: Notes runs in its own GPUI window (`WindowKind::PopUp`) with a `Root` wrapper and gpui-component UI. The window is toggled via a global handle (`NOTES_WINDOW`) and can be opened from hotkeys, built-in commands, tray, or stdin commands.
- **Window configuration**:
  - Positioned at the top-right of the display under the mouse cursor (fallback to stored bounds via `window_state`).
  - Uses vibrancy when enabled (`WindowBackgroundAppearance::Blurred`) and applies theme-based backgrounds.
  - Configured as a floating panel on macOS (NSFloatingWindowLevel, no Cmd+Tab, optional MoveToActiveSpace).
  - Persists bounds on close and debounces bound saves.
- **UI composition**:
  - Raycast-style single-note view (no sidebar). Titlebar and footer appear on hover.
  - Editor is a gpui-component `Input` in multi-line mode.
  - CommandBar is used for both actions (Cmd+K) and note switcher (Cmd+P), opening a separate actions window.
  - Legacy overlays (`NotesActionsPanel`, `BrowsePanel`) remain for backwards compatibility but are not the primary path.
- **State and data flow**:
  - Notes are loaded from storage on window creation and cached in `NotesApp`.
  - Editor changes update the in-memory note, set a dirty flag, and save with a debounce.
  - Auto-create note when typing with no selection to avoid data loss.
- **Storage**:
  - SQLite database at `~/.scriptkit/db/notes.sqlite`.
  - Notes table with metadata (timestamps, deleted_at, is_pinned, sort_order).
  - FTS5 virtual table with triggers for insert/update/delete, plus a LIKE fallback for special characters.

## Current Feature Inventory (confirmed in code)
- **Window lifecycle**
  - Toggle open/close via `open_notes_window` (closes if already open).
  - Opens from hotkeys, tray menu, built-in commands, and stdin `openNotes` command.
  - Hides main window when Notes opens to avoid focus conflicts.
  - Persists window bounds (per display) and restores on next open.
- **Editing and autosave**
  - Multi-line editor with placeholder and focus management.
  - Auto-create new note if user types when no selection exists.
  - Debounced save (300ms) and on-close flush of unsaved changes.
- **Autosizing**
  - Window height grows and shrinks with content line count.
  - Minimum height is the initial window height; maximum is capped.
  - Auto-sizing is disabled when a manual resize is detected; re-enabled via an action.
- **Notes management**
  - Create new note, duplicate note.
  - Soft delete (trash), restore, and permanently delete.
  - Pinned notes are supported in data model and storage, and pin toggling is implemented in the legacy browse panel.
- **Command bar actions (Cmd+K)**
  - New note, duplicate note, browse notes, find in note, copy note as, copy deeplink, create quicklink, export, format toolbar toggle, and enable auto-sizing.
- **Note switcher (Cmd+P)**
  - Lists notes with character count and a current-note indicator.
  - Provides search filtering via CommandBar.
  - If no notes exist, shows a placeholder action that creates a new note.
- **Export and clipboard**
  - Export to plain text, markdown, or HTML (clipboard on macOS via `pbcopy`).
  - Copy markdown, copy deeplink, and copy quicklink helpers.
- **Deep linking**
  - `scriptkit://notes/{id}` opens the Notes window (note selection is not yet implemented).

## Legacy and Partial Implementations
- `BrowsePanel` and `NotesActionsPanel` are still present, but the primary UI path now uses `CommandBar` windows. Legacy browse panel includes pin/delete actions on hover; the CommandBar note switcher does not expose pin/delete.
- `render_search` and `search_state` exist but are not wired into the main render tree; FTS search is implemented but not currently reachable from the primary UI.
- Formatting insertion is stubbed: `insert_formatting` builds a formatted string but does not apply it to the editor content at the cursor.
- `quick_capture` currently opens Notes but does not create a new note (TODO noted in code).
- Module docs mention markdown preview and menu bar integration; the codebase provides a tray menu entry to open Notes but does not show a markdown preview in the Notes window.

## Summary
The Notes window is a standalone GPUI PopUp window with gpui-component UI, backed by SQLite with FTS5. It favors a Raycast-style single-note view with hover-only chrome and CommandBar-driven actions/switcher. The core features (create, edit, autosave, autosize, trash/restore, export, and deep link opening) are implemented, while search UI, formatting insertion, pin management via the new switcher, and quick capture creation remain partial or legacy.
