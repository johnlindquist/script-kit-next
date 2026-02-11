# Other Prompt Shell Wrapper Audit

## Scope
- Target: `src/render_prompts/other.rs`
- Wrappers reviewed: `render_select_prompt`, `render_env_prompt`, `render_drop_prompt`, `render_template_prompt`, `render_chat_prompt`, `render_webcam_prompt`
- Comparison references:
  - Shared shell helper in `src/components/prompt_layout_shell.rs:79`
  - Prompt wrappers with inline actions overlays in `src/render_prompts/div.rs:134`, `src/render_prompts/form/render.rs:185`, `src/render_prompts/editor.rs:244`, `src/render_prompts/term.rs:282`
  - Global actions interceptor in `src/app_impl/startup_new_actions.rs:32`
  - Dismissability policy in `src/app_impl/shortcuts_hud_grid.rs:74`

## Current Wrapper Matrix (`other.rs`)

| Prompt | Outer shell construction | Radius/background | Sizing | Focus owner | Key interception | Actions UI mode |
|---|---|---|---|---|---|---|
| Select | `prompt_shell_container(...).child(prompt_shell_content(entity))` (`other.rs:119`) | Shared radius token + `get_vibrancy_background` (`other.rs:112`, `other.rs:115`) | Implicit `h_full` from helper (`prompt_layout_shell.rs:45`) | Entity (commented) | `other_prompt_shell_handle_key_default` (`other.rs:18`) | `Cmd+K` -> `toggle_arg_actions` if `sdk_actions.is_some()` (`other.rs:30`) |
| Env | Same as Select (`other.rs:137`) | Same (`other.rs:130`, `other.rs:133`) | Same | Entity | Same default handler | Same as Select |
| Drop | Same as Select (`other.rs:155`) | Same (`other.rs:148`, `other.rs:151`) | Same | Entity | Same default handler | Same as Select |
| Template | Same as Select (`other.rs:173`) | Same (`other.rs:166`, `other.rs:169`) | Same | Entity | Same default handler | Same as Select |
| Chat | Same shared shell (`other.rs:191`) | Same (`other.rs:184`, `other.rs:187`) | Same | Entity | Chat-specific handler (`other.rs:41`) with `route_key_to_actions_dialog` (`other.rs:63`) | Separate actions window via `toggle_chat_actions` |
| Webcam | Custom `div()` shell (`other.rs:232`) | Radius/vibrancy manually applied (`other.rs:205`, `other.rs:209`) | Explicit `STANDARD_HEIGHT` (`other.rs:212`, `other.rs:237`) | Wrapper (`track_focus`) (`other.rs:240`) | Webcam-specific handler (`other.rs:91`) | Separate actions window via `toggle_webcam_actions` |

## Findings

1. **High: Dismiss semantics conflict for webcam**
- `other_prompt_shell_handle_key_webcam` calls `handle_global_shortcut_with_options(..., true, ...)` (`other.rs:103`), so `Escape` is treated as dismissable.
- Global policy marks `WebcamView` as non-dismissable (`shortcuts_hud_grid.rs:82`).
- This creates a navigation contract mismatch: blur behavior and key behavior disagree on whether webcam is dismissable.

2. **High: Default handler can open actions without wrapper-level modal routing**
- Default handler toggles arg actions on `Cmd+K` (`other.rs:30`), used by select/env/drop/template.
- That handler does **not** call `route_key_to_actions_dialog` (`other.rs:18-38`), unlike chat/form/div/editor/term wrappers.
- Global interceptor host mapping excludes select/env/drop/template (`startup_new_actions.rs:204-215`), so these wrappers do not have the same explicit modal routing path as other prompts.

3. **Medium: Three different key-routing models exist inside one file**
- Default model (select/env/drop/template): global shortcuts + optional `Cmd+K`.
- Chat model: wrapper-level `Cmd+K` + wrapper-level actions routing + global shortcuts.
- Webcam model: global shortcuts only in wrapper, with `Cmd+K` delegated to global interceptor.
- This divergence makes behavior harder to predict and increases surprise when switching prompts.

4. **Medium: Shell construction split (shared helper vs manual shell)**
- Five wrappers use `prompt_shell_container`/`prompt_shell_content` (`other.rs:119`, `other.rs:137`, `other.rs:155`, `other.rs:173`, `other.rs:191`).
- Webcam duplicates shell assembly manually (`other.rs:232-271`), including radius/bg/flex/overflow setup.
- This invites visual drift over time (e.g., missing `relative`/key-context conventions compared with helper-based shells).

5. **Medium: Height policy is mixed and mostly implicit**
- Shared helper path relies on `h_full` (`prompt_layout_shell.rs:45`), while webcam hardcodes `STANDARD_HEIGHT` (`other.rs:212`).
- Other wrappers (editor/term/div/form) typically set explicit heights in wrapper root.
- Implicit vs explicit sizing in neighboring wrappers makes visual sizing behavior less obvious and harder to enforce via tests.

6. **Low: Global shortcut gating differs by wrapper**
- Webcam guards global shortcut handling behind `!show_actions_popup` (`other.rs:102`).
- Default/chat handlers call global shortcut helper without this explicit guard (`other.rs:37`, `other.rs:87`), relying on helper internals and/or prior routing.
- Behavior is probably equivalent for `Escape`, but the inconsistency obscures intent.

## Canonical Wrapper Contract (Proposed)

1. **Single shell primitive**
- All `other.rs` wrappers should use one shell constructor (directly or via helper) that guarantees:
  - `flex_col + w_full + overflow_hidden`
  - rounded radius from design tokens
  - vibrancy background only via `get_vibrancy_background`
  - optional explicit height override

2. **Explicit height policy per wrapper**
- Declare a per-wrapper height mode:
  - `FillParent` for select/env/drop/template/chat
  - `Fixed(layout::STANDARD_HEIGHT)` for webcam
- Avoid implicit sizing differences hidden inside ad-hoc shell code.

3. **Exactly one focus owner**
- If entity already tracks focus (select/env/drop/template/chat), wrapper does not track focus.
- If entity does not (webcam), wrapper must track focus and define key context.

4. **Unified key-routing order**
- For all wrappers:
  1. Hide mouse cursor.
  2. If actions popup is open: route via `route_key_to_actions_dialog(host, ...)` and return when handled.
  3. If `Cmd+K` and host supports actions: toggle host-specific actions.
  4. If popup is not open: apply `handle_global_shortcut_with_options(event, dismissable, ...)`.
- This mirrors form/div/editor/term patterns and removes per-wrapper routing surprises.

5. **Dismissability source of truth**
- Wrapper `dismissable` flag must align with app-level policy (`is_dismissable_view`) for that view.
- If a prompt is intentionally non-dismissable globally (e.g., webcam/env), wrapper global shortcut path should use `dismissable=false`.
- Entity-local `Escape` behavior can still submit/cancel when intentionally different from global window-dismiss behavior.

6. **Actions presentation contract**
- Inline actions hosts: wrapper includes absolute overlay/backdrop pattern.
- Separate-window hosts (chat/webcam): wrapper omits inline overlay but still uses shared host routing for open/close/execute semantics.
- Avoid duplicate `Cmd+K` ownership between global interceptor and wrapper/entity layers.

## Suggested Follow-up Checks
- Add source-level tests in `src/render_prompts/other.rs` for:
  - per-wrapper dismissable flag expectations
  - presence/absence of `route_key_to_actions_dialog` by host support
  - shell construction invariants (shared helper usage and explicit height policy)
- Decide whether `Cmd+K` ownership lives in global interceptor or wrapper layer, then remove the duplicate path.
