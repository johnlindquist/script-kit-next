---
name: storybook
description: Script Kit GPUI design explorer (storybook) system. Use when adding stories, footer/input variations, working with the StoryBrowser, design adoption, chrome audits, or the storybook CLI binary. Covers story registration, variant compare mode, selection persistence, and live adoption wiring.
---

# Storybook / Design Explorer

In-app design explorer for comparing, selecting, and adopting UI variations across prompt surfaces. Stories render real components — footer, input, header, action dialog variants — in a browsable, comparable format with persistent selection.

## Key Concepts

| Concept | What it means |
|---------|--------------|
| **Story** | A trait impl that renders one or more variants of a UI surface |
| **StoryBrowser** | The GPUI entity that hosts browsing, compare mode, and adoption |
| **Variation** | A declarative spec (e.g., FooterVariationSpec) defining a design option |
| **Adoption** | Persisting a selected variant so live prompt surfaces use it at runtime |
| **Chrome Audit** | Structured log that validates minimal chrome compliance per surface |

## File Map

| Path | Role |
|------|------|
| `src/storybook/mod.rs` | Module root; re-exports; JSON error payloads |
| `src/storybook/story.rs` | `Story` trait, `StorySurface` enum, `StoryVariant` |
| `src/storybook/registry.rs` | Lookup, filtering, `first_story_with_multiple_variants()` |
| `src/storybook/browser.rs` | `StoryBrowser` entity (~1400 lines) |
| `src/storybook/selection.rs` | `StorySelectionStore`, atomic persistence |
| `src/storybook/diagnostics.rs` | `StoryCatalogSnapshot` for machine-readable catalog |
| `src/storybook/footer_variations/` | 5 footer specs + rendering + adoption resolution |
| `src/storybook/input_variations/` | 5 input specs + rendering + adoption resolution |
| `src/storybook/layout.rs` | UI helpers (story_container, story_section, etc.) |
| `src/stories/mod.rs` | Story registration (`ALL_STORIES` LazyLock) |
| `src/bin/storybook.rs` | Standalone binary with CLI flags |

## Adding a New Story

1. Create struct implementing `Story` trait in `src/stories/`
2. Return stable `id()`, descriptive `name()`, `category()`, and `surface()`
3. Implement `variants()` returning `Vec<StoryVariant>` with stable IDs
4. Implement `render_variant(&self, variant)` to render each variant
5. Register in `ALL_STORIES` in `src/stories/mod.rs`

```rust
pub struct MyStory;
impl Story for MyStory {
    fn id(&self) -> &'static str { "my-component-variations" }
    fn name(&self) -> &'static str { "My Component Variations" }
    fn category(&self) -> &'static str { "Components" }
    fn surface(&self) -> StorySurface { StorySurface::Component }
    fn variants(&self) -> Vec<StoryVariant> {
        vec![
            StoryVariant::default_named("variant-a", "Variant A"),
            StoryVariant::default_named("variant-b", "Variant B"),
        ]
    }
    fn render_variant(&self, variant: &StoryVariant) -> AnyElement {
        match variant.stable_id() {
            "variant-a" => /* render A */,
            "variant-b" => /* render B */,
            _ => self.render(),
        }
    }
}
```

## Adding a New Variation System (Footer/Input Pattern)

Follow the footer_variations or input_variations pattern:

1. Define an ID enum with `stable_id() -> &'static str` and `from_stable_id()`
2. Define a declarative spec struct with design properties
3. Create a const array of specs
4. Implement `story_variants()` converting specs to `StoryVariant`
5. Implement `render_preview(stable_id)` rendering real components
6. Implement `adopted_*_variation()` reading from `StorySelectionStore`
7. Add tests for unique IDs, round-trip, and resolution

## CLI Usage

```bash
# Interactive browser
cargo run --bin storybook

# Pre-select story + compare mode
cargo run --bin storybook -- --story footer-layout-variations --compare

# Non-interactive adoption (JSON output, exit code 0/2)
cargo run --bin storybook -- --adopt --story footer-layout-variations --variant minimal

# Machine-readable catalog
cargo run --bin storybook -- --catalog-json

# Screenshot capture
cargo run --bin storybook -- --story footer-layout-variations --screenshot
```

## Selection Persistence

- File: `~/.kit/design-explorer-selections.json`
- Format: `{ "selections": { "<story_id>": "<variant_id>" } }`
- Atomic writes via temp file + rename
- `StorySelectionStore` in `selection.rs` handles load/save
- `save_selected_story_variant()` returns `StorySelectionWriteResult` with previous value

## Live Adoption Wiring

Footer adoption: `config_from_storybook_footer_selection()` reads persisted selection, resolves via `resolve_footer_selection_spec()`, falls back to RaycastExact.

Input adoption: `adopted_input_variation()` reads persisted selection, falls back to Bare.

Both are called at render time in live prompt surfaces.

## Surface Wiring Map

Use this map before claiming that a Storybook edit changes the real app.

| Surface | Wiring | What changes affect live UI |
|---------|--------|-----------------------------|
| Main Menu | Storybook-specific preview in `src/storybook/main_menu_variations/mod.rs` plus feature-gated live-spec overrides in `src/render_script_list/mod.rs` | Shared components/tokens affect both. Preview-only layout/data changes do not update the real app by themselves. |
| Footer Variations | Adopted live surface | Changing variation specs/config mapping affects live prompt footers when that selection is adopted. |
| Input Variations | Adopted live surface | Changing variation specs/config mapping affects live prompt inputs when that selection is adopted. |
| Actions Dialog | Shared presenter in `src/storybook/actions_dialog_presenter.rs` | Presenter changes affect both Storybook and the live dialog. |
| Mini AI Chat | Shared presenter in `src/storybook/mini_ai_chat_presenter.rs` | Presenter changes affect both Storybook and the live mini AI chat. |
| Notes Window | Runtime fixture preview | Fixture or preview-host changes affect Storybook only unless shared notes-window code also changes. |

Rule of thumb:
- `src/storybook/...` only: usually Storybook-only
- Shared presenter/component/theme code: affects both
- Adopted variation/spec code: can affect live surfaces that read that selection

## Chrome Audit System

`PromptChromeAudit` in `prompt_layout_shell.rs` validates minimal chrome compliance:
- `PromptChromeAudit::minimal(...)` for standard surfaces
- `PromptChromeAudit::exception(surface, reason)` for rich-chrome exceptions
- `emit_prompt_chrome_audit()` logs once per unique audit (deduped via HashSet)
- Warns if a non-exception surface uses `prompt_footer` mode

## Testing

```bash
cargo test footer          # Footer adoption + resolution
cargo test storybook       # Browser, catalog, errors
cargo test chrome_audit    # Audit dedup + emission
cargo test source_audit    # Minimal chrome compliance
cargo test input_variation # Input adoption + resolution
```

## References

- `references/architecture.md` — Detailed type definitions, browser state, rendering pipeline, keyboard shortcuts
