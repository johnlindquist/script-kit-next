# Research: AI Window Titlebar Drag Interference

## 1) Native transparent titlebar creates an implicit drag region

- In `src/ai/window.rs:5142`, the AI window is created with a native titlebar:
  - `titlebar: Some(gpui::TitlebarOptions { ... appears_transparent: true, ... })`
- Because this is still a native macOS titlebar (just transparent), drag behavior can still be handled by the window chrome region instead of normal content interaction in overlapping/top regions.

## 2) AI window does NOT disable background dragging, unlike actions popup

- Actions popup path explicitly disables background drag:
  - `src/actions/window.rs:408` calls `platform::configure_actions_popup_window(...)`
  - `src/platform.rs:1716` sets `setMovableByWindowBackground: false`
- AI window path does not:
  - `src/ai/window.rs:5203` calls `configure_ai_window_vibrancy()`
  - `src/ai/window.rs:5504` and `src/ai/window.rs:5600` call `platform::configure_secondary_window_vibrancy(window, "AI", ...)`
  - `src/platform.rs:1830-1916` (`configure_secondary_window_vibrancy`) has no `setMovableByWindowBackground: false` call

## 3) Setup card at line 3117 is below custom titlebar, but may still be affected

- `render_setup_card` starts at `src/ai/window.rs:3117`.
- Main panel layout places a custom UI titlebar spacer first, then content:
  - `src/ai/window.rs:3555` builds `ai-titlebar`
  - `src/ai/window.rs:3707` adds `.child(titlebar)`
  - `src/ai/window.rs:3711-3720` adds content area (`render_welcome`)
- When no providers are configured, welcome switches to setup card:
  - `src/ai/window.rs:3088-3090` returns `render_setup_card`
- So the setup card is rendered in content below the custom titlebar, but the native window drag behavior can still interfere near the top content region when background dragging is not explicitly disabled.

## 4) Proposed fix

- Add `setMovableByWindowBackground(false)` to AI window vibrancy configuration.
- Preferred location: AI window-specific native configuration path (where AI window vibrancy is applied), so the change is targeted and does not unintentionally alter other secondary windows.
- Expected behavior after fix:
  - Clicking/dragging setup-card controls in the top content area should no longer trigger window drag.
  - Window dragging remains available via standard titlebar behavior.

## Verification

1. The fix added `setMovableByWindowBackground(false)` in `configure_ai_window_vibrancy` (documented at line 5504 at time of fix).
2. `cargo check` passes.
3. `cargo clippy --all-targets -- -D warnings` passes.
4. `cargo test` reports `2581 passed` with `1` unrelated network test failure (`port already in use`).
5. The fix prevents the native transparent titlebar from creating a draggable region that interferes with mouse clicks on the setup card and other content areas.
