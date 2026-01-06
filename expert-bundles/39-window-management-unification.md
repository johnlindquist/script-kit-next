# Expert Bundle 39: Window Management Unification

## Goal
Consolidate duplicated window lifecycle management code across Notes, AI, and Actions windows into a unified registry and builder pattern.

## Current State

The codebase has **3 secondary windows** plus the main window, each with independent management:
- `src/notes/window.rs` (~2,100 lines) - `NOTES_WINDOW` OnceLock
- `src/ai/window.rs` (~2,200 lines) - `AI_WINDOW` OnceLock
- `src/actions/window.rs` (~250 lines) - `ACTIONS_WINDOW` OnceLock
- `src/windows/registry.rs` (~200 lines) - Exists but underutilized

Each window module independently implements nearly identical open/close/toggle logic.

## Specific Concerns

1. **Window Handle Singleton (3 copies)**: Each module has `static *_WINDOW: OnceLock<Mutex<Option<WindowHandle<Root>>>>` with identical lock patterns.

2. **`open_*_window()` Duplication (~120 lines x 2)**: Notes and AI window have nearly identical open functions with: theme init, existing handle check, vibrancy setup, bounds calculation, window creation, handle storage, theme watcher spawning.

3. **Vibrancy Helpers (5 copies!)**: `hex_to_rgba_with_opacity()` is duplicated in `ui_foundation.rs`, `app_impl.rs`, `notes/window.rs`, `ai/window.rs`, and `notes/actions_panel.rs`.

4. **`compute_box_shadows()` (2 copies)**: ~45-line identical function in both Notes and AI window modules.

5. **Theme Watcher Spawning (3 copies)**: Same async spawn + poll + theme sync pattern in Notes, AI, and main.

6. **Drop Implementation (2 copies)**: Identical `impl Drop` clearing global handle in Notes and AI.

## Key Questions

1. Should `WindowRegistry` be a global singleton managing all handles, or should each window type still maintain its own handle?

2. Is a `SecondaryWindowBuilder` pattern appropriate, or should we use a simpler factory function?

3. Should vibrancy helpers live in `ui_foundation.rs` (already has canonical versions) and all others just import?

4. Can `compute_box_shadows()` become a method on `Theme` struct since it only uses theme data?

5. Should theme watching be centralized in a `ThemeService` that broadcasts to all registered windows?

## Implementation Checklist

- [ ] Expand `src/windows/registry.rs` with full `WindowRegistry` API
- [ ] Move `hex_to_rgba_with_opacity()` to `ui_foundation.rs`, delete other copies
- [ ] Move `compute_box_shadows()` to `Theme::box_shadows()` method
- [ ] Create `spawn_window_theme_watcher(window_id)` helper in `theme/service.rs`
- [ ] Add `impl_window_drop!` macro for Drop implementation
- [ ] Create `SecondaryWindowBuilder` or `open_secondary_window()` helper
- [ ] Migrate Notes window to use unified patterns
- [ ] Migrate AI window to use unified patterns
- [ ] Update Actions window to use registry
