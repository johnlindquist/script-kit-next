# Form Prompt Spacing Audit

Snapshot date: 2026-02-11.

## Scope

- `src/render_prompts/form/render.rs`
- `src/components/prompt_footer.rs`
- Supporting call-chain context referenced where needed:
  - `src/main_sections/render_impl.rs`
  - `src/render_prompts/form.rs`
  - `src/render_prompts/arg/helpers.rs`
  - `src/panel.rs`
  - `src/window_resize/mod.rs`
  - `src/form_prompt.rs`

No Rust code changes were made. This is an audit-only document.

## Render Call Chain (Visible Path)

1. `ScriptListApp::render()` dispatches `AppView::FormPrompt` to `render_form_prompt(...)` (`src/main_sections/render_impl.rs:159`, `src/main_sections/render_impl.rs:173`).
2. `main.rs` includes `render_prompts/form.rs`, which includes `form/helpers.rs` and `form/render.rs` (`src/main.rs:299`, `src/render_prompts/form.rs:10`, `src/render_prompts/form.rs:11`).
3. `render_form_prompt(...)` resolves design tokens, footer config, and actions-dialog offsets (`src/render_prompts/form/render.rs:11`, `src/render_prompts/form/render.rs:17`, `src/render_prompts/form/render.rs:213`).
4. Form wrapper root is built as `relative + flex_col + w_full + h(content_height) + overflow_hidden + rounded(radius_lg)` (`src/render_prompts/form/render.rs:185`, `src/render_prompts/form/render.rs:192`, `src/render_prompts/form/render.rs:194`).
5. Scrollable content slot is the first child: `flex_1 + min_h(0) + overflow_y_scrollbar + p(padding_xl)` with `entity.clone()` inside (`src/render_prompts/form/render.rs:201`, `src/render_prompts/form/render.rs:205`, `src/render_prompts/form/render.rs:206`).
6. `Entity<FormPromptState>` renders its own internal stack with `.gap(16)` and optional empty-state `.p(16)` (`src/form_prompt.rs:229`, `src/form_prompt.rs:243`).
7. Footer is appended as second child via `PromptFooter::new(...)` (`src/render_prompts/form/render.rs:210`, `src/render_prompts/form/render.rs:222`).
8. Actions dialog is appended last as absolute overlay on the same root, anchored with `.top(actions_dialog_top)` and `.right(actions_dialog_right)` (`src/render_prompts/form/render.rs:233`, `src/render_prompts/form/render.rs:268`, `src/render_prompts/form/render.rs:269`).

## Wrapper Layout Audit

### 1) Container sizing

`render_form_prompt` computes wrapper height from local literals:

- `base_height = 150.0`
- `field_height = 60.0`
- `max_height = 700.0`
- `calculated_height = base_height + field_count * field_height`
- `content_height = min(calculated_height, max_height)`

Source: `src/render_prompts/form/render.rs:176`-`src/render_prompts/form/render.rs:180`.

Classification:
- Token/layout-metric driven:
  - None in this height formula.
- Ad-hoc/local values:
  - `150`, `60`, `700` are local literals in this function (with only a comment reference to `window_resize::layout::MAX_HEIGHT`).

### 2) Content scrolling slot

Outer scroll slot (wrapper-owned):
- `flex_1`, `w_full`, `min_h(0)`, `overflow_y_scrollbar`, `p(padding_xl)`
- Source: `src/render_prompts/form/render.rs:202`-`src/render_prompts/form/render.rs:207`.

Inner form entity (entity-owned):
- Vertical stack gap: `.gap(px(16.))`
- Empty-state fallback padding: `.p(px(16.))`
- Source: `src/form_prompt.rs:229`, `src/form_prompt.rs:243`.

Classification:
- Token/layout-metric driven:
  - Wrapper content inset uses `design_spacing.padding_xl`.
- Ad-hoc/local values:
  - Inner entity gap/padding are fixed `16` literals.

### 3) Footer placement

Placement behavior in wrapper:
- Footer is a sibling below the scroll slot (content first, footer second).
- Content slot uses `flex_1 + min_h(0)` and footer uses fixed height + `flex_shrink_0`, so footer remains pinned at bottom and content absorbs overflow.
- Source: `src/render_prompts/form/render.rs:200`-`src/render_prompts/form/render.rs:231`, `src/components/prompt_footer.rs:493`, `src/components/prompt_footer.rs:496`.

Footer geometry sources:
- Height: `FOOTER_HEIGHT = 30.0` (`src/window_resize/mod.rs:229`, used in `src/components/prompt_footer.rs:493`).
- Footer internal spacing constants (non-tokenized):
  - `PROMPT_FOOTER_PADDING_X_PX = 12.0` (`src/components/prompt_footer.rs:54`)
  - `PROMPT_FOOTER_PADDING_BOTTOM_PX = 2.0` (`src/components/prompt_footer.rs:56`)
  - Section gap `8.0` (`src/components/prompt_footer.rs:42`)
  - Buttons gap `4.0` (`src/components/prompt_footer.rs:44`)
  - Button inner spacing/radius `gap 6`, `px 8`, `py 6`, `rounded 4` (`src/components/prompt_footer.rs:366`-`src/components/prompt_footer.rs:369`)

Classification:
- Token/layout-metric driven:
  - Footer height uses shared layout metric `FOOTER_HEIGHT`.
- Ad-hoc/local values:
  - Most footer spacing is hardcoded via module constants, not design spacing tokens.

### 4) Actions dialog overlay offsets

Offset resolver used by form wrapper:
- `prompt_actions_dialog_offsets(design_spacing.padding_sm, design_visual.border_thin)`
- Source usage: `src/render_prompts/form/render.rs:16`-`src/render_prompts/form/render.rs:17`.

Shared formula definition:
- `top = HEADER_TOTAL_HEIGHT + padding_sm - border_thin`
- `right = padding_sm`
- Source: `src/render_prompts/arg/helpers.rs:2`-`src/render_prompts/arg/helpers.rs:6`.

Shared constants:
- `HEADER_TOTAL_HEIGHT = 45.0` (`src/panel.rs:72`)
- Default-variant regression test validates `top=52`, `right=8` (`src/render_prompts/arg/tests.rs:11`-`src/render_prompts/arg/tests.rs:13`).

Overlay composition in form wrapper:
- Full-surface absolute layer (`inset_0`) with clickable backdrop and top-right dialog anchor.
- Source: `src/render_prompts/form/render.rs:255`-`src/render_prompts/form/render.rs:271`.

Classification:
- Token/layout-metric driven:
  - Right offset uses spacing token (`padding_sm`).
  - Top offset depends on shared panel metric (`HEADER_TOTAL_HEIGHT`) and token inputs.
- Ad-hoc/local values:
  - None in the formula itself, but formula assumes a header geometry that this wrapper does not render.

## Spacing Source Map (Token vs Shared Metric vs Ad-hoc)

| Area | Value(s) | Source Type | Where |
|---|---|---|---|
| Root corner radius | `design_visual.radius_lg` | Design token | `src/render_prompts/form/render.rs:194` |
| Root height formula | `150 + field_count*60`, cap `700` | Ad-hoc literals | `src/render_prompts/form/render.rs:176`-`src/render_prompts/form/render.rs:180` |
| Content inset | `design_spacing.padding_xl` | Design token | `src/render_prompts/form/render.rs:206` |
| Inner form stack gap | `16` | Ad-hoc literal | `src/form_prompt.rs:229` |
| Empty-state inset | `16` | Ad-hoc literal | `src/form_prompt.rs:243` |
| Footer height | `FOOTER_HEIGHT = 30` | Shared layout metric | `src/window_resize/mod.rs:229`, `src/components/prompt_footer.rs:493` |
| Footer horizontal inset | `12` | Ad-hoc module constant | `src/components/prompt_footer.rs:54` |
| Footer bottom inset | `2` | Ad-hoc module constant | `src/components/prompt_footer.rs:56` |
| Footer section/button gaps | `8`, `4` | Ad-hoc module constants | `src/components/prompt_footer.rs:42`, `src/components/prompt_footer.rs:44` |
| Actions offset right | `padding_sm` | Design token | `src/render_prompts/arg/helpers.rs:5` |
| Actions offset top | `HEADER_TOTAL_HEIGHT + padding_sm - border_thin` | Shared metric + tokens | `src/render_prompts/arg/helpers.rs:4` |

## Findings

1. Height sizing in `render_form_prompt` is currently local and non-tokenized.
- The `150/60/700` literals are not tied to `window_resize::layout` constants except by comment.

2. Content spacing ownership is split across wrapper and entity with mixed sources.
- Wrapper uses tokenized `padding_xl`.
- `FormPromptState::render` applies fixed `16px` spacing internally.

3. Footer placement is structurally solid, but spacing rhythm is mixed.
- Footer pinning behavior is correct (`flex_shrink_0` with fixed height).
- Footer uses mostly fixed constants, while content inset is token-driven.

4. Actions overlay top anchor inherits “header-aware” math from shared helper.
- The formula uses `HEADER_TOTAL_HEIGHT`, but `render_form_prompt` itself does not render an explicit header block.
- That may be intentional for cross-prompt consistency, but it is coupling-by-assumption.

## Normalization Suggestions (No Rust Changes Applied)

1. Normalize form wrapper height metrics into shared constants.
- Replace local `150/60/700` literals with named constants (or reuse `window_resize::layout::*` directly where appropriate).
- Keep formula behavior, but move ownership to one shared location to reduce drift.

2. Define a single horizontal inset contract for form content and footer.
- Today: content defaults to token `padding_xl`, footer defaults to fixed `12`.
- Suggested: choose one canonical inset source per prompt class (either tokenized or shared constant), then apply to content and footer consistently.

3. Move inner form spacing literals toward token-backed metrics.
- Convert `FormPromptState`’s `gap(16)` and empty-state `p(16)` into design-token-derived values (or a small shared `FORM_PROMPT_*` metric set).

4. Split overlay offset strategies by prompt chrome type.
- Keep current `HEADER_TOTAL_HEIGHT`-based offset for prompts that render explicit header+divider chrome.
- Add a form-specific variant (or parameter) for headerless wrappers so top anchoring derives from visible form shell geometry instead of inherited header math.

5. Keep footer height on shared metric, but document and/or align footer spacing constants.
- `FOOTER_HEIGHT` is already centralized and should remain so.
- Footer internal spacing constants can be grouped into a token/metric map to make cross-prompt alignment intentional.

## Known Visibility Boundaries

- This audit covers wrapper and footer spacing surfaces plus the immediately visible inner form entity spacing.
- Field-level component internals (`FormTextField`, `FormTextArea`, `FormCheckbox`) were not expanded here.
- No runtime screenshot validation was performed because this assignment was documentation-only and requested no UI code changes.
