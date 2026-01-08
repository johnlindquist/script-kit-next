# Window Control Enhanced (window_control_enhanced/)

Enhanced window control types and utilities. Sub-modules: bounds, capabilities, coords, display, spaces.

## WindowBounds (bounds.rs)

Canonical bounds type in AX coordinates (top-left origin, Y grows downward).

```rust
pub struct WindowBounds {
    pub x: f64,      // Pixels from left of primary display
    pub y: f64,      // Pixels from TOP of primary display
    pub width: f64,
    pub height: f64,
}

impl WindowBounds {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self;
    pub fn from_ints(x: i32, y: i32, width: u32, height: u32) -> Self;
    
    pub fn center(&self) -> (f64, f64);
    pub fn right(&self) -> f64;   // x + width
    pub fn bottom(&self) -> f64;  // y + height
    
    pub fn contains_point(&self, x: f64, y: f64) -> bool;
    pub fn intersection_area(&self, other: &WindowBounds) -> f64;
    pub fn area(&self) -> f64;
    pub fn clamp_to(&self, container: &WindowBounds) -> WindowBounds;
}
```

### SizeConstraints
```rust
pub struct SizeConstraints {
    pub min_width: Option<f64>,
    pub min_height: Option<f64>,
    pub max_width: Option<f64>,
    pub max_height: Option<f64>,
}

impl SizeConstraints {
    pub fn clamp_size(&self, width: f64, height: f64) -> (f64, f64);
}
```

## Capability Detection (capabilities.rs)

Detect what operations are permitted on a window via `AXUIElementIsAttributeSettable`.

### WindowCapabilities
```rust
pub struct WindowCapabilities {
    pub can_move: bool,           // AXPosition is settable
    pub can_resize: bool,         // AXSize is settable
    pub can_minimize: bool,       // Has AXMinimizeButton
    pub can_close: bool,          // Has AXCloseButton
    pub can_fullscreen: bool,     // Has AXFullScreenButton
    pub supports_space_move: bool, // Always false by default
}
```

### Detection Functions
```rust
pub fn can_move_window(ax_element: *const c_void) -> bool;
pub fn can_resize_window(ax_element: *const c_void) -> bool;
pub fn can_minimize_window(ax_element: *const c_void) -> bool;
pub fn can_close_window(ax_element: *const c_void) -> bool;
pub fn can_fullscreen_window(ax_element: *const c_void) -> bool;
pub fn detect_window_capabilities(ax_element: *const c_void) -> WindowCapabilities;
```

### EnhancedWindowInfo
```rust
pub struct EnhancedWindowInfo {
    pub id: u32,
    pub app: String,
    pub bundle_id: Option<String>,
    pub title: String,
    pub pid: i32,
    pub bounds: WindowBounds,
    pub capabilities: WindowCapabilities,
    pub size_constraints: SizeConstraints,
}

impl EnhancedWindowInfo {
    pub fn can_resize_to(&self, width: f64, height: f64) -> bool;
}
```

## Coordinate Conversion (coords.rs)

Convert between AX (top-left origin) and AppKit (bottom-left origin) coordinates.

```rust
/// AppKit -> AX: y = main_height - (appkit_y + rect_height)
pub fn appkit_to_ax(x: f64, y: f64, height: f64, main_screen_height: f64) -> (f64, f64);

/// AX -> AppKit: y = main_height - (ax_y + rect_height)
pub fn ax_to_appkit(x: f64, y: f64, height: f64, main_screen_height: f64) -> (f64, f64);

/// NSRect (AppKit) -> WindowBounds (AX)
pub fn nsrect_to_bounds(origin_x: f64, origin_y: f64, width: f64, height: f64, main_screen_height: f64) -> WindowBounds;

/// WindowBounds (AX) -> NSRect components (AppKit)
pub fn bounds_to_nsrect(bounds: &WindowBounds, main_screen_height: f64) -> (f64, f64, f64, f64);
```

## Display Info (display.rs)

Multi-monitor detection and display-relative positioning.

### DisplayInfo
```rust
pub struct DisplayInfo {
    pub id: u32,
    pub name: Option<String>,
    pub frame: WindowBounds,           // Full display bounds
    pub visible_bounds: WindowBounds,  // Excludes dock/menu bar
    pub is_main: bool,
}

impl DisplayInfo {
    /// Get relative position [0.0, 1.0] within visible bounds
    pub fn relative_position(&self, bounds: &WindowBounds) -> (f64, f64);
    
    /// Apply relative position to get absolute bounds
    pub fn apply_relative_position(&self, rel_x: f64, rel_y: f64, width: f64, height: f64) -> WindowBounds;
}
```

### Display Functions
```rust
/// Get all connected displays
pub fn get_all_displays() -> Result<Vec<DisplayInfo>>;

/// Find display with largest overlap for a window
pub fn get_display_for_window<'a>(window_bounds: &WindowBounds, displays: &'a [DisplayInfo]) -> Option<&'a DisplayInfo>;
```

## Space Management (spaces.rs)

Virtual desktop (Spaces) backend. Default is unsupported (requires private APIs or yabai).

### Types
```rust
pub struct SpaceInfo {
    pub id: u64,
    pub index: u32,       // 1-based Mission Control index
    pub is_active: bool,
    pub space_type: SpaceType,
}

pub enum SpaceType {
    Desktop,
    Fullscreen,
    Unknown,
}

pub enum SpaceError {
    NotSupported(String),
    SpaceNotFound(u64),
    WindowNotMovable(u32),
    ExternalToolNotAvailable(String),
    Other(String),
}
```

### SpaceManager Trait
```rust
pub trait SpaceManager: Send + Sync {
    fn get_all_spaces(&self) -> Result<Vec<SpaceInfo>, SpaceError>;
    fn get_active_space(&self) -> Result<SpaceInfo, SpaceError>;
    fn move_window_to_space(&self, window_id: u32, space_id: u64) -> Result<(), SpaceError>;
    fn is_supported(&self) -> bool;
    fn unsupported_reason(&self) -> Option<String>;
}
```

### Global Backend
```rust
/// Get current space manager (default: UnsupportedSpaceBackend)
pub fn get_space_manager() -> Arc<dyn SpaceManager>;

/// Set custom backend (e.g., yabai integration)
pub fn set_space_manager(manager: Arc<dyn SpaceManager>);
```

**Note**: Space operations require either:
- Private WindowServer/Dock APIs (fragile, requires SIP disable)
- External tools like [yabai](https://github.com/koekeishiya/yabai)
