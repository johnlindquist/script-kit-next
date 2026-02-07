# Div Prompt Improvements Audit

Date: 2026-02-07  
Agent: `codex-div-prompt`  
Scope: `src/prompts/div.rs`, `src/render_prompts/div.rs`

## Executive Summary

The div prompt pipeline works for basic rich text, but it currently behaves more like a styled plain-text viewer than a true HTML/CSS prompt.

Highest-impact improvements:

1. Preserve inline semantics (bold/italic/link/code) instead of flattening most blocks into plain text.
2. Expand CSS/Tailwind support beyond the current minimal subset.
3. Fix scroll state/UX gaps (unused scroll offset, no keyboard scrolling, no visible scroll affordance).
4. Add keyboard-accessible interactive elements (at minimum links, then button-like submit actions).
5. Reduce render-time work by avoiding full HTML parse/tree rebuild on every render pass.

## Current Responsibility Map

`src/render_prompts/div.rs` currently:

1. Wraps `DivPrompt` in prompt shell/header/footer chrome (`src/render_prompts/div.rs:123`).
2. Uses fixed shell height (`window_resize::layout::STANDARD_HEIGHT`) (`src/render_prompts/div.rs:107`).
3. Handles parent-level key routing for Cmd+K/actions/global shortcuts (`src/render_prompts/div.rs:25`).
4. Renders actions overlay/backdrop (`src/render_prompts/div.rs:189`).

`src/prompts/div.rs` currently:

1. Parses HTML on each render (`src/prompts/div.rs:887`).
2. Renders supported `HtmlElement` nodes to GPUI divs (`src/prompts/div.rs:336`).
3. Applies subset Tailwind classes via `TailwindStyles` mapping (`src/prompts/div.rs:555`).
4. Handles click-only links and submit links (`submit:<value>`) (`src/prompts/div.rs:508`, `src/prompts/div.rs:892`).
5. Enables vertical scroll with a tracked `ScrollHandle` (`src/prompts/div.rs:1000`).

## Findings (Ranked)

### P1: Block rendering flattens rich inline content into plain text

Evidence:

1. Paragraph/header/list/blockquote rendering calls `collect_text(children)` (`src/prompts/div.rs:357`, `src/prompts/div.rs:371`, `src/prompts/div.rs:424`, `src/prompts/div.rs:494`).
2. `collect_text` strips structure by concatenating descendants (`src/prompts/div.rs:709`).
3. A richer inline renderer exists (`render_inline`) but is never used (`src/prompts/div.rs:751`).

Impact:

1. Nested formatting is lost in common cases (for example links/code/strong inside paragraphs and list items).
2. Output fidelity diverges from expected HTML behavior.

Recommendation:

1. Use `render_inline` (or a new inline-run renderer) inside paragraph/header/list/blockquote branches instead of `collect_text`.
2. Keep `collect_text` only for fallback/plain extraction paths.

### P1: HTML feature surface is narrow and silently degrades unsupported semantics

Evidence:

1. Parser supports a small tag set (`h1-h6`, `p`, `strong/em`, `code/pre`, lists, `blockquote`, `a`, `div`, `span`) (`src/utils/html.rs:375`).
2. Unknown tags collapse into child content with no semantic preservation (`src/utils/html.rs:487`).
3. `CodeBlock.language` is parsed but ignored by renderer (`src/utils/html.rs:69`, `src/prompts/div.rs:402`).

Impact:

1. Tables/images/semantic containers lose meaning.
2. Authors have little feedback when markup is dropped.

Recommendation:

1. Add explicit unsupported-tag fallback rendering (for example neutral container with optional warning in debug logs).
2. Surface `language` for code blocks as a label/badge in the rendered block.

### P1: CSS/Tailwind support is limited for real-world div prompt layouts

Evidence:

1. `apply_tailwind_styles` only maps a subset: flex/sizing basics, spacing, colors, font weight/size, border/border radius (`src/prompts/div.rs:555`).
2. `TailwindStyles` lacks overflow, max/min sizes (beyond `min-w/h-0`), text alignment, opacity, positioning, width/height scales, underline/decoration, shadows (`src/utils/tailwind.rs:5`).
3. Unknown classes are ignored without diagnostics (`src/utils/tailwind.rs:122`).

Impact:

1. Many SDK-authored div UIs cannot be expressed reliably.
2. Styling failures are hard to debug because unsupported classes fail silently.

Recommendation:

1. Expand style surface in phases: overflow + width/height scales + text alignment + opacity + decoration first.
2. Add optional debug logging for dropped/unsupported classes (behind compact debug category).

### P1: Scroll state is partially implemented but not surfaced

Evidence:

1. `DivPrompt` stores `scroll_offset` and exposes `scroll_offset_y()` (`src/prompts/div.rs:159`, `src/prompts/div.rs:245`).
2. `scroll_offset` is never updated in the prompt implementation.
3. Content is scrollable (`overflow_y_scroll`) but there is no scrollbar/status affordance in this prompt (`src/prompts/div.rs:1000`, `src/render_prompts/div.rs`).

Impact:

1. No visibility into long-content position.
2. Existing scroll-state fields create false confidence and dead-state complexity.

Recommendation:

1. Either wire scroll offset updates and render a scrollbar/status indicator, or remove unused offset fields.
2. Add keyboard scroll controls (PageUp/PageDown/Space/Shift+Space/Home/End) in div prompt context.

### P1: Interactive support is mouse-first and not keyboard accessible

Evidence:

1. Links use `on_mouse_down(MouseButton::Left, ...)` only (`src/prompts/div.rs:519`).
2. No focusable link model, no Enter/Space activation path for links.
3. Key handling in `DivPrompt` only recognizes Enter/Escape submit (`src/prompts/div.rs:872`).

Impact:

1. Keyboard-only users cannot activate in-content links/actions.
2. Accessibility and launcher-like power-user workflows are limited.

Recommendation:

1. Introduce focusable interactive runs for links (tab order + Enter/Space activation).
2. Add visible focus style for active interactive element.
3. Keep parent-level prompt submit behavior, but do not block focused element activation.

### P2: Keyboard semantics are split across wrapper and prompt entity

Evidence:

1. Parent wrapper intercepts key events “before DivPrompt” for global/actions logic (`src/render_prompts/div.rs:23`).
2. `DivPrompt` also binds its own key handler (`src/prompts/div.rs:872`).

Impact:

1. Harder to reason about precedence (global shortcut vs prompt submit vs in-content interaction).
2. Future interactive-element additions become brittle.

Recommendation:

1. Define explicit key-routing contract for div prompt:
   - Layer 1: actions modal/global
   - Layer 2: focused interactive element
   - Layer 3: prompt-level submit/dismiss
2. Encode this in one shared helper to reduce divergence.

### P2: Expensive render path for large HTML content

Evidence:

1. `parse_html(&self.html)` is called every render (`src/prompts/div.rs:887`).
2. Render tree is rebuilt every frame for all elements (`src/prompts/div.rs:325`).

Impact:

1. Potential frame cost spikes for long or dynamic HTML content.
2. Repeated allocations/parsing work when HTML is unchanged.

Recommendation:

1. Cache parsed AST by `html` content hash/version and only reparse on changes.
2. Consider lightweight memoization for transformed Tailwind class maps.

### P2: Container options docs and behavior are inconsistent

Evidence:

1. `ContainerOptions.opacity` doc says “applies to entire container” (`src/prompts/div.rs:30`).
2. Implementation only applies opacity to computed background color (`src/prompts/div.rs:950`).

Impact:

1. Script authors can misconfigure expecting full-content opacity.
2. API contract ambiguity increases support/debug burden.

Recommendation:

1. Either update docs to “background opacity” or implement full container opacity behavior.
2. Add focused unit tests for background + opacity interactions.

### P3: Dead/duplicate code paths in div prompt internals

Evidence:

1. `handle_link_click` duplicates link protocol logic already in render callback (`src/prompts/div.rs:261`, `src/prompts/div.rs:892`).
2. `RenderContext::with_link_callback` is unused (`src/prompts/div.rs:318`).
3. `render_inline` is unused (`src/prompts/div.rs:751`).

Impact:

1. Maintenance overhead and drift risk.
2. Harder to reason about intended architecture.

Recommendation:

1. Consolidate into a single link handling path.
2. Remove dead helpers or integrate them into the main renderer.

## Priority Implementation Plan

### Phase 1 (Fidelity + Accessibility)

1. Switch block rendering to preserve inline structure.
2. Implement keyboard focus/activation for links.
3. Unify key routing precedence between wrapper and prompt entity.

### Phase 2 (CSS + Layout)

1. Expand Tailwind mapping for core missing layout/typography utilities.
2. Add diagnostics for unsupported classes in debug logs.
3. Resolve `ContainerOptions.opacity` contract mismatch.

### Phase 3 (Scroll + Performance)

1. Wire real scroll offset state + visible scroll affordance.
2. Add keyboard scroll controls.
3. Cache parsed HTML tree and class maps to avoid repeated parse/alloc churn.

## Suggested Tests (TDD Names)

1. `test_div_prompt_preserves_inline_formatting_inside_paragraphs`
2. `test_div_prompt_renders_link_inside_list_item_without_flattening_text`
3. `test_div_prompt_activates_focused_link_with_enter`
4. `test_div_prompt_supports_page_navigation_keys_for_scroll`
5. `test_div_prompt_reports_or_logs_unsupported_tailwind_classes`
6. `test_div_prompt_uses_cached_html_ast_when_html_unchanged`
7. `test_div_prompt_container_opacity_matches_documented_behavior`
8. `test_div_prompt_key_routing_prioritizes_actions_then_interactive_elements_then_submit`

## Risks / Known Gaps

1. Full HTML fidelity is intentionally bounded; adding too many tags/utilities without a clear support matrix may increase regressions.
2. Keyboard-focusable inline interactions need careful integration with GPUI focus model to avoid stealing global shortcuts.
3. HTML AST caching must be invalidated correctly when theme/design variant affects rendered output.
