# EnvPrompt Structure Analysis

## Scope
- `src/prompts/env/render.rs`
- `src/render_prompts/other.rs`
- `src/components/prompt_footer.rs`
- `src/components/focusable_prompt_wrapper.rs`
- `src/components/prompt_input.rs`

## Current EnvPrompt Structure (Observed)
- EnvPrompt renders as a centered, form-like stack (icon/title/description/input/messages/status) plus `PromptFooter` (`src/prompts/env/render.rs:49`, `src/prompts/env/render.rs:277`).
- The prompt still sits inside the shared non-arg wrapper shell (`prompt_shell_container` + `prompt_shell_content`) used by other prompt types (`src/render_prompts/other.rs:125`, `src/render_prompts/other.rs:139`).
- Key routing uses the shared `FocusablePrompt` two-level interception contract, but EnvPrompt only consumes `Escape` in app-level interception (`src/components/focusable_prompt_wrapper.rs:11`, `src/prompts/env/render.rs:319`).

## Divergences That Increase Cognitive Load

### 1. EnvPrompt bypasses the shared input primitive
- `PromptInput` explicitly defines a single-source-of-truth input model for prompts (`src/components/prompt_input.rs:3`, `src/components/prompt_input.rs:8`).
- EnvPrompt hand-builds its own input shell, cursor, placeholder, and masking visuals (`src/prompts/env/render.rs:116`, `src/prompts/env/render.rs:151`, `src/prompts/env/render.rs:173`).
- Impact: users see slightly different input behavior/visual semantics between EnvPrompt and other prompts; maintainers have two input conventions to reason about.

### 2. Message channels are split across body + footer
- Inline `validation_error` appears under the input block (`src/prompts/env/render.rs:190`).
- A separate running-status row is also rendered in the body (`src/prompts/env/render.rs:200`).
- Footer helper text repeats status as `"Script running"` (`src/prompts/env/render.rs:288`).
- Impact: status and guidance are duplicated across locations, so users must scan multiple zones to understand state.

### 3. Footer secondary action diverges from broader prompt mental model
- Shared footer conventions are oriented around a primary action plus optional secondary action button in a fixed right-side action area (`src/components/prompt_footer.rs:456`, `src/components/prompt_footer.rs:472`).
- EnvPrompt uses secondary as `Cancel (Esc)` (`src/prompts/env/render.rs:290`), while non-arg wrapper logic still supports `Cmd+K` actions routing when actions exist (`src/render_prompts/other.rs:30`).
- Impact: users lose the consistent “secondary = actions” expectation in this prompt, while actions may still exist at app-level shortcuts.

### 4. Button + key semantics are fragmented across three places
- Escape is represented in footer (`Cancel`), app-level interception, and entity-level submit-cancel handling (`src/prompts/env/render.rs:290`, `src/prompts/env/render.rs:320`).
- `FocusablePrompt` abstraction expects clear split between intercepted global keys and entity keys (`src/components/focusable_prompt_wrapper.rs:5`).
- Impact: action discoverability is lower because visible affordances and key routing ownership are not aligned to one clear interaction model.

## Standardized Approach (Recommended)

### Error Messages
- Keep exactly one inline message slot directly below the input field (same width as input block).
- Use this slot for both validation and persistence failures; avoid additional body-level error/status rows.
- Keep styling severity-based (`error` token for failures, `muted` token for non-blocking helper copy).

### Helper Text
- Reserve input-adjacent helper text for field semantics only (for EnvPrompt: storage behavior/security notes).
- Reserve footer helper text for runtime/navigation guidance only (for EnvPrompt: Enter/Cmd+K/Esc guidance).
- Remove duplicate running-state copy from the body when footer helper already communicates prompt runtime state.

### Button Placement
- Keep `PromptFooter` right-side action region as the canonical action cluster.
- Standardize secondary button semantics:
  - If actions are available: secondary should be `Actions (⌘K)`.
  - If actions are unavailable: hide secondary button and communicate `Esc cancel` in helper text.
- Keep cancel as a keyboard-global affordance (`Esc`) instead of repurposing the footer secondary button away from actions.

### Centered Variant Compatibility
- The centered EnvPrompt card can remain (it is useful for credential setup), but it should still follow shared message and footer semantics above.
- Keep one consistent narrow column for input, inline message, and helper copy so users read state in one vertical channel.

## Suggested Follow-up Implementation (Not done in this task)
1. Switch EnvPrompt input rendering to `PromptInput` (or extract a shared secure-input variant based on `PromptInput`) to remove duplicate cursor/placeholder logic.
2. Consolidate EnvPrompt status/help copy into footer helper text and remove the in-body running-status row.
3. Normalize EnvPrompt footer config to the same secondary-action policy used by other prompts (`Actions` when available).
