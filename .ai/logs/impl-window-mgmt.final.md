Implemented the window-management P1 fixes in scoped files for resize lifecycle/positioning.

**Changed Files**
- `src/window_resize.rs`
- `src/panel.rs`

**What Changed**
- `src/window_resize.rs`
  - Added a new resize-frame computation path that:
    - preserves top-edge intent,
    - normalizes geometry to backing scale (mixed-DPI safe),
    - clamps window frame to `visible_bounds`.
  - Added helpers for scale sanitization, frame normalization, axis/dimension clamping, and final frame calculation.
  - Updated macOS screen selection to prefer the screen containing window center, then window’s screen, then main screen.
  - Wired `resize_first_window_to_height` to use the new screen geometry + clamped frame pipeline.
  - Added tests for top-edge behavior, bottom clamping, max-height capping, and backing-scale rounding.
- `src/panel.rs`
  - Added `WINDOW_VISIBLE_EDGE_MARGIN: f64 = 4.0`.
  - Added test `window_visible_edge_margin_is_positive`.

**Verification Run**
- `cargo fmt` passed.
- `cargo check` passed.
- Stdin JSON protocol smoke test passed (editor-height flow), with resize logs confirming expected transitions:
  - `Resize: 500 -> 700`
  - `Resize: 700 -> 500`

**How To Test**
1. Build/check:
   - `cargo check`
2. Run resize smoke test via stdin protocol:
   - `echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-editor-height.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1`
3. Verify logs:
   - `grep -iE 'Resize:|height_for_view' ~/.scriptkit/logs/latest-session.jsonl`

**Risks / Known Gaps**
- Full `cargo test` is currently blocked by unrelated pre-existing compile issues outside this scope (`src/ai/window.rs`, duplicate `tests` module elsewhere).  
- No changes were made outside assigned scope.

**Commits**
- No commits were created in this pass.