# Template Prompt Improvements Audit

Date: 2026-02-07  
Agent: `codex-template-prompt`  
Scope: `src/prompts/template.rs`

## Executive Summary

`TemplatePrompt` works for basic `{{name}}` workflows, but it is still a scaffold-level implementation with major gaps in editing ergonomics, parsing/substitution consistency, and configuration/reuse.

Highest-impact issues:

1. Field editing is character-only and bypasses shared input primitives.
2. Parsing/substitution logic is duplicated and inconsistent with shared template modules.
3. Submission does multi-pass global replacement, which is both inefficient and can mutate user-entered literals.
4. Protocol shape (`Message::Template`) cannot carry field metadata, so validation/layout behavior is hardcoded.

## Current Behavior Map

`src/prompts/template.rs` currently does:

1. Extract placeholders via `Regex("\\{\\{(\\w+)\\}\\}")` and infer field metadata heuristically (`src/prompts/template.rs:100`).
2. Keep one `String` per input and one `current_input` index (`src/prompts/template.rs:49`).
3. Handle keyboard input manually (`tab`, `enter`, `escape`, `backspace`, otherwise first `key_char`) (`src/prompts/template.rs:401`).
4. Render a static preview area and a vertically stacked field list with group headers (`src/prompts/template.rs:468`).
5. Validate fields using name-based heuristics and slug checks (`src/prompts/template.rs:191`).
6. Submit by replacing placeholders with trimmed values in a loop (`src/prompts/template.rs:328`).

Relevant integration constraints:

1. Protocol only sends `{ id, template }` for template prompts (`src/protocol/message.rs:261`, `src/main.rs:1098`).
2. Prompt handler passes only raw template string into `TemplatePrompt::new` (`src/prompt_handler.rs:1362`).
3. Scriptlet `template` tool routes stdout to `ShowTemplate`, again with no field schema (`src/app_impl.rs:6013`).

## Findings (Ranked)

### P1: Input editing model is too limited for production use

Evidence:

1. Manual key handling only supports tab/enter/escape/backspace plus appending one char (`src/prompts/template.rs:401`).
2. `handle_char` only `push`es chars and `handle_backspace` only pops (`src/prompts/template.rs:363`, `src/prompts/template.rs:375`).
3. No shared editing engine usage (`TextInputState`/`ScriptKitInput`) unlike other prompts (`src/prompts/env.rs:336`, `src/components/script_kit_input.rs:1`).

Impact:

1. No cursor movement inside a field, delete key behavior, selection, clipboard shortcuts, or robust IME/composition support.
2. Multi-character insert paths (paste/input method) degrade to fragile key-char behavior.

Recommendation:

1. Replace per-field `String` editing with shared input state (`TextInputState` or `gpui_component::Input` via `ScriptKitInput`).
2. Keep `Tab`/`Shift+Tab` field navigation as an outer concern, but delegate text editing semantics to shared input code.

### P1: Parsing/substitution logic is duplicated and inconsistent with existing template modules

Evidence:

1. TemplatePrompt parser only matches `{{\w+}}` (`src/prompts/template.rs:105`).
2. Shared module already supports extracting both `${var}` and `{{var}}` and skips control tags (`src/template_variables.rs:202`).
3. SDK APIs/documentation describe richer template/snippet semantics in other paths (`scripts/kit-sdk.ts:2893`, `scripts/kit-sdk.ts:4533`, `src/snippet.rs:1`).

Impact:

1. Behavior diverges across template-related features, increasing user confusion and maintenance cost.
2. Placeholder syntaxes and parsing edge cases are handled differently depending on entry point.

Recommendation:

1. Consolidate placeholder extraction/substitution into one reusable Rust module used by TemplatePrompt, scriptlets, and compile-like helpers.
2. Define supported syntax explicitly (for this prompt) and enforce it consistently across protocol + SDK + docs.

### P1: Multi-pass global replacement can cause recursive substitution and avoidable O(n*m) work

Evidence:

1. `filled_template`, `preview_template`, and `submit` each loop placeholders and call `String::replace` on entire template repeatedly (`src/prompts/template.rs:269`, `src/prompts/template.rs:287`, `src/prompts/template.rs:329`).

Impact:

1. Values containing placeholder-like text can be mutated by later replacement passes.
2. Large templates with many variables pay repeated full-string scans per field.

Recommendation:

1. Parse template once into tokens (literal + placeholder) and render/submit in a single pass.
2. Keep literal user input opaque (do not run placeholder substitution inside replacement values unless explicitly requested).

### P1: Protocol and constructor shape block layout/validation reusability

Evidence:

1. `Message::Template` and `PromptMessage::ShowTemplate` only carry `{id, template}` (`src/protocol/message.rs:263`, `src/main.rs:1099`).
2. `TemplatePrompt::new` derives label/placeholder/group/required entirely from name heuristics (`src/prompts/template.rs:67`, `src/prompts/template.rs:126`).

Impact:

1. Script authors cannot specify required flags, labels, defaults, field ordering, multiline controls, or validation rules.
2. Prompt behavior is non-portable and tightly coupled to English/name heuristics.

Recommendation:

1. Extend template prompt protocol with optional typed field schema, for example:
   - `name`, `label`, `default`, `placeholder`, `required`, `group`, `validator`, `multiline`.
2. Keep backward compatibility by falling back to placeholder extraction when schema is absent.

### P2: Validation heuristics are over-aggressive and not configurable

Evidence:

1. Any field named `name`, containing `slug`, or ending in `_name` gets slug validation (`src/prompts/template.rs:199`).
2. Validation enforces lowercase letters/numbers/hyphen only (`src/prompts/template.rs:243`).

Impact:

1. Legitimate free-text names (for example, person/project display names) are rejected.
2. Rules cannot be customized per field or per template.

Recommendation:

1. Replace heuristic validation with explicit per-field `ValidationMode` (none/slug/regex/custom).
2. Apply slug validation only to explicitly-marked slug fields.

### P2: Layout is rigid and not resilient for larger templates

Evidence:

1. No scroll container/virtualization for field list; all rows are appended to one full-height container (`src/prompts/template.rs:455`, `src/prompts/template.rs:504`).
2. Label and error alignment use fixed pixel widths (`src/prompts/template.rs:565`, `src/prompts/template.rs:588`).

Impact:

1. Large templates can overflow view height and become hard to complete.
2. Long labels/localization and smaller windows break alignment.

Recommendation:

1. Put field list in a scrollable region (or `uniform_list` if large counts are expected).
2. Make label column responsive (min/max width or stacked layout fallback).

### P2: Preview rendering is plain and loses useful context for code templates

Evidence:

1. Preview is rendered as plain text block with no language-aware formatting (`src/prompts/template.rs:470`).
2. Template options elsewhere include language support (`scripts/kit-sdk.ts:289`, `scripts/kit-sdk.ts:4582`), but TemplatePrompt has no equivalent.

Impact:

1. Harder to inspect multi-line code or structured templates.
2. Reduced confidence before submit for high-entropy templates.

Recommendation:

1. Add optional preview mode config (`plain` vs `code`) and language hint.
2. Highlight unresolved placeholders in preview rather than only replacing with bracket labels.

### P3: Reusability gap with existing editor/snippet stack

Evidence:

1. SDK `template()` uses editor/snippet flow (`scripts/kit-sdk.ts:4547`) while TemplatePrompt is separate `{{...}}` form UI (`src/prompts/template.rs:1`).
2. Snippet parser already exists and is feature-rich (`src/snippet.rs:1`).

Impact:

1. Two separate template UX paths must be maintained and tested independently.
2. Feature improvements (navigation, placeholders, choices) do not transfer automatically.

Recommendation:

1. Introduce a shared `TemplateModel` + rendering adapters:
   - form-style adapter (current TemplatePrompt UX)
   - inline-editor adapter (snippet UX)
2. Share parsing, validation metadata, and substitution engine between both adapters.

## Prioritized Roadmap

### Phase 1: Correctness + UX baseline

1. Migrate field editing to shared input state and preserve current tab navigation.
2. Replace multi-pass substitution with tokenized single-pass rendering/submission.
3. Add regression tests around replacement correctness and validation focus behavior.

### Phase 2: Configurability + layout flexibility

1. Extend protocol with optional template field schema.
2. Move validation/grouping from heuristic defaults to explicit schema-first behavior.
3. Add scrollable/responsive field layout.

### Phase 3: Reuse and feature convergence

1. Unify TemplatePrompt parser/substitution with `template_variables` core.
2. Add preview language mode and placeholder highlighting.
3. Define shared template model used by both TemplatePrompt and snippet/editor flows.

## Suggested Tests (TDD Names)

1. `test_template_prompt_substitute_single_pass_does_not_rewrite_user_literal_placeholders`
2. `test_template_prompt_supports_schema_defined_required_and_label_fields`
3. `test_template_prompt_preserves_cursor_and_selection_using_shared_input_state`
4. `test_template_prompt_does_not_apply_slug_validation_to_non_slug_name_fields`
5. `test_template_prompt_renders_scrollable_field_list_when_input_count_exceeds_view`
6. `test_template_prompt_preview_highlights_unfilled_placeholders`
7. `test_template_prompt_extracts_variables_with_shared_template_variable_parser`
8. `test_template_protocol_accepts_optional_template_field_schema`

## Risks / Known Gaps

1. Protocol shape changes (`Message::Template`) require SDK + app compatibility handling.
2. Moving to shared input/editor primitives may alter keyboard behavior; needs parity tests for existing tab-submit flows.
3. If both snippet-style and form-style templates remain supported, product-level rules are needed to decide which renderer is used by default.
