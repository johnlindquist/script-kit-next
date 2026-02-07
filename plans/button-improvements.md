# Button Component Improvements Audit

Date: 2026-02-07  
Agent: `codex-button-components`  
Scope: `src/components/button.rs`, `src/components/footer_button.rs`

## Executive Summary

The current button layer works for basic click/hover behavior, but it is missing several states and contracts needed for consistent UI across prompts:

1. Variant model is too narrow (`Primary`, `Ghost`, `Icon`) and split across two separate components.
2. Hover exists, but active/pressed/loading states are missing.
3. Icon support is label-text-based rather than typed icon slots.
4. Accessibility support is optional and inconsistently used (especially in footer buttons).
5. Styling is not fully token-driven; several fields are unused or duplicated across components.

## Current State

### `Button` (`src/components/button.rs`)

1. Supports variants via `ButtonVariant` (`Primary`, `Ghost`, `Icon`) (`src/components/button.rs:20`).
2. Supports optional shortcut, disabled flag, optional focused flag, optional click handler, optional focus handle (`src/components/button.rs:154`).
3. Has hover and focus visual states (`src/components/button.rs:337`, `src/components/button.rs:349`).
4. Keyboard activation only happens when `focus_handle` is set (`src/components/button.rs:364`).
5. Has no built-in loading, pressed, or selected/toggled states.

### `FooterButton` (`src/components/footer_button.rs`)

1. Supports label + optional shortcut + optional id + optional click handler (`src/components/footer_button.rs:18`).
2. Uses theme-derived hover background and text colors (`src/components/footer_button.rs:62`).
3. Has no variant model, no disabled/loading state, and no keyboard/focus behavior.

## Findings (Ranked)

### P1: Variant model is underpowered and split across two button primitives

Evidence:

1. `ButtonVariant` has only three variants (`src/components/button.rs:20`), with no semantic intent/tone (danger/success/subtle) and no size model.
2. `FooterButton` is a separate styling path with no variant API (`src/components/footer_button.rs:18`).

Impact:

1. Variant additions require one-off style logic in multiple places.
2. Footer action buttons and standard action buttons drift visually and behaviorally.

Recommendation:

1. Introduce shared typed model:
   - `ButtonIntent` (`Primary`, `Secondary`, `Danger`, `Subtle`)
   - `ButtonSize` (`Sm`, `Md`, `Lg`, `Icon`)
   - `ButtonKind` (`Text`, `IconOnly`, `TextWithIcon`)
2. Re-implement `FooterButton` as a thin wrapper over `Button` (or remove it after migration).

### P1: Hover is implemented, but active/pressed behavior is missing

Evidence:

1. `Button` defines hover styling (`src/components/button.rs:349`) but no pressed-state styling.
2. `FooterButton` defines hover styling (`src/components/footer_button.rs:81`) but no pressed-state styling.
3. Both components set pointer cursor even with no click handler (`src/components/button.rs:333`, `src/components/footer_button.rs:80`).

Impact:

1. No tactile feedback during click/hold.
2. Non-clickable buttons can still look interactive.

Recommendation:

1. Add explicit interaction states (`idle`, `hovered`, `pressed`, `focused`, `disabled`).
2. Only apply `cursor_pointer` when handler exists and button is enabled.
3. Add pressed visual token (darken/lift offset or overlay alpha) shared by both components.

### P1: Loading state is missing in both components

Evidence:

1. `Button` struct has no loading flag/slot (`src/components/button.rs:154`).
2. `FooterButton` struct has no loading flag/slot (`src/components/footer_button.rs:18`).

Impact:

1. Save/run/submit actions cannot communicate in-progress work or temporarily disable re-triggering.
2. Call sites must invent bespoke loading UI instead of using a consistent component contract.

Recommendation:

1. Add `loading(bool)` and `loading_label(Option<SharedString>)`.
2. While loading:
   - block click/keyboard activation,
   - show spinner/icon,
   - preserve width to avoid layout shift.

### P1: Accessibility support is incomplete and inconsistent

Evidence:

1. `Button` keyboard activation only works if `focus_handle` is provided (`src/components/button.rs:364`), and current call sites do not appear to use `.focus_handle(...)`.
2. `FooterButton` has no keyboard activation path or focus tracking (`src/components/footer_button.rs:60`).
3. `ElementId` defaults to label text in `Button` (`src/components/button.rs:318`), which can collide for repeated labels.

Impact:

1. Keyboard accessibility can silently regress by omission at call sites.
2. Footer actions rely on external key handling instead of button-local semantics.

Recommendation:

1. Add `id(...)` and `accessible_label(...)` to `Button` and require explicit id for repeated actions.
2. Add optional keyboard activation/focus support to `FooterButton` or migrate footer usage to `Button`.
3. Expose a single `activates_on` policy (`EnterOnly`, `EnterAndSpace`) for consistent behavior.

### P2: Icon support is text-based and limited

Evidence:

1. Icon variant still uses `label` text rendering (`src/components/button.rs:334`).
2. Stories show icon usage as `"▶"` text (`src/stories/button_stories.rs:44`).
3. No leading/trailing icon slots for text buttons.

Impact:

1. Visual consistency depends on ad-hoc glyph strings.
2. Icon-only buttons lack a clear accessible-name contract.

Recommendation:

1. Add typed icon slots:
   - `leading_icon(IconName)`
   - `trailing_icon(IconName)`
   - `icon_only(IconName).accessible_label("...")`
2. Normalize icon spacing, icon size, and icon opacity through shared tokens.

### P2: Styling contracts are inconsistent and partially unused

Evidence:

1. `ButtonColors.text_color`/`text_hover` are defined but render path uses `colors.accent` for text color in all variants (`src/components/button.rs:255`-`src/components/button.rs:290`).
2. `Button` and `FooterButton` use different corner radius, padding, border, and hover color logic (`src/components/button.rs:327`-`src/components/button.rs:344`, `src/components/footer_button.rs:76`-`src/components/footer_button.rs:81`).
3. `Button` font is coupled to `crate::list_item::FONT_SYSTEM_UI` (`src/components/button.rs:332`), while `FooterButton` uses only dynamic text size.

Impact:

1. Token surfaces do not match rendered output.
2. Header/footer buttons can look inconsistent across prompt variants.

Recommendation:

1. Make render logic consume all relevant token fields (`text_color`, `text_hover`) or remove dead fields.
2. Create shared `ButtonStyleTokens` used by both `Button` and `FooterButton`.
3. Decouple button font tokens from list-item internals.

## Proposed API Direction

1. `Button::intent(ButtonIntent)`
2. `Button::size(ButtonSize)`
3. `Button::state(ButtonState)` (or builder booleans mapped internally)
4. `Button::leading_icon(...)` / `Button::trailing_icon(...)`
5. `Button::loading(bool)`
6. `Button::id(...)`
7. `Button::accessible_label(...)`
8. `FooterButton` wraps `Button` with footer defaults (or is removed after migration)

## Suggested Test Plan (TDD Targets)

1. `test_button_does_not_show_pointer_when_no_handler`
2. `test_button_blocks_activation_when_loading`
3. `test_button_blocks_activation_when_disabled`
4. `test_button_icon_only_requires_accessible_label`
5. `test_button_uses_text_color_token_for_non_accent_variants`
6. `test_footer_button_loading_disables_clicks`
7. `test_footer_button_focus_activation_triggers_on_enter`
8. `test_button_element_id_is_stable_when_label_changes`

## Quick Wins

1. Add `has_handler` gating for pointer cursor in both components.
2. Add `id(...)` to `Button` instead of forcing label-based ids.
3. Implement a minimal `loading(bool)` state with spinner placeholder and disabled activation.
4. Refactor `FooterButton` to consume shared button style tokens to reduce drift.
