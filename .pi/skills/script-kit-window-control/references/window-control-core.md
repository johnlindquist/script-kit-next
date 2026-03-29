# Window Control Core (window_control.rs)

Core window management using macOS Accessibility APIs (AXUIElement).

## Requirements

**Accessibility Permission Required**: System Preferences > Privacy & Security > Accessibility

## Types

### Bounds
```rust
pub struct Bounds {
    pub x: i32,      // Screen pixels from left
    pub y: i32,      // Screen pixels from top (AX coords)
    pub width: u32,
    pub height: u32,
}

impl Bounds {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self;
}
```

### WindowInfo
```rust
pub struct WindowInfo {
    pub id: u32,        // Unique: (pid << 16) | window_index
    pub app: String,    // Application name
    pub title: String,  // Window title
    pub bounds: Bounds,
    pub pid: i32,       // Process ID
}
```

### TilePosition
```rust
pub enum TilePosition {
    LeftHalf,     // Left 50%
    RightHalf,    // Right 50%
    TopHalf,      // Top 50%
    BottomHalf,   // Bottom 50%
    TopLeft,      // Top-left quadrant
    TopRight,     // Top-right quadrant
    BottomLeft,   // Bottom-left quadrant
    BottomRight,  // Bottom-right quadrant
    Fullscreen,   // Fill entire visible display
}
```

## Public Functions

### Permission Checking
```rust
/// Check if accessibility permission is granted
pub fn has_accessibility_permission() -> bool;

/// Request permission (opens System Preferences)
pub fn request_accessibility_permission() -> bool;
```

### Window Discovery
```rust
/// List all visible windows across applications
/// Filters out small/utility windows (< 50x50)
pub fn list_windows() -> Result<Vec<WindowInfo>>;

/// Get PID of the menu bar owning application
/// Key for Script Kit: returns the app focused BEFORE Script Kit
pub fn get_menu_bar_owner_pid() -> Result<i32>;

/// Get the frontmost window of the previous app
/// Selection strategy:
/// 1. AXFocusedWindow (most accurate)
/// 2. AXMainWindow (fallback)
/// 3. First window in AXWindows array
pub fn get_frontmost_window_of_previous_app() -> Result<Option<WindowInfo>>;
```

### Window Operations
```rust
/// Move window to new position
pub fn move_window(window_id: u32, x: i32, y: i32) -> Result<()>;

/// Resize window to new dimensions
pub fn resize_window(window_id: u32, width: u32, height: u32) -> Result<()>;

/// Set complete bounds (position + size)
pub fn set_window_bounds(window_id: u32, bounds: Bounds) -> Result<()>;

/// Tile window to predefined position
pub fn tile_window(window_id: u32, position: TilePosition) -> Result<()>;

/// Minimize window
pub fn minimize_window(window_id: u32) -> Result<()>;

/// Maximize window (fills display without fullscreen mode)
pub fn maximize_window(window_id: u32) -> Result<()>;

/// Close window (may prompt for unsaved changes)
pub fn close_window(window_id: u32) -> Result<()>;

/// Focus and bring window to front
pub fn focus_window(window_id: u32) -> Result<()>;
```

## Window Caching

Windows are cached on `list_windows()` call. Cache is used for subsequent operations.
Call `list_windows()` to refresh the cache if windows have changed.

## AX Attributes Used

| Attribute | Purpose |
|-----------|---------|
| AXPosition | Window origin (CGPoint) |
| AXSize | Window dimensions (CGSize) |
| AXTitle | Window title string |
| AXWindows | App's window array |
| AXFocusedWindow | App's focused window |
| AXMainWindow | App's main window |
| AXMinimized | Minimize state (CFBoolean) |
| AXCloseButton | Close button element |

## Error Codes

| Code | Meaning |
|------|---------|
| kAXErrorSuccess (0) | Operation succeeded |
| kAXErrorAPIDisabled (-25211) | Accessibility disabled |
| kAXErrorNoValue (-25212) | Attribute has no value |

## Display Bounds

The module accounts for menu bar and dock:
- Uses `NSScreen.visibleFrame` for accurate bounds
- Falls back to estimates (25px menu bar, 70px dock) if NSScreen unavailable
- Handles coordinate conversion between Cocoa (bottom-left) and CG (top-left)
