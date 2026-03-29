# Window State (window_state.rs)

Window position persistence to `~/.sk/kit/window-state.json`.

## Architecture Principles

1. **Canonical coordinate space**: Global top-left origin (CoreGraphics-style), Y increases downward
2. **Persistence via WindowBounds**: Aligns with GPUI's `WindowBounds` (Windowed/Maximized/Fullscreen)
3. **Restore via WindowOptions.window_bounds**: No "jump after open"
4. **Validation via geometry intersection**: Not display IDs (which can change)
5. **Save on close/hide**: Main window saves on hide (often hidden not closed)

## Types

### WindowRole
```rust
pub enum WindowRole {
    Main,
    Notes,
    Ai,
}

impl WindowRole {
    pub fn as_str(&self) -> &'static str;  // "main", "notes", "ai"
    pub fn name(&self) -> &'static str;    // "Main", "Notes", "AI"
}
```

### PersistedWindowMode
```rust
#[derive(Serialize, Deserialize)]
pub enum PersistedWindowMode {
    Windowed,
    Maximized,
    Fullscreen,
}
```

### PersistedWindowBounds
```rust
#[derive(Serialize, Deserialize)]
pub struct PersistedWindowBounds {
    pub mode: PersistedWindowMode,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl PersistedWindowBounds {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self;
    pub fn to_gpui(&self) -> WindowBounds;
    pub fn from_gpui(wb: WindowBounds) -> Self;
}

impl Default for PersistedWindowBounds {
    fn default() -> Self {
        Self { mode: Windowed, x: 0.0, y: 0.0, width: 750.0, height: 475.0 }
    }
}
```

### WindowStateFile
```rust
#[derive(Serialize, Deserialize)]
pub struct WindowStateFile {
    pub version: u32,
    pub main: Option<PersistedWindowBounds>,
    pub notes: Option<PersistedWindowBounds>,
    pub ai: Option<PersistedWindowBounds>,
}
```

## File Operations

```rust
/// Get path: ~/.sk/kit/window-state.json
pub fn get_state_file_path() -> PathBuf;

/// Load entire state file
pub fn load_state_file() -> Option<WindowStateFile>;

/// Save state file (atomic: write tmp then rename)
pub fn save_state_file(state: &WindowStateFile) -> bool;

/// Load bounds for specific role
pub fn load_window_bounds(role: WindowRole) -> Option<PersistedWindowBounds>;

/// Save bounds for specific role
pub fn save_window_bounds(role: WindowRole, bounds: PersistedWindowBounds);

/// Reset all positions (delete file)
pub fn reset_all_positions();

/// Check if any positions are customized
pub fn has_custom_positions() -> bool;
```

## Visibility Validation

```rust
const MIN_VISIBLE_AREA: f64 = 64.0 * 64.0;  // 4096 sq pixels
const MIN_EDGE_MARGIN: f64 = 50.0;

/// Check if bounds are visible on current displays
pub fn is_bounds_visible(bounds: &PersistedWindowBounds, displays: &[DisplayBounds]) -> bool;

/// Clamp bounds to ensure visibility
pub fn clamp_bounds_to_displays(bounds: &PersistedWindowBounds, displays: &[DisplayBounds]) -> Option<PersistedWindowBounds>;
```

## High-Level API

### Get Initial Bounds
```rust
/// Get initial bounds: try saved -> clamp if offscreen -> default
pub fn get_initial_bounds(
    role: WindowRole,
    default_bounds: Bounds<Pixels>,
    displays: &[DisplayBounds],
) -> Bounds<Pixels>;
```

### Save From GPUI
```rust
/// Save from GPUI WindowBounds
pub fn save_window_from_gpui(role: WindowRole, window_bounds: WindowBounds);
```

## Usage Patterns

### On Window Open
```rust
use crate::window_state::{get_initial_bounds, WindowRole};
use crate::platform::get_macos_displays;

let displays = get_macos_displays();
let bounds = get_initial_bounds(
    WindowRole::Main,
    default_bounds,
    &displays
);

let options = WindowOptions {
    window_bounds: Some(WindowBounds::Windowed(bounds)),
    ..Default::default()
};
```

### On Window Hide/Close
```rust
use crate::window_state::{save_window_from_gpui, WindowRole};

// In hide handler:
let wb = window.bounds();
save_window_from_gpui(WindowRole::Main, wb);
```

### Multi-Monitor Restoration

The system automatically handles:
1. Saved position still visible -> restore exactly
2. Saved position partially visible -> clamp to nearest display
3. Saved position completely offscreen -> use default

```json
// ~/.sk/kit/window-state.json
{
  "version": 1,
  "main": {
    "mode": "windowed",
    "x": 585.0,
    "y": 290.0,
    "width": 750.0,
    "height": 500.0
  },
  "notes": null,
  "ai": null
}
```
