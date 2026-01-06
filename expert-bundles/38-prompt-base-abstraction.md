# Expert Bundle 38: Prompt Base Abstraction

## Goal
Consolidate duplicated struct fields, constructors, and trait implementations across 8 prompt types into a shared `PromptBase` abstraction.

## Current State

The prompts layer has **8 prompt types** with ~25% structural duplication:
- `src/prompts/arg.rs` - ArgPrompt (searchable list)
- `src/prompts/div.rs` - DivPrompt (HTML display)
- `src/prompts/path.rs` - PathPrompt (file picker)
- `src/prompts/env.rs` - EnvPrompt (secrets)
- `src/prompts/drop.rs` - DropPrompt (drag & drop)
- `src/prompts/select.rs` - SelectPrompt (multi-select)
- `src/prompts/template.rs` - TemplatePrompt (placeholders)
- `src/editor.rs` - EditorPrompt (~800 lines)

Each prompt independently declares the same base fields and implements identical trait methods.

## Specific Concerns

1. **Repeated Struct Fields (8 copies)**: Every prompt has `id`, `focus_handle`, `theme`, `design_variant`, `on_submit` fields declared separately.

2. **Focusable Boilerplate (8 copies)**: Identical 4-line `impl Focusable` blocks in every prompt module.

3. **Design Token Extraction (8 copies)**: Same 4-line pattern extracting `tokens`, `colors`, `spacing`, `visual` from design variant.

4. **Design Variant Branching (17 locations)**: Identical `if self.design_variant == DesignVariant::Default { theme } else { design }` conditionals scattered throughout.

5. **Render Wrapper Functions (4 nearly identical)**: `render_select_prompt`, `render_env_prompt`, `render_drop_prompt`, `render_template_prompt` are ~95% identical code.

6. **Actions Dialog Key Routing (6 copies)**: ~80-line identical key handling block duplicated in 6 render files.

## Key Questions

1. Should `PromptBase` be a struct that prompts embed via composition, or a trait with default implementations?

2. Can we use a derive macro for `Focusable` delegation, or is composition cleaner?

3. Is `DesignContext` (pre-computed tokens + theme-aware color getters) the right abstraction for eliminating variant branching?

4. Should the generic `render_entity_prompt<T>` wrapper handle all prompt types, or should some prompts opt out?

5. How should we handle the actions dialog key routing - method on `ScriptListApp`, standalone function, or trait?

## Implementation Checklist

- [ ] Create `src/prompts/base.rs` with `PromptBase` struct
- [ ] Add `PromptBase::new_focused()` constructor helper
- [ ] Create `DesignContext` struct with `text_primary()`, `background()`, etc. methods
- [ ] Add `impl_focusable_via_base!` macro or trait delegation
- [ ] Create generic `render_entity_prompt<T>()` function
- [ ] Add `ScriptListApp::route_key_to_actions_dialog()` method
- [ ] Migrate one prompt at a time to use new abstractions
- [ ] Update tests to verify behavior unchanged
