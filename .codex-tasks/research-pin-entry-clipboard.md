# Clipboard History Pin Entry - Research Notes

## 1) Files investigated
- `src/clipboard_history/database.rs:72-79,178-180,350-366,419-443,587-633`
  - History table schema includes `pinned` column, index on `(pinned DESC, timestamp DESC)`, prune excludes pinned, queries order pinned first, and pin/unpin helpers update cache.
- `src/clipboard_history/types.rs:71-107`
  - `ClipboardEntry` and `ClipboardEntryMeta` include `pinned: bool`.
- `src/render_builtins.rs:185-294,364-372,562-572`
  - Clipboard history key handler and UI rendering with pin emoji and "Pinned" badge.
- `src/actions/builders.rs:804-820,929-949`
  - Clipboard history actions include `clipboard_pin` / `clipboard_unpin` IDs (with shortcut labels).
- `src/actions/dialog.rs:311-363`
  - `ActionsDialog::with_clipboard_entry(...)` constructor for clipboard-specific actions.
- `src/execute_script.rs:416-486`
  - Clipboard history protocol actions include `Pin` and `Unpin` (wired to `clipboard_history::pin_entry` / `unpin_entry`).
- `src/app_impl.rs:3317-3386`
  - Actions dialog toggle uses `ActionsDialog::with_script(...)` (script list only).

## 2) Current behavior
- Pinned state is persisted in the clipboard history DB (`pinned INTEGER DEFAULT 0`) and preserved during retention pruning (pinned entries are excluded from delete). `src/clipboard_history/database.rs:72-79,350-366`
- History list queries already sort with pinned entries first (`ORDER BY pinned DESC, timestamp DESC`) and there is a supporting index. `src/clipboard_history/database.rs:178-180,419-443`
- Entry models expose `pinned: bool` for both full and metadata records. `src/clipboard_history/types.rs:71-107`
- UI already renders pin indicators: a leading pin emoji in list rows and a "Pinned" badge in the preview panel. `src/render_builtins.rs:364-372,562-572`
- Clipboard actions include `clipboard_pin` / `clipboard_unpin` action IDs (currently labeled with the Shift+Cmd+P shortcut). `src/actions/builders.rs:929-949`
- Protocol handling for `ClipboardHistoryAction::Pin` and `::Unpin` is implemented and routes to the DB helpers. `src/execute_script.rs:416-486`

## 3) Root cause analysis
- The clipboard history view key handler does not include any Cmd+K path to open the actions dialog, and only handles Escape/Cmd+W/arrow keys/Enter. `src/render_builtins.rs:185-294`
- The clipboard-specific actions dialog constructor exists (`ActionsDialog::with_clipboard_entry`), but there are no call sites outside its definition. `src/actions/dialog.rs:311-363`
- The actions dialog toggle code currently builds dialogs only for scripts (`ActionsDialog::with_script`), so clipboard history has no path to open actions from its view. `src/app_impl.rs:3317-3386`
- There is no direct keyboard shortcut in the clipboard history view to toggle pin status (no Cmd+P handling in the view key handler). `src/render_builtins.rs:185-294`

## 4) Proposed solution approach
- Wire Cmd+K in the clipboard history key handler to open a clipboard actions dialog for the selected entry using `ActionsDialog::with_clipboard_entry(...)`. `src/render_builtins.rs:185-294`, `src/actions/dialog.rs:311-363`
  - Follow the existing actions dialog open/close pattern in `src/app_impl.rs:3317-3386` (focus transfer, `show_actions_popup`, `open_actions_window`, `on_close` callback), but pass a `ClipboardEntryInfo` for the selected entry.
- Add a dedicated clipboard actions route/handler so `clipboard_pin` / `clipboard_unpin` action IDs update the DB via `clipboard_history::pin_entry` / `unpin_entry`, then refresh or update the cached list and re-render. `src/actions/builders.rs:929-949`, `src/clipboard_history/database.rs:587-633`
  - Ensure the list re-sorts after toggling (pinned-first ordering is already enforced by the query/index). `src/clipboard_history/database.rs:178-180,419-443`
- Add a direct Cmd+P shortcut in the clipboard history key handler to toggle the selected entry's pin state without opening the actions dialog. `src/render_builtins.rs:185-294`
  - After toggling, call `cx.notify()` and ensure cache updates so the pin emoji/badge update immediately. `src/render_builtins.rs:364-372,562-572`

## Verification
### What changed
- `src/clipboard_history/types.rs`: Added `Copy` trait to `ContentType` enum.
- `src/render_builtins.rs`: Added `toggle_clipboard_actions()` function, Cmd+P shortcut for pin toggle, Cmd+K shortcut for actions dialog, and actions dialog routing.
- `src/app_actions.rs`: Added `clipboard_pin` / `clipboard_unpin` handlers that call `clipboard_history::pin_entry` / `unpin_entry` and refresh cache.
- `src/app_impl.rs`: Added `focused_clipboard_entry_id` field.
- `src/main.rs`: Added `ClipboardHistory` variant to `ActionsDialogHost` enum and `clipboard_actions_tests` module.

### Test results
- `cargo test clipboard` passed (59 tests).
- `cargo check` failed due to pre-existing errors unrelated to this feature (`core_foundation_sys` import issues in `src/open_with.rs`).

### Before / after
- BEFORE: No way to pin/unpin entries from the clipboard history UI.
- AFTER: Cmd+P toggles pin/unpin; Cmd+K opens the actions dialog with pin/unpin option.

### Deviations from proposed solution
- None. Implemented as proposed.
