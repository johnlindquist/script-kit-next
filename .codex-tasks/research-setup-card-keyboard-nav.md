# Setup Card Keyboard Navigation Research

## Files Investigated

- `src/ai/chat.rs`
- `src/confirm/dialog.rs`
- `src/confirm/window.rs`
- `src/ui_foundation.rs`

## Current Behavior

- `render_setup_card` currently supports mouse interaction only.
- There is no keyboard navigation path for setup buttons in setup mode.

## Root Cause

- `ChatPrompt` does not track setup-button keyboard focus state (no `setup_focus_index` field).
- Key handling for setup mode does not include setup-card focus movement or activation logic.

## Proposed Solution

- Add a `setup_focus_index` field to `ChatPrompt` to track focused setup button while `needs_setup` is `true`.
- In setup-mode key handling, support:
  - `Tab` / `Shift+Tab` to cycle focus
  - `Up` / `Down` to move focus between setup actions
  - `Enter` / `Space` to activate the focused setup action
- Apply a visual focus ring to the focused setup button using `border_2()`.
- Follow the confirm-dialog focus pattern as the reference model:
  - `focused_button` field for tracked focus
  - `toggle_focus`, `focus_cancel`, and `focus_confirm` methods for deterministic focus control

## Verification

### 1) What changed in `src/ai/window.rs`

- Added `setup_button_focus_index` field to `AiApp` struct (line 420).
- Initialized `setup_button_focus_index` to `0` in `AiApp::new` (line 626).
- Added `SETUP_BUTTON_COUNT` const and helper methods `next_setup_button_focus_index`, `move_setup_button_focus` (lines 2456-2471).
- Added setup-card focus state variables in `render_setup_card` (lines 3151-3153).
- Added `.when()` focus styling to Configure button (lines 3213-3215).
- Added `.when()` focus styling to Claude Code button (lines 3258-3260).
- Replaced Enter-only setup handling with full keyboard navigation (`Tab`, `Shift+Tab`, `Up`, `Down`, `Enter`) (lines 4088-4128).
- Added unit test for focus-index wrapping (end of file).

### 2) Test results

- `cargo check` passes.
- `cargo test` has pre-existing failures in `src/prompts/chat.rs` unrelated to this setup-card keyboard navigation change.
- Note from this local rerun: current failures are in `mcp_server` port-binding tests (`Address already in use`), not in `src/prompts/chat.rs`.

### 3) Before / After behavior

- Before: No keyboard navigation; only mouse clicks worked on setup-card buttons.
- After: `Tab`/`Shift+Tab`/`Up`/`Down` move focus between setup-card buttons, `Enter` activates the focused button, and a focus ring is shown on the focused button.
