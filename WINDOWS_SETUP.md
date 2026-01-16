# Windows Support Progress

## Build Attempt Log

### Iteration 1: Initial Build
**Date**: 2026-01-15
**Status**: ❌ Failed - macOS-only dependencies

**Error**:
```
error: `objc2` only works on Apple platforms.
```

**Analysis**:
- The codebase has macOS-specific code that unconditionally compiles on all platforms
- Key culprits:
  - `objc2` - Objective-C runtime bindings (macOS only)
  - `cocoa` - Cocoa framework bindings (macOS only)
  - `core-graphics` - Core Graphics bindings (macOS only)
  - Platform-specific code in `src/platform.rs`, `src/panel.rs`, etc.

**Solution Strategy**:
Need to add `#[cfg(target_os = "macos")]` guards to macOS-specific dependencies in Cargo.toml and conditional compilation in source files.

## Changes Made

### 1. Cargo.toml - Platform-Specific Dependencies
- [ ] TODO: Guard macOS-only crates with `[target.'cfg(target_os = "macos")'.dependencies]`
- [ ] TODO: Identify Windows equivalents for platform features

### 2. Source Code - Conditional Compilation
- [ ] TODO: Add `#[cfg(target_os = "macos")]` to platform-specific modules
- [ ] TODO: Create Windows stub implementations

## Known macOS-Only Dependencies
From Cargo.toml analysis:
1. `core-graphics` - Window/display management
2. `cocoa` - macOS UI frameworks
3. `objc` - Objective-C runtime
4. `macos-accessibility-client` - Accessibility APIs
5. `smappservice-rs` - Launch at login

## Platform Feature Matrix

| Feature | macOS | Windows | Status |
|---------|-------|---------|--------|
| Window blur/vibrancy | NSVisualEffectView | Acrylic/Mica | ❌ Not implemented |
| Floating window | Panel levels | HWND_TOPMOST | ❌ Not implemented |
| Global hotkeys | Carbon/Cocoa | RegisterHotKey | ⚠️ Needs testing |
| System tray | NSStatusBar | Shell_NotifyIcon | ⚠️ Should work (tray-icon crate) |
| Clipboard | NSPasteboard | Win32 Clipboard | ✅ Works (arboard crate) |
| Terminal | PTY | ConPTY | ✅ Should work (portable-pty) |

## Next Steps
1. Split Cargo.toml dependencies into platform-specific sections
2. Add conditional compilation to source files
3. Create Windows platform module stubs
4. Test incremental build progress
