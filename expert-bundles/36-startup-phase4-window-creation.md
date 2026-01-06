# Script Kit GPUI - Expert Review Request

## Project Context

Script Kit GPUI is a **Rust desktop app** built with GPUI (Zed's UI framework) that serves as a command launcher and script runner. Think: Raycast/Alfred but scriptable with TypeScript.

**Architecture:**
- **GPUI** for UI rendering (custom immediate-mode reactive UI framework from Zed)
- **Bun** as the TypeScript runtime for user scripts
- **Stdin/stdout JSON protocol** for bidirectional script ↔ app communication
- **SQLite** for persistence (clipboard history, notes, chat)
- **macOS-first** with floating panel window behavior

**Key Constraints:**
- Must maintain backwards compatibility with existing Script Kit scripts
- Performance-critical: launcher must appear instantly, list scrolling at 60fps
- Multi-window: main launcher + Notes window + AI chat window (all independent)
- Theme hot-reload across all windows

---

## Bundle: Phase 4 - Main Window Creation

This bundle covers the creation of the main launcher window and ScriptListApp initialization.

---

## Phase 4 Sequence (main.rs lines 1454-1537)

```rust
// 1. Calculate window bounds
let window_size = size(px(750.), initial_window_height());
let default_bounds = calculate_eye_line_bounds_on_mouse_display(window_size);
let displays = platform::get_macos_displays();
let bounds = window_state::get_initial_bounds(
    window_state::WindowRole::Main,
    default_bounds,
    &displays,
);

// 2. Determine window background (vibrancy)
let initial_theme = theme::load_theme();
let window_background = if initial_theme.is_vibrancy_enabled() {
    WindowBackgroundAppearance::Blurred
} else {
    WindowBackgroundAppearance::Opaque
};

// 3. Holder for ScriptListApp entity (needed since Root wraps the view)
let app_entity_holder: Arc<Mutex<Option<Entity<ScriptListApp>>>> = 
    Arc::new(Mutex::new(None));

// 4. Open the window
let window: WindowHandle<Root> = cx.open_window(
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: None,
        is_movable: true,
        window_background,
        show: false,    // Start hidden
        focus: false,   // Don't focus on creation
        ..Default::default()
    },
    |window, cx| {
        // Create ScriptListApp
        let view = cx.new(|cx| 
            ScriptListApp::new(config_for_app, bun_available, window, cx)
        );
        // Store entity for external access
        *app_entity_holder.lock().unwrap() = Some(view.clone());
        // Wrap in Root for gpui-component
        cx.new(|cx| Root::new(view, window, cx))
    },
).unwrap();

// 5. Extract the app entity
let app_entity = app_entity_holder.lock().unwrap().clone()
    .expect("App entity should be set");

// 6. Set initial focus
window.update(cx, |_root, win, root_cx| {
    app_entity.update(root_cx, |view, ctx| {
        let focus_handle = view.focus_handle(ctx);
        win.focus(&focus_handle, ctx);
    });
}).unwrap();

// 7. Register with WindowManager
window_manager::find_and_register_main_window();

// 8. Swizzle GPUI's BlurredView for vibrancy
platform::swizzle_gpui_blurred_view();
```

---

## Window Positioning Strategy

### Eye-Line Position

The window appears at "eye-line" - approximately 1/3 from the top of the screen:

```rust
pub fn calculate_eye_line_bounds_on_mouse_display(size: Size<Pixels>) -> Bounds<Pixels> {
    // Find display containing mouse
    let mouse_pos = get_mouse_position();
    let display = find_display_containing(mouse_pos);
    
    // Get visible bounds (excludes menu bar, dock)
    let visible = display.visible_bounds();
    
    // Center horizontally
    let x = visible.center().x - size.width / 2.0;
    
    // Position at eye-line (1/3 from top)
    let y = visible.origin.y + visible.size.height * 0.33 - size.height / 2.0;
    
    Bounds { origin: point(x, y), size }
}
```

### Saved Position Fallback

```rust
let bounds = window_state::get_initial_bounds(
    window_state::WindowRole::Main,
    default_bounds,  // eye-line fallback
    &displays,       // validate against current displays
);
```

If a saved position exists and is still valid (on a connected display), use it. Otherwise, fall back to eye-line.

---

## Window Options Breakdown

```rust
WindowOptions {
    // Position and size
    window_bounds: Some(WindowBounds::Windowed(bounds)),
    
    // No title bar (custom UI)
    titlebar: None,
    
    // User can drag the window
    is_movable: true,
    
    // Vibrancy or opaque based on theme
    window_background,
    
    // CRITICAL: Start hidden, show on hotkey
    show: false,
    focus: false,
    
    ..Default::default()
}
```

---

## ScriptListApp Initialization

```rust
// In ScriptListApp::new()
impl ScriptListApp {
    pub fn new(
        config: Config,
        bun_available: bool,
        window: &mut Window,
        cx: &mut Context<Self>
    ) -> Self {
        // Load scripts and scriptlets
        let scripts = scripts::read_scripts();
        let scriptlets = scripts::read_scriptlets();
        
        // Load theme
        let theme = theme::load_theme();
        
        // Load frecency data
        let frecency_store = FrecencyStore::load();
        
        // Get builtin entries
        let builtin_entries = builtins::get_builtin_entries();
        
        // Spawn background app scanning
        cx.spawn(async move |cx| {
            let apps = app_launcher::scan_applications().await;
            // Update app list
        }).detach();
        
        // Spawn cursor blink timer
        cx.spawn(async move |cx| {
            loop {
                Timer::after(Duration::from_millis(530)).await;
                // Toggle cursor visibility
            }
        }).detach();
        
        // Create input state
        let gpui_input_state = cx.new(|_| InputState::new());
        
        // ... initialize all fields
        
        Self { /* ... */ }
    }
}
```

---

## Root Wrapper Pattern

GPUI requires a `Root` wrapper for gpui-component integration:

```rust
// View hierarchy:
// WindowHandle<Root>
//     └── Root
//           └── ScriptListApp

// Why this pattern?
// - Root provides theme context for gpui-component widgets
// - Root manages notification list rendering
// - ScriptListApp is the actual app logic

// Creating the hierarchy:
|window, cx| {
    let view = cx.new(|cx| ScriptListApp::new(...));
    cx.new(|cx| Root::new(view, window, cx))
}

// Accessing ScriptListApp later:
window.update(cx, |_root, win, root_cx| {
    app_entity.update(root_cx, |view, ctx| {
        // view is ScriptListApp
    });
});
```

---

## Vibrancy Configuration

### Initial Background

```rust
let window_background = if initial_theme.is_vibrancy_enabled() {
    WindowBackgroundAppearance::Blurred  // Native macOS blur
} else {
    WindowBackgroundAppearance::Opaque   // Solid background
};
```

### BlurredView Swizzle (CRITICAL)

```rust
platform::swizzle_gpui_blurred_view();
```

**Why?** GPUI hides macOS's native `CAChameleonLayer` (vibrancy tint). By swizzling the `updateLayer` method, we preserve native vibrancy appearance like Raycast/Spotlight.

See `expert-bundles/vibrancy-*.md` for the full vibrancy investigation.

---

## Window Manager Registration

```rust
window_manager::find_and_register_main_window();
```

This scans NSWindows to find our main window (by expected size ~750x500) and stores a reference for later operations like:
- Moving to specific display
- Showing/hiding
- Configuring as floating panel

---

## Why Window Starts Hidden

```rust
show: false,    // Start hidden
focus: false,   // Don't focus on creation
```

**Rationale:**
1. **Launcher UX** - App should be invisible until summoned
2. **No Dock flash** - Prevents brief appearance in Dock
3. **Hotkey ready** - User expects Cmd+; to show the window

---

## Panel Configuration (First Show)

Panel configuration is deferred until first show:

```rust
// In show_main_window_helper():
if !PANEL_CONFIGURED.load(Ordering::SeqCst) {
    platform::configure_as_floating_panel();
    platform::swizzle_gpui_blurred_view();
    platform::configure_window_vibrancy_material();
    PANEL_CONFIGURED.store(true, Ordering::SeqCst);
}
```

**Why defer?**
- Some panel APIs require the window to exist
- Reduces startup time (not needed until first show)

---

## Performance: ScriptListApp::new()

| Operation | Duration |
|-----------|----------|
| read_scripts() (331 scripts) | ~5ms |
| read_scriptlets() | ~5ms |
| load_theme() | <1ms |
| FrecencyStore::load() | ~1ms |
| get_builtin_entries() | <1ms |
| Background app scan spawn | <1ms (async) |
| **Total** | ~12ms |

The window appears immediately; app scanning happens in background.

---

## Script Loading Details

```rust
pub fn read_scripts() -> Vec<Arc<Script>> {
    let scripts_dir = get_kit_path()
        .join("kit")
        .join("main")
        .join("scripts");
    
    // Walk directory for .ts and .js files
    // Parse metadata from each file
    // Return Arc-wrapped for cheap cloning
}
```

Scripts are wrapped in `Arc<Script>` for efficient sharing across filter operations.

---

## Review Request

Please analyze the code above and provide:

1. **Critical Issues** - Bugs, race conditions, or architectural problems
2. **Performance Concerns** - Bottlenecks, memory leaks, or inefficiencies
3. **API Design Feedback** - Better patterns or abstractions
4. **Simplification Opportunities** - Over-engineering or unnecessary complexity
5. **Specific Recommendations** - Concrete code changes with examples

Focus on **actionable feedback** rather than general observations.
