<!-- markdownlint-disable MD013 -->

# DivPrompt Entity Layout Audit

## Scope

- Audited file: `src/prompts/div/render.rs`
- Supporting checks: `src/prompts/div/tests.rs`, wrapper shell in `src/render_prompts/div.rs`
- Focus: scroll region ownership, padding rhythm, background/vibrancy behavior, Tailwind class layering
- Audit date: 2026-02-11

## Current Contract (Observed)

- `DivPrompt` is rendered inside the shared shell content slot (`prompt_shell_content`) and is responsible for its own internal scrolling (`src/prompts/div/render.rs:114`-`src/prompts/div/render.rs:119`).
- Background resolution uses explicit precedence: `container_options.background` -> `container_options.opacity` -> vibrancy foundation (`src/prompts/div/render.rs:57`-`src/prompts/div/render.rs:77`).
- Root `tailwind` classes apply to parsed HTML content (`src/prompts/div/render.rs:90`-`src/prompts/div/render.rs:95`), while `containerClasses` apply to the scroll-container base before `.id()` and `overflow_y_scroll()` (`src/prompts/div/render.rs:97`-`src/prompts/div/render.rs:119`).

## Findings

### 1) Scroll rhythm risk from outer-owned padding

- Before this audit, padding was applied on the outer full-height container instead of the scroll owner.
- This produced edge pressure on long content because top/bottom breathing room was not owned by the scrolled region itself.
- Result: first/last lines could feel visually clipped against the viewport edge during overflow scenarios.

### 2) Background and vibrancy layering is structurally correct

- The renderer applies at most one background surface and defers to root vibrancy when no override is requested.
- This matches prompt-shell expectations and avoids double-tinting.

### 3) Tailwind layering is mostly safe, but should stay narrowly scoped

- Current order is correct: content styles first, then container classes, then stateful scroll decorators.
- This avoids breaking `overflow_y_scroll()` requirements while preserving legacy `tailwind` behavior.

## Canonical Rules For Content-Only Prompts

1. **Scroll container ownership**
- Exactly one element should own `overflow_y_scroll()` and `track_scroll(...)`.
- The outer root should remain non-scrolling and only provide sizing/background shell behavior.

2. **Padding standards**
- Default content inset uses design token `spacing.padding_md` unless explicitly overridden.
- Content inset must be applied on the scroll owner, not the outer root, so breathing room participates in scroll range.

3. **Background/vibrancy policy**
- Background precedence: explicit `background` > explicit `opacity` over base theme color > vibrancy fallback.
- If no background override exists, allow vibrancy to come from root (`None` from foundation helper).
- Avoid stacking multiple opaque backgrounds across wrapper + entity layers.

4. **Tailwind class policy**
- `tailwind` (legacy root classes) styles parsed HTML content only.
- `containerClasses` styles the scroll container base and must be applied before `.id()` + `overflow_y_scroll()`.
- Do not move Tailwind application after stateful scroll decorators.

## Changes Made In This Audit

- Moved `container_padding` from outer root to the scroll container base in `src/prompts/div/render.rs:99`-`src/prompts/div/render.rs:106` and removed outer `.p(...)` at `src/prompts/div/render.rs:123`-`src/prompts/div/render.rs:132`.
- Added regression coverage in `src/prompts/div/tests.rs:118`-`src/prompts/div/tests.rs:143` to enforce padding ownership on scroll container.

## Remaining Risk / Follow-up

- `container_options.background` parsing currently applies `opacity` for named colors but not uniformly for all hex forms. This is outside this audit’s scoped change, but should be normalized if background APIs are tightened.
