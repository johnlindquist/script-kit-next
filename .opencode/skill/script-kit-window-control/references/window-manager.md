# Window Manager (window_manager.rs)

Thread-safe registry to track windows by role. Solves the problem of finding "our" window
among NSApp's windows array (which includes tray icons, menu bar items, system overlays).

## Problem

When GPUI creates windows, `NSApp.windows` contains multiple windows in unpredictable order:
```
Window[0]: 34x24   - Tray icon popup
Window[1]: 0x37   - Menu bar
Window[2]: 0x24   - System window
Window[3]: 750x501 - Our main window (the one we want!)
```

Using `objectAtIndex:0` fails because tray icons appear first.

## Solution

Register windows by role after creation, then retrieve reliably.

## Types

### WindowRole
```rust
pub enum WindowRole {
    Main,   // Primary launcher window
    Notes,  // Notes window
    Ai,     // AI window
}
```

Re-exported from `window_state.rs` for single source of truth.

## Public API (macOS)

```rust
/// Register a window with a role (overwrites previous)
pub fn register_window(role: WindowRole, window_id: id);

/// Get window by role
pub fn get_window(role: WindowRole) -> Option<id>;

/// Convenience: get main window
pub fn get_main_window() -> Option<id>;

/// Find and register main window by size (~750x400-600px)
pub fn find_and_register_main_window() -> bool;

/// Unregister a window
pub fn unregister_window(role: WindowRole) -> Option<id>;

/// Check if role is registered
pub fn is_window_registered(role: WindowRole) -> bool;
```

## Thread Safety

Uses `OnceLock<Mutex<WindowManager>>`:
- `OnceLock` - One-time initialization (std alternative to lazy_static)
- `Mutex` - Protects HashMap access
- All functions handle locking internally

### WindowId Wrapper
```rust
struct WindowId(usize);

// Safety: Window ID is numeric identifier. Reading window properties
// is safe from any thread. Mutations done on main thread by caller.
unsafe impl Send for WindowId {}
unsafe impl Sync for WindowId {}
```

## Usage Pattern

```rust
// After GPUI creates window:
cx.open_window(options, |_window, _cx| {...});

// Register with WindowManager:
register_window(WindowRole::Main, ns_window_id);

// Later, in window_resize.rs:
let window = window_manager::get_main_window()?;
```

## Size-Based Discovery

For scenarios where you don't have the NSWindow ID after creation:

```rust
// Searches NSApp.windows for window matching ~750x400-600px
let found = find_and_register_main_window();
```

Expected main window dimensions:
- Width: 750 +/- 50px
- Height: 100-800px

## Non-macOS Stubs

All functions return `None` or `false` on non-macOS platforms.
