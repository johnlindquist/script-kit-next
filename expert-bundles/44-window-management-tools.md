# Script Kit GPUI - Expert Bundle 44: Window Management Tools

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner. Think: Raycast/Alfred but scriptable with TypeScript.

**macOS Window Features:**
- Floating panel window behavior (appears above all apps)
- Multiple spaces (virtual desktops) support
- Mission Control integration
- Window snapping/tiling
- Stage Manager compatibility

---

## Goal

Provide comprehensive **window management tools** for:
1. Moving windows between spaces (virtual desktops)
2. Window tiling and positioning (left half, right half, maximize, etc.)
3. Window focus switching and cycling
4. Window state queries (bounds, space, screen)
5. Multi-monitor support

---

## Current State

### Existing Capabilities

| Feature | Location | Status |
|---------|----------|--------|
| Window switcher | `builtins.rs` | Basic list of windows |
| Focus window | `window_control.rs` | AppleScript-based |
| Get window list | `window_control.rs` | Accessibility API |
| Move to mouse display | `platform.rs` | Works for main window |

### Current Window Control API

```rust
// src/window_control.rs
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub app: String,
    pub bounds: WindowBounds,
    pub is_minimized: bool,
    pub is_focused: bool,
}

pub fn get_all_windows() -> Vec<WindowInfo> { ... }
pub fn focus_window(window_id: u64) -> Result<()> { ... }
pub fn get_focused_window() -> Option<WindowInfo> { ... }
```

### Missing Capabilities

1. **No space management** - Can't move windows between spaces
2. **No tiling** - Can't snap windows to screen regions
3. **No window state modification** - Can't minimize, maximize, close
4. **No multi-monitor tiling** - Can't move between displays
5. **Limited window queries** - Missing space ID, screen ID

---

## Proposed Architecture

### 1. Enhanced Window Info

```rust
pub struct EnhancedWindowInfo {
    // Existing fields
    pub id: u64,
    pub title: String,
    pub app: String,
    pub bundle_id: Option<String>,
    pub bounds: WindowBounds,
    
    // New fields
    pub space_id: Option<u64>,      // Virtual desktop ID
    pub display_id: Option<u32>,    // Monitor/display ID
    pub is_minimized: bool,
    pub is_maximized: bool,
    pub is_fullscreen: bool,
    pub is_focused: bool,
    pub can_resize: bool,
    pub can_move: bool,
}
```

### 2. Window Positioning API

```rust
/// Predefined window positions for tiling
#[derive(Debug, Clone, Copy)]
pub enum WindowPosition {
    // Halves
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
    
    // Quarters
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    
    // Thirds
    LeftThird,
    CenterThird,
    RightThird,
    LeftTwoThirds,
    RightTwoThirds,
    
    // Special
    Maximize,
    Center,
    AlmostMaximize,  // 90% of screen, centered
    
    // Custom
    Custom(WindowBounds),
}

impl WindowPosition {
    /// Calculate bounds for this position on a given display
    pub fn bounds_on_display(&self, display: &DisplayInfo) -> WindowBounds {
        let screen = display.visible_bounds; // Excludes menu bar, dock
        match self {
            Self::LeftHalf => WindowBounds {
                x: screen.x,
                y: screen.y,
                width: screen.width / 2,
                height: screen.height,
            },
            Self::RightHalf => WindowBounds {
                x: screen.x + screen.width / 2,
                y: screen.y,
                width: screen.width / 2,
                height: screen.height,
            },
            // ... etc
        }
    }
}
```

### 3. Space Management API

```rust
/// Information about a macOS space (virtual desktop)
pub struct SpaceInfo {
    pub id: u64,
    pub index: usize,          // 1-based index shown in Mission Control
    pub display_id: u32,
    pub is_fullscreen: bool,   // Contains a fullscreen app
    pub windows: Vec<u64>,     // Window IDs in this space
}

pub trait SpaceManager {
    /// Get all spaces across all displays
    fn get_all_spaces() -> Vec<SpaceInfo>;
    
    /// Get the currently active space
    fn get_current_space() -> Option<SpaceInfo>;
    
    /// Move a window to a specific space
    fn move_window_to_space(window_id: u64, space_id: u64) -> Result<()>;
    
    /// Move a window to the next/previous space
    fn move_window_to_adjacent_space(window_id: u64, direction: Direction) -> Result<()>;
    
    /// Create a new space
    fn create_space() -> Result<SpaceInfo>;
    
    /// Switch to a specific space
    fn switch_to_space(space_id: u64) -> Result<()>;
}

pub enum Direction {
    Next,
    Previous,
}
```

### 4. Window Manipulation API

```rust
pub trait WindowManager {
    // Focus
    fn focus_window(window_id: u64) -> Result<()>;
    fn focus_app(bundle_id: &str) -> Result<()>;
    fn cycle_windows(app_bundle_id: Option<&str>) -> Result<()>;
    
    // Position
    fn move_window(window_id: u64, position: WindowPosition) -> Result<()>;
    fn resize_window(window_id: u64, bounds: WindowBounds) -> Result<()>;
    fn move_window_to_display(window_id: u64, display_id: u32) -> Result<()>;
    
    // State
    fn minimize_window(window_id: u64) -> Result<()>;
    fn maximize_window(window_id: u64) -> Result<()>;
    fn toggle_fullscreen(window_id: u64) -> Result<()>;
    fn close_window(window_id: u64) -> Result<()>;
    
    // Query
    fn get_window_info(window_id: u64) -> Option<EnhancedWindowInfo>;
    fn get_windows_for_app(bundle_id: &str) -> Vec<EnhancedWindowInfo>;
    fn get_focused_window() -> Option<EnhancedWindowInfo>;
}
```

### 5. SDK Integration

```typescript
// TypeScript SDK for scripts
interface WindowManager {
  // Query
  getAllWindows(): Promise<WindowInfo[]>;
  getFocusedWindow(): Promise<WindowInfo | null>;
  getWindowsByApp(bundleId: string): Promise<WindowInfo[]>;
  
  // Focus
  focusWindow(windowId: number): Promise<void>;
  focusApp(bundleId: string): Promise<void>;
  cycleWindows(bundleId?: string): Promise<void>;
  
  // Position
  moveWindow(windowId: number, position: WindowPosition): Promise<void>;
  moveWindowToBounds(windowId: number, bounds: Bounds): Promise<void>;
  moveWindowToDisplay(windowId: number, displayId: number): Promise<void>;
  
  // Space
  moveWindowToSpace(windowId: number, spaceIndex: number): Promise<void>;
  moveWindowToNextSpace(windowId: number): Promise<void>;
  moveWindowToPreviousSpace(windowId: number): Promise<void>;
  
  // State
  minimizeWindow(windowId: number): Promise<void>;
  maximizeWindow(windowId: number): Promise<void>;
  toggleFullscreen(windowId: number): Promise<void>;
  closeWindow(windowId: number): Promise<void>;
}

// Convenience shortcuts
type WindowPosition = 
  | "left-half" | "right-half" | "top-half" | "bottom-half"
  | "top-left" | "top-right" | "bottom-left" | "bottom-right"
  | "left-third" | "center-third" | "right-third"
  | "maximize" | "center" | "almost-maximize";
```

---

## Built-in Commands

### Proposed Built-in Entries

```rust
// In builtins.rs
pub fn get_window_management_builtins() -> Vec<BuiltInEntry> {
    vec![
        BuiltInEntry {
            id: "window-left-half",
            name: "Tile Window Left",
            description: "Move focused window to left half of screen",
            icon: "rectangle.lefthalf.filled",
            feature: BuiltInFeature::WindowAction(WindowAction::Position(WindowPosition::LeftHalf)),
            keywords: vec!["tile", "snap", "left", "half"],
        },
        BuiltInEntry {
            id: "window-right-half",
            name: "Tile Window Right",
            description: "Move focused window to right half of screen",
            icon: "rectangle.righthalf.filled",
            feature: BuiltInFeature::WindowAction(WindowAction::Position(WindowPosition::RightHalf)),
            keywords: vec!["tile", "snap", "right", "half"],
        },
        BuiltInEntry {
            id: "window-maximize",
            name: "Maximize Window",
            description: "Maximize the focused window",
            icon: "arrow.up.left.and.arrow.down.right",
            feature: BuiltInFeature::WindowAction(WindowAction::Position(WindowPosition::Maximize)),
            keywords: vec!["maximize", "full", "expand"],
        },
        BuiltInEntry {
            id: "window-center",
            name: "Center Window",
            description: "Center the focused window on screen",
            icon: "rectangle.center.inset.filled",
            feature: BuiltInFeature::WindowAction(WindowAction::Position(WindowPosition::Center)),
            keywords: vec!["center", "middle"],
        },
        BuiltInEntry {
            id: "window-next-space",
            name: "Move to Next Space",
            description: "Move focused window to the next space",
            icon: "arrow.right.to.line",
            feature: BuiltInFeature::WindowAction(WindowAction::MoveToSpace(Direction::Next)),
            keywords: vec!["space", "desktop", "next"],
        },
        BuiltInEntry {
            id: "window-prev-space",
            name: "Move to Previous Space",
            description: "Move focused window to the previous space",
            icon: "arrow.left.to.line",
            feature: BuiltInFeature::WindowAction(WindowAction::MoveToSpace(Direction::Previous)),
            keywords: vec!["space", "desktop", "previous"],
        },
        BuiltInEntry {
            id: "window-minimize",
            name: "Minimize Window",
            description: "Minimize the focused window",
            icon: "minus",
            feature: BuiltInFeature::WindowAction(WindowAction::Minimize),
            keywords: vec!["minimize", "hide", "dock"],
        },
        BuiltInEntry {
            id: "window-close",
            name: "Close Window",
            description: "Close the focused window",
            icon: "xmark",
            feature: BuiltInFeature::WindowAction(WindowAction::Close),
            keywords: vec!["close", "quit", "window"],
        },
    ]
}
```

---

## macOS Implementation Details

### Space Management via CGS Private APIs

```rust
// Note: These are private APIs, use with caution
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGSGetActiveSpace(connection: u32) -> u64;
    fn CGSCopySpaces(connection: u32, mask: u32) -> CFArrayRef;
    fn CGSMoveWindowToSpace(connection: u32, window_id: u32, space_id: u64);
    fn CGSGetWindowOwner(connection: u32, window_id: u32) -> u32;
}

// Safer alternative: Accessibility API + AppleScript hybrid
pub fn move_window_to_space_safe(window_id: u64, space_index: usize) -> Result<()> {
    // 1. Use Accessibility API to get window reference
    // 2. Use AppleScript to invoke Mission Control
    // 3. Programmatically drag window to target space
    // This is slower but more reliable
}
```

### Window Positioning via Accessibility API

```rust
use accessibility::{AXUIElement, AXValue};

pub fn move_and_resize_window(window_id: u64, bounds: WindowBounds) -> Result<()> {
    let ax_window = get_ax_window(window_id)?;
    
    // Set position
    let position = AXValue::new_point(bounds.x as f64, bounds.y as f64);
    ax_window.set_attribute(kAXPositionAttribute, position)?;
    
    // Set size
    let size = AXValue::new_size(bounds.width as f64, bounds.height as f64);
    ax_window.set_attribute(kAXSizeAttribute, size)?;
    
    Ok(())
}
```

---

## Implementation Checklist

### Phase 1: Core APIs
- [ ] Enhance `WindowInfo` with space/display IDs
- [ ] Implement `WindowPosition` enum with bounds calculation
- [ ] Create `WindowManager` trait with position/resize methods
- [ ] Add multi-monitor support to positioning

### Phase 2: Space Management
- [ ] Research CGS private APIs for space management
- [ ] Implement fallback using AppleScript
- [ ] Create `SpaceManager` trait
- [ ] Add space info to window queries

### Phase 3: Built-in Commands
- [ ] Add tiling commands to built-ins
- [ ] Add space movement commands
- [ ] Add minimize/maximize/close commands
- [ ] Add keyboard shortcuts for tiling

### Phase 4: SDK Integration
- [ ] Add `windowManager` to SDK protocol
- [ ] Implement TypeScript types
- [ ] Create convenience functions
- [ ] Document SDK usage

### Phase 5: UI Integration
- [ ] Add tiling options to window switcher actions
- [ ] Add position presets to actions dialog
- [ ] Show space indicator in window list
- [ ] Add drag-to-tile support (future)

---

## Key Questions

1. Should we use private CGS APIs for space management, or stick to AppleScript?
2. How to handle windows that can't be resized (e.g., some dialogs)?
3. Should tiling respect user's dock position (left, bottom, right)?
4. How to handle Stage Manager when it's enabled?
5. Should we persist window positions/layouts?

---

## Related Bundles

- Bundle 39: Window Management Unification - registry for Script Kit windows
- Bundle 12: macOS Platform - platform integration layer
- Bundle 32: System Events - system action patterns
