# Actions UX Audit (Built-ins + Actions Menu)

Date: 2026-02-06
Agent: `codex-actions-audit`

## Scope
- `src/builtins.rs` (`BuiltInFeature`, `get_builtin_entries()`)
- `src/app_execute.rs` (`execute_builtin()`, confirmation flow, command handlers)
- `src/app_actions.rs` (`handle_action()` for actions menu IDs)
- `src/app_impl.rs` (`execute_path_action()`, `execute_chat_action()`, `execute_webcam_action()`, popup routing)
- `src/actions/builders.rs`, `src/actions/dialog.rs`, `src/render_builtins.rs`, `src/render_prompts/*.rs`, `src/prompts/path.rs`, `src/prompts/chat.rs`

## High-Impact Gaps
1. `Webcam -> Capture` does not capture/save anything and provides no feedback; it only closes the window (`src/app_impl.rs:4195`).
2. Multiple destructive actions have no confirmation in actions menu paths (`clipboard_delete_all`, `clipboard_delete_multiple`, `move_to_trash`).
3. Several actions execute OS commands but ignore errors and show no success/failure HUD (`open_file`, `quick_look`, `open_with`, `show_info` in file-search actions path).
4. Confirmation-modal failure currently auto-executes dangerous built-ins (`src/app_execute.rs:88-91`).
5. Label inconsistency for same intent: `Reveal in Finder` vs `Show in Finder` vs `Open in Finder` across contexts.
6. Clipboard pin/unpin succeeds silently (no HUD/toast), reducing confidence that state changed.
7. Path actions mostly write `last_output` only (often invisible in-context) instead of HUD/toasts.
8. `ActionsDialog` rows render title+shortcut only; `Action.description` is not surfaced in row UI, so users lose disambiguating context (`src/actions/dialog.rs:1450+`).

## Built-in Features Audit
Notes:
- Default confirmation list: `builtin-shut-down`, `builtin-restart`, `builtin-log-out`, `builtin-empty-trash`, `builtin-sleep`, `builtin-quit-script-kit`, `builtin-test-confirmation` (`src/config/defaults.rs:37`).
- Confirmation UX: modal asks `Are you sure you want to {entry_name}?` (Yes/Cancel). Cancel has no user-facing feedback.

### Core Built-ins
- `builtin-clipboard-history` — Label: `Clipboard History` — Does: opens clipboard history view. Feedback now: view switch only. Should: optional transient HUD (`Clipboard History`) only if open is delayed/fails.
- `builtin-window-switcher` — Label: `Window Switcher` — Does: opens window switcher view. Feedback now: view switch only; error toast only on load failure. Should: keep as-is, but add empty-state guidance when no windows.
- `builtin-ai-chat` — Label: `AI Chat` — Does: hides main window, opens AI window async. Feedback now: no success HUD; error toast on failure. Should: HUD on open failure only (success toast optional/noisy).
- `builtin-notes` — Label: `Notes` — Does: hides main window, opens notes window. Feedback now: no success HUD; error toast on failure. Should: same as AI chat.
- `builtin-design-gallery` (debug) — Label: `Design Gallery` — Does: opens design gallery view. Feedback now: view switch only. Should: acceptable.
- `builtin-test-confirmation` (debug) — Label: `Test Confirmation` — Does: confirmation flow test; on execute shows success toast. Feedback now: good (success toast), but cancel is silent. Should: add cancel HUD (`Cancelled`).

### System Actions
- `builtin-empty-trash` — `Empty Trash` — Executes Finder `empty trash`. Feedback now: confirmation modal (default), then closes window; error toast on failure. Should: success HUD/toast (`Trash emptied`) and keep cancel feedback.
- `builtin-lock-screen` — `Lock Screen` — Sends lock keystroke. Feedback now: closes window; no success feedback. Should: no toast needed (screen locks immediately).
- `builtin-sleep` — `Sleep` — System sleep. Feedback now: confirmation modal (default), then closes window. Should: acceptable; optional pre-sleep HUD not necessary.
- `builtin-restart` — `Restart` — System restart. Feedback now: confirmation modal (default), then closes window. Should: keep modal; add explicit confirmation copy (`Restart now? Unsaved docs may prompt`).
- `builtin-shut-down` — `Shut Down` — System shutdown. Feedback now: confirmation modal (default), then closes window. Should: keep modal; stronger wording.
- `builtin-log-out` — `Log Out` — Logs out user. Feedback now: confirmation modal (default), then closes window. Should: keep modal + clearer consequence text.
- `builtin-toggle-dark-mode` — `Toggle Dark Mode` — Toggles appearance. Feedback now: closes window only. Should: HUD (`Dark Mode On/Off`) for immediate confirmation.
- `builtin-show-desktop` — `Show Desktop` — Mission-control shortcut. Feedback now: closes window only. Should: no HUD required.
- `builtin-mission-control` — `Mission Control` — triggers mission control. Feedback now: closes window only. Should: no HUD required.
- `builtin-launchpad` — `Launchpad` — activates Launchpad. Feedback now: closes window only. Should: no HUD required.
- `builtin-force-quit` — `Force Quit Apps` — opens macOS Force Quit dialog (not in-app app list). Feedback now: closes window only. Should: align label/description to actual behavior or implement in-app picker.
- `builtin-volume-0` — `Volume 0%` — sets output volume. Feedback now: closes window only. Should: HUD (`Volume 0%`).
- `builtin-volume-25` — `Volume 25%` — sets volume. Feedback now: closes window only. Should: HUD (`Volume 25%`).
- `builtin-volume-50` — `Volume 50%` — sets volume. Feedback now: closes window only. Should: HUD (`Volume 50%`).
- `builtin-volume-75` — `Volume 75%` — sets volume. Feedback now: closes window only. Should: HUD (`Volume 75%`).
- `builtin-volume-100` — `Volume 100%` — sets volume max. Feedback now: closes window only. Should: HUD (`Volume 100%`).
- `builtin-volume-mute` — `Toggle Mute` — toggles mute state. Feedback now: closes window only. Should: HUD (`Muted`/`Unmuted`).
- `builtin-quit-script-kit` — `Quit Script Kit` — quits app. Feedback now: confirmation modal (default), then quit; no success toast. Should: keep modal; no success toast needed.
- `builtin-toggle-dnd` — `Toggle Do Not Disturb` — toggles Focus via ControlCenter script. Feedback now: closes window only. Should: HUD (`Focus toggled`) and robust error copy when scripting fails.
- `builtin-screen-saver` — `Start Screen Saver` — activates screensaver. Feedback now: closes window only. Should: no HUD needed.
- `builtin-system-preferences` — `Open System Settings` — opens System Settings app. Feedback now: closes window only; error toast on failure. Should: acceptable.
- `builtin-privacy-settings` — `Privacy & Security Settings` — opens pane URL. Feedback now: closes window only; error toast on failure. Should: acceptable.
- `builtin-display-settings` — `Display Settings` — opens pane URL. Feedback now: closes window only. Should: acceptable.
- `builtin-sound-settings` — `Sound Settings` — opens pane URL. Feedback now: closes window only. Should: acceptable.
- `builtin-network-settings` — `Network Settings` — opens pane URL. Feedback now: closes window only. Should: acceptable.
- `builtin-keyboard-settings` — `Keyboard Settings` — opens pane URL. Feedback now: closes window only. Should: acceptable.
- `builtin-bluetooth-settings` — `Bluetooth Settings` — opens pane URL. Feedback now: closes window only. Should: acceptable.
- `builtin-notifications-settings` — `Notification Settings` — opens pane URL. Feedback now: closes window only. Should: acceptable.

### Notes Commands
- `builtin-new-note` — `New Note` — hides main, opens notes window. Feedback now: no success HUD. Should: optional HUD only on failure.
- `builtin-search-notes` — `Search Notes` — hides main, opens notes window. Feedback now: no success HUD. Should: same.
- `builtin-quick-capture` — `Quick Capture` — hides main, opens quick capture. Feedback now: no success HUD; error toast on failure. Should: HUD (`Quick Capture ready`) if open latency is noticeable.

### AI Commands
- `builtin-new-conversation` — `New AI Conversation` — hides main, opens AI window. Feedback now: no success HUD. Should: acceptable if instant.
- `builtin-send-screen-to-ai` — `Send Screen to AI Chat` — captures full screen, opens AI, seeds image prompt. Feedback now: no success HUD; error toast on capture failure. Should: HUD (`Screen attached to AI`) after successful attach.
- `builtin-send-window-to-ai` — `Send Focused Window to AI Chat` — captures focused window, opens AI, seeds image prompt. Feedback now: no success HUD; error toast on failure. Should: HUD (`Window attached to AI`).
- `builtin-send-selected-text-to-ai` — `Send Selected Text to AI Chat` — reads selection, opens AI, seeds text prompt. Feedback now: info toast when no selection, error toast on failure, no success HUD. Should: HUD (`Selection sent to AI`).
- `builtin-send-browser-tab-to-ai` — `Send Focused Browser Tab to AI Chat` — fetches browser URL, opens AI, seeds prompt. Feedback now: error toast on failure, no success HUD. Should: HUD (`Tab sent to AI`).
- `builtin-send-screen-area-to-ai` — `Send Screen Area to AI Chat` — placeholder only. Feedback now: info toast `coming soon`. Should: keep until implemented.
- `builtin-create-ai-preset` — `Create AI Chat Preset` — placeholder; opens AI window. Feedback now: info toast `coming soon`. Should: either remove from menu until implemented, or open concrete preset flow.
- `builtin-import-ai-presets` — `Import AI Chat Presets` — placeholder; opens AI window. Feedback now: info toast `coming soon`. Should: same as above.
- `builtin-search-ai-presets` — `Search AI Chat Presets` — placeholder; opens AI window. Feedback now: info toast `coming soon`. Should: same as above.

### Script / Permission / Settings / Utility
- `builtin-new-script` — `New Script` — creates script file and opens editor. Feedback now: success toast on success; error toast on failure. Should: optionally add `Reveal in Finder` quick action in toast.
- `builtin-new-extension` — `New Extension` — creates extension file and opens editor. Feedback now: success toast; error toast on failure. Should: same as above.
- `builtin-check-permissions` — `Check Permissions` — checks all permission statuses. Feedback now: success/warning toast. Should: good.
- `builtin-request-accessibility` — `Request Accessibility Permission` — triggers accessibility request. Feedback now: success/warning toast. Should: good.
- `builtin-accessibility-settings` — `Open Accessibility Settings` — opens settings pane. Feedback now: closes window on success, error toast on failure. Should: optional HUD (`Accessibility Settings opened`).
- `builtin-clear-suggested` — `Clear Suggested` — clears frecency store and resets list. Feedback now: success/error toast. Should: good.
- `builtin-reset-window-positions` (conditional) — `Reset Window Positions` — resets saved bounds, closes window. Feedback now: success toast. Should: good; optionally offer immediate reopen.
- `builtin-configure-vercel-api` — `Configure Vercel AI Gateway` — opens EnvPrompt and saves key. Feedback now: completion success toast only on submit; cancel is silent. Should: add explicit cancel HUD (`No changes saved`).
- `builtin-configure-openai-api` — `Configure OpenAI API Key` — same flow/feedback as above.
- `builtin-configure-anthropic-api` — `Configure Anthropic API Key` — same flow/feedback as above.
- `builtin-choose-theme` — `Choose Theme` — opens theme chooser view with live preview. Feedback now: view switch only. Should: show explicit save/cancel confirmations when leaving chooser.
- `builtin-scratch-pad` — `Scratch Pad` — opens editor prompt. Feedback now: no success HUD. Should: acceptable.
- `builtin-quick-terminal` — `Quick Terminal` — opens terminal prompt. Feedback now: no success HUD. Should: acceptable.
- `builtin-file-search` — `Search Files` — opens file-search view. Feedback now: no success HUD. Should: acceptable.
- `builtin-webcam` — `Webcam` — opens webcam prompt. Feedback now: no success HUD. Should: acceptable for open; see webcam action gap below.

## Actions Menu Audit

### Script/Main Actions (from `get_script_context_actions*` + `handle_action`)
- `run_script` — Label: dynamic (`Run "..."`, `Launch "..."`, `Switch to "..."`) — Does: delegates to `execute_selected()`. Feedback now: indirect; depends on selected item. Should: show immediate HUD (`Running...`) before asynchronous operations.
- `add_shortcut` / `update_shortcut` — Labels: `Add/Update Keyboard Shortcut` — Does: script/agent -> opens editor; others -> inline recorder modal. Feedback now: no success HUD on open, errors only for unsupported types. Should: HUD when recorder opens/saves.
- `remove_shortcut` — Label: `Remove Keyboard Shortcut` — Does: removes override and refreshes scripts. Feedback now: HUD success/error. Should: good.
- `add_alias` / `update_alias` — Label: `Add/Update Alias` — Does: opens alias input. Feedback now: modal only. Should: HUD on save success/cancel.
- `remove_alias` — Label: `Remove Alias` — Does: removes override. Feedback now: HUD success/error. Should: good.
- `edit_script` / `edit_scriptlet` — Labels: `Edit Script` / `Edit Scriptlet` — Does: opens source in editor. Feedback now: no success HUD. Should: HUD (`Opened in <editor>`).
- `view_logs` — Label: `View Logs` — Does: toggles logs panel. Feedback now: panel change only. Should: acceptable.
- `reveal_in_finder` / `reveal_scriptlet_in_finder` — Label mismatch by context (`Reveal in Finder` / `Show in Finder`) — Does: reveal path and hide main. Feedback now: HUD for success. Should: unify wording to one label.
- `copy_path` / `copy_scriptlet_path` — Label: `Copy Path` — Does: copies path and usually hides main. Feedback now: HUD success/error. Should: good.
- `copy_content` — Label: `Copy Content` — Does: reads file and copies entire content. Feedback now: HUD success/error. Should: consider confirmation for large files.
- `copy_deeplink` — Label: `Copy Deeplink` — Does: copies `scriptkit://run/...`. Feedback now: HUD success/error. Should: good.
- `reset_ranking` — Label: `Reset Ranking` — Does: removes item frecency and refreshes list. Feedback now: HUD success/no-op. Should: good.
- `scriptlet_action:<command>` — Label: dynamic from H3 action title — Does: executes parsed scriptlet action code. Feedback now: HUD success/error. Should: add spinner HUD for long-running actions.

### Global Shortcut-Only Actions (not discoverable in ActionsDialog list)
- `create_script` (Cmd+N) — Label/intent mismatch: named create, behavior opens scripts folder only. Feedback now: HUD `Opened scripts folder`. Should: either rename to `Open Scripts Folder` or actually scaffold a new script.
- `reload_scripts` (Cmd+R) — Feedback now: HUD `Scripts reloaded`. Should: good.
- `settings` (Cmd+,) — Label implies settings UI; behavior opens `~/.scriptkit/kit/config.ts`. Feedback now: HUD with editor name. Should: rename to `Open Config` or add real settings UI.
- `quit` (Cmd+Q) — Immediate quit. Feedback now: none. Should: optional confirmation depending on unsaved state.

### File Search Actions (`ActionsDialog::with_file` + `handle_action`)
- `open_file` / `open_directory` — Labels: `Open "..."` — Does: opens file/folder via OS and closes window. Feedback now: no success HUD, errors ignored (`let _ = ...`). Should: show HUD on success and error toast/HUD on failure.
- `quick_look` — Label: `Quick Look` — Does: opens Quick Look. Feedback now: no success HUD, errors ignored. Should: at least show error HUD when command fails.
- `open_with` — Label: `Open With...` — Does: opens Finder info/open-with flow. Feedback now: no success HUD, errors ignored. Should: show error HUD if launch fails.
- `show_info` — Label: `Get Info` (ID `show_info`) — Does: opens Finder info window. Feedback now: no success HUD, errors ignored. Should: show error HUD on failure.
- `copy_filename` — Label: `Copy Filename` — Does: copies filename and hides main. Feedback now: HUD success. Should: good.
- `reveal_in_finder` / `copy_path` are reused shared handlers with HUD.

### Clipboard Actions (`ActionsDialog::with_clipboard_entry` + `handle_action`)
- `clipboard_paste` — `Paste to Active App` — Copies selected entry then simulates paste and hides window. Feedback now: HUD `Pasted` or error HUD. Should: good.
- `clipboard_copy` — `Copy to Clipboard` — Copies entry only. Feedback now: HUD success/error. Should: good.
- `clipboard_paste_keep_open` — `Paste and Keep Window Open` — Pastes but keeps UI open. Feedback now: HUD `Pasted`. Should: good.
- `clipboard_share` — `Share...` — Opens system share sheet. Feedback now: no success HUD; errors only for decode/missing content. Should: HUD (`Share sheet opened`) to confirm action.
- `clipboard_attach_to_ai` — `Attach to AI Chat` — Opens AI and inserts text/image; hides main. Feedback now: no success HUD; failure HUD for missing/open errors. Should: HUD (`Attached to AI`) on success.
- `clipboard_quick_look` — `Quick Look` — Opens Quick Look preview. Feedback now: silent success. Should: error HUD when preview fails.
- `clipboard_open_with` — `Open With...` — For image entries, opens Open With flow from temp file. Feedback now: silent success. Should: success/error HUD.
- `clipboard_annotate_cleanshot` — `Annotate in CleanShot X` — Opens CleanShot URL after copying image. Feedback now: HUD success/failure. Should: good.
- `clipboard_upload_cleanshot` — `Upload to CleanShot X` — Writes temp PNG and opens upload URL. Feedback now: HUD success/failure. Should: good.
- `clipboard_pin` / `clipboard_unpin` — `Pin/Unpin Entry` — Updates pin state and list order. Feedback now: silent success (no HUD). Should: HUD (`Pinned` / `Unpinned`).
- `clipboard_ocr` — `Copy Text from Image` — OCRs image, caches result, copies text. Feedback now: progress and result HUDs; failure HUD. Should: good.
- `clipboard_save_snippet` — `Save Text as Snippet` — Appends snippet markdown and refreshes scripts. Feedback now: HUD success/failure. Should: include quick action to open snippet file.
- `clipboard_save_file` — `Save as File...` — Saves to Desktop/home with timestamp name. Feedback now: HUD filename/failure. Should: offer `Reveal in Finder` after save.
- `clipboard_delete` — `Delete Entry` — Deletes selected entry. Feedback now: HUD success/failure. Should: optional undo.
- `clipboard_delete_multiple` — `Delete Entries...` — Deletes filtered entries. Feedback now: HUD counts. Should: add confirmation dialog before batch delete.
- `clipboard_delete_all` — `Delete All Entries` — Clears all unpinned entries. Feedback now: HUD count. Should: add mandatory confirmation.

### Path Prompt Actions (`ActionsDialog::with_path` + `execute_path_action`)
- `select_file` — `Select "..."` — Submits file path to script callback. Feedback now: callback-driven only, no UI confirmation. Should: optional HUD if callback is async/slow.
- `open_directory` — `Open "..."` — Navigates into directory in-path prompt. Feedback now: visual navigation only. Should: acceptable.
- `copy_path` / `copy_filename` — Labels: `Copy Path` / `Copy Filename` — Does: clipboard copy and sets `last_output`. Feedback now: often no visible HUD in path flow. Should: use HUD, not `last_output`.
- `open_in_finder` — `Open in Finder` — Reveals in Finder and hides main. Feedback now: no HUD on success. Should: HUD (`Revealed in Finder`).
- `open_in_editor` — `Open in Editor` — Launches configured editor and hides main. Feedback now: no HUD on success. Should: HUD (`Opened in <editor>`).
- `open_in_terminal` — `Open in Terminal` — Opens Terminal at path and hides main. Feedback now: no HUD on success. Should: HUD (`Opened Terminal`).
- `move_to_trash` — `Move to Trash` — Finder delete script; refreshes list. Feedback now: `last_output` only, no confirmation modal. Should: confirmation dialog + HUD + undo hint.

### Chat Prompt Actions (`ActionsDialog::with_chat` + `execute_chat_action`)
- `select_model_<id>` — Label: model name (current one with `✓`) — Does: sets current model. Feedback now: checkmark state + log only. Should: HUD (`Model: <name>`).
- `continue_in_chat` — `Continue in Chat` — Transfers messages to AI window and closes prompt. Feedback now: no explicit success HUD. Should: HUD (`Opened in AI Chat`) or inline transition indicator.
- `copy_response` — `Copy Last Response` — Copies last assistant message to clipboard. Feedback now: silent success. Should: HUD (`Copied response`).
- `clear_conversation` — `Clear Conversation` — clears all messages immediately. Feedback now: silent, no confirmation. Should: confirmation or undo toast.

### Webcam Prompt Actions (`webcam_actions_for_dialog` + `execute_webcam_action`)
- `capture` — Label: `Capture` — Current behavior: just `close_and_reset_window()`. Feedback now: none. Should: actually capture current frame and show HUD with destination (`Saved photo` + `Reveal`).
- `close` — Label: `Close` — Closes prompt. Feedback now: none. Should: acceptable.

### SDK Actions (`set_sdk_actions` + `trigger_sdk_action_internal`)
- Action ID/Label source: `ProtocolAction.name` used for both ID and title (`src/actions/dialog.rs:608+`).
- Behavior:
  - `has_action=true` => sends `ActionTriggered` protocol message.
  - `has_action=false` + `value` => submits value.
  - Missing sender/full/disconnected => logs only.
- Feedback now: no user-visible success/failure state for dispatched actions. Should: add optional HUD (`Action sent`) and error HUD when channel is full/disconnected.

## Label Consistency Findings (Rendering + Naming)
- Finder intent uses three labels across contexts:
  - `Reveal in Finder` (`script` context)
  - `Show in Finder` (`file` context)
  - `Open in Finder` (`path` context)
  - Recommendation: normalize to one phrase (`Reveal in Finder`).
- `Get Info` (label) uses ID `show_info`; acceptable technically but inconsistent language. Recommendation: either label `Show Info` or keep Finder-native `Get Info` and align ID comments.
- `Settings` shortcut opens config file, not settings UI. Recommendation: rename shortcut action to `Open Config.ts` unless full settings UI exists.
- `Create Script` shortcut opens scripts folder only. Recommendation: either scaffold a file or rename to `Open Scripts Folder`.
- Action row rendering currently omits descriptions; only title + shortcut are visible. Recommendation: optional second-line description for ambiguous actions.

## Coverage Notes
- `BuiltInFeature::AppLauncher` and `BuiltInFeature::App(...)` execution paths still exist, but `get_builtin_entries()` no longer adds an App Launcher built-in entry.
- `NotesCommandType::OpenNotes` and `AiCommandType::OpenAi/ClearConversation` variants exist but are not currently exposed as built-in entries.

## Recommended Priority Order
1. Fix webcam `capture` behavior + success feedback.
2. Add confirmation for destructive action-menu operations (`clipboard_delete_all`, `clipboard_delete_multiple`, `move_to_trash`).
3. Add error handling feedback for file-search actions that currently ignore `Result`.
4. Remove dangerous auto-execute fallback when confirmation modal fails to open.
5. Unify label language (`Reveal in Finder`, etc.) and tighten action naming (`Create Script`, `Settings`).
