# PromptContainer Component Audit

## Scope
- Component reviewed: `src/components/prompt_container.rs`
- Supporting shell contract reviewed: `src/components/prompt_layout_shell.rs`
- Current renderer usage reviewed: `src/prompts/path/render.rs` (entity) and `src/render_prompts/path.rs` (outer shell)

## Current Contract (What PromptContainer Actually Guarantees)

### 1) Frame and clipping contract
- `PromptContainer` root is built through `prompt_frame_root(...)` with:
  - `flex_col`, `w_full`, `h_full`
  - `min_h(0)`
  - `overflow_hidden`
  - rounded corners from config
- Source:
  - `src/components/prompt_container.rs:243`
  - `src/components/prompt_container.rs:262`
  - `src/components/prompt_layout_shell.rs:40`
  - `src/components/prompt_layout_shell.rs:46`
  - `src/components/prompt_layout_shell.rs:49`

Implication: PromptContainer is always a clipping boundary by default. Any child that depends on overflow visibility will be clipped unless rendered outside the container.

### 2) Slot composition contract (header/content/footer)
- Slots are appended in order as direct flex children:
  1. header (optional)
  2. divider (optional, only if header exists)
  3. content (optional)
  4. footer (optional) or hint fallback
- Source:
  - `src/components/prompt_container.rs:267`
  - `src/components/prompt_container.rs:273`
  - `src/components/prompt_container.rs:278`
  - `src/components/prompt_container.rs:286`

Implication: PromptContainer does not impose section wrappers or section padding. Header/content/footer components own their own internal spacing.

### 3) Divider rhythm contract
- Divider is a 1px line with horizontal margin (`divider_margin`, default `16.0`) and alpha-adjusted border color.
- Divider appears only when both:
  - `config.show_divider == true`
  - header exists
- Source:
  - `src/components/prompt_container.rs:95`
  - `src/components/prompt_container.rs:218`
  - `src/components/prompt_container.rs:273`

Implication: Divider alignment is tied to the chosen margin, not auto-aligned to header internals.

### 4) Content sizing and scroll containment contract
- Default mode is `Fill`.
- `Fill` wraps content in `prompt_frame_fill_content(...)` (`flex_1 + min_h(0) + overflow_hidden`).
- `Intrinsic` mode renders content as-is.
- Source:
  - `src/components/prompt_container.rs:68`
  - `src/components/prompt_container.rs:98`
  - `src/components/prompt_container.rs:109`
  - `src/components/prompt_container.rs:279`
  - `src/components/prompt_layout_shell.rs:63`

Implication: In `Fill`, scrolling must happen inside the content child (e.g., list/scroll view). The wrapper itself clips overflow.

### 5) Footer fallback contract
- If `footer` slot exists, it is used.
- Otherwise, if `hint_text` is configured, PromptContainer renders a lightweight centered hint footer.
- Source:
  - `src/components/prompt_container.rs:287`
  - `src/components/prompt_container.rs:289`
  - `src/components/prompt_container.rs:226`

Implication: `footer` has precedence over `hint_text`.

## Observed Consistency Risk in Current Usage
- `PathPrompt` currently nests `PromptContainer` inside an outer shell that also applies `rounded(...)` + `overflow_hidden(...)`.
  - Entity side: `src/prompts/path/render.rs:140`
  - Outer wrapper side: `src/render_prompts/path.rs:395`
  - Outer wrapper rounding: `src/render_prompts/path.rs:396`

Risk: dual clipping boundaries can cause subtle edge clipping differences if radii diverge or if children expect overflow visibility.

## Canonical Container Rules

### Rule 1: Single source of truth for shell clipping/radius
- If a prompt uses `PromptContainer`, renderer authors must keep shell radius/clipping synchronized.
- Preferred options:
  1. Let `PromptContainer` own radius + clipping and keep outer wrapper non-rounded.
  2. If outer wrapper must stay rounded/clipped (for overlays/key context), set `PromptContainerConfig::rounded_corners(...)` to the same radius token.

### Rule 2: Slot components own padding; PromptContainer owns only structure
- Do not add generic section padding in PromptContainer callers.
- Keep spacing local to slot components:
  - header component controls header insets
  - content component/list controls content insets
  - footer component controls footer insets
- Treat container divider as the only shared horizontal rhythm token.

### Rule 3: Divider margin must track header rhythm
- Default `divider_margin = 16` should be used only when header horizontal inset is effectively 16px.
- If header inset changes, update divider margin explicitly to maintain vertical rhythm and avoid visual drift.

### Rule 4: Use `Fill` for scrollable content; scroll inside content child
- For list/table/log panels: keep `content_layout = Fill`.
- Content child should be the scroll owner and should generally include:
  - `flex_1`
  - `min_h(0)` where needed
  - its own scrolling behavior
- Avoid adding another parent scroll layer above PromptContainer content unless intentionally nesting scroll regions.

### Rule 5: Use `Intrinsic` only for non-fill content
- For intrinsically-sized content blocks/forms that should not consume remaining height, opt into `content_layout = Intrinsic`.
- In `Intrinsic`, caller is responsible for preventing overflow and layout collapse.

### Rule 6: Render overlays outside PromptContainer clipping boundary
- Menus/dialog overlays that should escape content bounds should be rendered in the outer relative shell as absolutely positioned siblings, not inside container slots.
- Current path/term wrappers already follow this pattern.

### Rule 7: Footer policy
- Use a real `footer` component when actions/status affordances are needed.
- Use `hint_text` fallback only for lightweight informational copy.
- Do not set both expecting merge behavior; `footer` will override hint.

## Recommended Renderer Usage Pattern

```rust
let container = PromptContainer::new(container_colors)
    .config(
        PromptContainerConfig::new()
            .rounded_corners(shell_radius) // keep in sync with outer shell if both clip
            .show_divider(true)
            .divider_margin(16.0)
            .content_layout(PromptContainerContentLayout::Fill),
    )
    .header(header)
    .content(scroll_owner_content)
    .footer(footer);
```

Where `scroll_owner_content` is the element that owns scrolling/virtualization; PromptContainer only provides the bounded slot.

## Practical Checklist for Prompt Authors
- Is radius/clipping defined in one place, or synchronized if duplicated?
- Does the divider margin match header horizontal inset?
- Is content scroll owned by the content child (not by parent shell wrappers)?
- Are overlays rendered outside PromptContainer if they need to escape clipping?
- Are footer/hint semantics intentional (no accidental footer-overrides-hint)?
