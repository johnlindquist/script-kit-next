//! Platform-specific window configuration abstraction.
//!
//! This module provides cross-platform abstractions for window behavior configuration,
//! with macOS-specific implementations for floating panel behavior and space management.
//!
//! # macOS Behavior
//!
//! On macOS, this module configures windows as floating panels that:
//! - Float above normal windows (NSFloatingWindowLevel = 3)
//! - Move to the active space when shown (NSWindowCollectionBehaviorMoveToActiveSpace = 2)
//! - Disable window state restoration to prevent position caching
//!
//! # Other Platforms
//!
//! On non-macOS platforms, these functions are no-ops, allowing cross-platform code
//! to call them without conditional compilation at the call site.

use crate::logging;

#[cfg(target_os = "macos")]
use cocoa::appkit::NSApp;
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
use crate::window_manager;

// ============================================================================
// Thread Safety
// ============================================================================

/// Assert that the current thread is the main thread.
///
/// AppKit APIs (NSApp, NSWindow, NSScreen, etc.) are NOT thread-safe and MUST
/// be called from the main thread. This function provides a cheap debug assertion
/// that will panic in debug builds if called from a background thread.
///
/// # Panics (debug builds only)
///
/// Panics if called from a thread other than the main thread.
///
/// # Safety
///
/// Uses Objective-C message sending to query NSThread.isMainThread.
#[cfg(target_os = "macos")]
fn debug_assert_main_thread() {
    unsafe {
        let is_main: bool = msg_send![class!(NSThread), isMainThread];
        debug_assert!(
            is_main,
            "AppKit calls must run on the main thread. \
             This function was called from a background thread, which can cause \
             crashes, undefined behavior, or silent failures."
        );
    }
}

// ============================================================================
// Application Activation Policy
// ============================================================================

/// Configure the app as an "accessory" application.
///
/// This is equivalent to setting `LSUIElement=true` in Info.plist, but done at runtime.
/// Accessory apps:
/// - Do NOT appear in the Dock
/// - Do NOT take menu bar ownership when activated
/// - Can still show windows that float above other apps
///
/// This is critical for window management actions (tile, maximize, etc.) because
/// it allows us to query `menuBarOwningApplication` to find the previously active app.
///
/// # macOS Behavior
///
/// Sets NSApplicationActivationPolicyAccessory (value = 1) on the app.
/// Must be called early in app startup, before any windows are shown.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
pub fn configure_as_accessory_app() {
    debug_assert_main_thread();
    unsafe {
        let app: id = NSApp();
        // NSApplicationActivationPolicyAccessory = 1
        // This makes the app not appear in Dock and not take menu bar ownership
        let _: () = msg_send![app, setActivationPolicy: 1i64];
        logging::log(
            "PANEL",
            "Configured app as accessory (no Dock icon, no menu bar ownership)",
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_as_accessory_app() {
    // No-op on non-macOS platforms
}

// ============================================================================
// Space Management
// ============================================================================

/// Ensure the main window moves to the currently active macOS space when shown.
///
/// This function sets NSWindowCollectionBehaviorMoveToActiveSpace on the main window,
/// which causes it to move to whichever space is currently active when the window
/// becomes visible, rather than forcing the user back to the space where the window
/// was last shown.
///
/// # macOS Behavior
///
/// Uses the WindowManager to get the main window (not keyWindow, which may not exist
/// yet during app startup) and sets the collection behavior.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
///
/// # Safety
///
/// Uses Objective-C message sending internally on macOS.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn ensure_move_to_active_space() {
    debug_assert_main_thread();
    unsafe {
        // Use WindowManager to get the main window (not keyWindow, which may not exist yet)
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "PANEL",
                    "WARNING: Main window not registered, cannot set MoveToActiveSpace",
                );
                return;
            }
        };

        // Get current collection behavior to preserve existing flags
        let current: u64 = msg_send![window, collectionBehavior];

        // OR in our desired flags:
        // - MoveToActiveSpace: window moves to current space when shown
        // - FullScreenAuxiliary: window can show over fullscreen apps without disrupting
        let desired = current
            | NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE
            | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY;

        let _: () = msg_send![window, setCollectionBehavior:desired];

        logging::log(
            "PANEL",
            &format!(
                "Set collection behavior: {} -> {} (MoveToActiveSpace + FullScreenAuxiliary)",
                current, desired
            ),
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn ensure_move_to_active_space() {
    // No-op on non-macOS platforms
}

// ============================================================================
// Floating Panel Configuration
// ============================================================================

/// Configure the main window as a floating macOS panel.
///
/// This function configures the main window (via WindowManager) with:
/// - Floating window level (NSFloatingWindowLevel = 3) - appears above normal windows
/// - MoveToActiveSpace collection behavior - moves to current space when shown
/// - Disabled window restoration - prevents macOS from remembering window position
/// - Empty frame autosave name - prevents position caching
///
/// # macOS Behavior
///
/// Uses WindowManager to get the main window (more reliable than NSApp.keyWindow,
/// which is timing-sensitive and can return nil during startup or the wrong window
/// in multi-window scenarios). If no main window is registered, logs a warning.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
///
/// # Safety
///
/// Uses Objective-C message sending internally on macOS.
///
#[cfg(target_os = "macos")]
pub fn configure_as_floating_panel() {
    debug_assert_main_thread();
    unsafe {
        // Use WindowManager to get the main window (more reliable than keyWindow)
        // keyWindow is timing-sensitive and can return nil during startup,
        // or the wrong window (Notes/AI) in multi-window scenarios.
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "PANEL",
                    "WARNING: Main window not registered, cannot configure as floating panel",
                );
                return;
            }
        };

        // NSFloatingWindowLevel = 3
        // This makes the window float above normal windows
        // Use i64 (NSInteger) for proper ABI compatibility on 64-bit macOS
        let _: () = msg_send![window, setLevel:NS_FLOATING_WINDOW_LEVEL];

        // Get current collection behavior to preserve existing flags set by GPUI/AppKit
        let current: u64 = msg_send![window, collectionBehavior];

        // OR in our desired flags instead of replacing:
        // - MoveToActiveSpace: window moves to current space when shown
        // - FullScreenAuxiliary: window can show over fullscreen apps without disrupting
        let desired = current
            | NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE
            | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY;

        let _: () = msg_send![window, setCollectionBehavior:desired];

        // CRITICAL: Disable macOS window state restoration
        // This prevents macOS from remembering and restoring the window position
        // when the app is relaunched or the window is shown again
        let _: () = msg_send![window, setRestorable:false];

        // Also disable the window's autosave frame name which can cause position caching
        let empty_string: id = msg_send![class!(NSString), string];
        let _: () = msg_send![window, setFrameAutosaveName:empty_string];

        logging::log(
            "PANEL",
            &format!(
                "Configured window as floating panel (level={}, behavior={}->{}, restorable=false)",
                NS_FLOATING_WINDOW_LEVEL, current, desired
            ),
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_as_floating_panel() {
    // No-op on non-macOS platforms
}

// ============================================================================
// Main Window Visibility Control
// ============================================================================

/// Hide the main window without hiding the entire app.
///
/// This is used when opening secondary windows (Notes, AI) to ensure the main
/// window stays hidden while the secondary window is shown. Unlike cx.hide(),
/// this doesn't hide all windows - only the main window.
///
/// # macOS Behavior
///
/// Uses NSWindow orderOut: to remove the main window from the screen without
/// affecting other windows. The window is not minimized, just hidden.
///
/// # Other Platforms
///
/// No-op on non-macOS platforms.
#[cfg(target_os = "macos")]
pub fn hide_main_window() {
    debug_assert_main_thread();
    unsafe {
        // Use WindowManager to get the main window
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "PANEL",
                    "hide_main_window: Main window not registered, nothing to hide",
                );
                return;
            }
        };

        // orderOut: removes the window from the screen without affecting other windows
        // nil sender means the action is programmatic, not from a menu item
        let _: () = msg_send![window, orderOut:nil];

        logging::log("PANEL", "Main window hidden via orderOut:");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn hide_main_window() {
    // No-op on non-macOS platforms
}

/// Get the current main window bounds in canonical top-left coordinates.
/// Returns (x, y, width, height) or None if window not available.
#[cfg(target_os = "macos")]
pub fn get_main_window_bounds() -> Option<(f64, f64, f64, f64)> {
    debug_assert_main_thread();
    unsafe {
        let window = window_manager::get_main_window()?;
        let frame: NSRect = msg_send![window, frame];

        // Get primary screen height for coordinate conversion
        let primary_height = primary_screen_height()?;

        // Convert from AppKit bottom-left origin to our top-left canonical space
        let top_left_y = flip_y(primary_height, frame.origin.y, frame.size.height);

        Some((
            frame.origin.x,
            top_left_y,
            frame.size.width,
            frame.size.height,
        ))
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_main_window_bounds() -> Option<(f64, f64, f64, f64)> {
    None
}

// ============================================================================
// App Active State Detection
// ============================================================================

/// Check if the application is currently active (has focus).
///
/// On macOS, this uses NSApplication's isActive property to determine
/// if our app is the frontmost app receiving keyboard events.
///
/// # Returns
/// - `true` if the app is active (user is interacting with our windows)
/// - `false` if another app is active (user clicked on another app)
///
/// # Platform Support
/// - macOS: Uses NSApplication isActive
/// - Other platforms: Always returns true (not yet implemented)
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn is_app_active() -> bool {
    debug_assert_main_thread();
    unsafe {
        let app: id = NSApp();
        let is_active: bool = msg_send![app, isActive];
        is_active
    }
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn is_app_active() -> bool {
    // TODO: Implement for other platforms
    // On non-macOS, assume always active
    true
}

/// Check if the main window is currently focused (key window).
///
/// This is used to detect focus loss even when the app remains active
/// (e.g., when switching focus to Notes/AI windows).
#[cfg(target_os = "macos")]
pub fn is_main_window_focused() -> bool {
    debug_assert_main_thread();
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(window) => window,
            None => return false,
        };

        let is_key: bool = msg_send![window, isKeyWindow];
        is_key
    }
}

#[cfg(not(target_os = "macos"))]
pub fn is_main_window_focused() -> bool {
    // TODO: Implement for other platforms
    // On non-macOS, assume focused to avoid auto-dismiss behavior.
    true
}

// ============================================================================
// Constants
// ============================================================================

/// NSFloatingWindowLevel constant value (3)
/// Windows at this level float above normal windows but below modal dialogs.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub const NS_FLOATING_WINDOW_LEVEL: i64 = 3;

/// NSWindowCollectionBehaviorMoveToActiveSpace constant value (1 << 1 = 2)
/// When set, the window moves to the currently active space when shown.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub const NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE: u64 = 1 << 1;

/// NSWindowCollectionBehaviorFullScreenAuxiliary constant value (1 << 8 = 256)
/// Allows the window to be shown over fullscreen apps without disrupting their space.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub const NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY: u64 = 1 << 8;

// ============================================================================
// Window Vibrancy Material Configuration
// ============================================================================

/// NSVisualEffectMaterial values
/// See: https://developer.apple.com/documentation/appkit/nsvisualeffectmaterial
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub mod ns_visual_effect_material {
    pub const TITLEBAR: isize = 3;
    pub const SELECTION: isize = 4; // What GPUI uses by default (colorless)
    pub const MENU: isize = 5;
    pub const POPOVER: isize = 6;
    pub const SIDEBAR: isize = 7;
    pub const HEADER_VIEW: isize = 10;
    pub const SHEET: isize = 11;
    pub const WINDOW_BACKGROUND: isize = 12;
    pub const HUD_WINDOW: isize = 13; // Dark, high contrast - good for dark UIs
    pub const FULL_SCREEN_UI: isize = 15;
    pub const TOOL_TIP: isize = 17;
    pub const CONTENT_BACKGROUND: isize = 18;
    pub const UNDER_WINDOW_BACKGROUND: isize = 21;
    pub const UNDER_PAGE_BACKGROUND: isize = 22;

    // Private/undocumented materials that Raycast might use
    // These provide more control over the appearance
    pub const DARK: isize = 2; // NSVisualEffectMaterialDark (deprecated but works)
    pub const MEDIUM_DARK: isize = 8; // Darker variant
    pub const ULTRA_DARK: isize = 9; // Darkest variant

    /// All materials to cycle through, with names for logging
    pub const ALL_MATERIALS: &[(isize, &str)] = &[
        (DARK, "Dark (2) - deprecated"),
        (TITLEBAR, "Titlebar (3)"),
        (SELECTION, "Selection (4) - GPUI default"),
        (MENU, "Menu (5)"),
        (POPOVER, "Popover (6)"),
        (SIDEBAR, "Sidebar (7)"),
        (MEDIUM_DARK, "MediumDark (8) - undocumented"),
        (ULTRA_DARK, "UltraDark (9) - undocumented"),
        (HEADER_VIEW, "HeaderView (10)"),
        (SHEET, "Sheet (11)"),
        (WINDOW_BACKGROUND, "WindowBackground (12)"),
        (HUD_WINDOW, "HudWindow (13)"),
        (FULL_SCREEN_UI, "FullScreenUI (15)"),
        (TOOL_TIP, "ToolTip (17)"),
        (CONTENT_BACKGROUND, "ContentBackground (18)"),
        (UNDER_WINDOW_BACKGROUND, "UnderWindowBackground (21)"),
        (UNDER_PAGE_BACKGROUND, "UnderPageBackground (22)"),
    ];
}

/// Current material index for cycling
#[cfg(target_os = "macos")]
static CURRENT_MATERIAL_INDEX: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// Current blending mode (0 = behindWindow, 1 = withinWindow)
#[cfg(target_os = "macos")]
static CURRENT_BLENDING_MODE: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// Current appearance index for cycling
#[cfg(target_os = "macos")]
static CURRENT_APPEARANCE_INDEX: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// All appearance options to try
#[cfg(target_os = "macos")]
const APPEARANCE_OPTIONS: &[&str] = &[
    "DarkAqua",
    "VibrantDark",
    "Aqua",
    "VibrantLight",
    "None", // No forced appearance - use system default
];

// NSAppearance name constants
#[cfg(target_os = "macos")]
#[link(name = "AppKit", kind = "framework")]
extern "C" {
    static NSAppearanceNameDarkAqua: id;
    #[allow(dead_code)]
    static NSAppearanceNameAqua: id;
    static NSAppearanceNameVibrantDark: id;
    #[allow(dead_code)]
    static NSAppearanceNameVibrantLight: id;
}

/// Swizzle GPUI's BlurredView class to preserve the CAChameleonLayer tint.
///
/// GPUI creates a custom NSVisualEffectView subclass called "BlurredView" that
/// hides the CAChameleonLayer (the native macOS tint layer) on every updateLayer call.
/// This function replaces that behavior to preserve the native tint effect.
///
/// Call this ONCE early in app startup, before any windows are created.
///
/// # Safety
///
/// Uses Objective-C runtime to replace method implementations.
#[cfg(target_os = "macos")]
static SWIZZLE_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Counter for patched_update_layer calls (for diagnostics)
#[cfg(target_os = "macos")]
static PATCHED_UPDATE_LAYER_CALLS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

#[cfg(target_os = "macos")]
pub fn swizzle_gpui_blurred_view() {
    use std::sync::atomic::Ordering;

    logging::log("VIBRANCY", "swizzle_gpui_blurred_view() called");

    // Only swizzle once
    if SWIZZLE_DONE.swap(true, Ordering::SeqCst) {
        logging::log("VIBRANCY", "Swizzle already done, skipping");
        return;
    }

    unsafe {
        // Get GPUI's BlurredView class
        let class_name = std::ffi::CString::new("BlurredView").unwrap();
        let blurred_class = objc::runtime::objc_getClass(class_name.as_ptr());

        logging::log(
            "VIBRANCY",
            &format!(
                "Looking for BlurredView class: {:?}",
                !blurred_class.is_null()
            ),
        );

        if blurred_class.is_null() {
            logging::log(
                "VIBRANCY",
                "BlurredView class not found (GPUI may not have created it yet)",
            );
            return;
        }

        // Get the updateLayer selector
        let update_layer_sel = sel!(updateLayer);

        // Get the original method
        let original_method =
            objc::runtime::class_getInstanceMethod(blurred_class as *const _, update_layer_sel);

        if original_method.is_null() {
            logging::log("VIBRANCY", "updateLayer method not found on BlurredView");
            return;
        }

        // Replace with our implementation that preserves the tint layer
        let new_imp: extern "C" fn(&objc::runtime::Object, objc::runtime::Sel) =
            patched_update_layer;
        let _ = objc::runtime::method_setImplementation(
            original_method as *mut _,
            #[allow(clippy::missing_transmute_annotations)]
            std::mem::transmute::<_, objc::runtime::Imp>(new_imp),
        );

        logging::log(
            "VIBRANCY",
            "Successfully swizzled BlurredView.updateLayer to preserve CAChameleonLayer tint!",
        );
    }
}

/// Our replacement for GPUI's updateLayer that preserves the CAChameleonLayer
#[cfg(target_os = "macos")]
extern "C" fn patched_update_layer(this: &objc::runtime::Object, _sel: objc::runtime::Sel) {
    use std::sync::atomic::Ordering;

    let call_count = PATCHED_UPDATE_LAYER_CALLS.fetch_add(1, Ordering::Relaxed);

    // Log first few calls and then periodically to confirm swizzle is active
    if call_count < 3
        || (call_count < 100 && call_count.is_multiple_of(20))
        || call_count.is_multiple_of(500)
    {
        logging::log(
            "VIBRANCY",
            &format!("patched_update_layer CALLED (count={})", call_count + 1),
        );
    }

    unsafe {
        // Call NSVisualEffectView's original updateLayer (skip GPUI's BlurredView implementation)
        // We use msg_send! with super() to call the parent class implementation
        let this_id = this as *const _ as id;
        let _: () = msg_send![super(this_id, class!(NSVisualEffectView)), updateLayer];

        // DON'T hide the CAChameleonLayer - this is the key difference from GPUI's version
        // The tint layer provides the native macOS vibrancy effect

        // On first call, log the sublayers recursively to find CAChameleonLayer
        if call_count == 0 {
            let layer: id = msg_send![this_id, layer];
            if !layer.is_null() {
                logging::log("VIBRANCY", "Inspecting BlurredView layer hierarchy:");
                dump_layer_hierarchy(layer, 0);
            }
        }

        // On second call (after window is visible), check layer state again
        if call_count == 1 {
            let layer: id = msg_send![this_id, layer];
            if !layer.is_null() {
                logging::log("VIBRANCY", "Second call - checking layer state after show:");
                dump_layer_hierarchy(layer, 0);
            }
        }
    }
}

/// Recursively dump layer hierarchy to find CAChameleonLayer
#[cfg(target_os = "macos")]
unsafe fn dump_layer_hierarchy(layer: id, depth: usize) {
    if layer.is_null() || depth > 5 {
        return;
    }

    let indent = "  ".repeat(depth);
    let class: id = msg_send![layer, class];
    let class_name: id = msg_send![class, className];
    let class_name_str: *const std::os::raw::c_char = msg_send![class_name, UTF8String];

    if !class_name_str.is_null() {
        let name = std::ffi::CStr::from_ptr(class_name_str).to_string_lossy();
        let is_hidden: bool = msg_send![layer, isHidden];
        let is_chameleon = name.contains("Chameleon");

        // Check for filters
        let filters: id = msg_send![layer, filters];
        let filter_count: usize = if !filters.is_null() {
            msg_send![filters, count]
        } else {
            0
        };

        // Check background color
        let bg_color: id = msg_send![layer, backgroundColor];
        let has_bg = !bg_color.is_null();

        logging::log(
            "VIBRANCY",
            &format!(
                "{}[d{}] {} (hidden={}, filters={}, bg={}){}",
                indent,
                depth,
                name,
                is_hidden,
                filter_count,
                has_bg,
                if is_chameleon { " <-- CHAMELEON!" } else { "" }
            ),
        );

        // Log filter names if any
        if filter_count > 0 {
            for i in 0..filter_count {
                let filter: id = msg_send![filters, objectAtIndex: i];
                let desc: id = msg_send![filter, description];
                let desc_str: *const std::os::raw::c_char = msg_send![desc, UTF8String];
                if !desc_str.is_null() {
                    let desc_s = std::ffi::CStr::from_ptr(desc_str).to_string_lossy();
                    logging::log(
                        "VIBRANCY",
                        &format!("{}  filter[{}]: {}", indent, i, desc_s),
                    );
                }
            }
        }

        // If we find CAChameleonLayer and it's hidden, unhide it!
        if is_chameleon && is_hidden {
            logging::log(
                "VIBRANCY",
                &format!("{}  -> Unhiding CAChameleonLayer!", indent),
            );
            let _: () = msg_send![layer, setHidden: false];
        }
    }

    // Recurse into sublayers
    let sublayers: id = msg_send![layer, sublayers];
    if !sublayers.is_null() {
        let count: usize = msg_send![sublayers, count];
        for i in 0..count {
            let sublayer: id = msg_send![sublayers, objectAtIndex: i];
            dump_layer_hierarchy(sublayer, depth + 1);
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn swizzle_gpui_blurred_view() {
    // No-op on non-macOS platforms
}

/// Get diagnostic info about the BlurredView swizzle status
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn get_swizzle_diagnostics() -> (bool, u64) {
    use std::sync::atomic::Ordering;
    (
        SWIZZLE_DONE.load(Ordering::Relaxed),
        PATCHED_UPDATE_LAYER_CALLS.load(Ordering::Relaxed),
    )
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn get_swizzle_diagnostics() -> (bool, u64) {
    (false, 0)
}

/// Log swizzle diagnostics - call periodically to monitor swizzle health
#[allow(dead_code)]
pub fn log_swizzle_diagnostics() {
    let (done, calls) = get_swizzle_diagnostics();
    logging::log(
        "VIBRANCY",
        &format!(
            "Swizzle diagnostics: done={}, patched_update_layer_calls={}",
            done, calls
        ),
    );
}

/// Configure the vibrancy blur for the main window to look good on ANY background.
///
/// The key insight from Raycast/Spotlight/Alfred: they force the window's appearance
/// to `NSAppearanceNameVibrantDark`, which makes NSVisualEffectView always use its
/// dark rendering path regardless of system appearance or what's behind the window.
///
/// This function:
/// 1. Forces window appearance to VibrantDark (CRITICAL - this is the main fix)
/// 2. Sets the blur material to HUD_WINDOW for a dark, high-contrast effect
/// 3. Ensures the effect is always active and uses behind-window blending
///
/// # macOS Behavior
///
/// Sets the window's NSAppearance to VibrantDark, then configures the
/// NSVisualEffectView with appropriate material and state.
///
/// # Safety
///
/// Uses Objective-C message sending internally.
#[cfg(target_os = "macos")]
pub fn configure_window_vibrancy_material() {
    debug_assert_main_thread();
    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "PANEL",
                    "WARNING: Main window not registered, cannot configure vibrancy material",
                );
                return;
            }
        };

        // Set window appearance to VibrantDark for consistent blur rendering
        // VibrantDark provides better vibrancy effects than DarkAqua - this is what
        // Raycast/Spotlight use for their blur effect
        let vibrant_dark: id = msg_send![
            class!(NSAppearance),
            appearanceNamed: NSAppearanceNameVibrantDark
        ];
        if !vibrant_dark.is_null() {
            let _: () = msg_send![window, setAppearance: vibrant_dark];
            logging::log("PANEL", "Set window appearance to VibrantDark");
        }

        // ╔════════════════════════════════════════════════════════════════════════════╗
        // ║ WINDOW BACKGROUND COLOR - DO NOT CHANGE WITHOUT TESTING                   ║
        // ╠════════════════════════════════════════════════════════════════════════════╣
        // ║ windowBackgroundColor provides the native ~1px border around the window.  ║
        // ║ Using clearColor removes the border but allows more blur.                 ║
        // ║ This setting was tested against Raycast/Spotlight appearance.             ║
        // ║ See: /Users/johnlindquist/dev/mac-panel-window/panel-window.mm           ║
        // ╚════════════════════════════════════════════════════════════════════════════╝
        let window_bg_color: id = msg_send![class!(NSColor), windowBackgroundColor];
        let _: () = msg_send![window, setBackgroundColor: window_bg_color];

        // Enable shadow for native depth perception (Raycast/Spotlight have shadows)
        let _: () = msg_send![window, setHasShadow: true];

        // Mark window as non-opaque to allow transparency/vibrancy
        let _: () = msg_send![window, setOpaque: false];

        logging::log(
            "PANEL",
            "Set window backgroundColor to windowBackgroundColor, hasShadow=true, opaque=false",
        );

        // Get the content view
        let content_view: id = msg_send![window, contentView];
        if content_view.is_null() {
            logging::log("PANEL", "WARNING: Window has no content view");
            return;
        }

        // Recursively find and configure ALL NSVisualEffectViews
        // Expert feedback: GPUI may nest effect views, so we need to walk the whole tree
        let mut count = 0;
        configure_visual_effect_views_recursive(content_view, &mut count);

        if count == 0 {
            logging::log(
                "PANEL",
                "WARNING: No NSVisualEffectView found in window hierarchy",
            );
        } else {
            logging::log(
                "PANEL",
                &format!(
                    "Configured {} NSVisualEffectView(s): VibrantDark + HUD_WINDOW + emphasized",
                    count
                ),
            );
        }
    }
}

/// Recursively walk view hierarchy and configure all NSVisualEffectViews
#[cfg(target_os = "macos")]
unsafe fn configure_visual_effect_views_recursive(view: id, count: &mut usize) {
    // Check if this view is an NSVisualEffectView
    let is_vev: bool = msg_send![view, isKindOfClass: class!(NSVisualEffectView)];
    if is_vev {
        // Log current state BEFORE configuration
        let old_material: isize = msg_send![view, material];
        let old_state: isize = msg_send![view, state];
        let old_blending: isize = msg_send![view, blendingMode];
        let old_emphasized: bool = msg_send![view, isEmphasized];

        // ╔════════════════════════════════════════════════════════════════════════════╗
        // ║ NSVISUALEFFECTVIEW SETTINGS - DO NOT CHANGE WITHOUT TESTING               ║
        // ╠════════════════════════════════════════════════════════════════════════════╣
        // ║ These settings match Electron's vibrancy:'popover' + visualEffectState:'followWindow' ║
        // ║ Combined with windowBackgroundColor + 37% tint alpha in gpui_integration.rs ║
        // ║ See: /Users/johnlindquist/dev/mac-panel-window/panel-window.mm           ║
        // ╚════════════════════════════════════════════════════════════════════════════╝
        // POPOVER (6) - matches Electron's vibrancy: 'popover'
        let _: () = msg_send![view, setMaterial: ns_visual_effect_material::POPOVER];
        // State 0 = followsWindowActiveState (matches Electron's visualEffectState: 'followWindow')
        let _: () = msg_send![view, setState: 0isize];
        // BehindWindow blending (0) - blur content behind the window
        let _: () = msg_send![view, setBlendingMode: 0isize];
        // Emphasized for more contrast
        let _: () = msg_send![view, setEmphasized: true];

        // Log state AFTER configuration
        let new_material: isize = msg_send![view, material];
        let new_state: isize = msg_send![view, state];
        let new_blending: isize = msg_send![view, blendingMode];
        let new_emphasized: bool = msg_send![view, isEmphasized];

        // Get effective appearance
        let effective_appearance: id = msg_send![view, effectiveAppearance];
        let appearance_name: id = if !effective_appearance.is_null() {
            msg_send![effective_appearance, name]
        } else {
            nil
        };
        let appearance_str = if !appearance_name.is_null() {
            let s: *const std::os::raw::c_char = msg_send![appearance_name, UTF8String];
            if !s.is_null() {
                std::ffi::CStr::from_ptr(s).to_string_lossy().to_string()
            } else {
                "nil".to_string()
            }
        } else {
            "nil".to_string()
        };

        logging::log(
            "VIBRANCY",
            &format!(
                "NSVisualEffectView config: mat {} -> {}, state {} -> {}, blend {} -> {}, emph {} -> {}, appearance={}",
                old_material, new_material,
                old_state, new_state,
                old_blending, new_blending,
                old_emphasized, new_emphasized,
                appearance_str
            ),
        );

        *count += 1;
    }

    // Recurse into subviews
    let subviews: id = msg_send![view, subviews];
    if !subviews.is_null() {
        let subview_count: usize = msg_send![subviews, count];
        for i in 0..subview_count {
            let child: id = msg_send![subviews, objectAtIndex: i];
            configure_visual_effect_views_recursive(child, count);
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_window_vibrancy_material() {
    // No-op on non-macOS platforms
}

/// Cycle through ALL vibrancy options - material, appearance, blending mode, emphasized.
/// Press Cmd+Shift+M repeatedly to find what works.
/// Returns a description of the current configuration.
#[cfg(target_os = "macos")]
pub fn cycle_vibrancy_material() -> String {
    use std::sync::atomic::Ordering;

    debug_assert_main_thread();

    let materials = ns_visual_effect_material::ALL_MATERIALS;
    let appearances = APPEARANCE_OPTIONS;

    // Get current indices
    let mat_idx = CURRENT_MATERIAL_INDEX.load(Ordering::SeqCst);
    let app_idx = CURRENT_APPEARANCE_INDEX.load(Ordering::SeqCst);
    let blend_mode = CURRENT_BLENDING_MODE.load(Ordering::SeqCst);

    // Increment material, wrap around and bump appearance when materials exhausted
    let new_mat_idx = (mat_idx + 1) % materials.len();
    CURRENT_MATERIAL_INDEX.store(new_mat_idx, Ordering::SeqCst);

    // When materials wrap, cycle appearance
    if new_mat_idx == 0 {
        let new_app_idx = (app_idx + 1) % appearances.len();
        CURRENT_APPEARANCE_INDEX.store(new_app_idx, Ordering::SeqCst);

        // When appearances wrap, toggle blending mode
        if new_app_idx == 0 {
            CURRENT_BLENDING_MODE.store(blend_mode ^ 1, Ordering::SeqCst);
        }
    }

    // Get current values after update
    let mat_idx = CURRENT_MATERIAL_INDEX.load(Ordering::SeqCst);
    let app_idx = CURRENT_APPEARANCE_INDEX.load(Ordering::SeqCst);
    let blend_mode = CURRENT_BLENDING_MODE.load(Ordering::SeqCst);

    let (material_value, material_name) = materials[mat_idx];
    let appearance_name = appearances[app_idx];
    let blend_name = if blend_mode == 0 { "Behind" } else { "Within" };

    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => return "ERROR: No main window".to_string(),
        };

        // Set window appearance
        if appearance_name != "None" {
            let appearance_id: id = match appearance_name {
                "DarkAqua" => NSAppearanceNameDarkAqua,
                "VibrantDark" => NSAppearanceNameVibrantDark,
                "Aqua" => NSAppearanceNameAqua,
                "VibrantLight" => NSAppearanceNameVibrantLight,
                _ => nil,
            };
            if !appearance_id.is_null() {
                let appearance: id =
                    msg_send![class!(NSAppearance), appearanceNamed: appearance_id];
                if !appearance.is_null() {
                    let _: () = msg_send![window, setAppearance: appearance];
                }
            }
        } else {
            // Clear appearance - use system default
            let _: () = msg_send![window, setAppearance: nil];
        }

        let content_view: id = msg_send![window, contentView];
        if content_view.is_null() {
            return "ERROR: No content view".to_string();
        }

        let subviews: id = msg_send![content_view, subviews];
        if subviews.is_null() {
            return "ERROR: No subviews".to_string();
        }

        let count: usize = msg_send![subviews, count];
        for i in 0..count {
            let subview: id = msg_send![subviews, objectAtIndex: i];
            let is_visual_effect_view: bool =
                msg_send![subview, isKindOfClass: class!(NSVisualEffectView)];

            if is_visual_effect_view {
                // Set material
                let _: () = msg_send![subview, setMaterial: material_value];

                // Set blending mode
                let _: () = msg_send![subview, setBlendingMode: blend_mode as isize];

                // Always active
                let _: () = msg_send![subview, setState: 1isize];

                // Toggle emphasized based on material index (try both)
                let emphasized = mat_idx.is_multiple_of(2);
                let _: () = msg_send![subview, setEmphasized: emphasized];

                // Force redraw
                let _: () = msg_send![subview, setNeedsDisplay: true];
                let _: () = msg_send![window, display];

                let msg = format!(
                    "{} | {} | {} | {}",
                    material_name,
                    appearance_name,
                    blend_name,
                    if emphasized { "Emph" } else { "NoEmph" }
                );
                logging::log("VIBRANCY", &msg);
                return msg;
            }
        }
    }

    "ERROR: No NSVisualEffectView found".to_string()
}

#[cfg(not(target_os = "macos"))]
pub fn cycle_vibrancy_material() -> String {
    "Vibrancy cycling not supported on this platform".to_string()
}

/// Toggle between blending modes (behindWindow vs withinWindow)
/// Call with Cmd+Shift+B
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn toggle_blending_mode() -> String {
    use std::sync::atomic::Ordering;

    debug_assert_main_thread();

    // Toggle between 0 (behindWindow) and 1 (withinWindow)
    let current = CURRENT_BLENDING_MODE.fetch_xor(1, Ordering::SeqCst);
    let new_mode = current ^ 1;
    let mode_name = if new_mode == 0 {
        "BehindWindow"
    } else {
        "WithinWindow"
    };

    unsafe {
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => return "ERROR: No main window".to_string(),
        };

        let content_view: id = msg_send![window, contentView];
        if content_view.is_null() {
            return "ERROR: No content view".to_string();
        }

        let subviews: id = msg_send![content_view, subviews];
        if subviews.is_null() {
            return "ERROR: No subviews".to_string();
        }

        let count: usize = msg_send![subviews, count];
        for i in 0..count {
            let subview: id = msg_send![subviews, objectAtIndex: i];
            let is_visual_effect_view: bool =
                msg_send![subview, isKindOfClass: class!(NSVisualEffectView)];

            if is_visual_effect_view {
                let _: () = msg_send![subview, setBlendingMode: new_mode as isize];
                let _: () = msg_send![subview, setNeedsDisplay: true];

                let msg = format!("Blending: {}", mode_name);
                logging::log("VIBRANCY", &msg);
                return msg;
            }
        }
    }

    "ERROR: No NSVisualEffectView found".to_string()
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
pub fn toggle_blending_mode() -> String {
    "Blending mode toggle not supported on this platform".to_string()
}

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
/// - VibrantDark appearance + POPOVER material for frosted glass effect
///
/// # Arguments
/// * `window` - The NSWindow pointer to configure
///
/// # Safety
/// - `window` must be a valid NSWindow pointer
/// - Must be called on the main thread
#[cfg(target_os = "macos")]
pub unsafe fn configure_actions_popup_window(window: id) {
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

    // Hide when app deactivates (loses focus to another app)
    let _: () = msg_send![window, setHidesOnDeactivate: true];

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

    // Set window appearance to VibrantDark for consistent blur rendering
    let vibrant_dark: id = msg_send![
        class!(NSAppearance),
        appearanceNamed: NSAppearanceNameVibrantDark
    ];
    if !vibrant_dark.is_null() {
        let _: () = msg_send![window, setAppearance: vibrant_dark];
    }

    // Use clearColor for window background to allow maximum blur transparency
    let clear_color: id = msg_send![class!(NSColor), clearColor];
    let _: () = msg_send![window, setBackgroundColor: clear_color];

    // Mark window as non-opaque to allow transparency/vibrancy
    let _: () = msg_send![window, setOpaque: false];

    // Configure NSVisualEffectViews in the window hierarchy
    let content_view: id = msg_send![window, contentView];
    if !content_view.is_null() {
        let mut count = 0;
        configure_visual_effect_views_recursive(content_view, &mut count);
        logging::log(
            "ACTIONS",
            &format!(
                "Configured {} NSVisualEffectView(s) in actions popup",
                count
            ),
        );
    }

    logging::log(
        "ACTIONS",
        "Configured actions popup window (non-movable, vibrancy, VibrantDark)",
    );
}

#[cfg(not(target_os = "macos"))]
pub fn configure_actions_popup_window(_window: *mut std::ffi::c_void) {
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

/// Represents a display's bounds in macOS global coordinate space
#[derive(Debug, Clone)]
pub struct DisplayBounds {
    pub origin_x: f64,
    pub origin_y: f64,
    pub width: f64,
    pub height: f64,
}

#[cfg(target_os = "macos")]
use cocoa::foundation::NSRect;

/// Get the height of the primary (main) screen for coordinate conversion.
/// macOS uses bottom-left origin; we convert to top-left origin.
#[cfg(target_os = "macos")]
pub fn primary_screen_height() -> Option<f64> {
    debug_assert_main_thread();
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
    debug_assert_main_thread();
    unsafe {
        let screens: id = msg_send![class!(NSScreen), screens];
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
            msg_send![screens, firstObject]
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

// ============================================================================
// Window Movement
// ============================================================================

#[cfg(target_os = "macos")]
use cocoa::foundation::{NSPoint, NSSize};

/// Move the application's main window to new bounds using WindowManager.
/// This uses the registered main window instead of objectAtIndex:0, which
/// avoids issues with tray icons and other system windows in the array.
///
/// IMPORTANT: macOS uses a global coordinate space where Y=0 is at the BOTTOM of the
/// PRIMARY screen, and Y increases upward. The primary screen's origin is always (0,0)
/// at its bottom-left corner. Secondary displays have their own position in this space.
#[cfg(target_os = "macos")]
pub fn move_first_window_to(x: f64, y: f64, width: f64, height: f64) {
    debug_assert_main_thread();
    unsafe {
        // Use WindowManager to get the main window reliably
        let window = match window_manager::get_main_window() {
            Some(w) => w,
            None => {
                logging::log(
                    "POSITION",
                    "WARNING: Main window not registered in WindowManager, cannot move",
                );
                return;
            }
        };

        // Get the PRIMARY screen's height for coordinate conversion
        // CRITICAL: Use mainScreen, not firstObject - they can differ when display arrangement changes
        let main_screen: id = msg_send![class!(NSScreen), mainScreen];
        let main_screen = if main_screen == nil {
            // Fallback to firstObject if mainScreen is nil (shouldn't happen but be safe)
            let screens: id = msg_send![class!(NSScreen), screens];
            msg_send![screens, firstObject]
        } else {
            main_screen
        };
        let main_screen_frame: NSRect = msg_send![main_screen, frame];
        let primary_screen_height = main_screen_frame.size.height;

        // Log current window position before move
        let current_frame: NSRect = msg_send![window, frame];
        logging::log(
            "POSITION",
            &format!(
                "Current window frame: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
                current_frame.origin.x,
                current_frame.origin.y,
                current_frame.size.width,
                current_frame.size.height
            ),
        );

        // Convert from top-left origin (y down) to bottom-left origin (y up)
        let flipped_y = primary_screen_height - y - height;

        logging::log(
            "POSITION",
            &format!(
                "Moving window: target=({:.0}, {:.0}) flipped_y={:.0}",
                x, y, flipped_y
            ),
        );

        let new_frame = NSRect::new(NSPoint::new(x, flipped_y), NSSize::new(width, height));

        // Move the window
        let _: () = msg_send![window, setFrame:new_frame display:true animate:false];

        // NOTE: We no longer call makeKeyAndOrderFront here.
        // Window ordering/activation is handled by GPUI's cx.activate() and win.activate_window()
        // which is called AFTER ensure_move_to_active_space() sets the collection behavior.

        // Verify the move worked
        let after_frame: NSRect = msg_send![window, frame];
        logging::log(
            "POSITION",
            &format!(
                "Window moved: actual=({:.0}, {:.0}) size={:.0}x{:.0}",
                after_frame.origin.x,
                after_frame.origin.y,
                after_frame.size.width,
                after_frame.size.height
            ),
        );
    }
}

#[cfg(not(target_os = "macos"))]
pub fn move_first_window_to(_x: f64, _y: f64, _width: f64, _height: f64) {
    // TODO: Implement for other platforms
    logging::log(
        "POSITION",
        "move_first_window_to is not implemented for this platform",
    );
}

use gpui::{point, px, Bounds, Pixels};

/// Move the first window to new bounds (wrapper for Bounds<Pixels>)
pub fn move_first_window_to_bounds(bounds: &Bounds<Pixels>) {
    let x: f64 = bounds.origin.x.into();
    let y: f64 = bounds.origin.y.into();
    let width: f64 = bounds.size.width.into();
    let height: f64 = bounds.size.height.into();
    move_first_window_to(x, y, width, height);
}

// ============================================================================
// Window Positioning (Eye-line)
// ============================================================================

/// Calculate window bounds positioned at eye-line height on the display containing the mouse cursor.
///
/// - Finds the display where the mouse cursor is located
/// - Centers the window horizontally on that display
/// - Positions the window at "eye-line" height (upper 14% of the screen)
///
/// This matches the behavior of Raycast/Alfred where the prompt appears on the active display.
pub fn calculate_eye_line_bounds_on_mouse_display(
    window_size: gpui::Size<Pixels>,
) -> Bounds<Pixels> {
    // Use native macOS API to get actual display bounds with correct origins
    // GPUI's cx.displays() returns incorrect origins for secondary displays
    let displays = get_macos_displays();

    logging::log("POSITION", "");
    logging::log(
        "POSITION",
        "╔════════════════════════════════════════════════════════════╗",
    );
    logging::log(
        "POSITION",
        "║  CALCULATING WINDOW POSITION FOR MOUSE DISPLAY             ║",
    );
    logging::log(
        "POSITION",
        "╚════════════════════════════════════════════════════════════╝",
    );
    logging::log(
        "POSITION",
        &format!("Available displays: {}", displays.len()),
    );

    // Log all available displays for debugging
    for (idx, display) in displays.iter().enumerate() {
        let right = display.origin_x + display.width;
        let bottom = display.origin_y + display.height;
        logging::log("POSITION", &format!(
            "  Display {}: origin=({:.0}, {:.0}) size={:.0}x{:.0} [bounds: x={:.0}..{:.0}, y={:.0}..{:.0}]",
            idx, display.origin_x, display.origin_y, display.width, display.height,
            display.origin_x, right, display.origin_y, bottom
        ));
    }

    // Try to get mouse position and find which display contains it
    let target_display = if let Some((mouse_x, mouse_y)) = get_global_mouse_position() {
        logging::log(
            "POSITION",
            &format!("Mouse cursor at ({:.0}, {:.0})", mouse_x, mouse_y),
        );

        // Find the display that contains the mouse cursor
        let mut found_display = None;
        for (idx, display) in displays.iter().enumerate() {
            let in_x = mouse_x >= display.origin_x && mouse_x < display.origin_x + display.width;
            let in_y = mouse_y >= display.origin_y && mouse_y < display.origin_y + display.height;
            let contains = in_x && in_y;

            if contains {
                logging::log("POSITION", &format!("  -> Mouse is on display {}", idx));
                found_display = Some(display.clone());
                break;
            } else {
                // Log why this display didn't match (helpful for debugging edge cases)
                logging::log(
                    "POSITION",
                    &format!(
                        "  Display {} rejected: in_x={} in_y={} (mouse: {:.0},{:.0} vs bounds: x={:.0}..{:.0}, y={:.0}..{:.0})",
                        idx, in_x, in_y, mouse_x, mouse_y,
                        display.origin_x, display.origin_x + display.width,
                        display.origin_y, display.origin_y + display.height
                    ),
                );
            }
        }

        found_display
    } else {
        logging::log(
            "POSITION",
            "Could not get mouse position, using primary display",
        );
        None
    };

    // Use the found display, or fall back to first display (primary)
    let display = target_display.or_else(|| {
        logging::log(
            "POSITION",
            "No display contains mouse, falling back to primary",
        );
        displays.first().cloned()
    });

    if let Some(display) = display {
        logging::log(
            "POSITION",
            &format!(
                "Selected display: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
                display.origin_x, display.origin_y, display.width, display.height
            ),
        );

        // Eye-line: position window top at ~14% from screen top (input bar at eye level)
        let eye_line_y = display.origin_y + display.height * 0.14;

        // Center horizontally on the display
        let window_width: f64 = window_size.width.into();
        let center_x = display.origin_x + (display.width - window_width) / 2.0;

        let final_bounds = Bounds {
            origin: point(px(center_x as f32), px(eye_line_y as f32)),
            size: window_size,
        };

        logging::log(
            "POSITION",
            &format!(
                "Final window bounds: origin=({:.0}, {:.0}) size={:.0}x{:.0}",
                center_x,
                eye_line_y,
                f64::from(window_size.width),
                f64::from(window_size.height)
            ),
        );

        final_bounds
    } else {
        logging::log(
            "POSITION",
            "No displays found, using default centered bounds",
        );
        // Fallback: just center on screen using 1512x982 as default (common MacBook)
        Bounds {
            origin: point(px(381.0), px(246.0)),
            size: window_size,
        }
    }
}

// ============================================================================
// Screenshot Capture
// ============================================================================

/// Capture a screenshot of the app window using xcap for cross-platform support.
///
/// Returns a tuple of (png_data, width, height) on success.
/// The function:
/// 1. Uses xcap::Window::all() to enumerate windows
/// 2. Finds the Script Kit window by app name or title
/// 3. Captures the window directly to an image buffer
/// 4. Optionally scales down to 1x resolution if hi_dpi is false
/// 5. Encodes to PNG in memory (no temp files)
///
/// # Arguments
/// * `hi_dpi` - If true, return full retina resolution (2x). If false, scale down to 1x.
pub fn capture_app_screenshot(
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use xcap::Window;

    let windows = Window::all()?;

    struct Candidate {
        window: Window,
        title: String,
        width: u32,
        height: u32,
    }

    let mut candidates = Vec::new();
    for window in windows {
        let title = window.title().unwrap_or_else(|_| String::new());
        let app_name = window.app_name().unwrap_or_else(|_| String::new());

        // Match our app window by name
        let is_our_window = app_name.contains("script-kit-gpui")
            || app_name == "Script Kit"
            || title.contains("Script Kit");

        let is_minimized = window.is_minimized().unwrap_or(true);

        // Get window dimensions to filter out tiny windows (tooltips, list items, etc.)
        let width = window.width().unwrap_or(0);
        let height = window.height().unwrap_or(0);

        // Only consider windows that are reasonably sized
        // Width >= 200 filters out small UI elements
        // Height >= 50 allows compact prompts (arg prompt without choices is ~76px)
        let is_reasonable_size = width >= 200 && height >= 50;

        if is_our_window && !is_minimized && is_reasonable_size {
            candidates.push(Candidate {
                window,
                title,
                width,
                height,
            });
        }
    }

    // Sort by size (largest first) - the main window is typically the largest
    candidates.sort_by(|a, b| {
        let area_a = a.width as u64 * a.height as u64;
        let area_b = b.width as u64 * b.height as u64;
        area_b.cmp(&area_a)
    });

    let mut target = candidates
        .iter()
        .filter(|candidate| candidate.title.contains("Notes") || candidate.title.contains("AI"))
        .find(|candidate| candidate.window.is_focused().unwrap_or(false))
        .map(|candidate| candidate.window.clone());

    if target.is_none() {
        target = candidates
            .iter()
            .find(|candidate| candidate.title.contains("Notes") || candidate.title.contains("AI"))
            .map(|candidate| candidate.window.clone());
    }

    if target.is_none() {
        target = candidates
            .iter()
            .find(|candidate| candidate.window.is_focused().unwrap_or(false))
            .map(|candidate| candidate.window.clone());
    }

    let Some(window) =
        target.or_else(|| candidates.first().map(|candidate| candidate.window.clone()))
    else {
        return Err("Script Kit window not found".into());
    };

    let title = window.title().unwrap_or_else(|_| String::new());
    let app_name = window.app_name().unwrap_or_else(|_| String::new());

    tracing::debug!(
        app_name = %app_name,
        title = %title,
        hi_dpi = hi_dpi,
        "Found Script Kit window for screenshot"
    );

    let image = window.capture_image()?;
    let original_width = image.width();
    let original_height = image.height();

    // Scale down to 1x if not hi_dpi mode (xcap captures at retina resolution on macOS)
    let (final_image, width, height) = if hi_dpi {
        (image, original_width, original_height)
    } else {
        // Scale down by 2x for 1x resolution
        let new_width = original_width / 2;
        let new_height = original_height / 2;
        let resized = image::imageops::resize(
            &image,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        tracing::debug!(
            original_width = original_width,
            original_height = original_height,
            new_width = new_width,
            new_height = new_height,
            "Scaled screenshot to 1x resolution"
        );
        (resized, new_width, new_height)
    };

    // Encode to PNG in memory (no temp files needed)
    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(&final_image, width, height, image::ExtendedColorType::Rgba8)?;

    tracing::debug!(
        width = width,
        height = height,
        hi_dpi = hi_dpi,
        file_size = png_data.len(),
        "Screenshot captured with xcap"
    );

    Ok((png_data, width, height))
}

/// Capture a screenshot of a window by its title pattern.
///
/// Similar to `capture_app_screenshot` but allows specifying which window to capture
/// by matching the title. This is useful for secondary windows like the AI Chat window.
///
/// # Arguments
/// * `title_pattern` - A string that the window title must contain
/// * `hi_dpi` - If true, return full retina resolution (2x). If false, scale down to 1x.
///
/// # Returns
/// A tuple of (png_data, width, height) on success.
pub fn capture_window_by_title(
    title_pattern: &str,
    hi_dpi: bool,
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use xcap::Window;

    let windows = Window::all()?;

    for window in windows {
        let title = window.title().unwrap_or_else(|_| String::new());
        let app_name = window.app_name().unwrap_or_else(|_| String::new());

        // Match window by title pattern (must also be our app)
        let is_our_app = app_name.contains("script-kit-gpui") || app_name == "Script Kit";
        let title_matches = title.contains(title_pattern);
        let is_minimized = window.is_minimized().unwrap_or(true);

        if is_our_app && title_matches && !is_minimized {
            tracing::debug!(
                app_name = %app_name,
                title = %title,
                title_pattern = %title_pattern,
                hi_dpi = hi_dpi,
                "Found window matching title pattern for screenshot"
            );

            let image = window.capture_image()?;
            let original_width = image.width();
            let original_height = image.height();

            // Scale down to 1x if not hi_dpi mode
            let (final_image, width, height) = if hi_dpi {
                (image, original_width, original_height)
            } else {
                let new_width = original_width / 2;
                let new_height = original_height / 2;
                let resized = image::imageops::resize(
                    &image,
                    new_width,
                    new_height,
                    image::imageops::FilterType::Lanczos3,
                );
                tracing::debug!(
                    original_width = original_width,
                    original_height = original_height,
                    new_width = new_width,
                    new_height = new_height,
                    "Scaled screenshot to 1x resolution"
                );
                (resized, new_width, new_height)
            };

            // Encode to PNG in memory
            let mut png_data = Vec::new();
            let encoder = PngEncoder::new(&mut png_data);
            encoder.write_image(&final_image, width, height, image::ExtendedColorType::Rgba8)?;

            tracing::debug!(
                width = width,
                height = height,
                hi_dpi = hi_dpi,
                file_size = png_data.len(),
                title_pattern = %title_pattern,
                "Screenshot captured for window by title"
            );

            return Ok((png_data, width, height));
        }
    }

    Err(format!("Window with title containing '{}' not found", title_pattern).into())
}

// ============================================================================
// Open Path with System Default
// ============================================================================

/// Open a path (file or folder) with the system default application.
/// On macOS: uses `open` command
/// On Linux: uses `xdg-open` command
/// On Windows: uses `cmd /C start` command
///
/// This can be used to open files, folders, URLs, or any path that the
/// system knows how to handle.
#[allow(dead_code)]
pub fn open_path_with_system_default(path: &str) {
    logging::log("UI", &format!("Opening path with system default: {}", path));
    let path_owned = path.to_string();

    std::thread::spawn(move || {
        #[cfg(target_os = "macos")]
        {
            match std::process::Command::new("open").arg(&path_owned).spawn() {
                Ok(_) => logging::log("UI", &format!("Successfully opened: {}", path_owned)),
                Err(e) => logging::log("ERROR", &format!("Failed to open '{}': {}", path_owned, e)),
            }
        }

        #[cfg(target_os = "linux")]
        {
            match std::process::Command::new("xdg-open")
                .arg(&path_owned)
                .spawn()
            {
                Ok(_) => logging::log("UI", &format!("Successfully opened: {}", path_owned)),
                Err(e) => logging::log("ERROR", &format!("Failed to open '{}': {}", path_owned, e)),
            }
        }

        #[cfg(target_os = "windows")]
        {
            match std::process::Command::new("cmd")
                .args(["/C", "start", "", &path_owned])
                .spawn()
            {
                Ok(_) => logging::log("UI", &format!("Successfully opened: {}", path_owned)),
                Err(e) => logging::log("ERROR", &format!("Failed to open '{}': {}", path_owned, e)),
            }
        }
    });
}

// ============================================================================
// Screen Capture for AI Commands
// ============================================================================

/// Capture a screenshot of the entire primary screen.
///
/// # Returns
/// A tuple of (png_data, width, height) on success.
pub fn capture_screen_screenshot(
) -> Result<(Vec<u8>, u32, u32), Box<dyn std::error::Error + Send + Sync>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use xcap::Monitor;

    let monitors = Monitor::all()?;

    // Get the primary monitor (first one, usually the main display)
    let monitor = monitors.into_iter().next().ok_or("No monitors found")?;

    tracing::debug!(
        name = %monitor.name().unwrap_or_default(),
        "Capturing primary monitor screenshot"
    );

    let image = monitor.capture_image()?;
    let width = image.width();
    let height = image.height();

    // Scale down to 1x for efficiency (monitors capture at retina resolution on macOS)
    let (final_image, final_width, final_height) = {
        let new_width = width / 2;
        let new_height = height / 2;
        let resized = image::imageops::resize(
            &image,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        (resized, new_width, new_height)
    };

    // Encode to PNG in memory
    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(
        &final_image,
        final_width,
        final_height,
        image::ExtendedColorType::Rgba8,
    )?;

    tracing::debug!(
        width = final_width,
        height = final_height,
        file_size = png_data.len(),
        "Screen screenshot captured"
    );

    Ok((png_data, final_width, final_height))
}

/// Capture a screenshot of the currently focused window (not our app).
///
/// This function finds the frontmost window that is NOT Script Kit and captures it.
///
/// # Returns
/// A tuple of (png_data, width, height, window_title) on success.
#[allow(clippy::type_complexity)]
pub fn capture_focused_window_screenshot(
) -> Result<(Vec<u8>, u32, u32, String), Box<dyn std::error::Error + Send + Sync>> {
    use image::codecs::png::PngEncoder;
    use image::ImageEncoder;
    use xcap::Window;

    let windows = Window::all()?;

    // Find the focused window that is NOT our app
    let mut target_window = None;
    for window in windows {
        let app_name = window.app_name().unwrap_or_else(|_| String::new());
        let is_minimized = window.is_minimized().unwrap_or(true);
        let is_focused = window.is_focused().unwrap_or(false);

        // Skip our own app
        let is_our_app = app_name.contains("script-kit-gpui")
            || app_name == "Script Kit"
            || app_name.contains("Script Kit");

        // Get window dimensions - skip tiny windows
        let width = window.width().unwrap_or(0);
        let height = window.height().unwrap_or(0);
        let is_reasonable_size = width >= 100 && height >= 100;

        if !is_our_app && !is_minimized && is_reasonable_size {
            if is_focused {
                target_window = Some(window);
                break;
            }
            // Keep the first reasonable non-our-app window as fallback
            if target_window.is_none() {
                target_window = Some(window);
            }
        }
    }

    let window = target_window.ok_or("No suitable window found to capture")?;
    let title = window.title().unwrap_or_else(|_| "Unknown".to_string());
    let app_name = window.app_name().unwrap_or_else(|_| "Unknown".to_string());

    tracing::debug!(
        app_name = %app_name,
        title = %title,
        "Capturing focused window screenshot"
    );

    let image = window.capture_image()?;
    let original_width = image.width();
    let original_height = image.height();

    // Scale down to 1x for efficiency
    let (final_image, width, height) = {
        let new_width = original_width / 2;
        let new_height = original_height / 2;
        let resized = image::imageops::resize(
            &image,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        (resized, new_width, new_height)
    };

    // Encode to PNG in memory
    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder.write_image(&final_image, width, height, image::ExtendedColorType::Rgba8)?;

    let display_title = if title.is_empty() {
        app_name
    } else {
        format!("{} - {}", app_name, title)
    };

    tracing::debug!(
        width = width,
        height = height,
        file_size = png_data.len(),
        title = %display_title,
        "Focused window screenshot captured"
    );

    Ok((png_data, width, height, display_title))
}

/// Get the URL of the currently focused browser tab.
///
/// Supports Safari, Google Chrome, Arc, Brave, Firefox, and Edge.
///
/// # Returns
/// The URL string on success.
#[cfg(target_os = "macos")]
pub fn get_focused_browser_tab_url() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use std::process::Command;

    // First, get the frontmost application name
    let frontmost_script = r#"
        tell application "System Events"
            set frontApp to name of first application process whose frontmost is true
            return frontApp
        end tell
    "#;

    let frontmost_output = Command::new("osascript")
        .arg("-e")
        .arg(frontmost_script)
        .output()?;

    if !frontmost_output.status.success() {
        return Err("Failed to get frontmost application".into());
    }

    let frontmost_app = String::from_utf8_lossy(&frontmost_output.stdout)
        .trim()
        .to_string();

    tracing::debug!(app = %frontmost_app, "Detected frontmost browser");

    // Map process name to application name and the AppleScript to get URL
    let (app_name, url_script) = match frontmost_app.as_str() {
        "Safari" => (
            "Safari",
            r#"tell application "Safari" to return URL of front document"#,
        ),
        "Google Chrome" => (
            "Google Chrome",
            r#"tell application "Google Chrome" to return URL of active tab of front window"#,
        ),
        "Arc" => (
            "Arc",
            r#"tell application "Arc" to return URL of active tab of front window"#,
        ),
        "Brave Browser" => (
            "Brave Browser",
            r#"tell application "Brave Browser" to return URL of active tab of front window"#,
        ),
        "Firefox" => {
            // Firefox doesn't support AppleScript well - return an error with helpful message
            return Err("Firefox doesn't fully support AppleScript for URL retrieval. Try Safari or Chrome.".into());
        }
        "Microsoft Edge" => (
            "Microsoft Edge",
            r#"tell application "Microsoft Edge" to return URL of active tab of front window"#,
        ),
        "Chromium" => (
            "Chromium",
            r#"tell application "Chromium" to return URL of active tab of front window"#,
        ),
        "Vivaldi" => (
            "Vivaldi",
            r#"tell application "Vivaldi" to return URL of active tab of front window"#,
        ),
        "Opera" => (
            "Opera",
            r#"tell application "Opera" to return URL of active tab of front window"#,
        ),
        _ => {
            return Err(format!(
                "Frontmost app '{}' is not a supported browser. Supported: Safari, Chrome, Arc, Brave, Edge, Vivaldi, Opera",
                frontmost_app
            ).into());
        }
    };

    tracing::debug!(app = %app_name, "Getting URL from browser");

    let url_output = Command::new("osascript")
        .arg("-e")
        .arg(url_script)
        .output()?;

    if !url_output.status.success() {
        let stderr = String::from_utf8_lossy(&url_output.stderr);
        return Err(format!("Failed to get URL from {}: {}", app_name, stderr).into());
    }

    let url = String::from_utf8_lossy(&url_output.stdout)
        .trim()
        .to_string();

    if url.is_empty() {
        return Err(format!("No URL found in {}", app_name).into());
    }

    tracing::debug!(url = %url, app = %app_name, "Browser URL retrieved");

    Ok(url)
}

#[cfg(not(target_os = "macos"))]
pub fn get_focused_browser_tab_url() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Err("Browser URL retrieval is only supported on macOS".into())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Main Thread Assertion Tests
    // =========================================================================

    /// Test that debug_assert_main_thread returns correct value via NSThread.isMainThread.
    /// Note: Rust test harness does NOT run tests on the main thread - it uses a thread pool.
    /// This test verifies that our assertion correctly detects we're NOT on the main thread.
    #[cfg(target_os = "macos")]
    #[test]
    fn test_debug_assert_main_thread_detects_non_main_thread() {
        // Rust tests run on thread pool workers, NOT the main thread.
        // Verify that NSThread.isMainThread returns false (as expected)
        unsafe {
            let is_main: bool = msg_send![class!(NSThread), isMainThread];
            // In tests, we should NOT be on the main thread
            assert!(
                !is_main,
                "Expected test to run on non-main thread (Rust test harness behavior)"
            );
        }
    }

    /// Test that debug_assert_main_thread would panic on a background thread.
    /// Note: This test only runs in debug mode since debug_assert is used.
    #[cfg(all(target_os = "macos", debug_assertions))]
    #[test]
    fn test_debug_assert_main_thread_panics_on_background_thread() {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            // Catch the panic from debug_assert_main_thread
            let result = std::panic::catch_unwind(|| {
                debug_assert_main_thread();
            });
            tx.send(result.is_err()).unwrap();
        })
        .join()
        .unwrap();

        let panicked = rx.recv().unwrap();
        assert!(
            panicked,
            "debug_assert_main_thread should panic on background thread"
        );
    }

    // =========================================================================
    // Characterization Tests (AppKit functions)
    // =========================================================================
    // NOTE: These tests are ignored on macOS because they require the main thread.
    // Rust's test harness runs tests on thread pool workers, not the main thread.
    // In production, these functions are called from GPUI's main thread event loop.

    /// Test that ensure_move_to_active_space can be called without panicking.
    /// This is a characterization test - it verifies the function doesn't crash.
    /// On non-macOS, this is a no-op. On macOS without a window, it logs a warning.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_ensure_move_to_active_space_does_not_panic() {
        // Should not panic even without a window registered
        ensure_move_to_active_space();
    }

    /// Test that configure_as_floating_panel can be called without panicking.
    /// This is a characterization test - it verifies the function doesn't crash.
    /// On non-macOS, this is a no-op. On macOS without NSApp/keyWindow, it handles gracefully.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_configure_as_floating_panel_does_not_panic() {
        // Should not panic even without an app running
        configure_as_floating_panel();
    }

    /// Verify the macOS constants have the correct values.
    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_constants() {
        assert_eq!(NS_FLOATING_WINDOW_LEVEL, 3);
        assert_eq!(NS_WINDOW_COLLECTION_BEHAVIOR_MOVE_TO_ACTIVE_SPACE, 2);
    }

    /// Test that both functions can be called in sequence.
    /// This mirrors the typical usage pattern in main.rs where both are called
    /// during window setup.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_functions_can_be_called_in_sequence() {
        // This is the typical call order in main.rs
        ensure_move_to_active_space();
        configure_as_floating_panel();
        // Should complete without panicking
    }

    /// Test that functions are idempotent - can be called multiple times safely.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_functions_are_idempotent() {
        for _ in 0..3 {
            ensure_move_to_active_space();
            configure_as_floating_panel();
        }
        // Should complete without panicking or causing issues
    }

    // =========================================================================
    // Mouse Position Tests
    // =========================================================================

    /// Test get_global_mouse_position returns valid coordinates or None.
    /// On macOS with display, returns Some((x, y)).
    /// On other platforms or without display, returns None.
    #[test]
    fn test_get_global_mouse_position_does_not_panic() {
        // Should not panic regardless of whether we can get the position
        let _ = get_global_mouse_position();
    }

    // =========================================================================
    // Display Information Tests
    // =========================================================================

    /// Test DisplayBounds struct creation and field access.
    #[test]
    fn test_display_bounds_struct() {
        let bounds = DisplayBounds {
            origin_x: 100.0,
            origin_y: 200.0,
            width: 1920.0,
            height: 1080.0,
        };

        assert_eq!(bounds.origin_x, 100.0);
        assert_eq!(bounds.origin_y, 200.0);
        assert_eq!(bounds.width, 1920.0);
        assert_eq!(bounds.height, 1080.0);
    }

    /// Test DisplayBounds Clone implementation.
    #[test]
    fn test_display_bounds_clone() {
        let original = DisplayBounds {
            origin_x: 0.0,
            origin_y: 0.0,
            width: 2560.0,
            height: 1440.0,
        };

        let cloned = original.clone();
        assert_eq!(cloned.width, 2560.0);
        assert_eq!(cloned.height, 1440.0);
    }

    /// Test get_macos_displays returns at least one display (or fallback).
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_get_macos_displays_returns_at_least_one() {
        let displays = get_macos_displays();
        assert!(!displays.is_empty(), "Should return at least one display");
    }

    /// Test get_macos_displays returns displays with valid dimensions.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_get_macos_displays_valid_dimensions() {
        let displays = get_macos_displays();
        for display in displays {
            assert!(display.width > 0.0, "Display width must be positive");
            assert!(display.height > 0.0, "Display height must be positive");
        }
    }

    // =========================================================================
    // Window Movement Tests
    // =========================================================================

    /// Test move_first_window_to does not panic without a window.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_move_first_window_to_does_not_panic() {
        // Should not panic even without a registered window
        move_first_window_to(100.0, 100.0, 800.0, 600.0);
    }

    /// Test move_first_window_to_bounds wrapper function.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_move_first_window_to_bounds_does_not_panic() {
        use gpui::size;
        let bounds = Bounds {
            origin: point(px(100.0), px(100.0)),
            size: size(px(800.0), px(600.0)),
        };
        // Should not panic
        move_first_window_to_bounds(&bounds);
    }

    // =========================================================================
    // Eye-line Positioning Tests
    // =========================================================================

    /// Test calculate_eye_line_bounds returns valid bounds.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_calculate_eye_line_bounds_returns_valid() {
        use gpui::size;
        let window_size = size(px(750.0), px(500.0));
        let bounds = calculate_eye_line_bounds_on_mouse_display(window_size);

        // Bounds should have the same size as input
        assert_eq!(bounds.size.width, window_size.width);
        assert_eq!(bounds.size.height, window_size.height);
    }

    /// Test eye-line calculation positions window in upper portion of screen.
    #[test]
    #[cfg_attr(target_os = "macos", ignore = "requires main thread (run via GPUI)")]
    fn test_calculate_eye_line_bounds_upper_portion() {
        use gpui::size;
        let window_size = size(px(750.0), px(500.0));
        let bounds = calculate_eye_line_bounds_on_mouse_display(window_size);

        // Get the display bounds for comparison
        let displays = get_macos_displays();
        if let Some(display) = displays.first() {
            let origin_y: f64 = bounds.origin.y.into();
            let display_height = display.height;

            // Eye-line should be in upper half of screen (top 50%)
            assert!(
                origin_y < display.origin_y + display_height * 0.5,
                "Window should be in upper half: origin_y={}, display_mid={}",
                origin_y,
                display.origin_y + display_height * 0.5
            );
        }
    }
}
