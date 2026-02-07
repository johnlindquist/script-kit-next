# Header/Footer Improvements Audit

Date: 2026-02-07  
Agent: `codex-header-footer`  
Scope: `src/components/prompt_header.rs`, `src/components/prompt_footer.rs`, `src/components/footer_button.rs`, `tests/prompt_footer.rs`

## Executive Summary

The current shared header/footer components are usable and consistent enough to ship, but they are still optimized for a narrow two-button flow and have limited primitives for richer prompt state.

Highest-impact improvements:

1. Introduce semantic metadata slots in header/footer (`status`, `breadcrumbs`, `meta chips`) rather than overloading plain strings.
2. Make button rendering action-driven (typed action model) and add active/disabled states so UI reflects runtime capability.
3. Improve information density with truncation and priority rules to prevent collisions when helper text, info labels, and long paths coexist.
4. Replace brittle string-based tests with contract tests around configuration behavior.

## Current State (What Exists)

### Header (`src/components/prompt_header.rs`)

1. Supports filter text + placeholder + optional path prefix (`src/components/prompt_header.rs:107`-`src/components/prompt_header.rs:130`).
2. Supports primary action + optional Actions button (`src/components/prompt_header.rs:114`-`src/components/prompt_header.rs:120`).
3. Supports actions-mode UI swap between buttons and actions search via stacked absolute layers (`src/components/prompt_header.rs:563`-`src/components/prompt_header.rs:629`).
4. Supports optional `Ask AI [Tab]` hint (`src/components/prompt_header.rs:509`-`src/components/prompt_header.rs:539`).
5. Uses fixed action-area minimum width (`.min_w(px(200.))`) regardless of available layout (`src/components/prompt_header.rs:623`).

### Footer (`src/components/prompt_footer.rs`)

1. Supports optional logo, helper text, info label, primary/secondary buttons (`src/components/prompt_footer.rs:94`-`src/components/prompt_footer.rs:110`).
2. Footer height is fixed to `FOOTER_HEIGHT` (currently 30px) and uses top border + overlay background (`src/components/prompt_footer.rs:320`-`src/components/prompt_footer.rs:344`, `src/window_resize.rs:36`).
3. Helper text is accent-colored and unbounded in width (no truncation constraints) (`src/components/prompt_footer.rs:355`-`src/components/prompt_footer.rs:360`).
4. `PromptFooterColors.background` is defined but not used in render; render always pulls from cached theme background (`src/components/prompt_footer.rs:51`, `src/components/prompt_footer.rs:312`).

### Footer Button (`src/components/footer_button.rs`)

1. Hover and text colors are always derived from global cached theme, not from `PromptFooterColors` (`src/components/footer_button.rs:62`-`src/components/footer_button.rs:65`).
2. No disabled/active/loading semantics, no pressed state, no ARIA-like semantic identifiers beyond optional id (`src/components/footer_button.rs:18`-`src/components/footer_button.rs:23`).

### Tests (`tests/prompt_footer.rs`)

1. Tests are source-string assertions instead of behavior/contract tests (`tests/prompt_footer.rs:4`-`tests/prompt_footer.rs:45`).
2. Coverage does not validate helper/info overflow behavior, button visibility logic, color-source consistency, or config composition.

## Findings (Ranked)

### P1: Information density degrades quickly with long helper/info text

Evidence:

1. Helper text and info text are rendered inline with no max width, no truncation, and no priority collapse (`src/components/prompt_footer.rs:274`-`src/components/prompt_footer.rs:308`, `src/components/prompt_footer.rs:355`-`src/components/prompt_footer.rs:360`).
2. Header input/path section has no explicit clipping strategy when path prefixes get long (`src/components/prompt_header.rs:292`-`src/components/prompt_header.rs:338`).
3. Header action area reserves hard minimum 200px (`src/components/prompt_header.rs:623`), reducing space for input content on narrow widths.

Impact:

1. Footer can visually crowd or clip in prompt variants that show rich helper text (chat model names, verbose running hints).
2. Path-style prompts can reduce visible query context due to fixed action footprint.

Recommendation:

1. Add `max_w` + ellipsis behavior to helper/info/prefix zones with explicit priority:
   - keep primary button visible first,
   - then keep query text,
   - collapse helper/info into short chips on width pressure.
2. Replace single fixed `min_w(200)` with responsive action width policy (`compact`, `normal`, `expanded`) controlled by config.

### P1: Footer color contract is inconsistent and partially dead

Evidence:

1. `PromptFooterColors.background` is never consumed in `render()` (`src/components/prompt_footer.rs:51`, `src/components/prompt_footer.rs:312`-`src/components/prompt_footer.rs:317`).
2. `FooterButton` ignores footer color inputs and reads global theme directly (`src/components/footer_button.rs:62`-`src/components/footer_button.rs:65`).

Impact:

1. `PromptFooterColors` appears configurable but does not fully control rendered output.
2. Design token refactors become error-prone because actual color source is split across component boundaries.

Recommendation:

1. Make color ownership explicit:
   - either remove unused `background` from `PromptFooterColors`, or
   - use it consistently for footer surface.
2. Pass resolved `FooterButtonColors` into `FooterButton` from `PromptFooter` so theme/color behavior is centralized.

### P1: Action buttons lack semantic state and runtime feedback

Evidence:

1. Footer buttons only support `label`, `shortcut`, click callback (`src/components/footer_button.rs:18`-`src/components/footer_button.rs:52`).
2. No disabled styling/click suppression path.
3. Header buttons are static text + shortcuts and cannot represent active actions mode except by full layer swap (`src/components/prompt_header.rs:379`-`src/components/prompt_header.rs:408`, `src/components/prompt_header.rs:568`-`src/components/prompt_header.rs:601`).

Impact:

1. UI cannot reflect action availability or transient states (running, unavailable, loading).
2. Users get less clarity about why certain actions are absent/inactive.

Recommendation:

1. Introduce a typed button model:
   - `ActionButtonSpec { id, label, shortcut, state: Enabled|Disabled|Active|Loading, tone }`.
2. Render consistent visual states and suppress callbacks when disabled.
3. For header actions mode, prefer active-state toggle indicator on the Actions button instead of hard swapping full layout where feasible.

### P2: Breadcrumb model is underpowered for path-like prompts

Evidence:

1. Header exposes only `path_prefix: Option<String>` as plain text (`src/components/prompt_header.rs:112`-`src/components/prompt_header.rs:113`, `src/components/prompt_header.rs:293`-`src/components/prompt_header.rs:298`).
2. No segment-level structure, no truncation-aware middle collapse, no clickable segments, no overflow summary.

Impact:

1. Deep paths become hard to parse and consume excessive horizontal space.
2. No clean route to richer breadcrumb UX across prompts.

Recommendation:

1. Replace/extend `path_prefix` with structured breadcrumbs:
   - `Vec<BreadcrumbSegment { label, full_path, is_clickable }>`.
2. Add optional render mode:
   - `InlinePrefix` (current),
   - `CollapsedBreadcrumb` (e.g. `~/…/project/src/`).
3. Preserve current API as compatibility shim to avoid call-site churn.

### P2: Status indicators are overloaded into plain helper strings

Evidence:

1. Running status text is composed as a generic string (`src/render_prompts/arg.rs:13`, `src/panel.rs:83`-`src/panel.rs:85`) and passed into `helper_text` (`src/render_prompts/arg.rs:520`-`src/render_prompts/arg.rs:524`).
2. Footer helper text styling is always accent-colored (`src/components/prompt_footer.rs:358`-`src/components/prompt_footer.rs:359`).

Impact:

1. No semantic distinction between info, warning, running, or paused states.
2. Visual hierarchy is weak: all helper text has same tone regardless of urgency.

Recommendation:

1. Add typed status block:
   - `FooterStatus { kind: Info|Running|Success|Warning|Error, text, icon? }`.
2. Map kind -> color/icon/style in one place.
3. Keep `helper_text` for backwards compatibility, but internally convert to `FooterStatus::Info`.

### P2: Header actions-mode implementation may be heavier than needed

Evidence:

1. Header keeps both button/search layers mounted and toggles visibility/opacity (`src/components/prompt_header.rs:579`-`src/components/prompt_header.rs:602`).
2. This helps avoid layout shift, but duplicates subtree render paths and keeps two states in sync.

Impact:

1. More complexity for maintenance and future feature additions.
2. Harder to evolve toward mixed mode (e.g., show active Actions button + small inline filter).

Recommendation:

1. Move to a single right-side layout with conditional children:
   - always keep primary action,
   - conditionally replace secondary label with search field when active,
   - preserve stable width via explicit slot sizing.

### P3: Footer tests are brittle and shallow

Evidence:

1. String-matching tests assert implementation text snippets (`tests/prompt_footer.rs:8`-`tests/prompt_footer.rs:44`).

Impact:

1. Refactors that preserve behavior can fail tests unnecessarily.
2. Regressions in behavior/layout semantics are not caught.

Recommendation:

1. Convert tests to behavioral unit tests where possible around config/builders and pure helper logic.
2. Add small pure helpers for render decisions to make them testable without GPUI macro-heavy integration.
3. Keep a minimal smoke string test only for critical invariant if needed.

## Proposed API Evolution

### Header

1. Add:
   - `breadcrumbs: Option<Vec<BreadcrumbSegment>>`
   - `status_chip: Option<HeaderStatusChip>`
   - `actions_density: HeaderActionDensity`
2. Keep existing fields (`path_prefix`, `show_ask_ai_hint`) as compatibility inputs.

### Footer

1. Add:
   - `status: Option<FooterStatus>`
   - `meta_items: Vec<FooterMetaItem>` (chips/pills instead of one `info_label` string)
   - `button_specs: Vec<ActionButtonSpec>` (replace fixed primary/secondary model over time)
2. Keep old `primary_/secondary_/helper_/info_` fields; internally map to new model.

## Layout Rules To Adopt

1. Header:
   - Input zone grows/shrinks first.
   - Actions zone uses density presets (`compact`, `default`) rather than hard `200px`.
   - Breadcrumbs collapse from middle when width constrained.
2. Footer:
   - Left zone (`logo + status`) max width at ~55%.
   - Right zone (`meta + actions`) max width at ~45%.
   - Helper/status and info/meta use truncation and optional tooltip/title pattern.

## Test Plan (TDD Targets)

1. `test_prompt_footer_collapses_meta_before_hiding_primary_action_when_narrow`
2. `test_prompt_footer_maps_helper_text_to_info_status_by_default`
3. `test_prompt_footer_status_kind_controls_color_token`
4. `test_footer_button_does_not_fire_callback_when_disabled`
5. `test_footer_button_uses_injected_color_tokens_not_global_theme`
6. `test_prompt_header_collapses_breadcrumb_segments_for_long_paths`
7. `test_prompt_header_actions_density_compact_reduces_reserved_width`
8. `test_prompt_header_keeps_primary_action_visible_in_actions_mode`
9. `test_prompt_footer_config_backwards_compatibility_maps_primary_secondary_fields`
10. `test_prompt_footer_contract_does_not_require_source_string_matching`

## Implementation Phasing

### Phase 1 (low-risk, high ROI)

1. Fix color ownership inconsistency (footer + footer button).
2. Add truncation/width caps for helper/info/path prefix.
3. Add disabled state support to `FooterButton`.
4. Replace brittle string tests with behavior-first tests for new pure helpers.

### Phase 2 (semantic model)

1. Add typed status and meta item models.
2. Add breadcrumb segment model and collapse strategy.
3. Add action density presets for header.

### Phase 3 (full action model)

1. Migrate fixed primary/secondary config to list-based action specs.
2. Align header and footer action primitives for shared behavior/state rendering.

## Risks / Known Gaps

1. GPUI macro recursion constraints limit direct render tree assertions; small pure helper extraction is needed for robust unit tests.
2. Prompt call sites are numerous (arg/editor/chat/script-list/path), so migration should preserve old config fields until all call sites are upgraded.
3. Status/meta chips can increase visual noise if defaults are not conservative; density policy must be explicit per prompt type.
