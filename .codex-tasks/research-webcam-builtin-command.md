# Webcam Built-in Command Research Notes

- **Built-in command registration (src/builtins.rs)**
  - `enum BuiltInFeature` lists all built-in feature variants (`ClipboardHistory`, `AppLauncher`, `WindowSwitcher`, `AiCommand`, `UtilityCommand`, etc.). Each variant is a concrete command category.
  - `struct BuiltInEntry` holds metadata for each command: `id`, `name`, `description`, `keywords`, `feature` (variant), optional `icon`, and `group` (`Core` or `MenuBar`).
  - `BuiltInEntry::new` / `new_with_icon` helpers construct entries with `BuiltInGroup::Core`, default icon optional, and set metadata.
  - `get_builtin_entries(config)` returns a `Vec<BuiltInEntry>` based on `BuiltInConfig` flags (`clipboard_history`, `window_switcher`, etc.) and always includes AI/Notes and design gallery entries. Add new commands by pushing a new `BuiltInEntry` (e.g., `id: "builtin-webcam"`, `BuiltInFeature::Webcam`).

- **Protocol message flow (src/protocol/message.rs -> execute_script.rs -> src/main.rs -> src/prompt_handler.rs)**
  - `enum Message` (`src/protocol/message.rs`) is serde-tagged by `type` and includes prompt variants (`Arg`, `Div`, `Editor`, `Webcam`, etc.) and action variants (`Exit`, `Hide`, etc.).
  - `Message::prompt_id()` and `Message::request_id()` provide helpers for extracting IDs and correlate responses.
  - `execute_script.rs` maps incoming SDK JSON messages to internal `PromptMessage` variants:
    - `Message::Div` -> `PromptMessage::ShowDiv`
    - `Message::Editor` -> `PromptMessage::ShowEditor`
    - `Message::Arg` -> `PromptMessage::ShowArg`
    - (No existing `Message::Webcam` mapping yet; would map to `PromptMessage::ShowWebcam`.)
  - `enum PromptMessage` in `src/main.rs` captures app-level prompt/message events (`ShowDiv`, `ShowEditor`, `ShowArg`, `ShowPath`, etc.)
  - `prompt_handler.rs` matches `PromptMessage` variants and instantiates UI views (`DivPrompt`, `EditorPrompt`, `SelectPrompt`, etc.) and sets `AppView` states + focus/resizes.

- **Prompt implementation examples**
  - **Div prompt (src/prompts/div.rs):**
    - `DivPrompt::with_options` builds prompt UI from HTML (parsed via `utils::parse_html`), container options (`ContainerOptions`), and theme colors.
    - Renders HTML elements into GPUI `Div` nodes (headers, paragraphs, list, links).
  - **Editor prompt (src/editor.rs):**
    - `EditorPrompt::with_height` / `with_template` build full-featured `gpui-component::InputState` prompt (syntax highlighting, templates, tabstops).
    - Delays InputState creation until first render (`ensure_initialized`) and supports tab-navigation for templates.

- **Step-by-step: add a Webcam built-in command + custom `WebcamPrompt`**
  1. Add a new `BuiltInFeature::Webcam` variant in `src/builtins.rs` and register it in `get_builtin_entries` (`BuiltInEntry::new_with_icon("builtin-webcam", "Webcam", ...)`).
  2. Add command handling in `src/app_execute.rs` under `execute_builtin`:
     - Match `BuiltInFeature::Webcam` and set `current_view` to `AppView::WebcamPrompt` (new variant).
     - Optionally trigger permissions checks or open OS camera access before rendering.
  3. Extend `AppView` in `src/main.rs` with `WebcamPrompt { id, entity }` and update render paths (`src/app_render.rs`) to display the webcam UI component.
  4. Implement `src/prompts/webcam.rs` (new component) and export it in `src/prompts/mod.rs`; keep a matching `SubmitCallback` if the view returns values.
  5. If protocol support is needed, add `Message::Webcam` handling in `src/protocol/message.rs` and map it in `execute_script.rs` to `PromptMessage::ShowWebcam` (new variant), then handle `ShowWebcam` in `prompt_handler.rs` to instantiate `WebcamPrompt` and set `current_view` accordingly.
