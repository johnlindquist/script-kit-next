# Windows Support - Ralph Loop Session Summary

**Date**: 2026-01-15
**Iterations Completed**: 3/10
**Status**: 🎯 Significant Progress - Identified Windows Build Path

---

## ✅ Achievements

### 1. Fixed: macOS-Only Dependencies (Iteration 1-2)
**Problem**: `objc2` crate causing compilation failure on Windows

**Solution**: Moved macOS-specific dependencies to platform-conditional section in Cargo.toml:
```toml
[target.'cfg(target_os = "macos")'.dependencies]
core-graphics = "0.24"
cocoa = "0.26"
objc = "0.2"
macos-accessibility-client = "0.0.1"
core-foundation = "0.10"
foreign-types = "0.5"
smappservice-rs = "0.1"
```

**Result**: ✅ objc2 error resolved!

### 2. Fixed: Module Declaration Guards
- Added `#[cfg(target_os = "macos")]` to `login_item` module in src/lib.rs
- Added `#[cfg(target_os = "macos")]` to Cocoa/objc imports in src/main.rs
- Existing guards already present for: menu_bar, menu_executor, menu_cache, keyboard_monitor, keyword_manager, ocr

---

## 🔄 Remaining Issues (32 compilation errors)

### Error Breakdown by Category:

| Category | Count | Severity |
|----------|-------|----------|
| `core_graphics` unresolved | 11 | High |
| `objc` unresolved/use errors | 12 | High |
| `macos_accessibility_client` | 3 | Medium |
| `menu_bar` module import | 2 | Low |
| Thread safety (*mut c_void not Send) | 2 | Medium |
| Type inference | 1 | Low |
| `login_item` import | 1 | Fixed ✅ |

### Files Requiring Platform Abstraction:

#### Tier 1: Core Window Management (High Priority)
These files are central to the app's window management and need Windows implementations:
- `src/window_resize.rs` - Window resizing logic (uses Cocoa/objc)
- `src/window_manager.rs` - Window state management (uses Cocoa NSApp)
- `src/window_control.rs` - Window positioning/control (uses core_graphics + AX)

#### Tier 2: Platform Integration (Medium Priority)
- `src/selected_text.rs` - Selected text detection (uses macos_accessibility_client)
- `src/permissions_wizard.rs` - Permission checks (uses macos_accessibility_client)
- `src/app_launcher.rs` - App launching (uses Cocoa)
- `src/clipboard_history/change_detection.rs` - Clipboard monitoring (uses objc)

#### Tier 3: Secondary Windows (Lower Priority)
- `src/notes/window.rs` - Notes window (uses Cocoa)
- `src/ai/window.rs` - AI chat window (uses Cocoa)

### Special Case: Hotkey Manager Thread Safety
**Files**: `src/hotkeys.rs:155`, `src/hotkeys.rs:548`

**Issue**: `GlobalHotKeyManager` on Windows contains `*mut c_void` which is not `Send`, preventing use in `OnceLock<Mutex<T>>`.

**Potential Solutions**:
1. Wrap in `parking_lot::Mutex` (may still fail)
2. Use `thread_local!` instead of static
3. Check if `global-hotkey` crate has a Windows-specific API
4. Report upstream bug to global-hotkey crate

---

## 📋 Recommended Next Steps

### Phase 1: Make It Compile (Minimal Viable Windows Build)
Goal: Get `cargo build` to succeed, even with limited functionality

1. **Add module-level platform guards**:
   ```rust
   // In src/lib.rs, wrap macOS-only modules:
   #[cfg(target_os = "macos")]
   pub mod window_resize;
   #[cfg(target_os = "macos")]
   pub mod window_manager;
   // etc.
   ```

2. **Create Windows stub modules** in `src/`:
   ```rust
   // src/window_resize_windows.rs
   #[cfg(target_os = "windows")]
   pub mod window_resize {
       pub fn initial_window_height() -> f32 { 500.0 }
       // Minimal stubs for all public functions
   }
   ```

3. **Fix hotkey manager** - Try `parking_lot::Mutex` or platform-specific initialization

4. **Guard platform-specific code in main.rs**:
   - Wrap Cocoa/objc usage in `#[cfg(target_os = "macos")]`
   - Provide Windows alternatives or no-ops

### Phase 2: Implement Windows Platform Layer
Goal: Feature parity with macOS where possible

1. **Window Management**:
   - Use Win32 APIs for window positioning/sizing
   - Implement Windows equivalent of panel behavior (always-on-top)
   - Research Windows 11 Acrylic/Mica for blur effects

2. **Accessibility**:
   - Implement selected text using Win32 UI Automation
   - Permission checks via Windows APIs

3. **System Integration**:
   - Clipboard monitoring via Win32
   - App launching via Shell APIs
   - Launch-at-login via Windows Task Scheduler or registry

### Phase 3: Testing & Refinement
1. Test each SDK function on Windows
2. Document platform differences
3. Update README with Windows build instructions
4. CI/CD for Windows builds

---

## 🎯 Immediate Next Action

**Recommended**: Continue with Phase 1 (minimal viable build) by:
1. Running a script to add `#[cfg(target_os = "macos")]` to all modules using macOS crates
2. Creating stub Windows modules
3. Testing incremental compilation

**Alternative**: Create detailed issue/task breakdown for community contribution

---

## 📁 Files Modified This Session

- `Cargo.toml` - Moved macOS dependencies to platform-specific section
- `src/lib.rs` - Added `#[cfg(target_os = "macos")]` to `login_item`
- `src/main.rs` - Added guards to Cocoa/objc imports
- `WINDOWS_SETUP.md` - Created (this file)
- `WINDOWS_BUILD_STRATEGY.md` - Created

---

## 🧠 Key Learnings

1. **Cross-platform foundations exist**: Many dependencies (`arboard`, `portable-pty`, `notify`, `global-hotkey`, `tray-icon`) are already cross-platform
2. **Code is well-structured**: Existing `#[cfg]` guards show John anticipated multi-platform support
3. **Main challenges are**:
   - Window management (heavily macOS-specific)
   - Accessibility APIs (completely different on Windows)
   - System integration (launch-at-login, permissions)
4. **Hotkey manager issue**: May be a bug in `global-hotkey` crate on Windows

---

## 💡 Fundamental Blocker Assessment

**Verdict**: ❌ No fundamental blockers found

- GPUI itself is cross-platform
- Most dependencies support Windows
- Issues are implementation-level, not architectural
- Windows support is definitely achievable with proper platform abstraction

**Estimated Effort**: 
- Phase 1 (compile): 4-8 hours
- Phase 2 (feature parity): 20-40 hours
- Phase 3 (polish): 10-20 hours

**Total**: ~35-70 hours for full Windows support
