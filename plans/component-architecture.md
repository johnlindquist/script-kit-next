# Component Architecture Review

Date: 2026-02-07  
Agent: `codex-component-arch`  
Scope: `src/components/**/*.rs`

## Summary

The component layer has strong building blocks but is currently split across overlapping abstractions. The biggest architecture issues are:

1. Input state/rendering is fragmented across multiple component families (`PromptInput`, `TextInputState`, form-local editing logic, and an orphaned `ScriptKitInput` file).
2. `UnifiedListItem` exposes API surface (`Custom` content + a11y fields) that is not fully implemented at render time.
3. Prompt shell composition is split between `prompt_layout_shell` and `PromptContainer`, with duplicated responsibilities and partial adoption.
4. Button behavior/styling is duplicated between `Button` and `FooterButton`, with additional per-caller color assembly.

## Current Architecture Map

### Input-related components

- `src/components/prompt_input.rs` defines a display-only prompt input renderer with manual cursor/placeholder logic.
- `src/components/text_input.rs` defines `TextInputState` (editing model used by alias modal).
- `src/components/form_fields.rs` contains another full text editing implementation inside `FormTextField` and `FormTextArea`.
- `src/components/script_kit_input.rs` defines a wrapper around `gpui_component::Input`, but is not wired into module exports/usages.

### Prompt shell components

- `src/components/prompt_layout_shell.rs` provides shared shell helpers and is used by render wrappers in `src/render_prompts/*`.
- `src/components/prompt_container.rs` also provides shell/header/content/footer composition, but current runtime usage is limited (`src/prompts/path.rs:668`).
- `src/components/prompt_header.rs` and `src/components/prompt_footer.rs` are partially reused; footer is broadly used, header is mostly path/stories.

### List item components

- `src/components/unified_list_item/*` is used by select prompt list rendering (`src/prompts/select.rs:755`) and provides typed leading/title/trailing content and density.

### Buttons

- `src/components/button.rs` is a general button component with variants.
- `src/components/footer_button.rs` is a separate footer-specific label+shortcut button.

## Findings

## P0 - Input Architecture Is Fragmented

### 1) Orphaned `ScriptKitInput` implementation (not in module tree)

- Evidence:
  - `src/components/script_kit_input.rs` exists with a full component API.
  - `src/components/mod.rs:29-45` does not declare `mod script_kit_input`.
  - No runtime references to `ScriptKitInput` (`rg` usage only returns the file itself).
- Impact:
  - A complete input abstraction exists but is unreachable, increasing confusion and maintenance cost.
- Recommendation:
  - Choose one direction explicitly:
    - Integrate it into `mod.rs` and migrate callers, or
    - Remove/archive it and document that `PromptInput` + `TextInputState` are canonical.

### 2) Multiple input editing models duplicate behavior

- Evidence:
  - `FormTextField` and `FormTextArea` each implement selection, clipboard, cursor movement, and editing (`src/components/form_fields.rs:326-563`, `src/components/form_fields.rs:833-1139`).
  - `AliasInput` uses a different editing model (`TextInputState`) (`src/components/alias_input.rs:24`, `src/components/alias_input.rs:160-161`, `src/components/alias_input.rs:516-523`).
  - `PromptHeader` and `PromptInput` both hand-roll cursor/placeholder rendering logic (`src/components/prompt_header.rs:266-357`, `src/components/prompt_input.rs:444-483`).
- Impact:
  - Keyboard behavior can drift between surfaces.
  - Fixes to selection/clipboard/IME behavior must be repeated in several places.
- Recommendation:
  - Extract a single `TextEditModel` (or adopt `gpui_component::InputState`) and use it across:
    - form text field/area,
    - alias modal,
    - prompt header/search inputs.
  - Keep rendering-specific concerns separate from editing state transitions.

### 3) Legacy handlers coexist with unified handlers in form fields

- Evidence:
  - `FormTextArea` still contains legacy `handle_input`/`handle_key_down` (`src/components/form_fields.rs:997-1059`) alongside newer `handle_key_event` (`src/components/form_fields.rs:1066-1139`).
  - `FormTextField` also has both older and unified entry points (`src/components/form_fields.rs:260-310`, `src/components/form_fields.rs:490-563`).
- Impact:
  - Increases API surface and ambiguity for callers.
- Recommendation:
  - Keep only one public event path per component (`handle_key_event`) and remove legacy internals once callers are migrated.

## P0 - Unified List Item API Is Not Internally Consistent

### 4) `Custom` variants are declared but not rendered

- Evidence:
  - Custom variants exist in public enums: `TextContent::Custom`, `LeadingContent::Custom`, `TrailingContent::Custom` (`src/components/unified_list_item/types.rs:30-31`, `src/components/unified_list_item/types.rs:116-117`, `src/components/unified_list_item/types.rs:143-144`).
  - Render path drops them: leading/trailing return `None`, text returns empty `div()` (`src/components/unified_list_item/render.rs:299`, `src/components/unified_list_item/render.rs:344`, `src/components/unified_list_item/render.rs:390`).
- Impact:
  - Public API implies extensibility that does not actually work.
- Recommendation:
  - Either implement actual custom rendering slots or remove these variants from public API.

### 5) Accessibility fields are settable but unused

- Evidence:
  - `UnifiedListItem` stores `a11y_label` and `a11y_hint` and has builder methods (`src/components/unified_list_item/render.rs:29-30`, `src/components/unified_list_item/render.rs:97-104`).
  - Render path never applies these fields (`src/components/unified_list_item/render.rs:113-217`).
- Impact:
  - Callers can provide a11y metadata that has no effect.
- Recommendation:
  - Wire these into element accessibility metadata, or remove until supported.

## P1 - Prompt Shell Components Have Overlap and Partial Adoption

### 6) `PromptContainer` and `prompt_layout_shell` both define shell responsibilities

- Evidence:
  - Shared shell helpers exist in `prompt_layout_shell` (`src/components/prompt_layout_shell.rs:10-33`) and are used by multiple prompt wrappers.
  - `PromptContainer` also defines full shell + slots (`src/components/prompt_container.rs:135-300`) but is primarily used in path prompt (`src/prompts/path.rs:668`).
- Impact:
  - Two competing composition patterns increase cognitive overhead and API sprawl.
- Recommendation:
  - Consolidate around one shell primitive:
    - Option A: `PromptContainer` internally composes `prompt_shell_container` / `prompt_shell_content`.
    - Option B: keep shell helpers and reduce `PromptContainer` to optional sugar wrapper.

### 7) `PromptContainer` has dead internal helper methods and duplicated inline rendering

- Evidence:
  - `render_divider` and `render_hint` exist (`src/components/prompt_container.rs:195-227`).
  - Equivalent divider/hint rendering is duplicated inline in `render()` (`src/components/prompt_container.rs:256-265`, `src/components/prompt_container.rs:281-295`).
- Impact:
  - Duplicate render code creates drift risk.
- Recommendation:
  - Use helper methods from render path or delete helpers.

## P1 - Button APIs Are Duplicated

### 8) Separate `Button` and `FooterButton` duplicate core behavior

- Evidence:
  - Both manage label/shortcut/click closure wiring (`src/components/button.rs:153-387`, `src/components/footer_button.rs:16-106`).
  - Callback aliases are duplicated (`src/components/button.rs:141`, `src/components/footer_button.rs:14`, `src/components/prompt_footer.rs:184`, `src/components/prompt_header.rs:224`).
- Impact:
  - Inconsistent hover/focus/accessibility behavior across action surfaces.
- Recommendation:
  - Unify on one button primitive with style variants (`Ghost`, `Footer`, etc.) and one shared callback type alias.

### 9) Button colors are often assembled ad-hoc by parent components

- Evidence:
  - `PromptHeader` constructs `ButtonColors` inline (`src/components/prompt_header.rs:363-373`).
  - Footer button colors are resolved separately in `FooterButton` using global theme lookup (`src/components/footer_button.rs:62-67`).
- Impact:
  - Color policy is split between parent and child layers.
- Recommendation:
  - Centralize color resolution in one place (theme-aware button style factory).

## P2 - Testing and Rollout Signals Suggest Incomplete Integration

### 10) Heavy `#![allow(dead_code)]` usage indicates partially integrated components

- Evidence:
  - Present in key modules (`button.rs`, `prompt_input.rs`, `prompt_header.rs`, `prompt_footer.rs`, `prompt_container.rs`, `alias_input.rs`, `form_fields.rs`, unified list modules).
- Impact:
  - Hard to tell canonical vs experimental components.
- Recommendation:
  - Replace broad `allow(dead_code)` with explicit deprecation or removal after convergence.

### 11) Tests verify helpers/tokens but not architectural invariants

- Evidence:
  - `form_fields_tests.rs` duplicates text helper functions instead of importing shared ones (`src/components/form_fields_tests.rs:6-37`).
  - unified list tests do not exercise custom variant rendering/a11y wiring (`src/components/unified_list_item_tests.rs:5-55`).
- Impact:
  - Regression risk during architecture cleanup.
- Recommendation:
  - Add invariant tests for canonical APIs (single input model, custom variant behavior, a11y field wiring).

## Refactor Plan (Suggested)

1. **Input convergence (highest priority)**
   - Introduce `components/input_core.rs` with a shared editing state machine.
   - Make `FormTextField`, `FormTextArea`, and `AliasInput` use it.
   - Decide fate of `ScriptKitInput` (integrate or remove).

2. **List item API hardening**
   - Implement or remove `Custom` variants.
   - Implement or remove `a11y_*` setters.
   - Add tests proving behavior.

3. **Prompt shell consolidation**
   - Define one canonical shell API and migrate path prompt + wrappers.
   - Remove duplicated divider/hint paths in `PromptContainer`.

4. **Button unification**
   - Merge footer button behavior into `Button` variants.
   - Keep a single callback alias and style factory.

5. **Cleanup pass**
   - Remove unnecessary `allow(dead_code)` and stale APIs.
   - Expand smoke/story coverage for migrated components.

## Recommended Tests for Follow-up Changes

- `test_unified_list_item_renders_custom_content_when_custom_variant_used`
- `test_unified_list_item_applies_a11y_metadata_when_label_and_hint_set`
- `test_form_textfield_and_textarea_share_identical_clipboard_shortcuts_when_cmd_pressed`
- `test_alias_input_uses_shared_text_edit_model_when_handling_selection`
- `test_prompt_container_uses_prompt_shell_layout_contract_when_rendered`
- `test_button_footer_variant_matches_footer_button_shortcut_layout_when_configured`

## Risks / Migration Notes

- Keyboard/input behavior is user-visible and easy to regress; migrate input internals behind compatibility adapters first.
- Prompt rendering is spread across multiple prompt modules; shell API migration should be staged per prompt type.
- Footer/header visual parity depends on token mapping; centralizing button styles may reveal existing theme assumptions that need explicit codification.

## Validation Notes

This agent task was an architecture audit and report pass. No runtime component behavior was changed in this task; only this plan document was added.
