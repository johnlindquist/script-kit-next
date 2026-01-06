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

## Bundle: Phase 3 - GPUI Application Startup

This bundle covers the GPUI event loop initialization and early app configuration.

---

## Phase 3 Sequence (main.rs lines 1415-1452)

```rust
Application::new().run(move |cx: &mut App| {
    logging::log("APP", "GPUI Application starting");

    // 1. Configure as accessory app FIRST
    // This must happen before any windows are created
    platform::configure_as_accessory_app();

    // 2. Start frontmost app tracker
    #[cfg(target_os = "macos")]
    frontmost_app_tracker::start_tracking();

    // 3. Register bundled fonts
    register_bundled_fonts(cx);

    // 4. Initialize gpui-component
    gpui_component::init(cx);

    // 5. Sync theme with gpui-component
    theme::sync_gpui_component_theme(cx);

    // 6. Initialize tray icon
    let tray_manager = TrayManager::new()?;

    // ... continues to Phase 4 (window creation)
});
```

---

## Component Details

### 1. Accessory App Configuration (CRITICAL)

```rust
pub fn configure_as_accessory_app() {
    #[cfg(target_os = "macos")]
    unsafe {
        use cocoa::appkit::NSApplication;
        use cocoa::appkit::NSApplicationActivationPolicy;

        let app: id = NSApp();
        let _: () = msg_send![app, 
            setActivationPolicy: NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory
        ];
    }
}
```

**What this does:**
- No Dock icon
- No menu bar ownership
- App runs in background until summoned

**Why FIRST?** If any window is created before this, the app will briefly appear in the Dock, causing visual flicker.

---

### 2. Frontmost App Tracker

```rust
#[cfg(target_os = "macos")]
frontmost_app_tracker::start_tracking();
```

This background observer:
- Tracks which app is currently active
- Pre-fetches menu bar items for menu bar integration
- Must start after `configure_as_accessory_app()` so we're correctly classified

---

### 3. Font Registration

```rust
fn register_bundled_fonts(cx: &mut App) {
    // Fonts embedded at compile time
    static JETBRAINS_MONO_REGULAR: &[u8] =
        include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
    static JETBRAINS_MONO_BOLD: &[u8] = 
        include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf");
    // ... 4 more styles

    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(JETBRAINS_MONO_REGULAR),
        Cow::Borrowed(JETBRAINS_MONO_BOLD),
        // ...
    ];

    cx.text_system().add_fonts(fonts)?;
}
```

**Why embedded?**
- Guarantees font availability
- No file system dependency
- Consistent rendering across machines

---

### 4. gpui-component Initialization

```rust
gpui_component::init(cx);
```

This initializes the gpui-component library which provides:
- Input components
- Notification system
- Root wrapper for theme context
- Modal/popover system

**Must be called before opening any windows** that use gpui-component widgets.

---

### 5. Theme Sync

```rust
theme::sync_gpui_component_theme(cx);
```

Syncs Script Kit's custom theme system with gpui-component's `ThemeColor` system:

```rust
pub fn sync_gpui_component_theme(cx: &mut App) {
    // Load Script Kit theme
    let theme = load_theme();
    
    // Convert to gpui-component colors
    let gpui_theme = convert_to_gpui_component_theme(&theme);
    
    // Apply globally
    gpui_component::set_theme(gpui_theme, cx);
}
```

---

### 6. Tray Manager

```rust
let tray_manager = match TrayManager::new() {
    Ok(tm) => {
        logging::log("TRAY", "Tray icon initialized successfully");
        Some(tm)
    }
    Err(e) => {
        logging::log("TRAY", &format!("Failed to initialize tray icon: {}", e));
        None
    }
};
```

The tray manager:
- Renders the Script Kit icon in the system tray
- Creates a context menu with actions (Show, Notes, AI, Quit)
- Provides an alternative entry point if hotkey fails

**Tray Menu Actions:**
```rust
pub enum TrayMenuAction {
    Show,           // Show main window
    OpenNotes,      // Open Notes window
    OpenAI,         // Open AI chat window
    Settings,       // Open settings
    Quit,           // Quit application
}
```

---

## Initialization Order Rationale

```
configure_as_accessory_app()  ← Must be FIRST (before windows)
         │
         ▼
frontmost_app_tracker()       ← Needs correct activation policy
         │
         ▼
register_bundled_fonts()      ← Before any text rendering
         │
         ▼
gpui_component::init()        ← Before windows using components
         │
         ▼
sync_gpui_component_theme()   ← After gpui_component init
         │
         ▼
TrayManager::new()            ← After all core init complete
```

---

## macOS-Specific Considerations

### Activation Policy

```rust
NSApplicationActivationPolicyAccessory
```

This is equivalent to `LSUIElement=true` in Info.plist but set programmatically.

**Behaviors:**
- App doesn't appear in Cmd+Tab switcher by default
- No automatic menu bar
- Must explicitly activate windows

### NSApp() Timing

The `NSApp()` function returns the global `NSApplication` instance. It MUST be called after `Application::new()` creates the Cocoa application.

---

## Error Recovery

Each component can fail independently:

```rust
// Tray failure doesn't crash the app
let tray_manager = match TrayManager::new() {
    Ok(tm) => Some(tm),
    Err(e) => {
        // Log but continue
        logging::log("TRAY", &format!("Failed: {}", e));
        None  // App works without tray
    }
};
```

Later, if both tray AND hotkey fail, a fallback shows the window at startup.

---

## Performance Notes

All Phase 3 operations are fast (<10ms total):

| Operation | Duration |
|-----------|----------|
| configure_as_accessory_app | <1ms |
| frontmost_app_tracker start | <1ms |
| Font registration | ~5ms (6 font files) |
| gpui_component init | <1ms |
| Theme sync | <1ms |
| Tray init | ~2ms |

---

## Review Request

Please analyze the code above and provide:

1. **Critical Issues** - Bugs, race conditions, or architectural problems
2. **Performance Concerns** - Bottlenecks, memory leaks, or inefficiencies
3. **API Design Feedback** - Better patterns or abstractions
4. **Simplification Opportunities** - Over-engineering or unnecessary complexity
5. **Specific Recommendations** - Concrete code changes with examples

Focus on **actionable feedback** rather than general observations.
