# Cmd+K Actions Investigation

## Files Investigated
- `src/app_impl.rs:929` (global Cmd+K interceptor + view routing)
- `src/render_script_list.rs:495` (main script list Cmd+K toggle)
- `src/render_prompts/arg.rs:136` (ArgPrompt Cmd+K + actions routing)
- `src/render_prompts/div.rs:46` (DivPrompt Cmd+K + actions routing)
- `src/render_prompts/form.rs:57` (FormPrompt Cmd+K + actions routing)
- `src/render_prompts/editor.rs:80` (EditorPrompt Cmd+K + actions routing)
- `src/render_prompts/term.rs:74` (TermPrompt Cmd+K + actions routing)
- `src/render_prompts/path.rs:123` (PathPrompt Cmd+K toggle)
- `src/render_prompts/other.rs:24` (Select/Env/Drop/Template prompt key handling)
- `src/render_prompts/other.rs:207` (ChatPrompt Cmd+K + actions routing)
- `src/prompt_handler.rs:39` (ShowArg stores sdk_actions + shortcuts)
- `src/prompt_handler.rs:1880` (setActions stores sdk_actions + shortcuts)
- `src/prompt_handler.rs:1089` (ShowPath/Env/Drop/Template/Select prompt setup)
- `src/protocol/message.rs:63` (Arg/Div actions fields)
- `src/protocol/message.rs:137` (Editor actions field)
- `src/protocol/message.rs:210` (Fields/Form actions fields)
- `src/protocol/message.rs:281` (Chat actions field)
- `src/protocol/message.rs:410` (Term actions field)
- `src/execute_script.rs:1213` (Message → PromptMessage mapping; no Fields case)
- `src/main.rs:762` (AppView prompt inventory)
- `src/ui_foundation.rs:605` (is_key_k helper)
- `scripts/kit-sdk.ts:4264` (fields() sends actions with type="fields")
- `src/actions/command_bar.rs:12` (CommandBar component for Cmd+K menus)
- `src/ai/window.rs:1230` (AI command bar Cmd+K in simulateKey handler)
- `src/notes/window.rs:2191` (Notes window Cmd+K toggles actions panel)
- `src/notes/actions_panel.rs:1` (Notes actions panel description)
- `src/app_shell/keymap.rs:18` (ShellAction::OpenActions binding)

## Current Behavior: Cmd+K Coverage

### Main window (ScriptListApp)
- **Script list**: Cmd+K toggles the actions dialog in the script list view via the list key handler. `src/render_script_list.rs:495`
- **Global interceptor**: Cmd+K is intercepted at the app level only for ScriptList, FileSearchView, ArgPrompt, and ChatPrompt. Other views are explicitly skipped. `src/app_impl.rs:929`
- **ArgPrompt**: Cmd+K calls `toggle_arg_actions` and routes keys to the actions dialog. `src/render_prompts/arg.rs:136`
- **DivPrompt**: Cmd+K calls `toggle_arg_actions` and routes keys to the actions dialog. `src/render_prompts/div.rs:46`
- **FormPrompt**: Cmd+K calls `toggle_arg_actions` and routes keys to the actions dialog. `src/render_prompts/form.rs:57`
- **EditorPrompt**: Cmd+K calls `toggle_arg_actions` and routes keys to the actions dialog. `src/render_prompts/editor.rs:80`
- **TermPrompt**: Cmd+K calls `toggle_arg_actions` and routes keys to the actions dialog. `src/render_prompts/term.rs:74`
- **ChatPrompt**: Cmd+K calls `toggle_chat_actions` and routes keys to the actions dialog. `src/render_prompts/other.rs:218`
- **PathPrompt**: Cmd+K toggles a path-specific actions dialog. `src/render_prompts/path.rs:123`

### Prompts that **do not** handle Cmd+K today
- **SelectPrompt, EnvPrompt, DropPrompt, TemplatePrompt** only intercept global shortcuts (Cmd+W / Esc). There is no Cmd+K handling or actions routing in these renderers. `src/render_prompts/other.rs:24`

### Actions availability from the protocol/SDK
- Prompts with **actions fields in protocol**: Arg, Div, Editor, Fields, Form, Chat, Term. `src/protocol/message.rs:63` `src/protocol/message.rs:137` `src/protocol/message.rs:210` `src/protocol/message.rs:281` `src/protocol/message.rs:410`
- `setActions` sets `self.sdk_actions` and registers shortcuts globally, independent of prompt type. `src/prompt_handler.rs:1880`
- SDK `fields()` sends `type="fields"` with actions (serialized) when provided. `scripts/kit-sdk.ts:4264`

### Secondary windows (not main prompts)
- **AI window**: Cmd+K toggles a CommandBar (shown in simulateKey handler). `src/ai/window.rs:1230`
- **Notes window**: Cmd+K toggles actions panel / command bar. `src/notes/window.rs:2191`

## Root Cause Analysis (Why Cmd+K Isn’t Universal)

1) **Cmd+K routing is explicitly limited to a subset of views.**  
The global interceptor only handles ScriptList, FileSearchView, ArgPrompt, and ChatPrompt, and explicitly skips other views. This means any prompt without its own Cmd+K handler won’t open actions. `src/app_impl.rs:929`

2) **Several prompt renderers don’t implement Cmd+K at all.**  
Select/Env/Drop/Template prompts only handle global shortcuts and never call `toggle_arg_actions` or `route_key_to_actions_dialog`. So even if `setActions()` is used, Cmd+K has no handler in those prompts. `src/render_prompts/other.rs:24` `src/prompt_handler.rs:1880`

3) **`fields()` actions are defined in the SDK but never reach a prompt.**  
The SDK sends `type="fields"` with actions, but the app’s message-to-prompt mapping doesn’t handle `Message::Fields`. That means no prompt is shown, and Cmd+K can’t work. `scripts/kit-sdk.ts:4264` `src/execute_script.rs:1213` `src/protocol/message.rs:210`

4) **Key handling is fragmented across multiple layers.**  
Some prompts rely on per-prompt `on_key_down`, while others need the global interceptor because focused Input components consume keystrokes (as the comment notes). This split makes coverage brittle for any prompt that embeds a focused input but isn’t included in the interceptor. `src/app_impl.rs:960`

## Proposed Solution Approach

1) **Centralize Cmd+K routing in one place.**  
Add a `toggle_actions_for_current_view()` helper in `ScriptListApp` that checks if actions are available (SDK actions or built-in view actions) and opens/closes the correct dialog. Then call it from the global interceptor instead of hard-coding the view list. `src/app_impl.rs:929`

2) **Expand Cmd+K handling to all prompt renderers that can show actions.**  
Add Cmd+K + `route_key_to_actions_dialog` handling in `render_prompts/other.rs` for Select/Env/Drop/Template (gated by `self.sdk_actions.is_some()` so it’s a no-op if no actions are set). `src/render_prompts/other.rs:24` `src/prompt_handler.rs:1880`

3) **Wire `fields` prompts into the prompt pipeline.**  
Handle `Message::Fields` in `execute_script.rs` and map it to a prompt (either reuse `ShowForm` or add a new `ShowFields`) so the actions payload reaches the UI. This will allow Cmd+K to work on `fields()` prompts the same way it does for `form()`. `src/execute_script.rs:1213` `src/protocol/message.rs:210` `scripts/kit-sdk.ts:4264`

4) **Make actions availability explicit and consistent.**  
Create a helper to set/clear `sdk_actions` + `action_shortcuts` whenever the prompt changes (not just in `ShowArg` and `setActions`), so action visibility and Cmd+K behavior are deterministic. `src/prompt_handler.rs:39` `src/prompt_handler.rs:1880`

5) **Optional consolidation:**  
Adopt the existing `CommandBar` abstraction for main-window prompts too (or implement a shared `ActionsHost` interface) to reduce per-prompt key handling and ensure Cmd+K works consistently everywhere. `src/actions/command_bar.rs:12`

## Verification
- What was changed: Added Cmd+K handling to SelectPrompt, EnvPrompt, DropPrompt, and TemplatePrompt in `src/render_prompts/other.rs`.
- Test results: `cargo check` passes; `cargo clippy --all-targets -- -D warnings` passes; `cargo test` passes (16 passed, 0 failed).
- Before/after comparison: Before: these 4 prompts did not handle Cmd+K. After: they now check for Cmd+K + `sdk_actions.is_some()` and call `toggle_arg_actions()`.
- Pattern used: Same pattern as existing prompts (FormPrompt, EditorPrompt, TermPrompt) that already handle Cmd+K.
