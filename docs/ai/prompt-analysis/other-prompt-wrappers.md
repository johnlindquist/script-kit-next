# Other Prompt Wrappers: Shortcut, Dismissability, and Actions Ownership

## Scope
Reviewed:
- `src/render_prompts/other.rs`
- `src/components/prompt_layout_shell.rs`
- `src/components/prompt_footer.rs`
- `src/components/focusable_prompt_wrapper.rs`

Cross-checked against shared routing state:
- `src/app_impl/shortcuts_hud_grid.rs`
- `src/app_impl/startup_new_actions.rs`
- `src/app_impl/actions_dialog.rs`
- representative prompt entities (`select/env/drop/template/chat/webcam`)

## Current Behavior Matrix

| Wrapper | Key handler in wrapper | Dismissability flag passed to global handler | Actions toggle path | Modal key routing when actions open | Actions surface model |
|---|---|---|---|---|---|
| Select (`render_select_prompt`) | `other_prompt_shell_handle_key_default` | `true` | `Cmd+K -> toggle_arg_actions` if `sdk_actions.is_some()` | none in wrapper | no inline overlay in wrapper |
| Env (`render_env_prompt`) | `other_prompt_shell_handle_key_default` | `true` | same as Select | none in wrapper | no inline overlay in wrapper |
| Drop (`render_drop_prompt`) | `other_prompt_shell_handle_key_default` | `true` | same as Select | none in wrapper | no inline overlay in wrapper |
| Template (`render_template_prompt`) | `other_prompt_shell_handle_key_default` | `true` | same as Select | none in wrapper | no inline overlay in wrapper |
| Chat (`render_chat_prompt`) | `other_prompt_shell_handle_key_chat` | `true` | `Cmd+K -> toggle_chat_actions` | explicit `route_key_to_actions_dialog(..., ChatPrompt, ...)` | separate actions window (no inline overlay) |
| Webcam (`render_webcam_prompt`) | `other_prompt_shell_handle_key_webcam` | `true` (but only evaluated when popup closed) | footer click -> `toggle_webcam_actions`; global interceptor also handles `Cmd+K` | none in wrapper | separate actions window (no inline overlay) |

## Key Inconsistencies

1. Dismissability source-of-truth is split.
- `other.rs` hardcodes `handle_global_shortcut_with_options(..., true, ...)` for all six wrappers.
- `is_dismissable_view()` marks `EnvPrompt` and `WebcamView` as non-dismissable (`src/app_impl/shortcuts_hud_grid.rs`).
- Result: wrapper code and app-level policy can disagree.

2. `Cmd+K` ownership is inconsistent.
- Default wrapper handler toggles arg actions for select/env/drop/template.
- Chat toggles in wrapper and is also covered by central interceptor host mapping.
- Webcam relies on central interceptor for `Cmd+K`, but also has a footer actions button in wrapper.

3. Modal actions routing is inconsistent.
- Chat wrapper routes keys via `route_key_to_actions_dialog`.
- Webcam wrapper does not; central interceptor handles its host.
- Select/env/drop/template wrappers neither route locally nor appear in central host mapping.

4. Behavior while actions popup is open differs.
- Default/chat handlers still call global shortcut logic (Escape is guarded internally by `!show_actions_popup`, but `Cmd+W` still applies).
- Webcam handler skips global shortcut handling entirely while popup is open.

5. Overlay expectations vary by wrapper family without explicit contract.
- Div/Form/Arg/Path wrappers include inline overlay/backdrop patterns.
- Chat/Webcam use separate actions windows.
- Select/env/drop/template wrappers do not render overlays but can still invoke arg-style actions toggle.

6. Shell abstraction is layout-only, but wrapper comments imply key/focus guarantees.
- `prompt_shell_container`/`prompt_shell_content` only normalize frame/layout.
- Key ownership and action semantics are still ad hoc in each renderer.

## Unified Wrapper Mental Model

### What wrappers must always provide

1. Frame contract.
- Use shared shell container/content helpers (or equivalent explicit layout) for consistent sizing/overflow/rounding.

2. Global shortcut gate with policy from app state.
- Wrapper calls global shortcut handling through a single policy input derived from current view dismissability (not hardcoded literals).
- Rule: Escape dismisses only when current view is dismissable and no modal actions UI is active.

3. Explicit actions capability declaration.
- Wrapper must declare one of:
  - `NoActions`
  - `ActionsInline { host }`
  - `ActionsWindow { host }`
- `Cmd+K` handling, footer secondary label, and modal key routing must all derive from this declaration.

4. Single owner for actions-modal key routing.
- For each host, choose exactly one routing owner:
  - wrapper-local `route_key_to_actions_dialog`, or
  - central interceptor mapping.
- Avoid duplicate ownership for the same host.

5. Consistent popup-open shortcut semantics.
- Define and apply one policy for `Cmd+W` when actions are open (either always allowed or always blocked), shared across wrappers.

### What prompt entities must own

1. Domain-specific input/navigation behavior.
- Arrow, tab, enter/submit, character input, local validation.

2. Prompt-specific cancel semantics.
- If Escape means a domain action (`submit_cancel`, stop streaming, etc.), entity owns that behavior.
- Generic window dismissal stays wrapper/app-level.

3. Local focus model.
- Entity chooses FocusablePrompt or direct key handlers for inner behavior; wrapper should not re-implement domain keys.

4. Content/footer presentation details.
- Labels/status/hints belong to entity-level prompt UI.
- If an actions button is shown, callback should delegate to wrapper/app action capability instead of inventing local action state.

## Recommended Mapping for Other Wrappers

- Select/Env/Drop/Template:
  - default `NoActions` unless explicitly configured otherwise.
  - if actions are supported, they should declare a concrete host + routing owner, not rely on implicit arg-toggle behavior.

- Chat/Webcam:
  - `ActionsWindow` hosts (`ChatPrompt`, `WebcamPrompt`).
  - keep routing ownership centralized or wrapper-local, but not both.

## Practical Checklist for Future Wrapper Changes

When adding or modifying a non-arg wrapper:
1. Choose dismissability from app-level policy, do not hardcode.
2. Declare action capability (`NoActions` / `ActionsInline` / `ActionsWindow`).
3. Wire `Cmd+K`, footer secondary action, and modal key routing from that same declaration.
4. Ensure popup-open behavior for Escape/Cmd+W matches shared policy.
5. Keep prompt-specific input logic in the prompt entity.
