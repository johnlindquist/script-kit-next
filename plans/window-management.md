# Window Management Audit (codex-window-management)

## Scope
- `src/window_resize.rs`
- `src/panel.rs`
- `src/app_impl.rs`
- `src/app_actions.rs`
- Related context read for behavior parity: `src/main.rs`, `src/window_state.rs`

## Implemented Fixes

### 1) Main-window close path now persists per-display bounds (fixed)
- Problem: `close_and_reset_window` saved only legacy `WindowRole::Main` bounds, which does not update `main_per_display`. This could restore stale coordinates when reopening on multi-monitor setups.
- Before: `save_window_bounds(WindowRole::Main, ...)` in `src/app_impl.rs`.
- After: it now calls display-aware persistence via `save_main_position_with_display_detection(...)`.
- Reference: `src/app_impl.rs:6385`.

### 2) Action-triggered hide path now saves window bounds before hiding (fixed)
- Problem: `hide_main_and_reset` hid the main window without saving bounds at all, so action-heavy flows could lose latest position.
- After: it now persists bounds with display detection before toggling visibility.
- Reference: `src/app_actions.rs:304`.

### 3) Added centralized persistence helper + typed outcome (fixed)
- Added `MainPositionSaveOutcome` and `save_main_position_with_display_detection(...)`.
- Behavior:
  - Saves per-display when display is resolvable.
  - Falls back to legacy `main` bounds when display detection fails.
  - Honors save suppression and reports `Suppressed`.
- Reference: `src/window_state.rs:391`.

### 4) Added regression tests for persistence behavior (fixed)
- Added tests for:
  - per-display save path,
  - legacy fallback path,
  - suppression path.
- Reference: `src/window_state_persistence_tests.rs:286`.

## Remaining Findings / Improvements

### P1: Resize path does not clamp to visible display bounds after height changes
- `resize_first_window_to_height` keeps top edge fixed but does not clamp resulting frame to visible regions. After display topology changes or prompt height jumps, the window can end partially off-screen.
- Reference: `src/window_resize.rs:165`.
- Suggestion: clamp `new_frame` against `visible_bounds` of the display that currently contains the window center.

### P1: Resize logic uses raw NSWindow frame deltas without explicit DPI/display normalization
- Current resize uses frame-point math only (`height_delta`, `new_origin_y`) and does not reconcile scale-factor boundaries in mixed-DPI multi-monitor transitions.
- Reference: `src/window_resize.rs:203`.
- Suggestion: normalize target geometry against display metrics (or GPUI/display abstractions) before applying `setFrame`.

### P2: Window show/hide behavior still duplicated in stdin command branch
- `ExternalCommand::Show/Hide` in `main.rs` reimplements logic documented as centralized in helper functions.
- This increases drift risk for future positioning/vibrancy/persistence fixes.
- References: `src/main.rs:3078`, `src/main.rs:3149`, helper definitions at `src/main.rs:357` and `src/main.rs:508`.
- Suggestion: route stdin show/hide through `show_main_window_helper` / `hide_main_window_helper` (or a shared internal API).

### P2: Stdin hide path lacks fallback when display detection fails
- In stdin hide branch, save occurs only if `find_display_for_bounds(...)` resolves a display; no legacy fallback is written.
- Reference: `src/main.rs:3153`.
- Suggestion: reuse `save_main_position_with_display_detection(...)` for parity with fixed paths.

## Verification Performed
- `cargo check` ✅
- `cargo build` ✅
- Runtime stdin protocol smoke:
  - `{"type":"show"}` then `{"type":"hide"}` with `SCRIPT_KIT_AI_LOG=1`.
  - Confirmed logs include show/hide and per-display save events.
- `cargo clippy --all-targets -- -D warnings` ❌ (pre-existing branch failures outside this task)
- `cargo test` ❌ (pre-existing branch failure outside this task)

## External Blockers Observed (pre-existing)
- `src/ai/window.rs`: missing `AiApp::message_body_content` in tests.
- Multiple existing unused-import warnings/errors under strict clippy in `src/actions/dialog_builtin_action_validation_tests_*.rs`.
- Duplicate `tests` module error in current branch (`src/render_prompts/arg.rs` vs `src/main.rs`) under all-targets clippy/test compilation.
