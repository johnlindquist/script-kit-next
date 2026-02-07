# Accessibility Audit (Rust UI Surface)

Date: 2026-02-07
Agent: codex-accessibility
Scope: `src/**/*.rs`

## Summary

This codebase has a strong keyboard-event foundation (`track_focus` + `on_key_down` patterns are common), but it still has major accessibility gaps in reusable interactive components and visual affordances.

Highest-risk issues:

1. Keyboard activation is inconsistent for button-like controls.
2. Clickable `div`-based controls are used without keyboard semantics.
3. Default selected/hover contrast is too low for reliable state visibility.
4. There is no explicit screen-reader semantic layer for app-owned UI.

## Method

- Searched all Rust UI modules under `src/` for:
  - interactive handlers (`on_click`, `on_key_down`)
  - focus handling (`track_focus`, `focus_handle`, `Focusable`)
  - accessibility semantics (`aria`, `AX`, `accessibility`)
  - theme/opacity tokens affecting contrast
- Reviewed high-use components and prompts manually.
- Calculated representative contrast values from current default theme tokens and alpha overlays.

## Findings

### A11Y-001 (High): Keyboard activation in shared `Button` is opt-in and easy to miss

Evidence:

- `src/components/button.rs:162` stores `focus_handle` as `Option<FocusHandle>`.
- `src/components/button.rs:364` only attaches keyboard handler when a `focus_handle` is provided.
- `src/components/button.rs:370` maps `enter`/`space` only inside that optional path.
- Common call sites build click actions without `.focus_handle(...)`, e.g.:
  - `src/components/prompt_header.rs:382`
  - `src/components/prompt_header.rs:396`
  - `src/components/alias_input.rs:438`
  - `src/components/alias_input.rs:453`
  - `src/components/alias_input.rs:461`

Impact:

- Reusable buttons can become mouse-only unless every caller remembers extra focus wiring.
- This creates inconsistent keyboard behavior across prompts/dialogs.

Recommendation:

- Make keyboard activation a default behavior for `Button` (not opt-in).
- Add a safe default internal focus strategy for button instances.
- Add targeted tests for Enter/Space activation and disabled-state no-op.

---

### A11Y-002 (High): `FooterButton` is a clickable `div` with no keyboard path

Evidence:

- `src/components/footer_button.rs:71` builds a `div` with `.cursor_pointer()`.
- `src/components/footer_button.rs:99` only wires `.on_click(...)`.
- `src/components/prompt_footer.rs:243` uses `FooterButton` for primary/secondary footer actions.

Impact:

- Footer actions are not operable via keyboard when focus is in footer region.
- Keyboard-only users must rely on global shortcuts instead of interacting with visible controls.

Recommendation:

- Replace `FooterButton` with the shared `Button` component after fixing A11Y-001.
- Or add `track_focus` + `on_key_down` (`Enter`, `Space`) + visible focus styling to `FooterButton`.

---

### A11Y-003 (High): Several interactive-looking elements are not true keyboard controls

Evidence:

- `src/warning_banner.rs:170` banner root is clickable via `.on_click(...)` when callback exists.
- `src/warning_banner.rs:186` no focus tracking or key activation for that banner click area.
- `src/components/toast.rs:440` details toggle is styled as clickable (`cursor_pointer`, underline on hover) but has no keyboard interaction path.
- `src/render_script_list.rs:1091` “Ask AI [Tab]” is rendered as a pointer-styled visual hint, not an interactive control.

Impact:

- Pointer users see obvious click affordances; keyboard users cannot interact equivalently.
- Creates “discoverability mismatch” and inconsistent input parity.

Recommendation:

- Convert click surfaces to semantic button components (or equivalent keyboard-focusable controls).
- Ensure visual affordance matches actual interactivity.

---

### A11Y-004 (High): Focus ring rendering is decoupled from actual focus state

Evidence:

- `src/components/button.rs:160` stores `focused: bool` as manual state.
- `src/components/button.rs:337` focus ring uses that manual boolean.
- `src/components/button.rs:365` adds `track_focus`, but does not derive visual focus from `focus_handle.is_focused(...)`.

Impact:

- A control can be focusable/keyboard-active without showing a focus ring.
- Visual focus indicators become caller-dependent and easy to regress.

Recommendation:

- Derive focus visuals from actual focus state at render time (or enforce one centralized focus-state source).
- Add regression tests for visible focus transitions.

---

### A11Y-005 (Medium): Default selected/hover state contrast is below non-text contrast guidance

Evidence (tokens):

- Dark mode selected/hover opacity defaults:
  - `src/theme/types.rs:205` selected = `0.15`
  - `src/theme/types.rs:206` hover = `0.09`
- Light mode selected/hover opacity defaults:
  - `src/theme/types.rs:235` selected = `0.20`
  - `src/theme/types.rs:236` hover = `0.12`
- Selected subtle base colors:
  - `src/theme/types.rs:804` dark uses white subtle selection
  - `src/theme/types.rs:845` light uses black subtle selection
- Unfocused windows further reduce opacity by 10%:
  - `src/theme/types.rs:1142`

Representative contrast calculations from defaults:

- Dark main background `#1E1E1E`:
  - selected overlay result ≈ `#404040`, contrast vs main ≈ `1.61:1`
  - hover overlay result ≈ `#323232`, contrast vs main ≈ `1.30:1`
- Light main background `#FAFAFA`:
  - selected overlay result ≈ `#C8C8C8`, contrast vs main ≈ `1.60:1`
  - hover overlay result ≈ `#DCDCDC`, contrast vs main ≈ `1.31:1`

Impact:

- Active/hovered state can be hard to detect, especially on noisy vibrancy backgrounds.
- Weak state differentiation increases navigation and targeting errors.

Recommendation:

- Raise selected/hover contrast for interactive states to approach/exceed 3:1 against adjacent surfaces.
- Consider combined cues: stronger fill + border/outline.
- Validate on both focused and unfocused window modes.

---

### A11Y-006 (Medium): Actions popup focus model is intentionally indirect and fragile

Evidence:

- `src/actions/window.rs:117` explicitly avoids focusing popup focus handle.
- `src/actions/window.rs:123` says key handling is fallback-only in popup.
- `src/actions/window.rs:375` reiterates the non-focus behavior at open.

Impact:

- Keyboard behavior depends on parent-window event routing and timing.
- Focus predictability can degrade in edge cases (window activation changes, click-to-focus transitions).

Recommendation:

- Prefer direct focus ownership when popup is active.
- Keep explicit return-focus behavior on close/escape.
- Add deterministic tests for open → type/filter → navigate → close focus restoration.

---

### A11Y-007 (Medium): Screen-reader semantics are largely absent for app-owned UI

Evidence:

- No app UI modules in `src/` define ARIA-like roles/names/state mapping.
- Accessibility API usage is concentrated in external window control paths:
  - `src/window_control.rs:10`
  - `src/window_control.rs:493`

Impact:

- VoiceOver-style navigation/announcements for this app’s own controls are limited.
- Dynamic status changes (toasts, HUD, selection updates) are not exposed as announcements.

Recommendation:

- Define an accessibility abstraction layer for GPUI views (role, name, state).
- Start with high-value surfaces: primary list, prompt header/footer actions, dialog actions, toast/HUD announcements.

---

### A11Y-008 (Low/Medium): Icon-only/symbol-heavy labels reduce semantic clarity

Evidence:

- Dismiss controls use symbol-only labels:
  - `src/warning_banner.rs:159` label `×`
  - `src/components/toast.rs:466` label `×`
- Several toolbar controls are terse/symbolic in notes UI:
  - `src/notes/window.rs:2913`
  - `src/notes/window.rs:2940`
  - `src/notes/window.rs:2958`

Impact:

- If/when assistive semantics are added, symbol-only names will be low quality without explicit accessible labels.

Recommendation:

- Add explicit semantic labels separate from visual glyphs (e.g., “Dismiss notification”, “Toggle bullet list”).

---

### A11Y-009 (Low): Accessibility regression coverage is thin in key shared components

Evidence:

- Tests omitted for shared components due GPUI macro recursion notes:
  - `src/components/button.rs:390`
  - `src/warning_banner.rs:212`
  - `src/components/prompt_header.rs:636`
- No direct contrast regression tests for main theme tokens.

Impact:

- Accessibility regressions are likely to reappear during UI refactors.

Recommendation:

- Add isolated behavior tests (keyboard activation, focus-visible behavior, disabled behavior) and contrast-token guard tests in non-render-dependent modules.

## Prioritized Remediation Plan

### Phase 1 (Immediate, High impact)

1. Fix shared control semantics:
   - Make `Button` keyboard activation default.
   - Add clear focus-visible behavior bound to real focus state.
2. Replace/upgrade `FooterButton` to keyboard-operable semantics.
3. Remove clickable-but-non-semantic `div` patterns in warning/toast/header hint surfaces.

### Phase 2 (Short term)

1. Tune selected/hover contrast tokens and unfocused opacity policy.
2. Add focus reliability tests for actions popup lifecycle.
3. Add explicit labels for icon-only controls.

### Phase 3 (Medium term)

1. Introduce an app-level accessibility semantics layer for GPUI views.
2. Add announcement hooks for transient status (toast/HUD/errors).
3. Build an accessibility smoke-test checklist into CI.

## Suggested Tests To Add

- `test_button_activates_with_enter_when_focused`
- `test_button_activates_with_space_when_focused`
- `test_button_does_not_activate_when_disabled`
- `test_footer_button_supports_keyboard_activation`
- `test_actions_window_restores_focus_after_escape`
- `test_theme_selected_hover_contrast_meets_thresholds`

## Known Constraints

- This is a native GPUI desktop app, so ARIA attributes do not apply directly as in web DOM.
- Equivalent accessibility outcomes still require a platform semantics layer (role/name/state/announcement), which is currently not visible in app-owned UI rendering paths.
