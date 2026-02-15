# Code Map

## Core Application
- `src/main.rs` App entry point, gpui imports, module declarations.
- `src/app_impl.rs` Core app state, window creation, rendering logic.
- `src/app_render.rs` Render pass orchestration and layout composition.
- `src/app_shell/` GPUI shell layer (focus, keymap, shell/window spec).
- `src/app_navigation.rs` and `src/navigation.rs` App navigation flow and state.

## UI & Theme
- `src/ui_foundation.rs` UI base wrappers and root container setup.
- `src/components/` UI building blocks (buttons, inputs, prompts, toast).
- `src/editor.rs` and `src/terminal.rs` Editor and terminal UI (script text and output styling).
- `src/theme/` Theme definitions (`types.rs`, `helpers.rs`, `gpui_integration.rs`) and GPUI integration.
- `src/render_prompts/` Prompt-specific renderers and list item styles.

## Prompts, Scripts, and Lists
- `src/prompts/` Prompt types (arg/div/editor/chat/select).
- `src/form_prompt.rs`, `src/term_prompt.rs` Prompt rendering and input handling.
- `src/scriptlet_metadata.rs`, `src/scriptlet_cache.rs`, `src/scriptlets.rs` Scriptlet loading and metadata cache.
- `src/scripts/` Script loading helpers (loader, metadata, scheduling).

## Execution & Runtime
- `src/executor/` Script execution, runner, error handling, stderr buffering.
- `src/execute_script.rs` Script run orchestration and metadata.
- `src/executor.rs` Bun execution integration and process management.

## Features
- `src/builtins.rs` Built-in script and action registrations.
- `src/ai/` AI chat window and model/session/profiles (`window.rs`, `session.rs`, `providers.rs`).
- `src/notes/` Notes UI, storage, and markdown rendering (`window.rs`, `storage.rs`, `model.rs`).
- `src/clipboard_history/` Clipboard data capture and history UI (`clipboard.rs`, `database.rs`, `image.rs`).
- `src/file_search.rs` file search feature.
- `src/system_actions.rs` MacOS system action integrations (AppleScript).
- `src/actions/` actions dialog and action helpers.
- `src/menu_bar.rs` and `src/app_launcher.rs` Menu bar and app launch UI.
- `src/hotkeys.rs` and `src/shortcuts.rs` Global hotkeys and shortcut mappings.

## Windows & System
- `src/window_manager.rs`, `src/window_state.rs`, `src/window_resize.rs`, `src/window_ops.rs` Window management and sizing.
- `src/panel.rs` and `src/window_control.rs` panel-level behavior and control.
- `src/platform.rs` OS-specific APIs (MacOS, window positioning).

## Data, Logging, and Config
- `src/config/` Configuration loading and helpers (`get_*` defaults).
- `src/config/` contains `defaults.rs` values, `loader.rs` parser, and `types.rs` definitions.
- `src/permissions_wizard.rs` macOS permissions prompts and management.
- `src/logging.rs` Log output and JSONL tracing.
- `src/config.rs` and `src/secrets.rs` settings and secure storage.

## Utilities & Protocol
- `src/protocol/` stdin JSON protocol definitions and message handling.
- `src/mcp_protocol.rs` JSON-RPC 2.0 definitions; `src/mcp_server.rs` HTTP server; `src/mcp_*.rs` tools/resources/streaming support.
- `src/stdin_commands.rs` Supported stdin command handlers.
- `src/watchers.rs` File and script watcher support.

## Scripts and SDK
- `scripts/kit-sdk.ts` SDK runtime and type definitions used by scripts.
- `scripts/*.ts` Helper scripts (test runners, tooling).

## Tests
- `tests/` End-to-end and unit tests (smoke, SDK, mcp).
