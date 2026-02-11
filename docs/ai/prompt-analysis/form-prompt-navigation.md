# Form Prompt Navigation Analysis

## Scope

- `src/render_prompts/form/render.rs`
- `src/components/prompt_footer.rs`
- `src/components/focusable_prompt_wrapper.rs`
- Comparison references from `src/form_prompt.rs`
- Comparison references from `src/components/form_fields/*`
- Comparison references from `src/prompts/env/*`
- Comparison references from `src/prompts/template/*`

## Current Model

### Keyboard ownership

FormPrompt uses one parent-level `on_key_down` listener at:
`src/render_prompts/form/render.rs:38` and
`src/render_prompts/form/render.rs:198`.

The handler processes keys in this order:

1. Global dismiss shortcuts when actions are closed
   (`src/render_prompts/form/render.rs:54`).
1. Cmd+K actions toggle (`src/render_prompts/form/render.rs:66`).
1. Actions-dialog routing (`src/render_prompts/form/render.rs:75`).
1. Action shortcuts (`src/render_prompts/form/render.rs:96`).
1. Enter submit/newline decision (`src/render_prompts/form/render.rs:115`).
1. Tab/Shift+Tab index movement (`src/render_prompts/form/render.rs:142`).
1. Fallback key forwarding to active field
   (`src/render_prompts/form/render.rs:160`).

### Enter behavior

Enter behavior is centralized in `form_enter_behavior`:
`src/render_prompts/form/helpers.rs:11`.

Rules:

1. Non-textarea focused field: Enter submits.
1. Textarea focused field: Enter inserts newline.
1. Textarea submit requires Cmd+Enter.

Submit-time validation only checks email and number formats:
`src/render_prompts/form/helpers.rs:104` and
`src/render_prompts/form/helpers.rs:117`.

Validation failures are shown as transient HUD text:
`src/render_prompts/form/render.rs:122`.

### Focus delegation

`FormPromptState` stores `focused_index` and uses it for:

1. Tab navigation (`src/form_prompt.rs:118`).
1. Shift+Tab navigation (`src/form_prompt.rs:127`).
1. Key forwarding target (`src/form_prompt.rs:158`).

Fields own GPUI focus handles via `track_focus`:

- `src/components/form_fields/text_field/render.rs:168`
- `src/components/form_fields/text_area/render.rs:116`
- `src/components/form_fields/checkbox.rs:147`

Mouse clicks focus field handles, but do not write back to
`focused_index`:

- `src/components/form_fields/text_field/render.rs:160`
- `src/components/form_fields/text_area/render.rs:52`

### Footer affordances

FormPrompt footer currently uses:

1. Primary: `Continue`.
1. Secondary: `Actions`.
1. Helper text: Enter hint from focused field snapshot.

Reference: `src/render_prompts/form/render.rs:213`.

`PromptFooter` supports two action slots only:
`src/components/prompt_footer.rs:184`.

## UX Unpredictability

### 1) FormPrompt bypasses shared two-level key routing

Most form-like prompts use `FocusablePrompt::build` with:

1. App-level intercept for Escape/Cmd+W/Cmd+K.
1. Entity-level key handling for prompt-local behavior.

Reference: `src/components/focusable_prompt_wrapper.rs:92`.

FormPrompt inlines all behavior in one app-level listener
(`src/render_prompts/form/render.rs:38`), so it differs from the
common prompt mental model.

### 2) Focus index can diverge from real focused field

`focused_index` changes only on Tab/Shift+Tab, while mouse focus
changes do not update that index.

Enter mode uses `focused_index` to decide submit vs newline:
`src/render_prompts/form/helpers.rs:28`.

This makes clicked textarea behavior potentially inconsistent if the
index is stale.

### 3) Tab updates index but does not explicitly focus next handle

`focus_next` and `focus_previous` only change index and notify:
`src/form_prompt.rs:122` and `src/form_prompt.rs:136`.

There is no explicit call to focus the next field handle, so behavior
depends on implicit focus propagation.

### 4) Validation is transient and global, not local and persistent

FormPrompt reports invalid fields through HUD text at submit time.

Other form-like prompts keep inline validation state and render
messages near the field:

- Env: `src/prompts/env/prompt.rs:109`, `src/prompts/env/render.rs:347`
- Template: `src/prompts/template/prompt.rs:360`,
  `src/prompts/template/render.rs:166`

### 5) Footer semantics differ from other input prompts

EnvPrompt uses secondary footer action for Cancel/Esc:
`src/prompts/env/render.rs:290` and `src/prompts/env/render.rs:320`.

FormPrompt uses secondary for Actions and leaves cancel implicit via
Esc/global handling.

### 6) Footer submit hint can be stale

Helper text is computed from a snapshot at render start:
`src/render_prompts/form/render.rs:20` and
`src/render_prompts/form/render.rs:216`.

Field-level focus changes can happen without parent rerender, so the
Enter hint can lag behind actual focus.

## Proposed Consistent Form Structure

### Keyboard and focus contract

1. Use `FocusablePrompt` in FormPrompt so routing matches other prompts.
1. Keep app-level intercept for Escape/Cmd+W/Cmd+K.
1. Keep entity-level handling for Tab/Enter/input.
1. Update `focused_index` on mouse focus/click.
1. On Tab and Shift+Tab, update index and explicitly focus target handle.
1. Resolve Enter behavior from actual focused field entity.

### Validation and error placement contract

1. Store per-field errors in FormPrompt state.
1. Render each field error directly below its field row.
1. On submit failure, focus first invalid field.
1. Clear only the edited field error on change.
1. Use optional persistent summary above footer
   (example: `2 fields need attention`).

### Footer and action placement contract

1. Footer primary: Submit/Continue.
1. Footer secondary: Cancel (Esc) for form-like prompts.
1. Keep Cmd+K actions globally available.
1. Show actions affordance outside submit/cancel pair.
1. If no tertiary slot exists, show explicit helper text
   (`Cmd+K Actions`).

## Recommended Implementation Order

1. Move FormPrompt to `FocusablePrompt` two-level routing.
1. Wire field focus/click to synchronize `focused_index`.
1. Make Tab/Shift+Tab explicitly focus the next handle.
1. Add per-field validation state and inline error rendering.
1. Focus first invalid field on submit failure.
1. Switch footer secondary to Cancel and keep actions via Cmd+K.

## Tests To Add During Implementation

1. `test_form_prompt_click_focus_updates_focused_index`
1. `test_form_prompt_tab_moves_focus_handle_and_index`
1. `test_form_prompt_enter_in_clicked_textarea_inserts_newline_without_cmd`
1. `test_form_prompt_submit_focuses_first_invalid_field_and_renders_inline_error`
1. `test_form_prompt_footer_shows_cancel_secondary_and_actions_hint`

These target the current unpredictability points directly.
