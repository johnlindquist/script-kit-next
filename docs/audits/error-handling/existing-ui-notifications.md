# Existing UI Status/Notification Mechanisms (Inventory)

Scope searched:

- `src/render_prompts/*.rs`
- `src/prompts/*.rs` (including prompt submodules under this path)

## Summary

Within the scoped files, status/error communication is distributed across:

- inline validation and error text
- footer helper/status text
- transient HUD overlays (`show_hud`)
- actions popup overlays (modal-like)
- prompt-specific loading/progress placeholders

No dedicated `Toast`, `Snackbar`, `Banner`, or notification-center primitive was found in this scope.

## Mechanisms Found

### 1) Actions popup overlay (modal-like UI)

What it is:

- Prompt-local overlay dialog toggled by `show_actions_popup` and rendered from `actions_dialog`.

Where used:

- `src/render_prompts/arg/render.rs:457`
- `src/render_prompts/form/render.rs:234`
- `src/render_prompts/div.rs:201`
- `src/render_prompts/editor.rs:314`
- `src/render_prompts/term.rs:315`
- `src/render_prompts/path.rs:204`

Interaction model:

- Most prompts render a backdrop and close on backdrop click:
  - `src/render_prompts/arg/render.rs:484`
  - `src/render_prompts/form/render.rs:259`
  - `src/render_prompts/div.rs:226`
  - `src/render_prompts/editor.rs:339`
  - `src/render_prompts/term.rs:340`
- Path prompt uses custom key routing when actions are open:
  - `src/render_prompts/path.rs:273`

Status semantics:

- This is a command/action surface, not a success/warn/error surface by itself.

### 2) Footer helper/status text (instructional state)

What it is:

- Shared footer copy helpers for “running/waiting/what to do next” messaging.

Where defined/used:

- Helpers in:
  - `src/render_prompts/arg/helpers.rs:10`
  - `src/render_prompts/arg/helpers.rs:23`
- Used in prompt renderers:
  - `src/render_prompts/arg/render.rs:417`
  - `src/render_prompts/form/render.rs:216`
  - `src/render_prompts/div.rs:125`
  - `src/render_prompts/editor.rs:275`
  - `src/render_prompts/other.rs:222`
- Env running status copy:
  - `src/prompts/env/helpers.rs:60`
  - rendered in `src/prompts/env/render.rs:214`

Status semantics:

- Primarily informational/instructional.
- No explicit severity taxonomy (success/warn/error) in footer helper APIs.

### 3) Transient HUD messages (`show_hud`)

What it is:

- Ephemeral, timeout-based feedback for immediate user correction.

Where used:

- Arg empty submit guard:
  - `src/render_prompts/arg/helpers.rs:194`
- Form submit validation aggregate:
  - `src/render_prompts/form/render.rs:123`

Status semantics:

- Error-like validation feedback, but transient only and not persistent inline.

### 4) Inline validation/error text

What it is:

- Persistent inline text rendered near the relevant control.

Where used:

- Env prompt:
  - state: `src/prompts/env/prompt.rs:35`
  - render: `src/prompts/env/render.rs:190`
  - color token: `src/prompts/env/render.rs:196`
- Template prompt:
  - state: `src/prompts/template/prompt.rs:17`
  - render: `src/prompts/template/render.rs:166`
  - color token source: `src/prompts/template/render.rs:35`
- Chat turn errors:
  - render path: `src/prompts/chat/render_turns.rs:53`
  - error classification/messages:
    - `src/prompts/chat/types.rs:434`
  - retry affordance:
    - `src/prompts/chat/render_turns.rs:66`

Status semantics:

- Error is explicit in Env/Template/Chat.
- Chat is the richest implementation (typed error mapping + retry eligibility).

### 5) Success indicators

Where used:

- Env configured-state checkmark and success color:
  - `src/prompts/env/render.rs:241`
  - `src/prompts/env/render.rs:244`
- Chat script-generation status uses success/error color branch:
  - `src/prompts/chat/render_core.rs:224`
  - `src/prompts/chat/render_core.rs:228`

Status semantics:

- Success and error are represented in targeted prompts, but not through one shared status system.

### 6) Progress/loading indicators

Where used:

- Chat provider-loading placeholder:
  - `src/prompts/chat/render_core.rs:435`
  - `src/prompts/chat/render_core.rs:460`
  - `src/prompts/chat/render_core.rs:466`
- Chat streaming pending output:
  - “Thinking...” when stream has no content:
    - `src/prompts/chat/render_turns.rs:117`
  - streaming cursor indicator:
    - `src/prompts/chat/render_turns.rs:133`
- Webcam state label while initializing/error/no buffer:
  - state labels: `src/prompts/webcam.rs:90`
  - placeholder render: `src/prompts/webcam.rs:119`
  - unsupported/error stub fallback: `src/prompts/webcam_stub.rs:44`

Status semantics:

- Progress exists as prompt-specific copy/placeholder, not a shared spinner/progress component.

### 7) Empty-state informational copy

Where used:

- Select prompt “no choices” messaging:
  - `src/prompts/select/render.rs` (empty-state copy)
- Arg prompt no-match hint:
  - `src/render_prompts/arg/render.rs` (typed-value/no-match helper copy)
- Drop prompt file-count status:
  - `src/prompts/drop.rs` (dropped files message)

Status semantics:

- Informational only, no severity model.

## Severity/State Distinction in Current UX

Observed distinction patterns:

- Success:
  - explicit in Env and chat script-generation status.
- Error:
  - explicit inline errors (Env/Template/Chat), transient HUD validation (Arg/Form), webcam error states.
- Warning:
  - chat user-facing error messages are prefixed with warning symbol via `ChatErrorType::display_message`:
    - `src/prompts/chat/types.rs:436`
  - no shared warn styling/token contract observed.
- Progress:
  - chat loading and streaming placeholders.
  - webcam initializing label.

## Inconsistencies / Ambiguities

1. Validation feedback channel is inconsistent.

- Arg/Form use transient HUD (`show_hud`) for validation failures.
- Env/Template use persistent inline errors tied to fields.
- Effect: similar user mistakes surface through different persistence/visibility models.

2. Error color tokening is inconsistent.

- Env and chat use dedicated error color tokens (`ui.error`/design error colors).
- Template error text uses `theme.colors.accent.selected`:
  - `src/prompts/template/render.rs:35`
- Effect: “error” can look like selection/accent state rather than a dedicated error state.

3. Actions overlay behavior differs across prompts.

- Arg/Form/Div/Editor backdrops include `.cursor_pointer()`:
  - `src/render_prompts/arg/render.rs:487`
  - `src/render_prompts/form/render.rs:262`
  - `src/render_prompts/div.rs:229`
  - `src/render_prompts/editor.rs:342`
- Term backdrop omits `.cursor_pointer()`:
  - `src/render_prompts/term.rs:343`
- Path overlay rendering does not include the same full-screen clickable backdrop close pattern:
  - `src/render_prompts/path.rs:401`
- Effect: popup dismissal affordance and clickability cues vary by prompt.

4. Footer status is informational but lacks shared severity semantics.

- `prompt_footer_config_with_status` and related helpers carry helper text but no success/warn/error typing:
  - `src/render_prompts/arg/helpers.rs:23`
- Effect: footer status can read like “status” without strong visual severity semantics.

5. Retry affordance exists in chat but not in other error surfaces.

- Chat maps typed errors and conditionally shows Retry:
  - `src/prompts/chat/render_turns.rs:56`
  - `src/prompts/chat/render_turns.rs:66`
- Other prompts mostly present static error text or HUD.
- Effect: recoverability expectations differ by prompt type.

## Explicitly Not Found in Scope

- Dedicated toast primitive (`Toast`, `toast`)
- Snackbar primitive (`Snackbar`)
- Notification center construct (`NotificationCenter`, `notification_center`)
- Named banner primitive (`Banner`) as a reusable status component

These terms/components were not found in `src/render_prompts/*.rs` and `src/prompts/*.rs` during this audit.
