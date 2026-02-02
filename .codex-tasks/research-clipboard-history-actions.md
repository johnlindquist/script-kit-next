# Research: Clipboard History Actions + Clipboard Operations

## 1) Files investigated
- `src/main.rs:1950-1971`
  - Startup path: config load, `clipboard_history::set_max_text_content_len`, `clipboard_history::init_clipboard_history()`.
- `src/clipboard_history/monitor.rs:38-190`
  - Background clipboard polling, change detection, cache prewarm, pruning.
- `src/clipboard_history/clipboard.rs:15-74`
  - `copy_entry_to_clipboard()` reads DB, sets clipboard (text/image), updates timestamp, refreshes cache.
- `src/app_execute.rs:103-133`
  - Built-in Clipboard History entry: loads cached entries, sets `AppView::ClipboardHistoryView`.
- `src/render_builtins.rs:172-294`
  - Clipboard History view render, filtering and key handler; Enter copies entry, hides window, simulates paste.
- `src/selected_text.rs:159-259`
  - `set_selected_text()` (clipboard save/restore + Cmd+V), `simulate_paste_with_cg()` for paste.
- `src/executor/scriptlet.rs:602-616`
  - Scriptlet `paste` tool uses `selected_text::set_selected_text()`.
- `src/actions/types.rs:1-374`
  - Core action types, `has_action` routing, ID conventions.
- `src/actions/dialog.rs:311-363,600-648,1065-1101`
  - `ActionsDialog::with_clipboard_entry`, SDK action conversion, close behavior.
- `src/actions/builders.rs:48-64,806-861`
  - `ClipboardEntryInfo` and clipboard action IDs (`clipboard_copy`, `clipboard_paste_keep_open`).
- `src/app_impl.rs:3723-3861`
  - `route_key_to_actions_dialog` routing of key events and action execution.
- `src/app_actions.rs:49-1142`
  - `handle_action()` built-in action routing; SDK action trigger helper.

## 2) Current clipboard operations (history + paste mechanisms)

### Clipboard history data flow
- Startup initializes clipboard history monitoring, sets max text length, and starts polling. `src/main.rs:1950-1971` -> `src/clipboard_history/monitor.rs:38-190`.
- Clipboard monitor:
  - Uses `ClipboardChangeDetector` for change count (fast poll). `src/clipboard_history/monitor.rs:122-190`.
  - Reads clipboard payload only after change detection, hashes content, and writes to SQLite (via `add_entry`). `src/clipboard_history/monitor.rs:193-220`.
- Cache:
  - `refresh_entry_cache()` pre-warms metadata list. `src/clipboard_history/monitor.rs:83-85`.
  - `copy_entry_to_clipboard()` updates timestamp and refreshes cache after copy. `src/clipboard_history/clipboard.rs:58-73`.

### Clipboard history UI flow
- Built-in action `ClipboardHistory` opens the view and pulls metadata from cache:
  - `self.cached_clipboard_entries = clipboard_history::get_cached_entries(100);` and sets `AppView::ClipboardHistoryView`. `src/app_execute.rs:103-133`.
- UI rendering:
  - `render_clipboard_history()` filters `cached_clipboard_entries` by `filter`. `src/render_builtins.rs:172-183`.
  - List items show text preview or image thumbnail; selection tracked by `selected_index`.

### Paste mechanisms
- Clipboard History "Enter" behavior:
  - On Enter, view copies selected entry with `clipboard_history::copy_entry_to_clipboard(&entry.id)`.
  - Hides main window + sets `NEEDS_RESET`, then spawns a thread to `selected_text::simulate_paste_with_cg()` (Cmd+V). `src/render_builtins.rs:257-287`.
- Scriptlet "paste" tool:
  - `execute_paste` uses `selected_text::set_selected_text(text)` which saves/restores clipboard and simulates Cmd+V. `src/executor/scriptlet.rs:602-616`, `src/selected_text.rs:159-229`.
- Shared paste primitive:
  - `selected_text::simulate_paste_with_cg()` posts Cmd+V via Core Graphics. `src/selected_text.rs:232-259`.

## 3) Action system overview (types + dialog)

### Types and routing rules
- `Action` and `ScriptInfo` are defined in `src/actions/types.rs` with action ID conventions and `has_action` routing.
  - Built-in action IDs are snake_case; SDK actions keep their protocol names. `src/actions/types.rs:19-35,284-304`.
  - `has_action=true` -> send `ActionTriggered` to SDK; `has_action=false` -> handled locally. `src/actions/types.rs:29-35,296-301`.

### ActionsDialog behavior
- `ActionsDialog::with_clipboard_entry(...)` exists and builds clipboard actions from `get_clipboard_history_context_actions`. `src/actions/dialog.rs:311-363`.
- `set_sdk_actions()` converts `ProtocolAction` into `Action` and controls routing via `has_action`. `src/actions/dialog.rs:600-648`.
- Close behavior:
  - `selected_action_should_close()` returns `true` for built-ins; SDK actions can override via `close`. `src/actions/dialog.rs:1092-1101`.

### Key routing and execution
- `route_key_to_actions_dialog()` in `app_impl` routes Up/Down/Enter/typing to the actions dialog and returns `ActionsRoute::Execute` when an action is selected. `src/app_impl.rs:3723-3861`.
- Built-in actions are handled by `handle_action()` (which resets view to ScriptList). `src/app_actions.rs:49-56`.
- SDK actions are sent via `trigger_sdk_action_internal()` from `handle_action()`'s default case. `src/app_actions.rs:1096-1142`.

### Clipboard actions list (defined but not wired)
- Clipboard action IDs are already defined:
  - `clipboard_copy` and `clipboard_paste_keep_open` with shortcuts `cmd+enter` and `alt+enter`. `src/actions/builders.rs:841-861`.
- **Gap:** no call site currently opens `ActionsDialog::with_clipboard_entry` from the clipboard history view, and `handle_action()` doesn't handle clipboard_* IDs.

## 4) How to implement "Copy to Clipboard" action

**Goal:** wire `clipboard_copy` to copy the selected clipboard entry without pasting.

### Where to implement
1. **Open actions dialog for clipboard entries** (optional but aligns with current action system):
   - In the Clipboard History key handler (`src/render_builtins.rs:185-294`), add Cmd+K to open actions.
   - Build a `ClipboardEntryInfo` from the selected entry and call `ActionsDialog::with_clipboard_entry(...)`. `src/actions/dialog.rs:311-363`, `src/actions/builders.rs:48-64`.
2. **Add a clipboard-specific action handler** (recommended vs reusing `handle_action()`):
   - `handle_action()` currently **resets to ScriptList** (`self.current_view = AppView::ScriptList`). `src/app_actions.rs:49-56`.
   - Clipboard actions should likely keep the Clipboard History view open or close only the actions dialog, so implement a new `execute_clipboard_action(action_id, entry_id, cx)` in `app_impl.rs` (similar to `execute_path_action`).

### Execution logic
- Obtain selected entry ID:
  - Reuse filter logic from `render_clipboard_history` to map `selected_index` -> entry. `src/render_builtins.rs:172-235`.
- Copy to clipboard:
  - Call `clipboard_history::copy_entry_to_clipboard(&entry.id)`. This handles both text and images and updates timestamps + cache. `src/clipboard_history/clipboard.rs:23-73`.
- UI feedback:
  - Optionally show HUD (`self.show_hud(...)`), and **do not** call `hide_main_and_reset()`.

### Summary of changes
- Add action handler for `clipboard_copy` (new function or branch), use existing clipboard API.
- If using ActionsDialog, route `ActionsRoute::Execute { action_id }` to your clipboard handler instead of `handle_action()` when the host is Clipboard History.

## 5) How to implement "Paste and Keep Window Open" action

**Goal:** paste to the active app but reopen Clipboard History afterwards (don't reset to ScriptList).

### Required behavior
- Same as Enter in Clipboard History, but **avoid setting `NEEDS_RESET`** so the view persists.
  - Current Enter path sets `NEEDS_RESET.store(true, ...)` which forces a reset on next show. `src/render_builtins.rs:271-274`.

### Implementation steps
1. **Copy entry to clipboard:**
   - `clipboard_history::copy_entry_to_clipboard(&entry.id)`. `src/clipboard_history/clipboard.rs:23-73`.
2. **Hide main window without reset:**
   - Use `script_kit_gpui::set_main_window_visible(false)` + `platform::hide_main_window()`.
   - **Do NOT** set `NEEDS_RESET` for this action.
3. **Paste via Cmd+V:**
   - Spawn a short-lived thread and call `selected_text::simulate_paste_with_cg()`. `src/selected_text.rs:232-259`.
4. **Re-open the window after paste:**
   - Call `script_kit_gpui::set_main_window_visible(true)` + `script_kit_gpui::request_show_main_window()` (same pattern as `prompt_handler`). `src/lib.rs:283-303`.
   - This will show the main window without resetting view state because `NEEDS_RESET` stays `false`.

### Notes
- If you route this via ActionsDialog, you'll want a new `ActionsDialogHost::ClipboardHistory` variant for correct focus restoration, or you can handle shortcuts directly in the clipboard history key handler.
- The existing "Paste" action title already uses `frontmost_app_name` for display; no code changes required for the label. `src/actions/builders.rs:825-835`.

## Verification (2026-02-01)

### What was implemented

1. **`clear_unpinned_history` function** (`src/clipboard_history/database.rs`)
   - New function that deletes all unpinned clipboard entries while preserving pinned ones
   - Properly handles blob file cleanup and cache clearing
   - Exported from `src/clipboard_history/mod.rs`

2. **Clipboard action handlers** (`src/app_actions.rs`)
   - `clipboard_delete`: Deletes single selected entry using `remove_entry`
   - `clipboard_delete_all`: Deletes all unpinned entries using `clear_unpinned_history`
   - `clipboard_save_file`: Saves clipboard content to Desktop with timestamp
   - `clipboard_save_snippet`: Creates snippet file in `~/.kenv/extensions/clipboard-snippets.md`

### Test results
- `cargo check`: Passed
- `cargo clippy --all-targets -- -D warnings`: Passed
- `cargo test`: 16 passed, 0 failed, 58 ignored

### Files changed
- `src/clipboard_history/database.rs`: Added `clear_unpinned_history()` function
- `src/clipboard_history/mod.rs`: Exported `clear_unpinned_history`
- `src/app_actions.rs`: Added handlers for clipboard_delete, clipboard_delete_all, clipboard_save_file, clipboard_save_snippet

### Notes
- `clipboard_delete_multiple` action is defined in builders but not yet implemented (requires multi-select UI)
- Confirmation dialog for delete_all was simplified to direct action (complex confirm callback pattern requires channel-based communication)
- File save uses Desktop directory with timestamp naming (PathPrompt integration deferred for simplicity)
- Snippet save generates unique keyword with timestamp suffix and handles code fences properly

## Verification

### What was changed
Two new clipboard history action handlers were implemented in `src/app_actions.rs`:

1. **clipboard_copy** (Cmd+Enter shortcut):
   - Copies the selected clipboard entry to system clipboard without pasting
   - Shows HUD "Copied to clipboard"
   - Keeps the clipboard history window open

2. **clipboard_paste_keep_open** (Opt+Enter shortcut):
   - Copies the selected clipboard entry to system clipboard
   - Simulates Cmd+V paste after a 50ms delay
   - Shows HUD "Pasted"
   - Keeps the clipboard history window open (does not hide main window)

### Files modified
- `src/app_actions.rs`: Added handlers for `clipboard_copy` and `clipboard_paste_keep_open`
- `src/render_builtins.rs`: Added Ctrl+Cmd+A handler for `clipboard_attach_to_ai`
- `src/clipboard_actions_tests.rs`: Updated tests to cover new functionality

### Test results
```
cargo test --bin script-kit-gpui clipboard_actions
running 5 tests
test clipboard_actions_tests::test_app_actions_handles_clipboard_attach_to_ai ... ok
test clipboard_actions_tests::test_app_actions_handles_clipboard_copy_and_paste_keep_open ... ok
test clipboard_actions_tests::test_app_actions_handles_clipboard_delete ... ok
test clipboard_actions_tests::test_app_actions_handles_clipboard_pin_unpin ... ok
test clipboard_actions_tests::test_render_builtins_has_clipboard_pin_shortcut_and_actions_toggle ... ok
test result: ok. 5 passed; 0 failed
```

### Verification commands run
1. `cargo check` - Passed
2. `cargo clippy --all-targets -- -D warnings` - Passed
3. `cargo test --bin script-kit-gpui clipboard_actions` - 5 tests passed

### Implementation notes
- The actions use `clipboard_history::copy_entry_to_clipboard()` to copy entries
- For paste simulation, `selected_text::simulate_paste_with_cg()` is used
- Window visibility is controlled by NOT calling `hide_main_and_reset()` for keep-open behavior
- The `selected_clipboard_entry()` helper function retrieves the currently selected entry based on view state
