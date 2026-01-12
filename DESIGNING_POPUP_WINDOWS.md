# Designing Popup Windows in Script Kit GPUI

This document describes the architectural patterns for creating popup windows that float relative to a parent window. The **Actions Window** (`src/actions/`) serves as the canonical reference implementation.

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Parent-Relative Positioning](#2-parent-relative-positioning)
3. [Background Colors & Vibrancy](#3-background-colors--vibrancy)
4. [Resizing & "Pin to Bottom" Behavior](#4-resizing--pin-to-bottom-behavior)
5. [Singleton Window Management](#5-singleton-window-management)
6. [Shared Entity Pattern](#6-shared-entity-pattern)
7. [Focus Management](#7-focus-management)
8. [macOS Platform Configuration](#8-macos-platform-configuration)
9. [Layout Constants](#9-layout-constants)
10. [Complete Implementation Checklist](#10-complete-implementation-checklist)
11. [Search Input Component](#11-search-input-component)
12. [List Component (uniform_list)](#12-list-component-uniform_list)
13. [Keyboard Navigation](#13-keyboard-navigation)
14. [Item Rendering](#14-item-rendering)
15. [Design Tokens Summary](#15-design-tokens-summary)

---

## 1. Architecture Overview

A popup window in Script Kit consists of:

| Component | Purpose | Example File |
|-----------|---------|--------------|
| **Window module** | Window creation, positioning, singleton management | `src/actions/window.rs` |
| **Dialog/View** | UI rendering, keyboard handling, state | `src/actions/dialog.rs` |
| **Constants** | Layout dimensions, heights, margins | `src/actions/constants.rs` |
| **Types** | Data structures | `src/actions/types.rs` |
| **Platform config** | macOS-specific window configuration | `src/platform.rs` |

```
┌─────────────────────────────────────────────────────────────┐
│  Main Window (keeps focus)                                   │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                                                         ││
│  │              Main Content                               ││
│  │                                                         ││
│  │                                     ┌─────────────────┐ ││
│  │                                     │  Popup Window   │ ││
│  │                                     │  (no focus)     │ ││
│  │                                     │                 │ ││
│  │                                     │  - List items   │ ││
│  │                                     │  - Search box   │ ││
│  │                                     └─────────────────┘ ││
│  │                                     ↑                   ││
│  │                                     8px margin          ││
│  └─────────────────────────────────────────────────────────┘│
│  Footer (30px)                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Parent-Relative Positioning

### The Problem

Popup windows must:
- Appear at a specific position relative to the parent window
- Work correctly on **multi-monitor setups** (different displays, different origins)
- Use GPUI's screen-relative coordinate system

### The Solution

**Key file:** `src/actions/window.rs:81-145`

```rust
pub fn open_actions_window(
    cx: &mut App,
    main_window_bounds: Bounds<Pixels>,  // GPUI screen-relative coords
    display_id: Option<DisplayId>,        // Same display as main window
    dialog_entity: Entity<ActionsDialog>,
) -> anyhow::Result<WindowHandle<Root>>
```

#### Step 1: Capture Parent Window Bounds

When opening the popup, capture the parent's bounds from GPUI:

```rust
// In app_impl.rs - when opening the popup
let main_bounds = window.bounds();  // Screen-relative GPUI coords
let display_id = window.display(cx).map(|d| d.id());

open_actions_window(cx, main_bounds, display_id, dialog_entity)?;
```

#### Step 2: Calculate Position Relative to Parent

For a **bottom-right aligned popup** (like Actions):

```rust
// Calculate window position
let window_width = px(POPUP_WIDTH);   // 320px
let window_height = px(initial_height);

// X: Right-aligned with margin from right edge
let window_x = main_window_bounds.origin.x 
    + main_window_bounds.size.width
    - window_width
    - px(ACTIONS_MARGIN_X);  // 8px

// Y: Bottom-aligned, above footer, with margin
let window_y = main_window_bounds.origin.y 
    + main_window_bounds.size.height
    - window_height
    - px(FOOTER_HEIGHT)      // 30px
    - px(ACTIONS_MARGIN_Y);  // 8px

let bounds = Bounds::new(
    point(window_x, window_y),
    size(window_width, window_height),
);
```

#### Step 3: Pass Display ID

**Critical for multi-monitor:** Always pass the display ID to ensure the popup appears on the same screen as the parent:

```rust
let window_options = WindowOptions {
    bounds: Some(bounds),
    display_id,  // Same display as parent!
    // ... other options
};
```

### Position Variants

| Alignment | X Calculation | Y Calculation |
|-----------|---------------|---------------|
| Bottom-right | `parent.x + parent.width - popup.width - margin` | `parent.y + parent.height - popup.height - margin` |
| Bottom-left | `parent.x + margin` | `parent.y + parent.height - popup.height - margin` |
| Top-right | `parent.x + parent.width - popup.width - margin` | `parent.y + margin` |
| Centered | `parent.x + (parent.width - popup.width) / 2` | `parent.y + (parent.height - popup.height) / 2` |

---

## 3. Background Colors & Vibrancy

### The Problem

GPUI hides the native macOS `CAChameleonLayer` that provides automatic dark tinting. Without compensation, windows look washed out over light backgrounds.

### The Solution

**Key files:** `src/actions/window.rs:91-97`, `src/actions/dialog.rs:740-793`, `src/platform.rs:1130-1201`

#### Step 1: Configure Window Background Appearance

```rust
let window_background = if theme.is_vibrancy_enabled() {
    gpui::WindowBackgroundAppearance::Blurred  // Enable blur
} else {
    gpui::WindowBackgroundAppearance::Opaque   // Solid background
};

let window_options = WindowOptions {
    window_background,
    // ...
};
```

#### Step 2: Conditional Background in Dialog

Only apply background color when vibrancy is **disabled**:

```rust
impl Render for MyPopupDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let use_vibrancy = self.theme.is_vibrancy_enabled();
        let main_bg = self.theme.get_main_bg();
        
        div()
            // Only apply bg when vibrancy disabled - let blur show through otherwise
            .when(!use_vibrancy, |d| d.bg(main_bg))
            .rounded(px(12.0))
            .border_1()
            .border_color(border_color)
            // ... children
    }
}
```

#### Step 3: Use Semi-Transparent Colors

For selection highlights, hover states, and other overlays:

```rust
let opacity = self.theme.get_opacity();

// Selection: use white base with low alpha (33%)
let selected_alpha = (opacity.selected * 255.0) as u32;
let selected_bg = rgba((0xFFFFFF << 8) | selected_alpha);

// Hover: even lower alpha (15%)
let hover_alpha = (opacity.hover * 255.0) as u32;
let hover_bg = rgba((0xFFFFFF << 8) | hover_alpha);
```

#### Step 4: Configure macOS Visual Effects (platform.rs)

```rust
pub fn configure_popup_window_vibrancy(window_handle: AnyWindowHandle, cx: &mut App) {
    #[cfg(target_os = "macos")]
    unsafe {
        let ns_window: id = /* get from window handle */;
        
        // Use VibrantDark appearance
        let vibrant_dark: id = msg_send![
            class!(NSAppearance), 
            appearanceNamed: NSAppearanceNameVibrantDark
        ];
        let _: () = msg_send![ns_window, setAppearance: vibrant_dark];
        
        // Clear background color for maximum blur
        let clear_color: id = msg_send![class!(NSColor), clearColor];
        let _: () = msg_send![ns_window, setBackgroundColor: clear_color];
        
        // Mark as non-opaque
        let _: () = msg_send![ns_window, setOpaque: false];
        
        // Configure NSVisualEffectViews in the view hierarchy
        let content_view: id = msg_send![ns_window, contentView];
        configure_visual_effect_views_recursive(content_view);
    }
}
```

### Vibrancy Checklist

- [ ] `WindowBackgroundAppearance::Blurred` when vibrancy enabled
- [ ] Conditional `bg()` - only when vibrancy disabled
- [ ] Semi-transparent colors (70-85% alpha) for surfaces
- [ ] White base + low alpha for selection/hover highlights
- [ ] macOS: VibrantDark appearance + clearColor background

---

## 4. Resizing & "Pin to Bottom" Behavior

### The Problem

When the popup's content changes (e.g., filtering reduces items), the window should:
- Shrink/grow to fit content
- Keep the **bottom edge fixed** (so search input stays in place)
- Animate smoothly

### The Solution

**Key file:** `src/actions/window.rs:251-365`

#### Understanding macOS Coordinate Systems

```
GPUI Coordinates (top-left origin):     macOS Coordinates (bottom-left origin):
┌────────────────────┐ y=0              ┌────────────────────┐
│ origin ─────────►  │                  │                    │
│ │                  │                  │                    │
│ ▼                  │                  │                    │
│                    │                  │                    │
│                    │                  │ origin ─────────►  │
└────────────────────┘ y=max            └─│──────────────────┘ y=0
                                          ▼
```

#### Step 1: Calculate New Height

```rust
pub fn resize_actions_window(
    window_handle: &WindowHandle<Root>,
    num_actions: usize,
    hide_search: bool,
    has_header: bool,
) {
    // Calculate dynamic height
    let search_box_height = if hide_search { 0.0 } else { SEARCH_INPUT_HEIGHT };
    let header_height = if has_header { HEADER_HEIGHT } else { 0.0 };
    let items_height = (num_actions as f32 * ACTION_ITEM_HEIGHT)
        .min(POPUP_MAX_HEIGHT - search_box_height - header_height);
    
    let new_height = items_height + search_box_height + header_height + BORDER_HEIGHT;
}
```

#### Step 2: Keep Bottom Edge Fixed (macOS)

In macOS coordinates, `frame.origin.y` is the **bottom** of the window. To keep the bottom fixed while changing height:

```rust
#[cfg(target_os = "macos")]
unsafe {
    let ns_window: id = /* get window */;
    let frame: NSRect = msg_send![ns_window, frame];
    
    // Key insight: In macOS coords, origin.y IS the bottom edge
    // Keep origin.y the same = keep bottom fixed
    let new_frame = NSRect::new(
        NSPoint::new(frame.origin.x, frame.origin.y),  // Same origin = same bottom
        NSSize::new(frame.size.width, new_height as f64),
    );
    
    // Animate the resize
    let _: () = msg_send![ns_window, setFrame:new_frame display:YES animate:YES];
}
```

#### Pin Direction Variants

| Pin Edge | Origin Change | Size Change |
|----------|---------------|-------------|
| **Bottom** (default) | Keep origin.y | Change height |
| Top | `origin.y += old_height - new_height` | Change height |
| Left | Keep origin.x | Change width |
| Right | `origin.x += old_width - new_width` | Change width |

---

## 5. Singleton Window Management

### The Pattern

Use `OnceLock<Mutex<Option<WindowHandle>>>` for global singleton management:

**Key file:** `src/actions/window.rs:24-79`

```rust
use std::sync::{Mutex, OnceLock};

static POPUP_WINDOW: OnceLock<Mutex<Option<WindowHandle<Root>>>> = OnceLock::new();

fn get_window_lock() -> &'static Mutex<Option<WindowHandle<Root>>> {
    POPUP_WINDOW.get_or_init(|| Mutex::new(None))
}

/// Open the popup (or focus if already open)
pub fn open_popup_window(cx: &mut App, ...) -> anyhow::Result<WindowHandle<Root>> {
    let lock = get_window_lock();
    let mut guard = lock.lock().unwrap();
    
    // Close existing window first
    if let Some(handle) = guard.take() {
        handle.update(cx, |_, window, cx| window.remove(cx)).ok();
    }
    
    // Create new window
    let handle = cx.open_window(options, |window, cx| {
        // ... create view
    })?;
    
    *guard = Some(handle.clone());
    Ok(handle)
}

/// Close the popup
pub fn close_popup_window(cx: &mut App) {
    let lock = get_window_lock();
    let mut guard = lock.lock().unwrap();
    
    if let Some(handle) = guard.take() {
        handle.update(cx, |_, window, cx| window.remove(cx)).ok();
    }
}

/// Check if popup is open
pub fn is_popup_open() -> bool {
    get_window_lock().lock().unwrap().is_some()
}

/// Notify the popup to re-render
pub fn notify_popup(cx: &mut App) {
    if let Some(handle) = get_window_lock().lock().unwrap().as_ref() {
        handle.update(cx, |_, _, cx| cx.notify()).ok();
    }
}
```

---

## 6. Shared Entity Pattern

### The Problem

The popup window shows UI, but the **main window** should handle keyboard events. How do we route events?

### The Solution

Create the entity in the main app, but render it in the popup window:

**Key insight:** GPUI entities can be rendered by any window. The entity lives in the app, not the window.

```rust
// ═══════════════════════════════════════════════════════════════
// Step 1: Define the dialog (just a normal GPUI view)
// ═══════════════════════════════════════════════════════════════
pub struct PopupDialog {
    items: Vec<Item>,
    selected_index: usize,
    // ...
}

impl Render for PopupDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Render the UI
    }
}

impl PopupDialog {
    pub fn move_up(&mut self, cx: &mut Context<Self>) { /* ... */ }
    pub fn move_down(&mut self, cx: &mut Context<Self>) { /* ... */ }
    pub fn submit(&mut self, cx: &mut Context<Self>) { /* ... */ }
}

// ═══════════════════════════════════════════════════════════════
// Step 2: Create a wrapper that holds the shared entity
// ═══════════════════════════════════════════════════════════════
pub struct PopupWindow {
    pub dialog: Entity<PopupDialog>,  // Shared reference
}

impl Render for PopupWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.dialog.clone())  // Just render the entity
    }
}

// ═══════════════════════════════════════════════════════════════
// Step 3: Main app creates the entity and stores it
// ═══════════════════════════════════════════════════════════════
struct MainApp {
    popup_dialog: Option<Entity<PopupDialog>>,  // Stored for keyboard routing
}

impl MainApp {
    fn open_popup(&mut self, cx: &mut Context<Self>) {
        // Create the entity HERE in the main app
        let dialog = cx.new(|cx| PopupDialog::new(cx));
        self.popup_dialog = Some(dialog.clone());
        
        // Open window with the shared entity
        open_popup_window(cx, dialog).ok();
    }
    
    fn handle_key(&mut self, key: &str, cx: &mut Context<Self>) {
        // Route keyboard to the dialog entity
        if let Some(dialog) = &self.popup_dialog {
            dialog.update(cx, |dialog, cx| {
                match key {
                    "up" | "arrowup" => dialog.move_up(cx),
                    "down" | "arrowdown" => dialog.move_down(cx),
                    "enter" => dialog.submit(cx),
                    _ => {}
                }
            });
        }
    }
}
```

### Benefits

1. **Single source of truth** - One entity, one state
2. **Keyboard routing** - Main window handles keys, updates shared entity
3. **Automatic UI updates** - Both windows see state changes via `cx.notify()`

---

## 7. Focus Management

### The Rule

**Popup windows should NOT take focus.** The main window keeps focus and routes keyboard events.

**Key file:** `src/actions/window.rs:159`

```rust
let window_options = WindowOptions {
    focus: false,  // CRITICAL: Don't steal focus
    // ...
};
```

### Why?

1. User expects to keep typing in main window
2. Keyboard shortcuts should continue working
3. Closing popup shouldn't disrupt focus state

### Focus Flow Diagram

```
┌─────────────────────────────────────┐
│           Main Window               │
│         (HAS FOCUS)                 │
│                                     │
│  KeyDown("ArrowDown")               │
│         │                           │
│         ▼                           │
│  if popup_open {                    │
│      dialog.move_down()  ─────────────────► Popup Window
│  }                                  │       (NO FOCUS)
│                                     │       
│                                     │       Re-renders with
│                                     │       new selection
└─────────────────────────────────────┘
```

---

## 8. macOS Platform Configuration

### Complete Window Configuration

**Key file:** `src/platform.rs:1130-1201`

```rust
pub fn configure_popup_window(window_handle: AnyWindowHandle, cx: &mut App) {
    #[cfg(target_os = "macos")]
    window_handle.update(cx, |_, window, _cx| {
        let ns_window = window.as_raw_window();
        
        unsafe {
            // ═══════════════════════════════════════════════════════════
            // 1. Prevent dragging (popup is anchored to parent)
            // ═══════════════════════════════════════════════════════════
            let _: () = msg_send![ns_window, setMovable: false];
            let _: () = msg_send![ns_window, setMovableByWindowBackground: false];
            
            // ═══════════════════════════════════════════════════════════
            // 2. Auto-hide when app loses focus
            // ═══════════════════════════════════════════════════════════
            let _: () = msg_send![ns_window, setHidesOnDeactivate: true];
            
            // ═══════════════════════════════════════════════════════════
            // 3. Float above other windows (but below main window)
            // ═══════════════════════════════════════════════════════════
            // NSPopUpMenuWindowLevel = 101
            // NSFloatingWindowLevel = 3
            let _: () = msg_send![ns_window, setLevel: 3i64];
            
            // ═══════════════════════════════════════════════════════════
            // 4. Vibrancy configuration
            // ═══════════════════════════════════════════════════════════
            let vibrant_dark: id = msg_send![
                class!(NSAppearance), 
                appearanceNamed: NSAppearanceNameVibrantDark
            ];
            let _: () = msg_send![ns_window, setAppearance: vibrant_dark];
            
            let clear_color: id = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![ns_window, setBackgroundColor: clear_color];
            let _: () = msg_send![ns_window, setOpaque: false];
            
            // ═══════════════════════════════════════════════════════════
            // 5. Window style (no title bar, no shadow)
            // ═══════════════════════════════════════════════════════════
            let style_mask: NSUInteger = NSWindowStyleMask::Borderless as NSUInteger
                | NSWindowStyleMask::NonactivatingPanel as NSUInteger;
            let _: () = msg_send![ns_window, setStyleMask: style_mask];
            let _: () = msg_send![ns_window, setHasShadow: false];
            
            // ═══════════════════════════════════════════════════════════
            // 6. Configure visual effect views for blur
            // ═══════════════════════════════════════════════════════════
            let content_view: id = msg_send![ns_window, contentView];
            configure_visual_effect_views(content_view);
        }
    }).ok();
}
```

### Visual Effect View Configuration

```rust
unsafe fn configure_visual_effect_views(view: id) {
    let class_name: id = msg_send![view, className];
    let class_str: *const i8 = msg_send![class_name, UTF8String];
    let name = std::ffi::CStr::from_ptr(class_str).to_string_lossy();
    
    if name.contains("NSVisualEffectView") {
        // Material: .sidebar (26), .hudWindow (13), .popover (5)
        let _: () = msg_send![view, setMaterial: 26i64];
        
        // Blending mode: .behindWindow (1)
        let _: () = msg_send![view, setBlendingMode: 1i64];
        
        // State: .active (1) - always show vibrancy
        let _: () = msg_send![view, setState: 1i64];
    }
    
    // Recurse into subviews
    let subviews: id = msg_send![view, subviews];
    let count: usize = msg_send![subviews, count];
    for i in 0..count {
        let subview: id = msg_send![subviews, objectAtIndex: i];
        configure_visual_effect_views(subview);
    }
}
```

---

## 9. Layout Constants

### Recommended Constants File

Create a `constants.rs` for your popup:

```rust
// src/mypopup/constants.rs

/// Fixed width of the popup window
pub const POPUP_WIDTH: f32 = 320.0;

/// Maximum height before scrolling
pub const POPUP_MAX_HEIGHT: f32 = 400.0;

/// Height of each item in the list (for uniform_list)
pub const ITEM_HEIGHT: f32 = 44.0;

/// Height of the search input box
pub const SEARCH_INPUT_HEIGHT: f32 = 44.0;

/// Height of the header/title bar
pub const HEADER_HEIGHT: f32 = 24.0;

/// Horizontal margin from parent window edge
pub const MARGIN_X: f32 = 8.0;

/// Vertical margin from parent window edge
pub const MARGIN_Y: f32 = 8.0;

/// Border radius for the popup container
pub const BORDER_RADIUS: f32 = 12.0;

/// Inner padding for content
pub const CONTENT_PADDING: f32 = 6.0;

/// Selection highlight corner radius
pub const SELECTION_RADIUS: f32 = 8.0;
```

### Using Constants

```rust
use super::constants::*;

impl Render for PopupDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(POPUP_WIDTH))
            .max_h(px(POPUP_MAX_HEIGHT))
            .rounded(px(BORDER_RADIUS))
            .p(px(CONTENT_PADDING))
            .child(
                uniform_list("items", self.items.len(), |this, range, _, _| {
                    range.map(|i| {
                        div().h(px(ITEM_HEIGHT))  // Fixed height for virtualization
                            // ...
                    }).collect()
                })
            )
    }
}
```

---

## 10. Complete Implementation Checklist

Use this checklist when creating a new popup window:

### File Structure
- [ ] `src/mypopup/mod.rs` - Module exports
- [ ] `src/mypopup/window.rs` - Window management (singleton, open/close)
- [ ] `src/mypopup/dialog.rs` - UI component (render, keyboard, state)
- [ ] `src/mypopup/constants.rs` - Layout dimensions
- [ ] `src/mypopup/types.rs` - Data structures (if needed)

### Window Management (`window.rs`)
- [ ] `OnceLock<Mutex<Option<WindowHandle>>>` singleton
- [ ] `open_*_window()` function with parent bounds + display_id
- [ ] `close_*_window()` function
- [ ] `is_*_window_open()` function
- [ ] `resize_*_window()` function (if dynamic sizing needed)
- [ ] Root wrapper: `cx.new(|cx| Root::new(view, window, cx))`

### Positioning
- [ ] Accept `main_window_bounds: Bounds<Pixels>` parameter
- [ ] Accept `display_id: Option<DisplayId>` parameter
- [ ] Calculate X position relative to parent
- [ ] Calculate Y position relative to parent
- [ ] Pass `display_id` to `WindowOptions`

### WindowOptions
- [ ] `focus: false` (don't steal focus)
- [ ] `window_background: Blurred` when vibrancy enabled
- [ ] Correct `bounds` calculation
- [ ] `display_id` for multi-monitor

### Vibrancy
- [ ] Conditional `bg()` in dialog render
- [ ] Semi-transparent colors for overlays
- [ ] macOS: `configure_*_popup_window()` in platform.rs
- [ ] VibrantDark appearance
- [ ] clearColor background
- [ ] Visual effect view configuration

### Resizing (if dynamic)
- [ ] Calculate height based on content
- [ ] Pin to correct edge (usually bottom)
- [ ] macOS: Use `setFrame:display:animate:`
- [ ] Handle edge cases (min/max height)

### Shared Entity Pattern
- [ ] Entity created in main app
- [ ] Stored in main app for keyboard routing
- [ ] PopupWindow wrapper renders shared entity
- [ ] Main app routes keyboard events

### Platform Configuration (macOS)
- [ ] `setMovable: false`
- [ ] `setMovableByWindowBackground: false`
- [ ] `setHidesOnDeactivate: true` (optional)
- [ ] Appropriate window level
- [ ] Configure NSVisualEffectViews

### Testing
- [ ] Test via stdin JSON protocol
- [ ] Verify positioning on primary monitor
- [ ] Test multi-monitor positioning
- [ ] Test keyboard navigation
- [ ] Test resize behavior
- [ ] Visual test with `captureScreenshot()`

---

## 11. Search Input Component

The search input provides type-to-filter functionality. It's positioned at the **bottom** of the popup (search stays fixed while list shrinks from top).

**Key file:** `src/actions/dialog.rs:845-947`

### Dimensions

| Property | Value | Source |
|----------|-------|--------|
| Container height | 44px | `SEARCH_INPUT_HEIGHT` |
| Inner input width | 240px | Fixed |
| Inner input height | 28px | Fixed |
| Icon container width | 24px | Fixed |
| Cursor width | 2px | |
| Cursor height | 16px | |

### Structure

```
┌─────────────────────────────────────────────────────────────┐
│ ┌──────┐ ┌──────────────────────────────────────────────┐  │ 44px
│ │ ⌘K   │ │ Search text...█                              │  │
│ └──────┘ └──────────────────────────────────────────────┘  │
│   24px                    240px                            │
└─────────────────────────────────────────────────────────────┘
```

### Container Styling

```rust
div()
    .w(px(POPUP_WIDTH))                // 320px
    .h(px(SEARCH_INPUT_HEIGHT))        // 44px
    .px(px(16.0))                      // Horizontal padding
    .py(px(10.0))                      // Vertical padding
    .bg(search_box_bg)
    .border_t_1()                      // Top border (input at bottom)
    .border_color(border_color)
    .flex()
    .flex_row()
    .items_center()
    .gap(px(8.0))
```

### Inner Input Field

```rust
div()
    .flex_shrink_0()                   // CRITICAL: prevents flexbox shrinking
    .w(px(240.0))                      // Fixed width
    .h(px(28.0))                       // Fixed height
    .px(px(8.0))                       // Inner padding
    .py(px(4.0))
    .bg(input_bg)                      // Semi-transparent
    .rounded(px(4.0))                  // Small radius
    .border_1()
    .border_color(border_color)
    .flex()
    .flex_row()
    .items_center()
    .text_sm()
```

### State-Dependent Styling

```rust
// Background alpha varies by state
let bg_alpha = if search_text.is_empty() { 0x20 } else { 0x40 };  // 12.5% → 25%

// Border color changes when text present
let border = if !search_text.is_empty() {
    accent_color_with_alpha(0x60)  // 37.5% - highlighted
} else {
    border_color_with_alpha(0x80)  // 50% - subtle
};
```

### Blinking Cursor

```rust
div()
    .w(px(2.0))
    .h(px(16.0))
    .rounded(px(1.0))
    .when(cursor_visible, |d| d.bg(accent_color))
```

Cursor blink is controlled by a timer in the parent component toggling `cursor_visible`.

### Text Handling

The parent component routes character input to the dialog:

```rust
// In dialog
pub fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
    self.search_text.push(ch);
    self.refilter();
    cx.notify();
}

pub fn handle_backspace(&mut self, cx: &mut Context<Self>) {
    if !self.search_text.is_empty() {
        self.search_text.pop();
        self.refilter();
        cx.notify();
    }
}
```

---

## 12. List Component (uniform_list)

Use GPUI's `uniform_list` for virtualized scrolling with fixed-height items.

**Key file:** `src/actions/dialog.rs:1000-1213`

### Configuration

```rust
let list = uniform_list(
    "actions-list",                    // Element ID (for debugging)
    filtered_items.len(),              // Total item count
    cx.processor(move |this, visible_range, _window, _cx| {
        // Render only items in visible_range
        visible_range.map(|idx| this.render_item(idx)).collect()
    }),
)
.flex_1()
.w_full()
.track_scroll(&self.scroll_handle);   // UniformListScrollHandle
```

### Critical: Fixed Item Height

`uniform_list` **requires** all items to have identical height for virtualization:

```rust
pub const ACTION_ITEM_HEIGHT: f32 = 44.0;  // Must be consistent!

// Every item must use this exact height
div()
    .h(px(ACTION_ITEM_HEIGHT))
    // ...
```

### Item Structure (Pill-Style Selection)

```
┌─────────────────────────────────────────────────────────────┐ 44px
│ ┌─────────────────────────────────────────────────────────┐ │
│ │  Title                                        ⌘ ⇧ T    │ │ Selection pill
│ └─────────────────────────────────────────────────────────┘ │ (rounded)
└─────────────────────────────────────────────────────────────┘
  6px inset                                              6px inset
```

```rust
// Outer item (full width, provides inset)
let item = div()
    .w_full()
    .h(px(ACTION_ITEM_HEIGHT))         // 44px
    .px(px(ACTION_ROW_INSET))          // 6px horizontal inset
    .py(px(2.0))                       // 2px vertical padding
    .flex()
    .flex_col()
    .justify_center();

// Inner row (the selection pill)
let inner_row = div()
    .w_full()
    .flex_1()
    .flex()
    .flex_row()
    .items_center()
    .px(px(16.0))                      // Content padding
    .rounded(px(SELECTION_RADIUS))     // 8px - pill corners
    .bg(if is_selected { selected_bg } else { transparent })
    .hover(|s| s.bg(hover_bg))
    .cursor_pointer();
```

### Selection & Hover Colors

Use semi-transparent colors for vibrancy compatibility:

```rust
let opacity = theme.get_opacity();
let selected_alpha = (opacity.selected * 255.0) as u32;  // ~33% (0x54)
let hover_alpha = (opacity.hover * 255.0) as u32;        // ~15% (0x26)

// White base + low alpha = subtle brightening that lets blur through
let selected_bg = rgba((0xFFFFFF << 8) | selected_alpha);  // rgba(0xffffff54)
let hover_bg = rgba((0xFFFFFF << 8) | hover_alpha);        // rgba(0xffffff26)
```

### Category Separators

Add visual separation between action categories:

```rust
let is_category_start = idx > 0 && prev_action.category != action.category;

if is_category_start {
    item = item.border_t_1().border_color(separator_color);
}

// Separator: border color at 25% alpha
let separator_color = rgba(hex_with_alpha(theme.colors.ui.border, 0x40));
```

### Scroll Handling

```rust
// Store handle in struct
pub scroll_handle: UniformListScrollHandle,

// Scroll to selected item
self.scroll_handle.scroll_to_item(
    self.selected_index, 
    ScrollStrategy::Nearest  // Only scroll if item not visible
);
```

### Scrollbar Overlay

```rust
// Position scrollbar absolutely over the list
div()
    .relative()
    .child(list)
    .child(
        Scrollbar::new(total_items, visible_items, scroll_offset, colors)
            .absolute()
            .right_0()
            .top_0()
            .bottom_0()
    )
```

---

## 13. Keyboard Navigation

**Critical:** The popup does NOT handle its own keyboard events. The parent window routes all keyboard input to the shared dialog entity.

### Why Parent Handles Keyboard

1. Popup has `focus: false` - it can't receive key events
2. Main window keeps focus for consistent UX
3. Allows main window to intercept/filter events
4. Single source of truth for keyboard handling

### Navigation Methods

```rust
impl PopupDialog {
    pub fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.scroll_handle.scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }

    pub fn move_down(&mut self, cx: &mut Context<Self>) {
        let max_idx = self.filtered_items.len().saturating_sub(1);
        if self.selected_index < max_idx {
            self.selected_index += 1;
            self.scroll_handle.scroll_to_item(self.selected_index, ScrollStrategy::Nearest);
            cx.notify();
        }
    }

    pub fn submit_selected(&mut self) {
        if let Some(item) = self.filtered_items.get(self.selected_index) {
            (self.on_select)(item.id.clone());
        }
    }

    pub fn submit_cancel(&mut self) {
        (self.on_select)("__cancel__".to_string());
    }
}
```

### Parent Keyboard Handler

```rust
// In main app's key handler
fn handle_key(&mut self, key: &str, cx: &mut Context<Self>) {
    if let Some(dialog) = &self.popup_dialog {
        dialog.update(cx, |dialog, cx| {
            match key {
                "up" | "arrowup" => dialog.move_up(cx),
                "down" | "arrowdown" => dialog.move_down(cx),
                "enter" | "Enter" => dialog.submit_selected(),
                "escape" | "Escape" => dialog.submit_cancel(),
                _ => {}
            }
        });
    }
}

// Character input (for type-to-filter)
fn handle_char(&mut self, ch: char, cx: &mut Context<Self>) {
    if let Some(dialog) = &self.popup_dialog {
        dialog.update(cx, |dialog, cx| {
            dialog.handle_char(ch, cx);
        });
    }
}
```

### Fuzzy Filtering

Score-based filtering with ranked results:

```rust
fn refilter(&mut self) {
    if self.search_text.is_empty() {
        self.filtered_items = (0..self.items.len()).collect();
        return;
    }

    let search_lower = self.search_text.to_lowercase();
    
    // Score each item
    let mut scored: Vec<(usize, i32)> = self.items.iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            let score = Self::score_item(item, &search_lower);
            if score > 0 { Some((idx, score)) } else { None }
        })
        .collect();

    // Sort by score descending (best matches first)
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    
    self.filtered_items = scored.into_iter().map(|(idx, _)| idx).collect();
    self.selected_index = 0;  // Reset selection
}

fn score_item(item: &Item, search: &str) -> i32 {
    let title = item.title.to_lowercase();
    
    if title.starts_with(search) { return 100; }      // Prefix match
    if title.contains(search) { return 50; }          // Contains match
    if Self::fuzzy_match(&title, search) { return 25; }  // Subsequence
    
    0  // No match
}
```

---

## 14. Item Rendering

### Content Layout

```
┌─────────────────────────────────────────────────────────────┐
│  Title text                                    ⌘  ⇧  T     │
│  └─ flex_1 (fills space)                       └─ keycaps  │
└─────────────────────────────────────────────────────────────┘
```

```rust
let content = div()
    .flex_1()
    .flex()
    .flex_row()
    .items_center()
    .justify_between()   // Title left, shortcuts right
    .child(title_div)
    .child(keycap_row);
```

### Title Typography

```rust
div()
    .text_color(if is_selected { primary_text } else { secondary_text })
    .text_sm()           // ~14px
    .font_weight(if is_selected {
        FontWeight::MEDIUM   // 500 when selected
    } else {
        FontWeight::NORMAL   // 400 normally
    })
    .child(title_str)
```

### Keyboard Shortcut Badges (Keycaps)

Individual keys styled as keyboard-like badges:

```
 ┌───┐ ┌───┐ ┌───┐
 │ ⌘ │ │ ⇧ │ │ T │
 └───┘ └───┘ └───┘
   3px   3px
```

```rust
// Parse shortcut into individual keycaps
// "cmd+shift+t" → ["⌘", "⇧", "T"]
let keycaps = parse_shortcut_keycaps(&shortcut);

let mut keycap_row = div()
    .flex()
    .flex_row()
    .items_center()
    .gap(px(3.0));   // 3px gap between keycaps

for keycap in keycaps {
    keycap_row = keycap_row.child(
        div()
            .min_w(px(KEYCAP_MIN_WIDTH))   // 22px
            .h(px(KEYCAP_HEIGHT))          // 22px
            .px(px(6.0))                   // 6px horizontal padding
            .flex()
            .items_center()
            .justify_center()
            .bg(keycap_bg)                 // border @ 50% alpha
            .border_1()
            .border_color(keycap_border)  // border @ 62.5% alpha
            .rounded(px(5.0))             // 5px radius
            .text_xs()                    // ~10-12px
            .text_color(dimmed_text)
            .child(keycap)
    );
}
```

### Keycap Dimensions

```rust
pub const KEYCAP_MIN_WIDTH: f32 = 22.0;
pub const KEYCAP_HEIGHT: f32 = 22.0;
```

### Shortcut Symbol Mapping

```rust
fn format_shortcut(shortcut: &str) -> String {
    // "cmd+shift+t" → "⌘⇧T"
    shortcut.split('+').map(|part| {
        match part.trim().to_lowercase().as_str() {
            "cmd" | "command" | "meta" => "⌘",
            "ctrl" | "control" => "⌃",
            "alt" | "opt" | "option" => "⌥",
            "shift" => "⇧",
            "enter" | "return" => "↵",
            "escape" | "esc" => "⎋",
            "tab" => "⇥",
            "backspace" | "delete" => "⌫",
            "space" => "␣",
            "up" | "arrowup" => "↑",
            "down" | "arrowdown" => "↓",
            "left" | "arrowleft" => "←",
            "right" | "arrowright" => "→",
            other => other.to_uppercase(),
        }
    }).collect()
}
```

### Header (Optional Context Title)

```rust
div()
    .w_full()
    .h(px(HEADER_HEIGHT))              // 24px
    .px(px(16.0))
    .pt(px(8.0))
    .pb(px(4.0))
    .border_b_1()
    .border_color(header_border)       // 25% alpha
    .child(
        div()
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)  // 600
            .text_color(dimmed_text)
            .child(context_title)
    )
```

---

## 15. Design Tokens Summary

### Dimensions Quick Reference

| Component | Property | Value | Constant |
|-----------|----------|-------|----------|
| **Popup** | Width | 320px | `POPUP_WIDTH` |
| **Popup** | Max height | 400px | `POPUP_MAX_HEIGHT` |
| **Popup** | Corner radius | 12px | `radius_lg` |
| **Popup** | Border | 1px | `border_thin` |
| **Search** | Container height | 44px | `SEARCH_INPUT_HEIGHT` |
| **Search** | Input width | 240px | - |
| **Search** | Input height | 28px | - |
| **Search** | Input radius | 4px | `radius_sm` |
| **Item** | Height | 44px | `ACTION_ITEM_HEIGHT` |
| **Item** | Row inset | 6px | `ACTION_ROW_INSET` |
| **Item** | Selection radius | 8px | `SELECTION_RADIUS` |
| **Keycap** | Min width | 22px | `KEYCAP_MIN_WIDTH` |
| **Keycap** | Height | 22px | `KEYCAP_HEIGHT` |
| **Keycap** | Padding X | 6px | - |
| **Keycap** | Radius | 5px | - |
| **Keycap** | Gap | 3px | - |
| **Header** | Height | 24px | `HEADER_HEIGHT` |

### Spacing Defaults

```rust
padding_xs: 4.0,
padding_sm: 8.0,
padding_md: 12.0,
padding_lg: 16.0,
gap_sm: 4.0,
gap_md: 8.0,
item_padding_x: 16.0,
item_padding_y: 8.0,
```

### Alpha Values Quick Reference

| Use Case | Alpha | Hex | Percentage |
|----------|-------|-----|------------|
| Empty input bg | 0x20 | 32 | 12.5% |
| Active input bg | 0x40 | 64 | 25% |
| Keycap bg | 0x80 | 128 | 50% |
| Keycap border | 0xA0 | 160 | 62.5% |
| Selection highlight | 0x54 | 84 | 33% |
| Hover highlight | 0x26 | 38 | 15% |
| Separator | 0x40 | 64 | 25% |
| Subtle border | 0x80 | 128 | 50% |
| Accent border | 0x60 | 96 | 37.5% |

### Color Pattern

```rust
// Helper for hex color with alpha
fn hex_with_alpha(hex: u32, alpha: u8) -> u32 {
    (hex << 8) | (alpha as u32)
}

// Usage
let selected_bg = rgba(hex_with_alpha(0xFFFFFF, 0x54));  // White @ 33%
let hover_bg = rgba(hex_with_alpha(0xFFFFFF, 0x26));     // White @ 15%
```

---

## Reference Implementation

The **Actions Window** (`src/actions/`) is the canonical reference. Study these files:

| File | Key Patterns |
|------|--------------|
| `window.rs:24-79` | Singleton management |
| `window.rs:81-145` | Position calculation |
| `window.rs:251-365` | Pin-to-bottom resize |
| `dialog.rs:740-793` | Vibrancy-aware colors |
| `dialog.rs:1297-1307` | Conditional background |
| `platform.rs:1130-1201` | macOS configuration |
| `constants.rs` | Layout values |

---

## Common Mistakes

1. **Forgetting `focus: false`** - Popup steals focus, keyboard stops working in main window
2. **Missing `display_id`** - Popup appears on wrong monitor
3. **Hardcoded backgrounds** - Vibrancy doesn't show through
4. **Wrong resize origin** - Window jumps instead of growing from fixed edge
5. **No Root wrapper** - Theming breaks, vibrancy doesn't work
6. **Blocking main thread** - Popup positioning lags
7. **Not using shared entity** - Keyboard events don't reach popup

---

*Last updated: January 2026*
