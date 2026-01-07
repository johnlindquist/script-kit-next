# Multi-Window Management - Expert Bundle

## Overview

Script Kit manages multiple independent windows: Main launcher, Notes, and AI chat. Each uses a singleton pattern with proper lifecycle management.

## Window Architecture

### Window Types

```
~/.scriptkit/
├── Main Window    - Primary launcher (src/main.rs)
├── Notes Window   - Note-taking (src/notes/)
└── AI Window      - Chat interface (src/ai/)
```

### Singleton Pattern

```rust
use std::sync::{Mutex, OnceLock};
use gpui::{WindowHandle, Context};

/// Global handle for Notes window (single instance)
static NOTES_WINDOW: OnceLock<Mutex<Option<WindowHandle<Root>>>> = OnceLock::new();

fn notes_window() -> &'static Mutex<Option<WindowHandle<Root>>> {
    NOTES_WINDOW.get_or_init(|| Mutex::new(None))
}

/// Open Notes window (creates or focuses existing)
pub fn open_notes(cx: &mut AppContext) {
    let mut handle_guard = notes_window().lock().unwrap();
    
    if let Some(ref handle) = *handle_guard {
        // Window exists - focus it
        if handle.update(cx, |_, _, cx| {
            cx.activate(true);
        }).is_ok() {
            return;
        }
        // Update failed - window was closed, clear handle
        *handle_guard = None;
    }
    
    // Create new window
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(600.0), px(700.0)),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("Notes".into()),
            ..Default::default()
        }),
        kind: WindowKind::Normal,
        ..Default::default()
    };
    
    if let Ok(handle) = cx.open_window(options, |window, cx| {
        let view = cx.new(|cx| NotesApp::new(window, cx));
        cx.new(|cx| Root::new(view, window, cx))
    }) {
        *handle_guard = Some(handle);
    }
}
```

## Root Wrapper Pattern

### Why Root is Required

```rust
// GPUI-Component requires Root wrapper for theming and styling
use gpui_component::Root;

// Creating a window with Root
let handle = cx.open_window(opts, |window, cx| {
    let view = cx.new(|cx| NotesApp::new(window, cx));
    // Root MUST wrap the view
    cx.new(|cx| Root::new(view, window, cx))
})?;
```

### Root Configuration

```rust
impl Root {
    pub fn new(content: View<impl Render>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Root provides:
        // - Theme context
        // - Focus management
        // - Global event handling
        Self {
            content: content.into_any(),
            theme: load_theme(),
            focus_handle: cx.focus_handle(),
        }
    }
}
```

## Window Options

### Standard Configurations

```rust
// Floating panel (main launcher)
fn main_window_options(display: &Display, cx: &AppContext) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(
            Bounds::centered(Some(display.id()), size(px(680.0), px(400.0)), cx)
        )),
        titlebar: None, // No titlebar for launcher
        kind: WindowKind::PopUp,
        is_movable: true,
        focus: true,
        show: true,
        ..Default::default()
    }
}

// Standard window (Notes, AI)
fn standard_window_options(title: &str, width: f32, height: f32, cx: &AppContext) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(
            Bounds::centered(None, size(px(width), px(height)), cx)
        )),
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(10.0), px(10.0))),
        }),
        kind: WindowKind::Normal,
        ..Default::default()
    }
}
```

## Multi-Monitor Support

### Display Selection

```rust
fn get_display_for_window(cx: &AppContext) -> Display {
    // Prefer display containing mouse cursor
    let mouse_pos = cx.mouse_position();
    
    for display in cx.displays() {
        if display.bounds().contains(&mouse_pos) {
            return display;
        }
    }
    
    // Fallback to primary
    cx.primary_display()
}
```

### Centered Positioning

```rust
fn center_on_display(display: &Display, size: Size<Pixels>, cx: &AppContext) -> Bounds<Pixels> {
    let visible = display.visible_bounds(); // Excludes menu bar, dock
    
    Bounds::new(
        point(
            visible.origin.x + (visible.size.width - size.width) / 2.0,
            visible.origin.y + (visible.size.height - size.height) / 3.0, // Upper third
        ),
        size,
    )
}
```

## Window Lifecycle

### Opening

```rust
pub fn open_ai_window(cx: &mut AppContext) -> Result<()> {
    let mut guard = ai_window().lock().unwrap();
    
    // Check for existing
    if let Some(ref handle) = *guard {
        if handle.update(cx, |_, _, cx| cx.activate(true)).is_ok() {
            return Ok(());
        }
        *guard = None;
    }
    
    // Create new
    let display = get_display_for_window(cx);
    let options = standard_window_options("AI Chat", 700.0, 800.0, cx);
    
    let handle = cx.open_window(options, |window, cx| {
        let view = cx.new(|cx| AiApp::new(window, cx));
        cx.new(|cx| Root::new(view, window, cx))
    })?;
    
    *guard = Some(handle);
    
    // Configure macOS panel behavior
    #[cfg(target_os = "macos")]
    configure_as_panel(&handle);
    
    Ok(())
}
```

### Closing

```rust
pub fn close_notes_window(cx: &mut AppContext) {
    let mut guard = notes_window().lock().unwrap();
    
    if let Some(handle) = guard.take() {
        // Graceful close - save state first
        let _ = handle.update(cx, |view, _, cx| {
            view.save_state();
            cx.remove_window();
        });
    }
}
```

### Toggle Pattern

```rust
pub fn toggle_notes(cx: &mut AppContext) {
    let guard = notes_window().lock().unwrap();
    
    if let Some(ref handle) = *guard {
        // Check if visible
        let is_visible = handle.read(cx)
            .map(|_| true) // If we can read, it exists
            .unwrap_or(false);
        
        if is_visible {
            drop(guard);
            close_notes_window(cx);
        } else {
            handle.update(cx, |_, _, cx| cx.activate(true)).ok();
        }
    } else {
        drop(guard);
        open_notes(cx);
    }
}
```

## macOS Panel Configuration

### Floating Panel

```rust
#[cfg(target_os = "macos")]
fn configure_as_panel(handle: &WindowHandle<Root>) {
    use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior};
    use cocoa::base::id;
    use objc::{msg_send, sel, sel_impl};
    
    let _ = handle.update(|_, _, cx| {
        cx.activate(true);
        
        unsafe {
            let app: id = NSApp();
            let window: id = msg_send![app, keyWindow];
            
            if window != nil {
                // Float above normal windows
                let _: () = msg_send![window, setLevel: 3i32]; // NSFloatingWindowLevel
                
                // Show on all spaces
                let _: () = msg_send![
                    window, 
                    setCollectionBehavior: NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                ];
            }
        }
    });
}
```

## Hotkey Integration

### Window Hotkeys

```rust
// In hotkeys.rs
pub fn start_hotkey_listener(config: Config) {
    // Register window hotkeys
    register_builtin_hotkey(&manager, HotkeyAction::Main, &config.hotkey);
    register_builtin_hotkey(&manager, HotkeyAction::Notes, &config.get_notes_hotkey());
    register_builtin_hotkey(&manager, HotkeyAction::Ai, &config.get_ai_hotkey());
    
    // Event loop handles dispatch
    loop {
        if let Ok(event) = receiver.recv() {
            match routes().read().unwrap().get_action(event.id) {
                Some(HotkeyAction::Notes) => dispatch_notes_hotkey(),
                Some(HotkeyAction::Ai) => dispatch_ai_hotkey(),
                // ...
            }
        }
    }
}
```

### GCD Dispatch to Main Thread

```rust
fn dispatch_notes_hotkey() {
    let handler = NOTES_HANDLER.lock().unwrap().clone();
    
    if let Some(handler) = handler {
        gcd::dispatch_to_main(move || {
            handler();
        });
    } else {
        notes_hotkey_channel().0.try_send(()).ok();
    }
}
```

## Window Registry

### Tracking Multiple Windows

```rust
use std::collections::HashMap;

pub struct WindowRegistry {
    windows: RwLock<HashMap<String, WindowHandle<Root>>>,
}

impl WindowRegistry {
    pub fn register(&self, id: &str, handle: WindowHandle<Root>) {
        self.windows.write().unwrap().insert(id.to_string(), handle);
    }
    
    pub fn unregister(&self, id: &str) {
        self.windows.write().unwrap().remove(id);
    }
    
    pub fn get(&self, id: &str) -> Option<WindowHandle<Root>> {
        self.windows.read().unwrap().get(id).cloned()
    }
    
    pub fn close_all(&self, cx: &mut AppContext) {
        let handles: Vec<_> = self.windows.write().unwrap().drain().collect();
        for (_, handle) in handles {
            let _ = handle.update(cx, |_, _, cx| cx.remove_window());
        }
    }
}
```

## Theme Synchronization

### Shared Theme

```rust
// All windows load from same theme file
static SHARED_THEME: OnceLock<RwLock<Theme>> = OnceLock::new();

pub fn shared_theme() -> &'static RwLock<Theme> {
    SHARED_THEME.get_or_init(|| RwLock::new(load_theme()))
}

// When theme changes, notify all windows
pub fn reload_theme(cx: &mut AppContext) {
    let theme = load_theme();
    *shared_theme().write().unwrap() = theme.clone();
    
    // Notify all windows
    for (_, handle) in window_registry().windows.read().unwrap().iter() {
        let _ = handle.update(cx, |_, _, cx| cx.notify());
    }
}
```

## Best Practices

1. **Use singleton pattern** for special windows (Notes, AI)
2. **Always wrap in Root** for gpui-component compatibility
3. **Handle stale handles** - window may be closed externally
4. **Use visible_bounds** for positioning (respects dock/menubar)
5. **Configure as panel** for floating behavior on macOS
6. **Share theme** across all windows
7. **Use GCD dispatch** for hotkey-triggered opens on macOS

## Summary

| Window | Type | Hotkey | Pattern |
|--------|------|--------|---------|
| Main | PopUp | `Cmd+;` | Toggle visibility |
| Notes | Normal | `Cmd+Shift+N` | Singleton, focus or create |
| AI | Normal | `Cmd+Shift+Space` | Singleton, focus or create |
