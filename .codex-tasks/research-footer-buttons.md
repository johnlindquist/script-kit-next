# Research: Footer Buttons (Script Kit GPUI)

## 1) Where footer buttons are rendered

### Core components (the actual button rendering)
- `src/components/prompt_footer.rs:234-388` -- `PromptFooter::render_button` and `PromptFooter::render` build the footer container and the primary/secondary buttons for prompt UIs.
- `src/app_shell/shell.rs:250-358` -- `AppShell::render_footer` + `render_footer_button` render footer buttons for the generic shell frame (`ShellSpec`/`FooterSpec`).

### PromptFooter call sites (prompt + main menu footers)
- Main menu script list footer: `src/render_script_list.rs:957-1014` (primary label varies by selected item, secondary Actions button).
- Arg prompt footer: `src/render_prompts/arg.rs:459-503`.
- Div prompt footer: `src/render_prompts/div.rs:103-162`.
- Editor prompt footer: `src/render_prompts/editor.rs:191-247`.
- Terminal prompt footer: `src/render_prompts/term.rs:120-165`.
- Form prompt footer: `src/render_prompts/form.rs:181-199`.
- Env prompt footer: `src/prompts/env.rs:567-599`.
- Chat prompt footer: `src/prompts/chat.rs:2165-2187`.
- Built-ins (multiple views): `src/render_builtins.rs:698`, `1277`, `1601`, `2267`, `2909` (all use `PromptFooter::new`).

### App shell usage
- `ShellSpec` includes an optional footer and `FooterSpec` builder: `src/app_shell/spec.rs:12-338`.
- The shell footer is rendered when `ShellSpec.footer` is present: `src/app_shell/shell.rs:250-334`.

## 2) How buttons are configured (label, shortcut, visibility)

### PromptFooter configuration (prompt UIs)
- `PromptFooterConfig` fields for labels/shortcuts/visibility: `src/components/prompt_footer.rs:91-109`.
- Builder methods:
  - `primary_label`, `primary_shortcut`, `secondary_label`, `secondary_shortcut`: `src/components/prompt_footer.rs:133-153`.
  - `show_secondary` (visibility toggle) + `show_logo`, `helper_text`, `info_label`: `src/components/prompt_footer.rs:157-177`.
- Visibility logic: secondary button is only rendered when `config.show_secondary` is true: `src/components/prompt_footer.rs:316-325`.
- Example of dynamic primary label:
  - Script list footer pulls label from selected item: `src/render_script_list.rs:972-985`.
  - The label is resolved in `SearchResult::get_default_action_text`: `src/scripts/types.rs:202-236`.

### AppShell footer configuration (shell frame)
- `FooterSpec` fields (labels/shortcuts/visibility): `src/app_shell/spec.rs:274-288`.
- Builder methods:
  - `.primary(...)` and `.secondary(...)`: `src/app_shell/spec.rs:300-319`.
  - `.logo(...)`, `.helper(...)`, `.info(...)`: `src/app_shell/spec.rs:322-337`.
- Visibility logic in renderer:
  - Primary button only renders if `primary_label` is not empty: `src/app_shell/shell.rs:303-310`.
  - Secondary button only renders if `secondary_label` is `Some`: `src/app_shell/shell.rs:312-328`.

## 3) How actions are defined for prompts

### Protocol-level action model (SDK -> app)
- `ProtocolAction` schema (name/shortcut/value/has_action/visible/close): `src/protocol/types.rs:666-723`.
- Action routing semantics (`has_action` true -> `ActionTriggered`, false -> submit value): `src/protocol/message.rs:1043-1067`.

### Prompt messages carrying actions
- `Message::Arg` + `Message::Div` include `actions: Option<Vec<ProtocolAction>>`: `src/protocol/message.rs:63-103`.
- `Message::Editor` includes `actions`: `src/protocol/message.rs:137-155`.
- `Message::Fields` + `Message::Form` include `actions`: `src/protocol/message.rs:210-228`.
- `Message::Chat` includes `actions`: `src/protocol/message.rs:277-312`.
- `Message::Term` includes `actions`: `src/protocol/message.rs:410-419`.
- Runtime updates: `Message::SetActions` replaces the current action list: `src/protocol/message.rs:1043-1053`.

### How the app stores and uses actions
- Prompt handler stores SDK actions and registers shortcuts (only for visible actions): `src/prompt_handler.rs:39-58`.
- `setActions` updates the `ActionsDialog` if open and rebuilds shortcuts: `src/prompt_handler.rs:1880-1914`.
- Prompt UIs show/hide the Actions footer button based on `has_actions` and wire clicks to open the ActionsDialog, e.g. `render_prompts/arg.rs:462-503`.
- The actual Actions dialog open/close behavior is centralized in `toggle_arg_actions`: `src/app_impl.rs:3490-3543`.

## 4) Architecture for adding new footer buttons

### A) PromptFooter (used by most prompts)
1. Extend the configuration and callbacks:
   - Add fields to `PromptFooterConfig` for the new button label/shortcut/visibility and an `on_..._click` callback in `PromptFooter`.
   - Locations: `src/components/prompt_footer.rs:91-223`.
2. Render the new button:
   - Add a new `render_button(...)` call and update divider logic in `PromptFooter::render`.
   - Location: `src/components/prompt_footer.rs:286-387`.
3. Wire prompt-specific actions:
   - Update prompt renderers (e.g., `src/render_prompts/arg.rs`, `src/render_prompts/editor.rs`, `src/render_prompts/term.rs`, etc.) to set the new label/shortcut and provide the click handler.
   - Ensure key handling matches the displayed shortcut (many prompts already handle Cmd+K / Enter in their key handlers).
4. Optional: update stories/tests if you want visual coverage
   - Footer layout stories live in `src/stories/footer_*_variations.rs`.

### B) AppShell footer (ShellSpec/Footerspec usage)
1. Extend `FooterSpec` with new fields + builder method:
   - `src/app_shell/spec.rs:274-338`.
2. Render the new button in `AppShell::render_footer`:
   - `src/app_shell/shell.rs:250-334`.
3. Update any view that constructs `ShellSpec.footer(...)` to pass the new fields.
4. Update tests for the spec builder (footer tests live in `src/app_shell/tests.rs:221-243`).

### C) Action wiring (if the new button should trigger SDK actions)
- Add the action to `ProtocolAction` lists (SDK side) or `SetActions`, then use `toggle_arg_actions` or a new handler to open dialogs or submit values.
- Ensure visibility and shortcut registration honor `ProtocolAction::is_visible`: `src/prompt_handler.rs:46-58`.

