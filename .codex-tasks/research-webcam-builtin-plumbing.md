# Webcam Built-in Plumbing Research

## 1. BuiltInFeature enum (src/builtins.rs)

`BuiltInFeature` is defined around `src/builtins.rs` and includes:

- `ClipboardHistory`, `AppLauncher`, `App(String)` for individual apps,
- `WindowSwitcher`, `DesignGallery`, `AiChat`, `Notes`, `MenuBarAction(MenuBarActionInfo)`
- `SystemAction(SystemActionType)`
- `NotesCommand(NotesCommandType)`, `AiCommand(AiCommandType)`, `ScriptCommand(ScriptCommandType)`
- `PermissionCommand(PermissionCommandType)`, `FrecencyCommand(FrecencyCommandType)`
- `SettingsCommand(SettingsCommandType)`, `UtilityCommand(UtilityCommandType)`, `FileSearch`

Built-in entries are registered via `get_builtin_entries(config: &BuiltInConfig)` and
`BuiltInEntry::new_with_icon(...)` (for icon entries) or `new(...)`:

- Clipboard history, window switcher, AI chat, notes, design gallery (debug), system actions,
  notes commands, AI commands, script commands, etc.
- Each entry includes `id`, `name`, `description`, `keywords`, `feature` and icon.

See `src/builtins.rs` around `BuiltInFeature` and `get_builtin_entries` for exact variants
and entry registration details.

## 2. AppView enum and rendering (src/main.rs)

`AppView` enum in `src/main.rs` defines view states:

- `ScriptList`, `ActionsDialog`
- Prompt views: `ArgPrompt { id, placeholder, choices, actions }`, `DivPrompt { id, entity }`,
  `FormPrompt { id, entity }`, `TermPrompt { id, entity }`, `EditorPrompt { id, entity, focus_handle }`,
  `SelectPrompt { id, entity }`, `PathPrompt { id, entity, focus_handle }`,
  `EnvPrompt { id, entity }`, `DropPrompt { id, entity }`, `TemplatePrompt { id, entity }`,
  `ChatPrompt { id, entity }`
- Fixed views: `ClipboardHistoryView { filter, selected_index }`, `AppLauncherView { filter, selected_index }`,
  `WindowSwitcherView { filter, selected_index }`, `DesignGalleryView { filter, selected_index }`,
  `ScratchPadView { entity, focus_handle }`, `QuickTerminalView { entity }`,
  `FileSearchView { query, selected_index }`, `ThemeChooserView { filter, selected_index }`.

`main.rs` render path: match on `self.current_view` and call
`render_*` methods (`render_script_list`, `render_arg_prompt`, `render_div_prompt`,
`render_editor_prompt`, etc.) to produce `AnyElement`.

## 3. execute_builtin pattern (src/app_execute.rs)

`execute_builtin(&mut self, entry: &BuiltInEntry, cx: &mut Context<Self>)` handles:

- Confirmation check via `self.config.requires_confirmation(entry.id)`: open confirm modal and
  spawn async open_confirm_window; sends result via `builtin_confirm_sender`.
- Match on `entry.feature` and set UI state:
  - For list-style views (`ClipboardHistory`, `WindowSwitcher`, `AppLauncher`, `DesignGallery` etc.)
    set filter text, `current_view` variant, `selected_index`, focus states and
    `resize_to_view_sync` then `cx.notify()`.
  - For window/AI/notes actions, call `script_kit_gpui::set_main_window_visible(false)`,
    hide window, and open specific window (`ai::open_ai_window`, `notes::open_notes_window`).
  - System actions use `system_actions::` helper functions and hide toast/notifications.
  - API key and theme chooser use `show_api_key_prompt` and `AppView::ThemeChooserView`.

Full match statements are in `src/app_execute.rs` around `execute_builtin`.

## 4. Prompts and protocol state

- `src/prompts/mod.rs` exports prompt modules (`chat`, `div`, `env`, `drop`, `path`, `select`,
  `template`), re-exports prompt types (e.g., `DivPrompt`, `ChatPrompt`, `EnvPrompt`,
  `SelectPrompt`), and now includes `WebcamPrompt` via `webcam`.
- `src/protocol/message.rs` defines `Message::Webcam { id: String }` (plus `Message::mic`).
  Message helper: `pub fn webcam(id: String) -> Self` and it includes `Message::Webcam` in id
  extraction. However, no corresponding prompt struct or rendering path exists yet.

## 5. Code patterns to follow

- Add new built-in features by extending `BuiltInFeature` and registering entries via
  `get_builtin_entries` with `BuiltInEntry::new_with_icon(...)`.
- Add `AppView` variant and a corresponding `render_` function call in `main.rs`'s render match
  (same shape as `ClipboardHistoryView`/`AppLauncherView` etc.).
- In `execute_builtin`, add a match arm and set `current_view` + `pending_placeholder`/`resize_to_view_sync`,
  then `cx.notify()` as per other built-in list views.
- For webcam support, add new prompt module and export in `src/prompts/mod.rs`, and include
  handling in `execute_builtin` + `AppView` + protocol if needed.

## Verification

- Files changed: `src/builtins.rs` (BuiltInFeature::Webcam and entry registration),
  `src/main.rs` (AppView::WebcamView and rendering match),
  `src/app_execute.rs` (execute_builtin match arm for webcam),
  `src/prompts/mod.rs` (webcam module export), and supporting files
  (`src/app_impl.rs`, `src/app_layout.rs`, `src/app_render.rs`, `src/prompt_handler.rs`) for
  state, layout, and rendering details.
- Cargo check: passed with warnings.
- Actual implementation matches research patterns by extending `BuiltInFeature`, `AppView`, and
  adding webcam rendering and execution handling as described.
- Deviations: extra match arms were added in `src/app_impl.rs`, `src/app_layout.rs`,
  `src/app_render.rs`, and `src/prompt_handler.rs` to support UI state reporting, layout
  metadata, and prompt state responses for webcam views, as those details are beyond the
  original research patterns but required for runtime UI support.
