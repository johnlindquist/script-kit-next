// ============================================================================
// Actions Popup Window Configuration
// ============================================================================

/// Configure the actions popup window as a non-movable child window with vibrancy.
///
/// This function configures a popup window with:
/// - isMovable = false - prevents window dragging
/// - isMovableByWindowBackground = false - prevents dragging by clicking background
/// - Same window level as main window (NSFloatingWindowLevel = 3)
/// - hidesOnDeactivate = true - auto-hides when app loses focus
/// - hasShadow = true - shadow for depth perception
/// - Disabled restoration - no position caching
/// - animationBehavior = NSWindowAnimationBehaviorNone - no animation on close
/// - Appearance-aware vibrancy (VibrantDark/VibrantLight on views, window appearance nil) + POPOVER material for frosted glass effect
///
/// # Arguments
/// * `window` - The NSWindow pointer to configure
/// * `is_dark` - Whether to use dark vibrancy (true) or light vibrancy (false)
///
/// # Safety
///
/// - `window` must be a valid, non-null NSWindow pointer obtained from GPUI
///   window creation. The pointer is checked for null at entry.
/// - Must be called on the main thread (all AppKit property setters require it).
/// - NSAppearance pointers are nil-checked before use.
/// - Content view is nil-checked before recursing into visual effect views.
#[cfg(target_os = "macos")]
pub unsafe fn configure_actions_popup_window(window: id, is_dark: bool) {
    if window.is_null() {
        logging::log(
            "ACTIONS",
            "WARNING: Cannot configure null window as actions popup",
        );
        return;
    }

    // Disable window dragging
    let _: () = msg_send![window, setMovable: false];
    let _: () = msg_send![window, setMovableByWindowBackground: false];

    // Match main window level (NSFloatingWindowLevel = 3)
    let _: () = msg_send![window, setLevel: NS_FLOATING_WINDOW_LEVEL];

    // NOTE: We intentionally do NOT set setHidesOnDeactivate:true here.
    // The main window is a non-activating panel (WindowKind::PopUp), so the app
    // is never "active" in the macOS sense. If we set hidesOnDeactivate, the
    // actions popup would immediately hide since the app isn't active.
    // Instead, we manage visibility ourselves via close_actions_window().

    // Enable shadow - Raycast/Spotlight use shadow for depth perception
    let _: () = msg_send![window, setHasShadow: true];

    // Disable close animation (NSWindowAnimationBehaviorNone = 2)
    // This prevents the white flash on dismiss
    let _: () = msg_send![window, setAnimationBehavior: 2i64];

    // Disable restoration
    let _: () = msg_send![window, setRestorable: false];

    // Disable frame autosave
    let empty_string: id = msg_send![class!(NSString), string];
    let _: () = msg_send![window, setFrameAutosaveName: empty_string];

    // ═══════════════════════════════════════════════════════════════════════════
    // VIBRANCY CONFIGURATION - Match main window settings for consistent blur
    // ═══════════════════════════════════════════════════════════════════════════

    // Clear window appearance so GPUI can detect system appearance changes.
    // Appearance is set on individual NSVisualEffectViews instead.
    let _: () = msg_send![window, setAppearance: nil];
    logging::log(
        "ACTIONS",
        &format!(
            "Actions popup: Cleared window appearance (nil) for {} mode; appearance set on views",
            if is_dark { "dark" } else { "light" }
        ),
    );

    // Use windowBackgroundColor for semi-opaque background (reduces excessive transparency)
    // This matches the main window pattern and provides the native ~1px border
    let window_bg_color: id = msg_send![class!(NSColor), windowBackgroundColor];
    let _: () = msg_send![window, setBackgroundColor: window_bg_color];
    logging::log(
        "ACTIONS",
        "Actions popup: Set backgroundColor to windowBackgroundColor (semi-opaque)",
    );

    // Mark window as non-opaque to allow transparency/vibrancy
    let _: () = msg_send![window, setOpaque: false];

    // Configure NSVisualEffectViews in the window hierarchy
    let content_view: id = msg_send![window, contentView];
    if !content_view.is_null() {
        let mut count = 0;
        configure_visual_effect_views_recursive(content_view, &mut count, is_dark);
        let material_name = if is_dark { "HUD_WINDOW" } else { "POPOVER" };
        logging::log(
            "ACTIONS",
            &format!(
                "Configured {} NSVisualEffectView(s) in actions popup with {} material",
                count, material_name
            ),
        );
    }

    let appearance_name = if is_dark {
        "VibrantDark"
    } else {
        "VibrantLight"
    };
    let material_name = if is_dark { "HUD_WINDOW" } else { "POPOVER" };
    logging::log(
        "ACTIONS",
        &format!(
            "Configured actions popup window (non-movable, vibrancy, {}, {})",
            appearance_name, material_name
        ),
    );
}

#[cfg(not(target_os = "macos"))]
pub fn configure_actions_popup_window(_window: *mut std::ffi::c_void, _is_dark: bool) {
    // No-op on non-macOS platforms
}

// ============================================================================
// Secondary Window Vibrancy Configuration
// ============================================================================

/// Configure vibrancy for a secondary window (Notes, AI, etc.)
///
/// This applies the same VibrantDark appearance and NSVisualEffectView configuration
/// that the main window and actions popup use, ensuring consistent blur effect
/// across all Script Kit windows.
///
/// # Arguments
/// * `window` - The NSWindow pointer to configure
/// * `window_name` - Name for logging (e.g., "Notes", "AI")
/// * `is_dark` - Whether to use dark vibrancy (true) or light vibrancy (false)
///
/// # Safety
///
/// - `window` must be a valid, non-null NSWindow pointer obtained from GPUI
///   window creation. The pointer is checked for null at entry.
/// - Must be called on the main thread (all AppKit property setters require it).
/// - NSAppearance and content view pointers are nil-checked before use.
#[cfg(target_os = "macos")]
pub unsafe fn configure_secondary_window_vibrancy(window: id, window_name: &str, is_dark: bool) {
    if window.is_null() {
        logging::log(
            "PANEL",
            &format!(
                "WARNING: Cannot configure null window for {} vibrancy",
                window_name
            ),
        );
        return;
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // VIBRANCY CONFIGURATION - Match main window settings for consistent blur
    // ═══════════════════════════════════════════════════════════════════════════

    // Clear window appearance so GPUI can detect system appearance changes.
    // Appearance is set on individual NSVisualEffectViews instead.
    let _: () = msg_send![window, setAppearance: nil];
    logging::log(
        "PANEL",
        &format!(
            "{} window: Cleared window appearance (nil) for {} mode; appearance set on views",
            window_name,
            if is_dark { "dark" } else { "light" }
        ),
    );

    // Use windowBackgroundColor for semi-opaque background (reduces excessive transparency)
    // This matches the main window pattern and provides the native ~1px border
    let window_bg_color: id = msg_send![class!(NSColor), windowBackgroundColor];
    let _: () = msg_send![window, setBackgroundColor: window_bg_color];
    logging::log(
        "PANEL",
        &format!(
            "{} window: Set backgroundColor to windowBackgroundColor (semi-opaque)",
            window_name
        ),
    );

    // Mark window as non-opaque to allow transparency/vibrancy
    let _: () = msg_send![window, setOpaque: false];

    // Enable shadow for native depth perception
    let _: () = msg_send![window, setHasShadow: true];

    // Configure NSVisualEffectViews in the window hierarchy
    let content_view: id = msg_send![window, contentView];
    if !content_view.is_null() {
        let mut count = 0;
        configure_visual_effect_views_recursive(content_view, &mut count, is_dark);
        let material_name = if is_dark { "HUD_WINDOW" } else { "POPOVER" };
        logging::log(
            "PANEL",
            &format!(
                "{} window: Configured {} NSVisualEffectView(s) with {} material",
                window_name, count, material_name
            ),
        );
    }

    let appearance_name = if is_dark {
        "VibrantDark"
    } else {
        "VibrantLight"
    };
    let material_name = if is_dark { "HUD_WINDOW" } else { "POPOVER" };
    logging::log(
        "PANEL",
        &format!(
            "{} window vibrancy configured ({} + {} + blur)",
            window_name, appearance_name, material_name
        ),
    );
}

#[cfg(not(target_os = "macos"))]
pub fn configure_secondary_window_vibrancy(
    _window: *mut std::ffi::c_void,
    _window_name: &str,
    _is_dark: bool,
) {
    // No-op on non-macOS platforms
}

/// Update appearance for all secondary windows (Notes, AI, Actions) when system appearance changes.
/// This ensures consistency across all windows when user toggles light/dark mode.
///
/// # Arguments
/// * `is_dark` - true for dark mode (VibrantDark), false for light mode (VibrantLight)
///
/// # Safety
/// - Must be called on the main thread
/// - Uses Objective-C runtime to enumerate and update windows
#[cfg(target_os = "macos")]
#[allow(dead_code)] // TODO: Will be used in appearance change handler (Fix 2)
pub fn update_all_secondary_windows_appearance(is_dark: bool) {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    // SAFETY: NSApplication.sharedApplication is always valid after app launch.
    // We iterate windows with count-bounded indices. Each window, title, and
    // UTF8String pointer is checked for nil/null before use.
    // Window appearance set to nil; appearance applied to NSVisualEffectViews instead.
    unsafe {
        let app: id = msg_send![class!(NSApplication), sharedApplication];
        let windows: id = msg_send![app, windows];
        if windows.is_null() {
            return;
        }
        let count: usize = msg_send![windows, count];

        logging::log(
            "APPEARANCE",
            &format!("Updating {} windows to is_dark={}", count, is_dark),
        );

        for i in 0..count {
            let window: id = msg_send![windows, objectAtIndex: i];
            if window == nil {
                continue;
            }

            // Get window title to identify secondary windows
            let title: id = msg_send![window, title];
            if title == nil {
                continue;
            }

            let title_str: *const std::os::raw::c_char = msg_send![title, UTF8String];
            if title_str.is_null() {
                continue;
            }

            let title_string = std::ffi::CStr::from_ptr(title_str)
                .to_string_lossy()
                .to_string();

            // Match secondary window titles
            if title_string.contains("Script Kit AI")
                || title_string.contains("Script Kit Notes")
                || title_string.contains("Actions")
            {
                // Clear window appearance so GPUI can detect system appearance changes.
                // Set appearance on individual NSVisualEffectViews instead.
                let _: () = msg_send![window, setAppearance: nil];

                // Walk view hierarchy and set appearance + material on each NSVisualEffectView
                let content_view: id = msg_send![window, contentView];
                if content_view != nil {
                    let mut vev_count = 0;
                    configure_visual_effect_views_recursive(content_view, &mut vev_count, is_dark);
                    logging::log(
                        "APPEARANCE",
                        &format!(
                            "Updated window '{}': cleared window appearance, configured {} NSVisualEffectView(s) for {}",
                            title_string,
                            vev_count,
                            if is_dark { "dark" } else { "light" }
                        ),
                    );
                }
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn update_all_secondary_windows_appearance(_is_dark: bool) {
    // No-op on non-macOS platforms
}

// ============================================================================
// Mouse Position
// ============================================================================

/// Get the current global mouse cursor position using macOS Core Graphics API.
/// Returns the position in global display coordinates (top-left origin, Y increases down).
///
/// # Implementation Note
/// We use `CGEventCreate(NULL)` directly via FFI because the Rust core-graphics crate's
/// `CGEvent::new(source)` creates a null-type event with undefined location. According to
/// Apple's documentation, when `CGEventCreate` is passed NULL and then `CGEventGetLocation`
/// is called, it returns the CURRENT mouse position. This is the canonical way to get
/// mouse position in Core Graphics.
#[cfg(target_os = "macos")]
pub fn get_global_mouse_position() -> Option<(f64, f64)> {
    use core_foundation::base::CFRelease;
    use core_graphics::geometry::CGPoint;
    use std::ffi::c_void;

    // FFI declarations for direct CGEventCreate(NULL) call
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventCreate(source: *const c_void) -> *const c_void;
        fn CGEventGetLocation(event: *const c_void) -> CGPoint;
    }

    // SAFETY: CGEventCreate(NULL) is the documented way to get the current
    // mouse position. The returned event pointer is nil-checked before use.
    // CFRelease is called to free the event, preventing a memory leak.
    // CGEventGetLocation returns a value type (CGPoint), no pointer issues.
    unsafe {
        // CGEventCreate(NULL) returns an event that, when queried for location,
        // returns the current mouse cursor position
        let event = CGEventCreate(std::ptr::null());
        if event.is_null() {
            logging::log("POSITION", "WARNING: CGEventCreate returned null");
            return None;
        }

        let location = CGEventGetLocation(event);

        // Release the event to avoid memory leak
        CFRelease(event);

        Some((location.x, location.y))
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_global_mouse_position() -> Option<(f64, f64)> {
    // TODO: Implement for other platforms
    None
}

// ============================================================================
// Display Information
// ============================================================================

#[cfg(target_os = "macos")]
use cocoa::foundation::NSRect;

/// Get the height of the primary (main) screen for coordinate conversion.
/// macOS uses bottom-left origin; we convert to top-left origin.
#[cfg(target_os = "macos")]
pub fn primary_screen_height() -> Option<f64> {
    if require_main_thread("primary_screen_height") {
        return None;
    }
    // SAFETY: Main thread verified. NSScreen.mainScreen is a class method
    // that returns the primary screen. Nil checked before accessing frame.
    unsafe {
        let main_screen: id = msg_send![class!(NSScreen), mainScreen];
        if main_screen == nil {
            return None;
        }
        let frame: NSRect = msg_send![main_screen, frame];
        Some(frame.size.height)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn primary_screen_height() -> Option<f64> {
    // Fallback for non-macOS
    Some(1080.0)
}

/// Convert Y coordinate from top-left origin (y increases down) to
/// AppKit bottom-left origin (y increases up).
/// Same formula both directions (mirror transform).
#[allow(dead_code)]
pub fn flip_y(primary_height: f64, y: f64, height: f64) -> f64 {
    primary_height - y - height
}

/// Get all displays with their actual bounds in macOS global coordinates.
/// This uses NSScreen directly because GPUI's display.bounds() doesn't return
/// correct origins for secondary displays.
#[cfg(target_os = "macos")]
pub fn get_macos_displays() -> Vec<DisplayBounds> {
    if require_main_thread("get_macos_displays") {
        return Vec::new();
    }
    // SAFETY: Main thread verified. NSScreen.screens is a class method.
    // We check mainScreen for nil. Array iteration uses count-bounded indices.
    unsafe {
        let screens: id = msg_send![class!(NSScreen), screens];
        if screens.is_null() {
            return Vec::new();
        }
        let count: usize = msg_send![screens, count];

        // Get primary screen height for coordinate flipping
        // macOS coordinates: Y=0 at bottom of primary screen
        // CRITICAL: Use mainScreen, not firstObject - they can differ when display arrangement changes
        let main_screen: id = msg_send![class!(NSScreen), mainScreen];
        let main_screen = if main_screen == nil {
            // Fallback to firstObject if mainScreen is nil (shouldn't happen but be safe)
            logging::log(
                "POSITION",
                "WARNING: mainScreen returned nil, falling back to firstObject",
            );
            let fallback: id = msg_send![screens, firstObject];
            if fallback.is_null() {
                return Vec::new();
            }
            fallback
        } else {
            main_screen
        };
        let main_frame: NSRect = msg_send![main_screen, frame];
        let primary_height = main_frame.size.height;

        let mut displays = Vec::with_capacity(count);

        for i in 0..count {
            let screen: id = msg_send![screens, objectAtIndex:i];
            let frame: NSRect = msg_send![screen, frame];

            // Convert from macOS bottom-left origin to top-left origin
            // macOS: y=0 at bottom, increasing upward
            // We want: y=0 at top, increasing downward
            let flipped_y = primary_height - frame.origin.y - frame.size.height;

            displays.push(DisplayBounds {
                origin_x: frame.origin.x,
                origin_y: flipped_y,
                width: frame.size.width,
                height: frame.size.height,
            });
        }

        displays
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_macos_displays() -> Vec<DisplayBounds> {
    // Fallback: return a single default display
    vec![DisplayBounds {
        origin_x: 0.0,
        origin_y: 0.0,
        width: 1920.0,
        height: 1080.0,
    }]
}

