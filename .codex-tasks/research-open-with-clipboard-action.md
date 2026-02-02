# Research: clipboard_open_with action

## 1) Files investigated

### Clipboard history module structure
- `src/clipboard_history/mod.rs` - module map + public exports for clipboard history (types, config, cache, database, image, monitor, clipboard ops).
- `src/clipboard_history/monitor.rs` - background clipboard polling + change detection + DB insertion flow.
- `src/clipboard_history/database.rs` - SQLite CRUD; `get_entry_content()` for fetching full payload by entry id.
- `src/clipboard_history/types.rs` - `ContentType`, `ClipboardEntry`, `ClipboardEntryMeta` (list metadata vs full content).
- `src/clipboard_history/blob_store.rs` - image blob files stored at `~/.scriptkit/clipboard/blobs/<hash>.png`.
- `src/clipboard_history/image.rs` - encode/decode helpers; supports `blob:`, `png:`, `rgba:` formats.
- `src/clipboard_history/clipboard.rs` - `copy_entry_to_clipboard()` (text/image write-back to system clipboard).

### Actions system
- `src/actions/builders.rs` - `get_clipboard_history_context_actions()` builds clipboard action list; includes `clipboard_open_with` for image entries on macOS.
- `src/actions/dialog.rs` - `ActionsDialog::with_clipboard_entry()` wires clipboard entry actions into the actions dialog.
- `src/actions/types.rs` - actions routing architecture doc; actions are executed through `handle_action()` in main app.
- `src/app_actions.rs` - `handle_action()` implementation (has `open_with` for file search, but no clipboard handlers).
- `src/file_search.rs` - `open_with(path)` (macOS AppleScript "Get Info"/Open With UI for file search entries).
- `src/render_builtins.rs` - clipboard history UI + key handling (Enter copies + pastes; no actions dialog trigger here).

### Temp file patterns
- `Cargo.toml` - `tempfile = "3"` in main dependencies (available for runtime temp file creation).
- `src/config/loader.rs` - `tempfile::NamedTempFile` used for secure temp JS output during config load.
- `src/clipboard_history/blob_store.rs` - persistent image file storage pattern (PNG blobs on disk).

## 2) Current behavior (clipboard history + actions)

### Clipboard history data flow
- Startup enables clipboard history via `clipboard_history::init_clipboard_history()` in `src/main.rs`. The monitor thread uses `ClipboardChangeDetector` to detect changes, reads text/image payloads, and writes to SQLite (`monitor.rs`).
- Images are stored as blob files on disk (`~/.scriptkit/clipboard/blobs/<hash>.png`) and referenced in DB as `blob:<hash>` (`blob_store.rs`, `image.rs`).
- UI lists are populated from cached metadata (`clipboard_history::get_cached_entries()`), and preview panel fetches full content using `clipboard_history::get_entry_content()` when needed (`render_builtins.rs`, `database.rs`).
- Built-in entry for Clipboard History loads cached entries and switches view state in `src/app_execute.rs` (`BuiltInFeature::ClipboardHistory` branch).

### Clipboard history UI behavior
- In `render_clipboard_history()` the Enter key copies the selected entry to the system clipboard via `clipboard_history::copy_entry_to_clipboard()` and simulates paste (Cmd+V), then hides the main window (`render_builtins.rs`).
- There is no direct "Open With..." UI hook in clipboard history rendering/keyboard handlers (no Cmd+K or per-entry action invocation here).

### Clipboard actions that exist today
- `get_clipboard_history_context_actions()` includes:
  - `clipboard_paste`, `clipboard_copy`, `clipboard_paste_keep_open`, `clipboard_share`, `clipboard_attach_to_ai`, pin/unpin, delete, etc.
  - Image-only actions include `clipboard_open_with`, `clipboard_quick_look`, CleanShot actions (macOS-gated) (`actions/builders.rs`).
- Actions dialog has a constructor specifically for clipboard entries: `ActionsDialog::with_clipboard_entry()` (`actions/dialog.rs`).

### Missing execution handler
- `handle_action()` in `src/app_actions.rs` contains handlers for file search (`open_with`, `quick_look`, etc.), but has **no match arm** for `clipboard_open_with` (or other clipboard-specific action IDs).
- Result: even if `clipboard_open_with` is surfaced by the actions dialog, there is no handler to execute it, so it's effectively a no-op.

## 3) Root cause analysis

**Root cause:** `clipboard_open_with` is defined in the clipboard action builder but there is no execution path that handles it.

Evidence:
- Action is created in `get_clipboard_history_context_actions()` with id `clipboard_open_with` for image entries on macOS (`src/actions/builders.rs`).
- Actions dialog supports clipboard entries (`ActionsDialog::with_clipboard_entry()` in `src/actions/dialog.rs`).
- `handle_action()` in `src/app_actions.rs` only handles `open_with` (file search) and has no branch for `clipboard_open_with` or any clipboard action IDs.
- Clipboard history view currently handles Enter directly (copy + paste) in `render_clipboard_history()` and doesn't route clipboard actions through `handle_action()` (`src/render_builtins.rs`).

## 4) Proposed solution approach

### A) Save clipboard content to a temp file
- Use `clipboard_history::get_entry_content(id)` to fetch payload (`src/clipboard_history/database.rs`).
- For text: write to a `.txt` file (UTF-8). For image: resolve `blob:` content (preferred) via `blob_store::load_blob()` or decode `png:`/`rgba:` formats from `clipboard_history/image.rs`, then write a `.png`.
- Follow existing secure temp file pattern with `tempfile::NamedTempFile` (see `src/config/loader.rs`), or a deterministic temp dir under `~/.scriptkit/clipboard/tmp/` if persistence is desired.

### B) Get available apps for "Open With..."
- Implement a macOS Launch Services FFI helper (e.g., `LSCopyApplicationURLsForURL` / `LSCopyAllRoleHandlersForContentType`) to resolve candidate apps for the temp file's UTI.
- There is existing app discovery infrastructure (`src/app_launcher.rs`) but it scans directories; Launch Services is a better fit for per-file "Open With".

### C) Wire `clipboard_open_with` into action execution
- Add a handler in `handle_action()` (`src/app_actions.rs`) for `clipboard_open_with`.
- The handler should:
  1) Resolve selected clipboard entry id (from current selection in ClipboardHistory view).
  2) Save payload to a temp file (step A).
  3) Use Launch Services (step B) to show/select an app and open the temp file.

### D) Launch the chosen app with the temp file
- Existing file-search action uses `file_search::open_with(path)` which triggers Finder "Open With..." UI (`src/file_search.rs`); can be reused once a temp file path exists.
- Alternatively, use `open -a "AppName" <tempfile>` or NSWorkspace APIs directly (pattern already used in `app_launcher.rs` for `open -a`).

---

## Notes / cross-refs
- Existing clipboard history actions research: `.codex-tasks/research-clipboard-history-actions.md` (related context).
- Related UI actions: `.codex-tasks/research-quick-look-clipboard.md`, `.codex-tasks/research-share-sheet.md`.

---

## Verification

### What was changed

1. **`src/clipboard_history/temp_file.rs`** (new file)
   - Implements `save_entry_to_temp_file(entry: &ClipboardEntry) -> Result<PathBuf>`
   - Handles text content: saves as `.txt` file with UTF-8 encoding
   - Handles image content: decodes blob/base64 PNG and saves as `.png` file
   - Uses `tempfile::Builder` for secure temp file creation
   - Files are persisted (not auto-deleted) so external apps can open them

2. **`src/clipboard_history/open_with.rs`** (new file)
   - Implements macOS Launch Services FFI via `core_foundation` crate
   - `AppInfo` struct with name, bundle_id, and app_path
   - `get_apps_for_file(path: &Path) -> Vec<AppInfo>` - uses `LSCopyApplicationURLsForURL` to get apps
   - `open_file_with_app(file_path, app_path)` - uses `open -a` command
   - Non-macOS stubs return empty results / error

3. **`src/app_actions.rs`** - Added `clipboard_open_with` handler (around line 918)
   - Gets selected clipboard entry
   - Loads full content via `clipboard_history::get_entry_content()`
   - Saves to temp file via `clipboard_history::save_entry_to_temp_file()`
   - Opens macOS Finder "Open With" dialog via `crate::file_search::open_with()`
   - Shows HUD error messages on failure

4. **`src/clipboard_history/mod.rs`** - Updated exports
   - Added `mod open_with;`, `mod temp_file;`, `mod quick_look;`
   - Re-exports: `save_entry_to_temp_file`, `get_apps_for_file`, `open_file_with_app`, `AppInfo`

### Test results

- `cargo check`: **passed** (no errors)
- `cargo clippy --all-targets -- -D warnings`: **passed** (no warnings/errors)
- `cargo test`: **2557 passed, 0 failed, 34 ignored**

### Before/after comparison

| Aspect | Before | After |
|--------|--------|-------|
| Action definition | `clipboard_open_with` existed in `get_clipboard_history_context_actions()` | Same |
| Handler | No match arm in `handle_action()` - was a no-op | Full handler at line ~918 |
| Temp file support | None | `save_entry_to_temp_file()` for text/image |
| macOS app discovery | None | `get_apps_for_file()` via Launch Services |
| User experience | Action did nothing | Opens Finder's "Open With" dialog |

### Deviations from proposed solution

1. **App picker approach**: Used existing `file_search::open_with()` pattern which triggers Finder's built-in "Open With" UI instead of building a custom app picker dialog. This is simpler and provides native macOS experience.

2. **Launch Services APIs implemented for future use**: The `get_apps_for_file()` and `open_file_with_app()` functions are fully implemented but currently unused. They're available for a future custom app picker if needed.

3. **Added `#![allow(dead_code)]`**: To suppress warnings for the currently-unused Launch Services functions.
