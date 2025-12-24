# macOS Panel Window Implementation - Complete Guide

## Overview
This implementation adds floating panel configuration to Script Kit GPUI, making the window appear above other applications and remain visible when switching between apps.

## Changes Required

### 1. File: src/lib.rs
**Action:** Already updated ✓
**What it does:** Exposes the panel module publicly

### 2. File: src/panel.rs  
**Action:** Already created ✓
**What it does:** Defines the panel module (currently a stub that delegates to main.rs)

### 3. File: src/main.rs - THREE EDITS REQUIRED

#### Edit 1: Add module declaration
**Location:** After line 22 (after `mod prompts;`)
**Find:**
```rust
mod protocol;
mod prompts;

use std::sync::{Arc, Mutex, mpsc};
```

**Replace with:**
```rust
mod protocol;
mod prompts;
mod panel;

use std::sync::{Arc, Mutex, mpsc};
```

#### Edit 2: Add configure function  
**Location:** Before main() function (around line 1090-1095)
**Find:**
```rust
fn start_hotkey_poller(cx: &mut App, window: WindowHandle<ScriptListApp>) {
```

**Replace with:**
```rust
/// Configure the current window as a floating macOS panel that appears above other apps
#[cfg(target_os = "macos")]
fn configure_as_floating_panel() {
    unsafe {
        let app: id = NSApp();

        // Get the key window (the most recently activated window)
        let window: id = msg_send![app, keyWindow];

        if window != nil {
            // NSFloatingWindowLevel = 3
            // This makes the window float above normal windows
            let floating_level: i32 = 3;
            let _: () = msg_send![window, setLevel:floating_level];

            // NSWindowCollectionBehaviorCanJoinAllSpaces = (1 << 0)
            // This makes the window appear on all spaces/desktops
            let collection_behavior: u64 = 1;
            let _: () = msg_send![window, setCollectionBehavior:collection_behavior];

            logging::log(
                "PANEL",
                "Configured window as floating panel (NSFloatingWindowLevel)",
            );
        } else {
            logging::log("PANEL", "Warning: No key window found to configure as panel");
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_as_floating_panel() {}

fn start_hotkey_poller(cx: &mut App, window: WindowHandle<ScriptListApp>) {
```

#### Edit 3: Call configuration function
**Location:** In main() function, after `cx.activate(true);` (around line 1130-1135)
**Find:**
```rust
        cx.activate(true);
        
        start_hotkey_poller(cx, window.clone());
```

**Replace with:**
```rust
        cx.activate(true);
        
        // Configure window as floating panel on macOS
        configure_as_floating_panel();
        
        start_hotkey_poller(cx, window.clone());
```

### 4. File: GPUI_RESEARCH.md
**Action:** Already updated ✓
**What it does:** Documents the panel implementation approach and window level constants

## Implementation Details

### Key Cocoa Concepts

**NSFloatingWindowLevel (3)**
- Makes window float above normal windows
- Standard for floating palettes, HUD windows, utility windows
- Window remains visible when app loses focus

**NSWindowCollectionBehaviorCanJoinAllSpaces**
- Allows window to appear on all Mission Control spaces
- Ensures window is accessible regardless of current space
- User can switch spaces and panel stays accessible

**NSApp::keyWindow**
- Gets the most recently activated/focused window
- Reliable access immediately after window creation
- Works correctly when called in main() after cx.activate()

### Why This Order Matters

1. **Window is created** by `cx.open_window()`
2. **Window is focused** via `window.focus()`
3. **App is activated** by `cx.activate(true)`
4. **NOW we configure as panel** - window is ready to receive Cocoa calls
5. **Hotkey listener starts** - window is fully configured

The timing is important because the NSWindow must exist and be visible before we can set its window level.

## Verification

### Compilation
```bash
cargo check
cargo build --release
```

### Runtime Testing
1. Run: `./target/release/script-kit-gpui`
2. Press Cmd+; to show window
3. Switch to another app (e.g., Chrome, VS Code)
4. Verify Script Kit window remains visible above the other app
5. Test on secondary display if available

### Expected Behavior
- Window appears above all other applications
- Window stays visible when switching apps with Cmd+Tab
- Window appears on all Mission Control spaces
- Hotkey (Cmd+;) still brings window to focus
- Multi-monitor positioning still works (positions on monitor with mouse)
- Eye-line height positioning still works

## Platform Notes

- **macOS:** Full implementation with NSPanel configuration
- **Linux/Windows:** No-op via `#[cfg(not(target_os = "macos"))]`
- **Graceful:** If NSWindow can't be found, logs warning and continues

## Log Output

When working correctly, you should see:
```
[PANEL] Configured window as floating panel (NSFloatingWindowLevel)
```

In the log panel (Cmd+L) or console output.

## Potential Issues & Solutions

### Issue: Window doesn't float
**Solution:** Verify execute order - configure_as_floating_panel() must be called AFTER cx.activate(true)

### Issue: Compilation error about msg_send macro
**Solution:** Ensure #[macro_use] extern crate objc; is imported (it is in main.rs already)

### Issue: Multiple agents modifying main.rs
**Solution:** Coordinate edits carefully:
1. Add module declaration first
2. Add function definition next  
3. Add function call last
4. Each edit is independent and non-overlapping

## Dependencies & Versions

- cocoa = "0.26" (already in Cargo.toml)
- objc = "0.2" (already in Cargo.toml)  
- gpui (from zed repo)

No new dependencies required.

## Performance Impact

- **Negligible:** Single function call at startup
- **Zero runtime overhead:** Configuration happens once when window is created
- **No allocation:** Uses stack-only variables
- **Safe:** Uses unsafe blocks only for Cocoa FFI (industry standard)

## References

- NSFloatingWindowLevel docs
- NSWindow collection behavior docs  
- GPUI window creation flow
- Cocoa/Objective-C messaging via msg_send! macro
