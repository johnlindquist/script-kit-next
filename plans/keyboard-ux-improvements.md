# Keyboard UX Improvements Audit

Date: 2026-02-07
Agent: `codex-ux-keyboard`
Scope: `src/**/*.rs`

## Executive Summary

Keyboard handling is functional but fragmented. The main risks are:

1. Alias mismatches (`return` vs `enter`, `esc` vs `escape`) across handlers that should be equivalent.
2. Focus restoration paths that bypass overlay-stack cleanup.
3. Divergence between runtime key paths and stdin simulated key paths used for automated UI testing.
4. Multiple global interceptors with overlapping concerns, increasing double-handle regressions.

Quick inventory from scan:

- `app_impl.rs` has 4 global keystroke interceptors (`tab`, `arrow`, `home/end/page`, `actions`) at `src/app_impl.rs:515`, `src/app_impl.rs:712`, `src/app_impl.rs:1022`, `src/app_impl.rs:1079`.
- `capture_key_down` is used in 3 root windows (`main`, `notes`, `ai`) at `src/main.rs:1896`, `src/notes/window.rs:4068`, `src/ai/window.rs:6496`.
- 25 `event.keystroke.key.to_lowercase()` hot-path conversions remain.
- 14 direct `show_actions_popup = false` writes exist in `src/app_impl.rs`, not all routed through shared close logic.

## Keyboard/Focus Architecture Map

### Main window pipeline

- Global pre-bubble interception is centralized in `ScriptListApp` via `cx.intercept_keystrokes(...)` for tab/arrows/home-end/actions.
- Prompt-level renders still attach local `on_key_down` listeners (for prompt-specific behavior and fallbacks).
- Shared actions-dialog routing is centralized in `route_key_to_actions_dialog(...)` with `ui_foundation` helpers at `src/app_impl.rs:4507`.

### Secondary windows

- Notes and AI each use root-level `capture_key_down` to preempt focused text input behavior (`src/notes/window.rs:4068`, `src/ai/window.rs:6496`).
- Actions popup window intentionally does not own focus in the primary path; parent windows route keys, while popup keeps a fallback `on_key_down` (`src/actions/window.rs:117`, `src/actions/window.rs:123`).

### Existing normalization utilities

- `ui_foundation` provides allocation-free key checks (`is_key_up/down/left/right/enter/escape/backspace`) at `src/ui_foundation.rs:553`.
- Shortcut system has broader canonicalization (`arrow*`, `return`, `esc`, `pgup/pgdn`, etc.) at `src/shortcuts/types.rs:281` and `src/shortcuts/hotkey_compat.rs:96`.
- These utilities are not consistently used by runtime handlers.

## Findings (Ranked)

### P1: Confirm dialog routing mismatch (`return` and `esc` not uniformly handled)

Evidence:

- Confirm window local handler accepts `"enter" | "return"` and `"escape" | "esc"` at `src/confirm/window.rs:78` and `src/confirm/window.rs:88`.
- Global confirm dispatcher accepts only `"enter" | "Enter"` and `"escape" | "Escape"` at `src/confirm/window.rs:377` and `src/confirm/window.rs:389`.
- Interceptors call `dispatch_confirm_key(...)` in multiple paths (`src/app_impl.rs:534`, `src/app_impl.rs:726`, `src/app_impl.rs:1101`).

Impact:

- Depending on route path, `Return`/`Esc` variants can behave differently for the same confirm UI.
- High risk for regression where simulated/tested behavior differs from live behavior.

Recommendation:

- Replace dispatcher string matches with `ui_foundation::is_key_enter` / `is_key_escape`.
- Add dedicated unit tests for `dispatch_confirm_key` alias parity.

### P1: Simulated-key paths diverge from runtime keyboard paths

Evidence:

- `main.rs` `SimulateKey` logic often handles only `"enter"`/`"escape"` without aliases (`src/main.rs:3344`, `src/main.rs:3345`, `src/main.rs:3375`, `src/main.rs:3390`, `src/main.rs:3536`, `src/main.rs:3548`).
- AI simulated path supports `"escape" | "esc"` (`src/ai/window.rs:1931`, `src/ai/window.rs:2015`), while live capture path frequently checks only `"escape"` (`src/ai/window.rs:6569`, `src/ai/window.rs:6619`, `src/ai/window.rs:6646`, `src/ai/window.rs:6764`, `src/ai/window.rs:6814`).

Impact:

- Stdin-driven UI tests can pass while real keyboard interactions fail (or vice versa).
- This directly weakens the project’s required autonomous stdin test workflow.

Recommendation:

- Introduce one shared key canonicalization helper for both runtime and simulation paths.
- Add parity tests that replay the same key set (`enter/return`, `escape/esc`, `up/arrowup`) through both handlers.

### P1: Focus overlay stack can be bypassed by direct popup state resets

Evidence:

- Shared close path restores overlay focus (`close_actions_popup`) via `pop_focus_overlay` + `apply_pending_focus` at `src/app_impl.rs:4675`.
- Some close paths directly clear popup state without calling shared close logic, e.g. main-input focus handler: `src/app_impl.rs:199`.
- Similar direct clears exist in lifecycle paths (`src/app_impl.rs:4912`, `src/app_impl.rs:5188`, `src/app_impl.rs:6370`, `src/app_impl.rs:6580`, `src/app_impl.rs:7016`).

Impact:

- Overlay stack state and legacy focus fields can desync after non-standard close paths.
- Risk: focus returns to wrong target, stale overlay assumptions, intermittent keyboard capture bugs.

Recommendation:

- Enforce a single close API for actions popup state transitions.
- If a hard reset is required, call explicit `clear_focus_overlays(...)` or a dedicated `force_close_actions_popup_and_reset_focus(...)` to keep coordinator state coherent.

### P2: Alias coverage is inconsistent across prompt handlers

Evidence:

- Fully aliased handlers exist (good pattern): `SelectPrompt` and `PathPrompt` (`src/prompts/select.rs:629`, `src/prompts/path.rs:514`).
- Partial handlers remain:
  - `DivPrompt`: only `"enter" | "escape"` (`src/prompts/div.rs:880`).
  - `EnvPrompt`: only `"enter"` / `"escape"` (`src/prompts/env.rs:325`, `src/prompts/env.rs:329`).
  - `Actions` command bar: only `"escape"` (`src/actions/command_bar.rs:509`).
  - `ActionsWindow` fallback: only `"escape"` (`src/actions/window.rs:173`).
  - Global dismiss shortcut: only `"escape"` (`src/app_impl.rs:6672`).

Impact:

- Cross-view keyboard muscle memory is inconsistent.
- Platform-variant key naming issues can reappear in new features.

Recommendation:

- Use `ui_foundation` helpers or a shared canonical key enum in all hot handlers.
- Prohibit ad-hoc string matches for core navigation keys in new code.

### P2: Interceptor overlap increases double-processing risk

Evidence:

- Four separate global interceptors in one file with partially overlapping domains.
- Existing comment acknowledges prior double-processing issue and explicit guard for arrows at `src/app_impl.rs:1261`.

Impact:

- High change-surface: small feature edits can reintroduce duplicate handling or propagation mistakes.
- Hard to reason about precedence order across views.

Recommendation:

- Consolidate to a single primary router with ordered phases:
  1. global modal windows,
  2. actions dialog routing,
  3. view-specific navigation,
  4. fallthrough.
- Keep per-feature handlers pure functions returning a typed `Handled/NotHandled/Execute` result.

### P3: Actions dialog navigation is limited compared to main list UX

Evidence:

- Actions dialog supports up/down only (`src/actions/dialog.rs:1082`, `src/actions/dialog.rs:1104`).
- Main list already supports jump navigation (`Home/End/PageUp/PageDown`) via interceptor (`src/app_impl.rs:1020`).

Impact:

- Keyboard power-users cannot quickly traverse long action lists.

Recommendation:

- Add `home/end/pageup/pagedown` support to actions dialog (and command bars) for parity with script list.

### P3: Test coverage does not protect the highest-risk alias/focus cases

Evidence:

- `keyboard_routing_tests` are mostly static source-pattern assertions, not behavior-level alias/focus tests (`src/keyboard_routing_tests.rs:1`).
- No direct tests found for `dispatch_confirm_key` alias behavior.

Impact:

- Regressions in live key behavior may not be detected by existing tests.

Recommendation:

- Add behavior tests with explicit key variants and expected routing effects.

## Accessibility and UX Gaps

1. Inconsistent `Esc`/`Escape` handling creates non-deterministic keyboard exits.
2. Modal key swallowing is not uniformly paired with clear/consistent close affordances across windows.
3. Actions/search overlays lack fast jump navigation shortcuts already present in primary list views.

## Recommended Implementation Plan

### Phase 1 (stabilize correctness)

1. Fix confirm dispatcher alias parity (`return`, `esc`).
2. Normalize escape/enter alias handling in `DivPrompt`, `EnvPrompt`, `ActionsWindow`, `actions::command_bar`, and global dismiss shortcut path.
3. Route all actions-popup close events through shared close/focus API.

### Phase 2 (unify architecture)

1. Introduce `KeyIntent` canonicalizer (or extend `ui_foundation`) and migrate hot handlers from manual `to_lowercase()+match`.
2. Consolidate global interceptor responsibilities into one ordered router.

### Phase 3 (keyboard UX upgrades)

1. Add Home/End/PageUp/PageDown to actions dialog and command bars.
2. Keep runtime and simulate-key routing backed by shared helper functions.

## Proposed Tests (TDD)

1. `test_dispatch_confirm_key_handles_return_and_esc_when_confirm_open`
2. `test_runtime_and_simulated_ai_escape_aliases_produce_same_state_changes`
3. `test_main_simulate_key_return_matches_enter_for_prompt_submission`
4. `test_actions_popup_close_path_pops_focus_overlay_when_closed_via_input_focus`
5. `test_actions_dialog_handles_home_end_page_keys_for_long_lists`

## Risks and Migration Notes

1. Interceptor consolidation can cause behavioral churn; gate with focused keyboard regression tests before refactor.
2. Focus restoration changes can regress non-main windows (Notes/AI); validate with stdin scenario scripts per AGENTS workflow.
3. Canonicalization should avoid allocations in hot paths; preserve `eq_ignore_ascii_case` or pre-normalized static matches.
