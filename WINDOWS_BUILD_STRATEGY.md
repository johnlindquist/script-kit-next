# Windows Build Strategy

## Current Status
- Platform-specific Cargo dependencies: ✅ Done
- Module guards in src/lib.rs: ✅ Partially done (menu_bar, keyboard_monitor, keyword_manager, ocr already guarded)
- Missing guard added: login_item ✅

## Remaining Issues (32 errors)

### Files Needing macOS-Only Guards

Based on grep results, these files import macOS-only crates and need to be either:
1. Made macOS-only (whole file wrapped in `#[cfg(target_os = "macos")]`)
2. Have conditional imports and platform-specific implementations

#### Option 1: macOS-Only Files (wrap entire file)
These are pure macOS modules with no Windows equivalent yet:
- `src/window_resize.rs` - Uses Cocoa/objc for window management
- `src/window_manager.rs` - Uses Cocoa/objc for NSApp management
- `src/permissions_wizard.rs` - Uses macos_accessibility_client
- `src/ocr.rs` - Already has `#[cfg(feature = "ocr")]`, add macOS guard
- `src/clipboard_history/change_detection.rs` - Uses objc
- `src/app_launcher.rs` - Uses Cocoa for macOS app launching

#### Option 2: Cross-Platform Files Needing Platform-Specific Impl
These need both macOS and Windows implementations:
- `src/window_control.rs` - Uses core_graphics + macos_accessibility_client
- `src/selected_text.rs` - Uses macos_accessibility_client
- `src/platform.rs` - Already has platform guards
- `src/main.rs` - Main entry, needs conditional compilation
- `src/notes/window.rs` - Uses Cocoa for window management
- `src/ai/window.rs` - Uses Cocoa for window management

### Approach

**Phase 1: Make macOS-only modules conditional**
Add `#[cfg(target_os = "macos")]` to files that are purely macOS-specific.

**Phase 2: Add Windows stubs**
For cross-platform modules, create stub Windows implementations.

**Phase 3: Fix hotkey manager thread safety**
The `GlobalHotKeyManager` contains `*mut c_void` which is not `Send` on Windows.
This may be a bug in the `global-hotkey` crate or we need to use a different pattern.

## Implementation Plan

1. Guard macOS-only imports with `#[cfg(target_os = "macos")]`
2. Create Windows stub modules where needed
3. Test incremental build
4. Address hotkey manager thread safety (may need upstream fix or workaround)
