# Arg Prompt Layout Audit

## Scope

- Target files: `src/render_prompts/arg/render.rs`, `src/render_prompts/arg/helpers.rs`, `src/render_prompts/arg/render_input.rs`, `src/render_prompts/arg/render_prompt.rs`
- Audit focus:
  - spacing/layout drift vs shared prompt ecosystem (`prompt_layout_shell`, `PromptInput`, `PromptFooter`, `FocusablePrompt`)
  - scroll/layout-shift behavior during cursor blink and selection changes
  - navigation affordance surprises (selection preservation, tab completion, actions overlay)
  - proposed mapping to canonical shell/header/content/footer hierarchy

## Current Composition (Observed)

Arg prompt is currently rendered by bespoke layout code in `render.rs` instead of shared shell/input primitives.

Current structure (active path):

1. Root custom `div()` frame (`relative + flex_col + w_full + h_full + rounded + on_key_down`) in `src/render_prompts/arg/render.rs:319`
2. Custom header/input row (manual cursor/placeholder/selection rendering) in `src/render_prompts/arg/render.rs:334`
3. Optional divider + list content area when `has_choices` in `src/render_prompts/arg/render.rs:393`
4. Shared `PromptFooter` in `src/render_prompts/arg/render.rs:412`
5. Absolute actions dialog overlay/backdrop in `src/render_prompts/arg/render.rs:454`

Notably, the shared shell helpers (`prompt_shell_container`, `prompt_shell_content`) are not used in ArgPrompt, while they are used in most non-arg wrappers (`src/render_prompts/other.rs:119`).

## Findings

### F1: Shell-level drift from canonical prompt wrapper

- Evidence:
  - ArgPrompt root is hand-rolled: `src/render_prompts/arg/render.rs:319`
  - Canonical wrappers use `prompt_shell_container` and `prompt_shell_content`: `src/render_prompts/other.rs:119`
- Impact:
  - ArgPrompt layout policy (`overflow`, corner clipping, vibrancy wiring, relative overlay host semantics) can drift independently.
  - Future shell behavior changes in shared helpers will not automatically apply to ArgPrompt.

### F2: Header/input drift from `PromptInput` cursor alignment contract

- Evidence:
  - Arg placeholder overlay offsets by `-CURSOR_WIDTH`: `src/render_prompts/arg/render.rs:381`
  - Shared `PromptInput` and `PromptHeader` offset by `-(CURSOR_WIDTH + CURSOR_GAP_X)`: `src/components/prompt_input.rs:463`, `src/components/prompt_header/component.rs:127`
- Impact:
  - Cursor/placeholder x-position is subtly different from shared prompt input behavior.
  - Visual consistency across prompt types is weaker, especially when switching between ArgPrompt and other prompt modes.

### F3: Selection toggle can introduce horizontal micro-shift

- Evidence:
  - Non-selection path always renders a cursor slot (`CURSOR_WIDTH`) in `src/render_prompts/arg/render.rs:73`
  - Selection path renders no cursor slot in `src/render_prompts/arg/render.rs:47`
- Impact:
  - Transitioning between selection/non-selection can shift content by cursor width (~2px).
  - This is a layout-shift class issue tied to selection state changes (even if blink itself is width-stable).

### F4: Selection preservation logic does not guarantee visual visibility after filter change

- Evidence:
  - Selection is preserved by original choice index in `sync_arg_prompt_after_text_change`: `src/render_prompts/arg/helpers.rs:141`
  - Function updates `arg_selected_index` and queues resize, but does not call `scroll_to_item(...)`: `src/render_prompts/arg/helpers.rs:162`
  - Scroll-to-selected is only guaranteed for up/down key paths: `src/render_prompts/arg/render.rs:184`, `src/render_prompts/arg/render.rs:200`
- Impact:
  - After typing/filtering, preserved selection may be off-screen until additional navigation input.
  - Perceived selection “jump” or lost highlight can occur from the user’s perspective.

### F5: Actions overlay anchor uses shared header constant that does not match Arg header geometry

- Evidence:
  - Dialog top offset is derived from `HEADER_TOTAL_HEIGHT`: `src/render_prompts/arg/helpers.rs:4`
  - Arg header row is custom and shorter (input-only row + padding) in `src/render_prompts/arg/render.rs:334`
- Impact:
  - Overlay may sit lower than expected relative to ArgPrompt’s own header/divider anatomy.
  - This makes Arg overlay placement feel inconsistent with the prompt’s visual structure.

### F6: Tab completion behavior has weak visual affordance

- Evidence:
  - Tab completion is implemented (`apply_arg_tab_completion`) in `src/render_prompts/arg/helpers.rs:200`
  - Footer helper strings never mention Tab completion affordance in `src/render_prompts/arg/helpers.rs:100`
- Impact:
  - Feature discoverability is low.
  - Users can perceive Tab as non-functional in multi-match/no-match contexts because UI guidance is Enter-focused.

### F7: Duplicate arg render fragments increase audit and maintenance drift risk

- Evidence:
  - Active include path is `arg/render.rs` + `arg/helpers.rs` via `src/render_prompts/arg.rs:10`
  - Additional near-duplicate files exist: `src/render_prompts/arg/render_prompt.rs`, `src/render_prompts/arg/render_input.rs`
- Impact:
  - Future edits can land in inactive files.
  - Source-string tests and audits become noisier and less trustworthy.

## Blink / Selection / Scroll Stability Summary

- Cursor blink stability: mostly stable for empty and non-selection states because cursor width is always reserved (`src/render_prompts/arg/render.rs:73`).
- Selection-state stability: not fully stable due to missing cursor-width reservation in selection mode (`src/render_prompts/arg/render.rs:47`).
- Scroll stability: up/down navigation is visibility-safe (`scroll_to_item`), but typing/filtering path is not guaranteed visibility-safe (`src/render_prompts/arg/helpers.rs:141`).

## Navigation Surprises (User-Visible)

1. Preserved selection can be off-screen after text edits (selection is logically kept, but not necessarily scrolled into view).
2. Tab completion exists but lacks explicit footer hinting, so affordance is hidden.
3. Actions overlay vertical anchor can feel detached from Arg header because anchor math uses a shared header constant from a different geometry.

## Proposed Canonical ArgPrompt Hierarchy

Target composition contract:

1. Shell:
   - Use `prompt_shell_container(radius, vibrancy_bg)` as outer frame.
   - Keep shell root as the overlay host (`relative`) and key-routing host.
2. Header:
   - Use shared header/input primitive path (prefer `PromptInput` contract or an Arg-specific adapter over it) for cursor gap/alignment parity.
3. Content:
   - Divider + virtualized list content in a `prompt_shell_content(...)` fill slot.
   - Preserve existing list virtualization (`uniform_list`) and list row component (`ListItem`).
4. Footer:
   - Keep `PromptFooter` (already shared) and continue status/info label wiring.
5. Overlay:
   - Keep modal actions overlay behavior, but anchor against Arg header geometry (not just generic `HEADER_TOTAL_HEIGHT`).

## Recommended Refactor Sequence

1. Lift Arg root frame to `prompt_shell_container` + `prompt_shell_content` while preserving existing key handlers and actions overlay behavior.
2. Normalize Arg input cursor spacing to shared input contract (`CURSOR_GAP_X` parity).
3. Reserve cursor slot in selection state to eliminate selection-toggle x-shift.
4. In `sync_arg_prompt_after_text_change`, scroll selected item into view after selection restoration.
5. Add helper-text affordance for Tab completion when filtered choices are available.
6. Remove or quarantine inactive duplicate files (`render_prompt.rs`, `render_input.rs`) once call sites are confirmed.

## Test Coverage Suggestions (for follow-up implementation)

- Add focused tests for:
  - selection mode preserving cursor slot width contract
  - post-filter selection restoration triggers scroll visibility
  - Arg shell path uses shared shell primitives
  - helper text includes Tab affordance in eligible states

