# Theme Consistency Audit

Date: 2026-02-07  
Agent: `codex-theme-consistency`  
Scope: `src/prompts/**/*.rs`, `src/components/**/*.rs`, `src/render_prompts/**/*.rs`

## Summary

The codebase has strong theme-token adoption in many paths, but there are still consistency gaps in three areas:

1. Hardcoded color literals are still present in reusable components and prompt-specific UI (including alpha overlays and fallback palettes).
2. Several prompt render paths mix token systems (`theme.colors.*`, design tokens, and `cx.theme()`), which causes cross-prompt visual drift.
3. Spacing and sizing are partly tokenized, but `env/div/markdown/path/form` still contain many fixed pixel values and bespoke sizing logic.

## Findings

## P0 - Mixed Token Sources Cause Cross-Prompt Visual Drift

### 1) Design-token prompts still pull theme/footer colors from legacy theme paths

- Evidence:
  - `src/prompts/env.rs:353` to `src/prompts/env.rs:359` uses design colors for content, but footer uses theme colors at `src/prompts/env.rs:627`.
  - `src/render_prompts/term.rs:131` uses `PromptFooterColors::from_theme(&self.theme)` while the surrounding container/layout is design-token-driven (`src/render_prompts/term.rs:13` to `src/render_prompts/term.rs:18`).
  - `src/prompts/path.rs:615` and `src/prompts/path.rs:653` force `PromptHeaderColors::from_theme` / `PromptContainerColors::from_theme`, bypassing current design tokens entirely.
- Impact:
  - Same design variant can render with different footer/header surfaces depending on prompt type.
  - Theme switches can look inconsistent between content and footer/header in the same prompt.
- Recommendation:
  - Route all prompt shell/header/footer colors through one source of truth per render path (design tokens for non-default design variants, theme tokens only for default fallback).

### 2) Chat prompt mixes Script Kit theme tokens with GPUI global theme tokens

- Evidence:
  - `src/prompts/chat.rs:2533` and `src/prompts/chat.rs:2539` use `cx.theme().accent_foreground` while neighboring styles use `self.prompt_colors` / `self.theme`.
- Impact:
  - Chat setup CTA can diverge in contrast and brand tone from the rest of the Script Kit theme system.
- Recommendation:
  - Replace `cx.theme()` usage in chat CTA text/icon coloring with Script Kit token mapping derived from `self.theme` / `self.prompt_colors`.

### 3) Select prompt search input bypasses design token background

- Evidence:
  - `src/prompts/select.rs:651` to `src/prompts/select.rs:666` computes per-design colors, but search input background remains `self.theme.colors.background.search_box` at `src/prompts/select.rs:686`.
- Impact:
  - In non-default design variants, the search row can visually mismatch list rows and borders.
- Recommendation:
  - Use variant-resolved token for search input surface (design token in non-default variants, theme token in default).

## P1 - Hardcoded Colors Where Theme Tokens Should Be Used

### 4) Reusable components still encode literal colors in runtime paths

- Evidence:
  - `src/components/form_fields.rs:110` and `src/components/form_fields.rs:129` hardcode cursor cyan `0x00ffff`.
  - `src/components/prompt_header.rs:61` hardcodes `logo_icon: 0x000000`.
  - `src/components/alias_input.rs:74` and `src/components/shortcut_recorder.rs:72` hardcode modal overlay black `0x000000`.
  - `src/components/toast.rs:98` and `src/components/toast.rs:122` hardcode `details_bg: 0x00000020` (same for light/dark).
  - `src/components/button.rs:64` to `src/components/button.rs:68` hardcodes hover overlays as white/black alpha literals.
- Impact:
  - Color behavior is not fully governed by theme JSON/design systems.
  - Light-mode and vibrancy behavior can become inconsistent across components.
- Recommendation:
  - Replace literals with theme-token-derived helpers (for cursor, logo-on-accent, overlays, detail backgrounds).
  - Centralize alpha overlays in one helper API (e.g., `theme::overlay_for_surface(...)`).

### 5) Default color impls are heavily literal and can leak into runtime fallback paths

- Evidence:
  - Literal default palettes in:
    - `src/components/button.rs:127` to `src/components/button.rs:136`
    - `src/components/prompt_header.rs:90` to `src/components/prompt_header.rs:97`
    - `src/components/prompt_footer.rs:83` to `src/components/prompt_footer.rs:87`
    - `src/components/prompt_container.rs:62` to `src/components/prompt_container.rs:66`
    - `src/components/prompt_input.rs:302` to `src/components/prompt_input.rs:307`
    - `src/components/form_fields.rs:141` to `src/components/form_fields.rs:150`
    - `src/components/alias_input.rs:92` to `src/components/alias_input.rs:103`
    - `src/components/shortcut_recorder.rs:90` to `src/components/shortcut_recorder.rs:100`
    - `src/components/toast.rs:137` to `src/components/toast.rs:144`
    - `src/components/scrollbar.rs:117` to `src/components/scrollbar.rs:121`
    - `src/components/unified_list_item/types.rs:224` to `src/components/unified_list_item/types.rs:231`
- Impact:
  - In fallback/uninitialized paths, visuals can diverge from configured theme and design variants.
- Recommendation:
  - Keep defaults only as emergency fallback, but derive from a single canonical theme preset constant, not per-component literal copies.

## P1 - Spacing and Layout Inconsistencies

### 6) Form prompt uses hardcoded height formula instead of shared layout constants

- Evidence:
  - `src/render_prompts/form.rs:152` to `src/render_prompts/form.rs:156` uses `base_height = 150.0`, `field_height = 60.0`, `max_height = 700.0`.
- Impact:
  - Form sizing can drift from `window_resize::layout` changes and from other prompt types.
- Recommendation:
  - Move these values into shared layout constants or compute from design spacing tokens + `window_resize::layout`.

### 7) Path prompt container height behavior differs from other prompt renderers

- Evidence:
  - `src/render_prompts/path.rs:308` uses `.h_full()`.
  - `src/render_prompts/editor.rs:36` and `src/render_prompts/term.rs:35` use explicit `window_resize::layout::MAX_HEIGHT`.
  - `src/render_prompts/div.rs:107` uses explicit `window_resize::layout::STANDARD_HEIGHT`.
- Impact:
  - Path prompt vertical sizing and clipping behavior can differ from other prompt windows.
- Recommendation:
  - Align path renderer with explicit shared layout constants, or standardize root sizing contract for all prompt renderers.

### 8) Rich text prompts rely on many fixed px values outside design spacing tokens

- Evidence:
  - `src/prompts/div.rs:393` to `src/prompts/div.rs:531`, `src/prompts/div.rs:784` to `src/prompts/div.rs:856`.
  - `src/prompts/markdown.rs:509` to `src/prompts/markdown.rs:599`, `src/prompts/markdown.rs:639` to `src/prompts/markdown.rs:689`.
  - `src/prompts/env.rs:395` to `src/prompts/env.rs:460` and `src/prompts/env.rs:551`.
- Impact:
  - Spacing rhythm differs across prompt types and can drift during design refreshes.
- Recommendation:
  - Introduce prompt-level spacing tokens for rich text/content cards and map these fixed values to token names.

## P2 - Hardcoded Alpha Values Should Be Tokenized

### 9) Alpha overlays are consistent in intent but duplicated as literals

- Evidence:
  - `src/prompts/chat.rs:2027` to `src/prompts/chat.rs:2030`, `src/prompts/chat.rs:2225`, `src/prompts/chat.rs:2430` to `src/prompts/chat.rs:2434`.
  - `src/prompts/drop.rs:171` and `src/prompts/template.rs:528` use hardcoded `0x0f` selected-subtle overlays.
- Impact:
  - Hard to tune vibrancy/contrast globally and keep all prompts in sync.
- Recommendation:
  - Define semantic opacity tokens (e.g., `overlay_subtle`, `overlay_hover`, `overlay_selected`) and replace per-file literals.

## Prompt-Type Consistency Matrix

| Prompt type | Status | Key consistency notes |
|---|---|---|
| `select` | Partial | Uses variant-aware text/border tokens, but search box bg bypasses design tokens (`src/prompts/select.rs:686`). |
| `div` | Partial | Theme-aware colors, but many fixed spacing values and custom rich-text spacing (`src/prompts/div.rs:393`, `src/prompts/div.rs:506`). |
| `markdown` | Partial | Theme-aware colors with many fixed px spacings + fixed code font (`src/prompts/markdown.rs:605`). |
| `chat` | Partial | Mostly tokenized, but mixed token source (`cx.theme()`), many literal alpha overlays (`src/prompts/chat.rs:2533`). |
| `env` | Inconsistent | Design-token content + theme-token footer mismatch; many fixed layout spacings (`src/prompts/env.rs:627`). |
| `form` | Mostly consistent | Good token usage, but height formula is hardcoded (`src/render_prompts/form.rs:152`). |
| `editor` | Mostly consistent | Uses design tokens and shared footer helper consistently (`src/render_prompts/editor.rs:203`). |
| `term` | Inconsistent | Uses theme footer colors in design-token render path (`src/render_prompts/term.rs:131`), plus intentional no-radius shell (`src/render_prompts/term.rs:146`). |
| `path` | Inconsistent | Uses theme-only prompt components (`src/prompts/path.rs:615`, `src/prompts/path.rs:653`) while render shell is design-tokenized (`src/render_prompts/path.rs:55`). |

## Recommended Remediation Order

1. Unify token source in prompt shells (`env`, `term`, `path`, `select`) so each prompt resolves colors from one variant-aware path.
2. Replace runtime hardcoded colors in reusable components (`form_fields`, `prompt_header`, `alias_input`, `shortcut_recorder`, `toast`, `button`) with theme/design token helpers.
3. Tokenize alpha overlays and spacing constants used in `chat`, `div`, `markdown`, and `env`.
4. Standardize prompt root heights (`form`/`path` vs `editor`/`term`/`div`) using shared layout constants.

## Validation Notes

This task was an audit/report pass only; no runtime code paths were changed in this agent task.
