# Prompt Render Patterns Audit

## Goal

Inventory repeated prompt-render boilerplate, call out intentional divergences, and define a single mental model for adding new prompt types with less copy/paste.

Snapshot date: 2026-02-11.

## Scope

Audited render surfaces that assemble prompt chrome (shell, header/input/body/footer, actions overlay):

- `src/render_prompts/arg/helpers.rs`
- `src/render_prompts/arg/render.rs`
- `src/render_prompts/div.rs`
- `src/render_prompts/editor.rs`
- `src/render_prompts/form/render.rs`
- `src/render_prompts/other.rs`
- `src/render_prompts/path.rs`
- `src/render_prompts/term.rs`
- `src/prompts/select/render.rs`
- `src/prompts/env/render.rs`
- `src/prompts/template/render.rs`
- `src/prompts/drop.rs`
- `src/prompts/chat/render_core.rs`
- `src/prompts/chat/render_input.rs`
- `src/prompts/div/render.rs`
- Shared components: `src/components/prompt_layout_shell.rs`, `src/components/prompt_container.rs`, `src/components/prompt_header/component.rs`, `src/components/prompt_input.rs`, `src/components/prompt_footer.rs`
- Shared constants: `src/panel.rs`, `src/window_resize/mod.rs`

## Repeated Boilerplate Inventory

### 1. Shell frame + vibrancy lookup

Repeated pattern:

1. Read design tokens (`get_tokens(self.current_design)` or `get_tokens(self.design_variant)`)
2. Resolve vibrancy background (`get_vibrancy_background(&self.theme)`)
3. Build root container with `w_full`, explicit height/full height, `overflow_hidden`, rounded corners
4. Apply background only when vibrancy helper returns `Some`

Where it appears:

- Shared shell helper used: `src/render_prompts/other.rs`, `src/render_prompts/div.rs`, `src/render_prompts/editor.rs` via `prompt_shell_container(...)` / `prompt_shell_content(...)`
- Hand-rolled variants: `src/render_prompts/arg/render.rs`, `src/render_prompts/form/render.rs`, `src/render_prompts/path.rs`, `src/render_prompts/term.rs`, `src/prompts/select/render.rs`, `src/prompts/template/render.rs`, `src/prompts/drop.rs`, `src/prompts/chat/render_core.rs`, `src/prompts/div/render.rs`

Notable duplication detail:

- Several wrappers still compute now-unused compatibility values (`hex_to_rgba_with_opacity`, `create_box_shadows`) even when comments note vibrancy should pass through from root.

### 2. Theme/design token lookups and default-vs-design branching

Repeated pattern:

1. Load design tokens (`colors`, `spacing`, `visual`, `typography`)
2. For `DesignVariant::Default`, sometimes read directly from `self.theme.colors.*` instead of token set
3. Derive local color tuple variables (`text_primary`, `text_muted`, `border`, `accent`)

Where it appears:

- Branch-heavy examples: `src/prompts/select/render.rs`, `src/prompts/template/render.rs`, `src/prompts/drop.rs`, `src/prompts/div/render.rs`
- Wrapper-level token extraction for shell constants: `src/render_prompts/arg/render.rs`, `src/render_prompts/div.rs`, `src/render_prompts/editor.rs`, `src/render_prompts/form/render.rs`, `src/render_prompts/path.rs`, `src/render_prompts/term.rs`

### 3. Header/footer/input assembly

There are two parallel assembly styles:

- Shared component style
  - `PromptHeader` + `PromptContainer` + `PromptFooter`
  - Most visible in `src/prompts/path/render.rs`
- Inline/manual style
  - Header/input/footer built ad hoc per prompt
  - `src/render_prompts/arg/render.rs`, `src/prompts/env/render.rs`, `src/prompts/template/render.rs`, `src/prompts/select/render.rs`, `src/prompts/chat/render_core.rs`, `src/prompts/chat/render_input.rs`

Specific repeated input boilerplate:

- Cursor rendering and placeholder alignment logic is implemented multiple times:
  - `src/render_prompts/arg/render.rs`
  - `src/prompts/env/render.rs` + `src/prompts/env/prompt.rs`
  - `src/prompts/chat/render_input.rs`
  - `src/components/prompt_header/component.rs`
  - `src/components/prompt_input.rs` (already has canonical cursor-slot logic)

Footer assembly is partly standardized but still repeated:

- `PromptFooter::new(...)` used across arg/div/editor/form/env/term/webcam wrappers
- Repeated status + actions button wiring with similar click handlers

### 4. Spacing and size constants

Shared constants exist but usage is mixed:

- Canonical header/input constants in `src/panel.rs` (`HEADER_PADDING_X`, `HEADER_PADDING_Y`, `HEADER_GAP`, `PROMPT_INPUT_FIELD_HEIGHT`, `HEADER_TOTAL_HEIGHT`)
- Window height/footer constants in `src/window_resize/mod.rs` (`STANDARD_HEIGHT`, `MAX_HEIGHT`, `FOOTER_HEIGHT`)

Duplication/divergence examples:

- Some prompts use panel constants directly (`src/render_prompts/arg/render.rs`, `src/render_prompts/div.rs`)
- Some prompts define local ad hoc constants (`src/prompts/select/render.rs`)
- Some prompts hardcode per-view px values inline (`src/prompts/env/render.rs`, `src/prompts/chat/render_core.rs`)

### 5. Actions overlay/backdrop and dialog positioning

Common repeated pattern:

1. Compute dialog offsets using `prompt_actions_dialog_offsets(...)`
2. Condition on `self.show_actions_popup` + `self.actions_dialog.clone()`
3. Render absolute overlay layer with top-right positioned dialog
4. Optionally render click-capturing backdrop to dismiss popup

Where it appears:

- `src/render_prompts/arg/render.rs`
- `src/render_prompts/div.rs`
- `src/render_prompts/editor.rs`
- `src/render_prompts/form/render.rs`
- `src/render_prompts/path.rs`
- `src/render_prompts/term.rs`

Divergent backdrop behavior:

- Scrim backdrop color via `modal_overlay_bg(...)`: editor only (`src/render_prompts/editor.rs`)
- Transparent click-capture backdrops: arg/div/form
- No backdrop scrim and different overlay host shape: path/term

## Divergence Map (With Rationale)

| Area              | Current divergence                                                                        | Likely rationale                                                         | Cost of divergence                                                |
| ----------------- | ----------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ | ----------------------------------------------------------------- |
| Shell framing     | Some prompts use `prompt_shell_container`; others hand-roll root `div()` shell            | Prompt-specific historical evolution and different height policies       | Repeated vibrancy/root setup and harder consistency updates       |
| Header/input      | Path uses shared `PromptHeader`; Arg/Env/Select/Template/Chat implement custom input rows | Different UX needs (path prefix, multiline chat input, per-prompt hints) | Cursor/placeholder logic duplicated in multiple files             |
| Footer            | Shared `PromptFooter` widely used, but Chat has custom footer and custom footer buttons   | Chat footer includes model context and script-generation controls        | Footer spacing/visual behavior diverges from common shell         |
| Actions backdrop  | Editor uses themed scrim; others use transparent capture/no scrim                         | Editor readability + modal emphasis while editing                        | Inconsistent modal semantics and click affordance between prompts |
| Actions offsets   | Shared helper exists, but each wrapper repeats overlay wiring and close handlers          | Helper only solves offsets, not overlay host composition                 | Many near-identical overlay blocks across wrappers                |
| Spacing constants | Mixture of panel constants, design-token spacing, and inline px literals                  | Fast local tuning per prompt                                             | Hard to keep visual rhythm consistent across prompt types         |
| Height policy     | `STANDARD_HEIGHT`, `MAX_HEIGHT`, dynamic computed form height, full-height entities       | Content-specific behavior (editor/term tall, form dynamic)               | Shell construction split across many render paths                 |

## Proposed Mental Model: `PromptShell` + Slot Blocks

Treat every prompt as:

`PromptShell { chrome + key routing + optional actions overlay } + slots { header, body, footer }`

### Slot contract

- `HeaderSlot`
  - `None` or shared `PromptHeader` or custom element
  - Owns search/input affordance at top level when present
- `BodySlot`
  - Required
  - Scroll/list/editor/content surface
- `FooterSlot`
  - `None` or shared `PromptFooter` or custom footer
- `OverlaySlot`
  - Optional actions dialog host (dialog + backdrop + dismiss wiring)

### Shell responsibilities

- Resolve vibrancy background once
- Apply frame policy once (rounded/overflow/relative)
- Apply height policy once (`Standard`, `Tall`, `Intrinsic`, `EdgeToEdge`)
- Mount shared key interception once (global shortcuts + actions routing hooks)
- Mount overlay slot once using shared offsets and backdrop mode

### Suggested shell profiles

- `PromptShellProfile::Standard`
  - Rounded, `STANDARD_HEIGHT`, footer optional
- `PromptShellProfile::Tall`
  - Rounded, `MAX_HEIGHT` (editor/term-like)
- `PromptShellProfile::EntityFull`
  - Wrapper keeps shell chrome; body entity owns internal layout
- `PromptShellProfile::EdgeToEdge`
  - For terminal-like surfaces that intentionally skip rounded chrome

### Suggested overlay modes

- `OverlayBackdrop::None`
- `OverlayBackdrop::TransparentClickCapture`
- `OverlayBackdrop::Scrim { opacity/token }`

This directly models current behavior without forcing a single visual style.

## How to Add a New Prompt Type (Using the Model)

1. Pick a shell profile (`Standard`, `Tall`, `EdgeToEdge`, etc.).
2. Decide whether header/input uses shared components (`PromptHeader`, `PromptInput`) or a custom slot.
3. Compose body as the only required slot.
4. Use shared `PromptFooter` unless prompt-specific controls require custom footer.
5. If actions exist, select an overlay mode and reuse shared offsets + close wiring.
6. Only add local constants when they cannot map to panel/design tokens.
7. Keep key routing in shell-level handler; keep domain behavior in prompt entity.

## Short-Term Consolidation Targets

1. Extract shared actions overlay host from repeated blocks in:
   - `src/render_prompts/arg/render.rs`
   - `src/render_prompts/div.rs`
   - `src/render_prompts/editor.rs`
   - `src/render_prompts/form/render.rs`
   - `src/render_prompts/term.rs`
2. Migrate repeated single-line input cursor/placeholder rendering to `PromptInput` where multiline behavior is not required.
3. Move prompt-level shell creation to `prompt_shell_container`/`prompt_shell_content` (or equivalent shell builder) for wrappers still hand-rolling the same frame logic.
4. Normalize explicit spacing literals to either panel constants or design tokens, then keep prompt-local constants only for truly unique layout needs.

## Notes

- This audit is intentionally structural; it does not propose behavioral changes by itself.
- Chat and terminal are valid exceptions where custom body/footer semantics are part of the product surface and should remain explicit.
