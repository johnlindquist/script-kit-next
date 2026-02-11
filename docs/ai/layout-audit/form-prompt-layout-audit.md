# Form Prompt Layout Audit

## Scope
- `src/render_prompts/form.rs`
- `src/render_prompts/form/helpers.rs`
- `src/render_prompts/form/render.rs`
- Supporting field composition sources used by `render_form_prompt`:
  - `src/form_prompt.rs`
  - `src/components/form_fields/text_field/render.rs`
  - `src/components/form_fields/text_area/render.rs`
  - `src/components/form_fields/checkbox.rs`
- Template comparison baseline:
  - `src/prompts/template/render.rs`
  - `src/render_prompts/other.rs` (`render_template_prompt`)

## Current Form Prompt Composition

### Shell and Slots (`src/render_prompts/form/render.rs`)
- Root surface is assembled inline (not via `prompt_shell_container`): `relative + flex_col + w_full + rounded + overflow_hidden` with dynamic height (`src/render_prompts/form/render.rs:185`).
- Height is derived from field count (`base_height + 60px * fields`, capped at `700px`) (`src/render_prompts/form/render.rs:173`).
- Body slot is `flex_1 + min_h(0) + overflow_y_scrollbar + p(padding_xl)` and mounts `FormPromptState` directly (`src/render_prompts/form/render.rs:200`).
- Footer slot uses `PromptFooter` and contextual status text (Enter vs Cmd+Enter for textarea) (`src/render_prompts/form/render.rs:210`, `src/render_prompts/form/helpers.rs:36`).
- Overlay slot is custom actions-dialog backdrop and positioned dialog (`src/render_prompts/form/render.rs:232`).

### Field Stack and Row Shapes (`src/form_prompt.rs` + `src/components/form_fields/*`)
- `FormPromptState::render` builds a flat vertical stack (`gap(px(16.))`) with no sectioning/group headers (`src/form_prompt.rs:228`).
- `FormTextField` row is two-column horizontal layout:
  - fixed label width `7.5rem`
  - input fills remaining width
  (`src/components/form_fields/text_field/render.rs:131`).
- `FormTextArea` uses the same label width and a fixed-height editor (`rows * 1.5rem + 1rem`) with its own `overflow_y_scrollbar` (`src/components/form_fields/text_area/render.rs:34`, `src/components/form_fields/text_area/render.rs:130`).
- `FormCheckbox` preserves alignment by inserting an empty `7.5rem` label spacer before checkbox+label content (`src/components/form_fields/checkbox.rs:158`).

### Validation Messaging Placement
- Submit-time validation is centralized in `render_form_prompt` helpers (`email` and `number`) (`src/render_prompts/form/helpers.rs:67`, `src/render_prompts/form/helpers.rs:95`).
- On invalid submit, user gets one HUD message listing invalid fields; no inline row-level error slot is rendered (`src/render_prompts/form/render.rs:121`).
- Field controls do not render error/support text rows beneath inputs.

### Scroll Containment
- Primary body scroll owner: wrapper content viewport (`overflow_y_scrollbar`) (`src/render_prompts/form/render.rs:205`).
- Secondary scroll owner: each textarea control (`overflow_y_scrollbar`) (`src/components/form_fields/text_area/render.rs:131`).
- This creates nested vertical scroll regions when multi-line fields are present.

## TemplatePrompt Baseline (Comparison Target)

### Shell Pattern
- Template wrapper uses shared shell helpers (`prompt_shell_container` + `prompt_shell_content`) in `render_template_prompt` (`src/render_prompts/other.rs:161`).
- Template entity itself uses `w_full + h_full + p(spacing.padding_lg)` and owns all internal layout zones (`src/prompts/template/render.rs:39`).

### Field and Message Pattern
- Template rows are `flex_col` wrappers with:
  - top row: label column (`140px`) + input cell
  - optional second row: inline error text under the control column (`pl(144px)`)
  (`src/prompts/template/render.rs:133`, `src/prompts/template/render.rs:166`).
- Template includes explicit helper copy (field count instructions, naming tip, keyboard help) as in-flow text blocks (`src/prompts/template/render.rs:74`, `src/prompts/template/render.rs:185`, `src/prompts/template/render.rs:195`).

## Findings Against Requested Audit Areas

### 1) Field Layout Composition
- Form prompt has stable row alignment, but lacks a semantic row wrapper with dedicated support/error slot.
- Template rows are structurally richer (`field row` + `message row`) and easier to extend.
- Form field rhythm is mostly hard-coded (`gap(px(16.))`, `7.5rem`, `rounded(px(6.))`) vs template using spacing tokens.

### 2) Validation Messaging Placement
- Form validation appears as transient global HUD text only.
- Template keeps validation local to the field with consistent indent anchoring.
- Result: form gives lower spatial locality for correction and no persistent error context in the form body.

### 3) Spacing Rhythm
- Form: one global inter-field gap (`16px`) + per-control local paddings.
- Template: explicit section rhythm (`padding_lg`, `padding_md`, `padding_sm`) and distinct copy tiers.
- Result: template has clearer visual cadence between intro text, group headers, rows, and supporting hints.

### 4) Scroll Containment
- Form uses body scroll and control-level textarea scroll concurrently.
- Template has no nested control scroll in row rendering.
- Result: form is more prone to wheel/trackpad scroll handoff friction in dense/multiline forms.

## Proposed Unified Form-Layout Vocabulary

Use one shared vocabulary across `FormPrompt` and `TemplatePrompt` renderers:

- `form_surface`: outer rounded/vibrancy-aware shell, overlay anchor.
- `form_header_zone`: optional preview/title/instructions block.
- `form_body_viewport`: the single primary vertical scroll owner.
- `form_field_stack`: ordered field rows with standard inter-row rhythm.
- `form_field_row`: one label/control pair (type-agnostic).
- `form_field_support`: inline validation/hint row aligned to control column.
- `form_footer_zone`: fixed action/status footer.
- `form_overlay_zone`: actions dialog/backdrop layer.

Tokenized geometry primitives to support that vocabulary:
- `form_label_column_width`
- `form_row_gap`
- `form_section_gap`
- `form_body_padding`
- `form_support_indent` (`label_width + row_gap`)
- `form_max_surface_height`

## Proposed Component Hierarchy Mapping

Canonical hierarchy (top to bottom):

1. `PromptFormShell`
- Owns: `form_surface`, `form_footer_zone`, `form_overlay_zone`.
- Maps from current code:
  - Form: `src/render_prompts/form/render.rs`
  - Template: `src/render_prompts/other.rs` + `prompt_shell_container`

2. `PromptFormViewport`
- Owns: `form_header_zone` + `form_body_viewport` scroll policy.
- Maps from current code:
  - Form: wrapper content div in `src/render_prompts/form/render.rs:200`
  - Template: root container in `src/prompts/template/render.rs:39`

3. `PromptFormFieldStack`
- Owns: field ordering and row spacing rhythm.
- Maps from current code:
  - Form: `src/form_prompt.rs:228`
  - Template: in-loop row assembly in `src/prompts/template/render.rs:85`

4. `PromptFormFieldRow`
- Owns: label column + control slot + focus styling for each field type.
- Maps from current code:
  - FormTextField: `src/components/form_fields/text_field/render.rs:131`
  - FormTextArea: `src/components/form_fields/text_area/render.rs:88`
  - FormCheckbox: `src/components/form_fields/checkbox.rs:142`
  - Template pseudo-row: `src/prompts/template/render.rs:139`

5. `PromptFormFieldSupport`
- Owns: inline validation/hint/help row aligned under control column.
- Maps from current code:
  - Template inline error: `src/prompts/template/render.rs:166`
  - Form currently missing (global HUD only)

## Normalization Direction

To converge FormPrompt and TemplatePrompt behavior without changing business logic:

1. Move both prompt shells onto one shell API (`prompt_shell_container` family) with explicit slot names.
2. Extract one shared field-row primitive (label column width + support indent tokens) and apply to both form and template rows.
3. Add inline support-message slot for form field rows; keep HUD as optional summary, not sole validation surface.
4. Define one vertical scroll owner for the body; keep control-level scrolling only for bounded editors when strictly needed.
5. Centralize form spacing tokens so row rhythm is token-driven instead of mixed literals.

## Risk Notes
- Dynamic height in `render_form_prompt` assumes `60px` per field (`src/render_prompts/form/render.rs:177`), but textarea rows are taller and can introduce early scrolling; any unification work should treat row height by field type.
- Keyboard behavior currently depends on parent key routing + child delegated focus (`src/render_prompts/form/render.rs:109`, `src/form_prompt.rs:261`); shell/component refactors must preserve this routing contract.
