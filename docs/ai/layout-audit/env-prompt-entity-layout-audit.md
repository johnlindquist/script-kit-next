<!-- markdownlint-disable MD013 -->

# EnvPrompt Entity Layout Audit

## Scope

- Reviewed: `src/prompts/env/render.rs`
- Compared against shared shell/container vocabulary used by prompt wrappers and entity components.

## Current Contract (Observed)

- `EnvPrompt` is wrapped by the shared shell in `src/render_prompts/other.rs:125`-`src/render_prompts/other.rs:140` (`prompt_shell_container` + `prompt_shell_content`), so outer frame behavior is standardized.
- Inside the entity, content uses a full-height column with a centered main stack (`.flex_1().items_center().justify_center()`) and a bottom `PromptFooter` (`src/prompts/env/render.rs:49`-`src/prompts/env/render.rs:311`).
- The main stack is card-like: icon, title, description, input label/input/hint, error text, running-state row, and optional configured/delete row (`src/prompts/env/render.rs:58`-`src/prompts/env/render.rs:275`).
- Width constraints are partial: input/error/running rows are capped at `max_w(400)`, while title/description and configured/delete row are not consistently bound to the same width (`src/prompts/env/render.rs:104`, `src/prompts/env/render.rs:193`, `src/prompts/env/render.rs:203`, `src/prompts/env/render.rs:227`).
- Footer uses `PromptFooter` correctly but carries helper text "Script running" while a separate running-status row also exists in body (`src/prompts/env/render.rs:200`-`src/prompts/env/render.rs:215`, `src/prompts/env/render.rs:285`-`src/prompts/env/render.rs:289`).

## Inconsistencies / Potential Surprises

### 1. Centered layout vs top-aligned prompt rhythm

- Most prompt entities follow a top-anchored "header + content" rhythm (example: `PathPrompt` via `PromptHeader`/`PromptContainer` in `src/prompts/path/render.rs:90`-`src/prompts/path/render.rs:144`).
- EnvPrompt centers everything vertically (`src/prompts/env/render.rs:53`-`src/prompts/env/render.rs:56`).
- Result: switching between EnvPrompt and list/navigation prompts can feel like a different product mode, not just a different prompt type.

### 2. Mixed spacing vocabulary (tokenized + hard-coded)

- EnvPrompt uses many fixed pixel values (`32`, `24`, `16`, `12`, `8`) directly in the entity (`src/prompts/env/render.rs:55`, `src/prompts/env/render.rs:56`, `src/prompts/env/render.rs:118`-`src/prompts/env/render.rs:127`).
- Other prompts often route spacing through shared token/config surfaces (for example, `PromptContainer` and tokenized spacing usage in list prompts).
- Result: visual rhythm drift risk when spacing tokens evolve.

### 3. Width rhythm is not unified per vertical stack

- The input/status blocks are constrained to 400px, but adjacent title/description and "Configured/Delete" row can be wider (`src/prompts/env/render.rs:82`-`src/prompts/env/render.rs:99`, `src/prompts/env/render.rs:227`-`src/prompts/env/render.rs:274`).
- Result: horizontal alignment edges jump between rows, which weakens scanability.

### 4. Focus treatment is visually "always-on"

- Input border is always accent-tinted (`src/prompts/env/render.rs:123`), and empty-state cursor is always painted accent (`src/prompts/env/render.rs:159`-`src/prompts/env/render.rs:163`).
- No focus-dependent visual state is threaded in this renderer (unlike shared header input patterns that explicitly encode focused/cursor-visible state).
- Result: reduced clarity about active focus, especially when multiple interactive controls are present.

### 5. Duplicate running-state channel (body + footer)

- Running status appears in both body row and footer helper text (`src/prompts/env/render.rs:200`-`src/prompts/env/render.rs:215`, `src/prompts/env/render.rs:288`).
- Result: duplicated information and competing emphasis.

## Recommended Fit In Unified Vocabulary

### A. Classify EnvPrompt as a first-class "Setup Card" variant

- Keep centered layout as intentional for credential/setup moments.
- Explicitly document this as a sanctioned variant (similar to setup-card behavior in `src/prompts/chat/render_setup.rs:20`-`src/prompts/chat/render_setup.rs:60`), instead of treating it as a one-off exception.

### B. Use one narrow-column token for the entire content stack

- Introduce one shared max-width for all body rows in EnvPrompt (title, description, input, error, status, configured/delete) rather than per-row ad hoc caps.
- This preserves centered composition while restoring left/right edge consistency.

### C. Align focus language with shared prompt input behavior

- Add explicit focused/unfocused states for input border/cursor rendering so focus affordance is not always-on.
- Keep keyboard routing unchanged; this is purely visual consistency.

### D. Keep one status channel

- Prefer footer helper text for persistent runtime status, and reserve in-body status row for exceptional states (error/verification), or invert this rule consistently.
- This removes duplicate copy and clarifies hierarchy.

### E. Tokenize spacing constants used in EnvPrompt body

- Replace hardcoded body spacing values with design spacing tokens used elsewhere.
- This keeps EnvPrompt synced with system-wide rhythm changes.

## Suggested Canonical Rule

- `Top-aligned container rhythm` is the default for operational prompts (search/list/editor/form/path).
- `Centered setup-card rhythm` is allowed only for "blocking setup" prompts (credentials/onboarding) and must still obey shared width, spacing-token, focus-state, and footer-status rules.
