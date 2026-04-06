# Storybook Architecture Details

## Story Trait

```rust
pub trait Story: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> &'static str;
    fn surface(&self) -> StorySurface { StorySurface::Component }
    fn render(&self) -> AnyElement;
    fn render_variant(&self, _variant: &StoryVariant) -> AnyElement { self.render() }
    fn variants(&self) -> Vec<StoryVariant> { vec![StoryVariant::default_named("default", "Default")] }
}
```

## StorySurface Enum

Categorizes stories by the UI surface they represent:
- `Component` (default) — generic UI component
- `Footer` — prompt footer variations
- `Input` — prompt input field variations
- `Header` — prompt header variations
- `Shell` — prompt shell/scaffold variations
- `ActionDialog` — actions dialog variations
- `TurnCard` — AI turn card variations
- `FullPrompt` — complete prompt compositions

## StoryBrowser State

```rust
pub struct StoryBrowser {
    stories: Vec<&'static StoryEntry>,
    selected_index: usize,
    selected_variant_index: usize,
    filter: String,
    theme_name: String,
    design_variant: DesignVariant,
    preview_mode: PreviewMode,  // Single | Compare
    selection_store: StorySelectionStore,
    status_line: Option<String>,
    focus_handle: FocusHandle,
    screenshot_dir: PathBuf,
}
```

## Browser Keyboard Shortcuts

### Single Mode
- `Up/Down` — navigate story list
- `Enter` or `C` — enter compare mode (if variants > 1)
- `Escape` — close browser

### Compare Mode
- `Left/Right` — navigate variants
- `1-9` — jump to variant by number
- `Enter` — adopt selected variant (persist to disk)
- `Escape` — back to single mode

## Footer Variation Specs

Five footer variations with declarative specs:

| ID | Logo | Primary | Secondary | Helper | Info | Left Slot | Right Slot |
|----|------|---------|-----------|--------|------|-----------|------------|
| `raycast-exact` | yes | yes | yes | — | — | — | — |
| `scriptkit-branded` | yes | yes | yes | "ACP Chat" | "Built-in" | — | — |
| `minimal` | no | no | no | — | — | — | key hints |
| `status-bar` | yes | no | no | — | — | "Ready" | key hints |
| `invisible` | no | no | no | — | — | — | — |

## Input Variation IDs

| ID | Style |
|----|-------|
| `bare` | No decoration (default) |
| `underline` | Bottom border line |
| `pill` | Rounded pill container |
| `search-icon` | Magnifying glass prefix |
| `prompt-prefix` | ">" prefix character |

## Adoption Resolution Chain

### Footer
1. `config_from_storybook_footer_selection()` — entry point for live render
2. Reads `~/.kit/design-explorer-selections.json`
3. Looks up `"footer-layout-variations"` key
4. `resolve_footer_selection_spec(value)` — matches variant ID to spec
5. Falls back to `RaycastExact` if missing/unknown
6. Returns `(PromptFooterConfig, FooterSelectionResolution)`

### Input
1. `adopted_input_variation()` — entry point for live render
2. Reads selection store, looks up `"input-design-variations"` key
3. `InputVariationId::from_stable_id(value)` — matches to enum
4. Falls back to `Bare` if missing/unknown

## CLI JSON Envelopes

### Success (--adopt)
```json
{
  "schemaVersion": 1,
  "ok": true,
  "data": {
    "storyId": "footer-layout-variations",
    "variantId": "minimal",
    "previousVariantId": "raycast-exact",
    "selectionStorePath": "~/.kit/design-explorer-selections.json",
    "selectionCount": 2
  }
}
```

### Error
```json
{
  "schemaVersion": 1,
  "ok": false,
  "error": {
    "kind": "unknown_story",
    "message": "Story 'foo' not found",
    "hint": "Available: footer-layout-variations, ..."
  }
}
```

## Chrome Audit Contract

```rust
pub struct PromptChromeAudit {
    pub surface: &'static str,
    pub input_mode: &'static str,    // "bare" | "custom"
    pub divider_mode: &'static str,  // "section_divider" | "custom"
    pub footer_mode: &'static str,   // "hint_strip" | "prompt_footer"
    pub header_padding_x: u16,
    pub header_padding_y: u16,
    pub hint_count: usize,
    pub has_leading_status: bool,
    pub has_actions: bool,
    pub exception_reason: Option<&'static str>,
}
```

### Two Constructors
- `PromptChromeAudit::minimal(surface, hints, status, actions)` — standard minimal chrome
- `PromptChromeAudit::exception(surface, reason)` — rich chrome with documented reason

### Emission
- `emit_prompt_chrome_audit(&audit)` — logs once per unique audit per process
- Uses `Mutex<HashSet<PromptChromeAudit>>` for deduplication
- Warns if non-exception surface uses `prompt_footer` mode

## Test Coverage Map

| Module | Test file | What's tested |
|--------|-----------|--------------|
| `story.rs` | inline | Stable ID fallback to slugified name |
| `registry.rs` | inline | Surface queries; comparable story helpers |
| `selection.rs` | inline | Roundtrip; atomic writes; write results; camelCase JSON |
| `footer_variations` | `tests.rs` | Unique IDs; spec coverage; round-trips; resolution; config bridge |
| `input_variations` | `tests.rs` | Unique IDs; spec coverage; round-trips; adoption defaults |
| `diagnostics.rs` | inline | Catalog metadata; selection marking; variant props |
| `mod.rs` | inline | JSON error payload structure |
| `prompt_layout_shell.rs` | inline | Chrome audit emission for exception surfaces |
