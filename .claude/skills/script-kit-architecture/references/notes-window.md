# Notes Window

Separate floating window. gpui-component + SQLite `~/.scriptkit/db/notes.sqlite`. Theme synced from `~/.scriptkit/theme.json`.

## Files

- `window.rs` - NotesApp view + open/close + quick_capture
- `storage.rs` - SQLite persistence + FTS5 search + delete/restore
- `model.rs` - `NoteId`, `Note`, `ExportFormat`

## Features

- Markdown toolbar
- Sidebar list + search + trash
- FTS5 full-text search
- Soft delete/trash restore
- Export (copies to clipboard)
- Character count footer
- Hover icons

## Root Wrapper (Required)

```rust
let handle = cx.open_window(opts, |w, cx| {
  let view = cx.new(|cx| NotesApp::new(w, cx));
  cx.new(|cx| Root::new(view, w, cx))
})?;
```

## Theme Mapping

Script Kit colors → gpui-component `ThemeColor` (e.g. via `hex_to_hsla(...)`).

## Testing

- stdin: `{"type":"openNotes"}`
- captureScreenshot() captures the **main** window; Notes testing is mainly log-based
- log filter: `grep -i 'notes|PANEL'`

## Open Methods

- Hotkey: `Cmd+Shift+N` (configurable `notesHotkey`)
- Tray menu
- Stdin

## Single-Instance Pattern

Global `OnceLock<Mutex<Option<WindowHandle<Root>>>>`.
