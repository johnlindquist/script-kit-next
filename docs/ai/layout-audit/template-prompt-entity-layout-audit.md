<!-- markdownlint-disable MD013 -->

# Template Prompt Entity Layout Audit

## Scope

- Audited file: `src/prompts/template/render.rs`
- Shell context: `src/render_prompts/other.rs`, `src/components/prompt_layout_shell.rs`
- Audit date: 2026-02-11

## Unified Hierarchy Mapping

- App wrapper: `ScriptListApp::render_template_prompt(...)` uses `prompt_shell_container(...)` + `prompt_shell_content(...)` (`src/render_prompts/other.rs:161`).
- Shell contract: prompt shell content is flex-fill + overflow-clipped (`src/components/prompt_layout_shell.rs:79`, `src/components/prompt_layout_shell.rs:86`).
- Entity contract: `TemplatePrompt::render(...)` is responsible for internal sectioning, spacing, and any scrollable regions (`src/prompts/template/render.rs:10`).

## Findings

### 1) Form content has no internal scroll region and can be clipped

- Evidence: the root container is `h_full` with all sections appended in one vertical flow, but no `min_h(0)` + `overflow_y_scroll()` section exists (`src/prompts/template/render.rs:39`).
- Why this matters: the outer prompt shell intentionally clips overflow (`src/components/prompt_layout_shell.rs:86`), so long templates can hide lower fields/help text with no path to reveal them.

### 2) Label and validation indentation are hardcoded and drift-prone

- Evidence: label width is fixed to `140px` (`src/prompts/template/render.rs:146`) while validation text uses a separate hardcoded `pl(144px)` offset (`src/prompts/template/render.rs:169`).
- Why this matters: changing row gap, text size, or label width in one place will misalign error text in another; this breaks canonical form-column alignment.

### 3) Field height contract is ambiguous

- Evidence: row fields use `min_h(PROMPT_INPUT_FIELD_HEIGHT)` plus vertical padding (`src/prompts/template/render.rs:154` and `src/prompts/template/render.rs:156`).
- Why this matters: the visible field height can exceed the nominal prompt field token, producing inconsistent baseline alignment across prompt types.

### 4) Preview and helper copy are not protected against overflow pressure

- Evidence: preview uses direct `.child(preview)` with no wrapping/overflow policy (`src/prompts/template/render.rs:53`), and help text is appended as normal flow content (`src/prompts/template/render.rs:195`).
- Why this matters: large preview strings or many grouped inputs can push helper text off-screen, and long unbroken preview text can overflow horizontally.

### 5) Group header spacing is locally correct but not tokenized as a reusable contract

- Evidence: group transitions are rendered inline with ad-hoc margins in the main loop (`src/prompts/template/render.rs:87` and `src/prompts/template/render.rs:91`).
- Why this matters: future form prompts can re-implement spacing slightly differently, creating drift from a shared “grouped form” prompt pattern.

## Canonical Form Prompt Layout Rules

### 1) Shell-to-Entity contract

- Keep using `prompt_shell_container` + `prompt_shell_content` at wrapper level.
- Inside `TemplatePrompt`, define three vertical slots:
- `slot_preview`: shrink-to-content, non-scrolling.
- `slot_form`: `flex_1 + min_h(0) + overflow_y_scroll()`.
- `slot_help`: shrink-to-content, anchored after the form slot.

### 2) Form row geometry

- Define row constants once in template prompt render module:
- `FORM_LABEL_WIDTH_PX`: label column width.
- `FORM_COLUMN_GAP_PX`: horizontal gap between label and field.
- `FORM_ERROR_INDENT_PX = FORM_LABEL_WIDTH_PX + FORM_COLUMN_GAP_PX`.
- Keep field body at canonical height token (`PROMPT_INPUT_FIELD_HEIGHT`) using one explicit strategy (fixed height or min-height without additive vertical drift).

### 3) Group spacing contract

- Group headers should use one tokenized top spacing when group changes.
- Row spacing within a group should use a second tokenized spacing value.
- First group should not rely on ad-hoc conditional offsets distinct from subsequent groups.

### 4) Validation placement contract

- Validation text must align to field start column, not to an independent magic number.
- Validation text should wrap and remain inside form scroll region.

### 5) Preview/help overflow contract

- Preview text should wrap and optionally clamp/max-height with internal scroll for extreme content.
- Help text should remain visible after form scrolling, not be pushed outside clipped shell bounds.

## Follow-up Fix Plan (Not Implemented In This Audit)

- Extract template form layout constants and replace hardcoded `140`, `144`, and `4` values.
- Refactor render tree into `preview`, `scrollable_form`, and `help` slots.
- Move validation messages into a layout branch that derives indent from label+gap constants.
- Add render-level regression tests that assert no hardcoded indent drift and presence of scrollable form slot markers.
