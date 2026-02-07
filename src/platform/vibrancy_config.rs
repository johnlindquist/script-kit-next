/// Configure the vibrancy blur for the main window based on appearance mode.
///
/// This is the appearance-aware version that should be called after loading the theme.
/// Uses VibrantLight for light mode, VibrantDark for dark mode.
///
/// # Arguments
/// * `is_dark` - true for dark mode (VibrantDark), false for light mode (VibrantLight)
///
/// # macOS Behavior
///
/// Clears the window's NSAppearance (sets to nil) so GPUI can detect system appearance
/// changes, then sets the appearance on each NSVisualEffectView individually along with
/// appropriate material and state.
///
/// # Safety
///
/// Uses Objective-C message sending internally.
#[cfg(target_os = "macos")]
pub fn configure_window_vibrancy_material_for_appearance(is_dark: bool) {
    if require_main_thread("configure_window_vibrancy_material_for_appearance") {
        return;
    }
    // SAFETY: Main thread verified. Window from WindowManager is valid.
    // NSAppearance set to nil on window, set on views instead.
    // NSVisualEffectView methods are standard AppKit APIs.
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

        // Clear window appearance so GPUI can detect system appearance changes.
        // The appearance is set on individual NSVisualEffectViews instead (see
        // configure_visual_effect_views_recursive).
        let _: () = msg_send![window, setAppearance: nil];
        logging::log(
            "PANEL",
            &format!(
                "Cleared window appearance (nil) for {} mode; appearance set on views instead",
                if is_dark { "dark" } else { "light" }
            ),
        );

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
        configure_visual_effect_views_recursive(content_view, &mut count, is_dark);

        let material_name = if is_dark { "HUD_WINDOW" } else { "POPOVER" };
        if count == 0 {
            logging::log(
                "PANEL",
                "WARNING: No NSVisualEffectView found in window hierarchy",
            );
        } else {
            logging::log(
                "PANEL",
                &format!(
                    "Configured {} NSVisualEffectView(s): {} + {} + emphasized",
                    count,
                    if is_dark {
                        "VibrantDark"
                    } else {
                        "VibrantLight"
                    },
                    material_name
                ),
            );
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_window_vibrancy_material_for_appearance(_is_dark: bool) {
    // No-op on non-macOS platforms
}

/// Configure the vibrancy blur for the main window (backward compatible).
///
/// This function defaults to dark mode (VibrantDark) for backward compatibility.
/// For appearance-aware vibrancy, use `configure_window_vibrancy_material_for_appearance()`.
///
/// # macOS Behavior
///
/// Clears the window's NSAppearance and sets VibrantDark on each NSVisualEffectView,
/// then configures appropriate material and state.
///
/// # Safety
///
/// Uses Objective-C message sending internally.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub fn configure_window_vibrancy_material() {
    // Backward compatible: default to dark mode
    configure_window_vibrancy_material_for_appearance(true);
}

/// Recursively walk view hierarchy and configure all NSVisualEffectViews.
///
/// # Arguments
/// * `view` - The view to configure
/// * `count` - Counter for configured views
/// * `is_dark` - Whether to use dark mode material (HUD_WINDOW) or light mode material (POPOVER)
///
/// # Safety
///
/// `view` must be a valid NSView pointer. Caller must be on the main thread.
/// Subview array and its elements are accessed via count-bounded iteration.
/// UTF8String pointers are nil-checked before CStr::from_ptr.
#[cfg(target_os = "macos")]
unsafe fn configure_visual_effect_views_recursive(view: id, count: &mut usize, is_dark: bool) {
    // Check if this view is an NSVisualEffectView
    let is_vev: bool = msg_send![view, isKindOfClass: class!(NSVisualEffectView)];
    if is_vev {
        // Log current state BEFORE configuration
        let old_material: isize = msg_send![view, material];
        let old_state: isize = msg_send![view, state];
        let old_blending: isize = msg_send![view, blendingMode];
        let old_emphasized: bool = msg_send![view, isEmphasized];

        // Set appearance on the NSVisualEffectView (NOT on the window) so that
        // GPUI can still detect system appearance changes via the window.
        let view_appearance: id = if is_dark {
            msg_send![
                class!(NSAppearance),
                appearanceNamed: NSAppearanceNameVibrantDark
            ]
        } else {
            msg_send![
                class!(NSAppearance),
                appearanceNamed: NSAppearanceNameVibrantLight
            ]
        };
        if !view_appearance.is_null() {
            let _: () = msg_send![view, setAppearance: view_appearance];
        }

        // ╔════════════════════════════════════════════════════════════════════════════╗
        // ║ NSVISUALEFFECTVIEW SETTINGS - DO NOT CHANGE WITHOUT TESTING               ║
        // ╠════════════════════════════════════════════════════════════════════════════╣
        // ║ These settings use 'active' state to prevent dimming when child windows   ║
        // ║ (like Actions popup) take focus. Combined with tint alpha in              ║
        // ║ gpui_integration.rs. See: /Users/johnlindquist/dev/mac-panel-window/      ║
        // ╚════════════════════════════════════════════════════════════════════════════╝
        // Material selection based on appearance mode:
        // - Dark mode: HUD_WINDOW (13) - designed for dark UIs, high contrast
        // - Light mode: POPOVER (6) - cleaner light appearance with frosted glass effect
        let material = if is_dark {
            ns_visual_effect_material::HUD_WINDOW
        } else {
            ns_visual_effect_material::POPOVER
        };
        let _: () = msg_send![view, setMaterial: material];
        // State: 1=active for dark (prevents dimming), 0=followsWindow for light (cleaner look)
        // NSVisualEffectState: 0=followsWindowActiveState, 1=active, 2=inactive
        // Dark mode: active state prevents dimming when Actions popup opens
        // Light mode: followsWindow gives cleaner appearance like the POC
        let state = if is_dark { 1isize } else { 0isize };
        let _: () = msg_send![view, setState: state];
        // BehindWindow blending (0) - blur content behind the window
        let _: () = msg_send![view, setBlendingMode: 0isize];
        // Emphasized adds more contrast/tint - use only in dark mode
        // Light mode without emphasis matches POC's cleaner look
        let _: () = msg_send![view, setEmphasized: is_dark];

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

        let material_name = if is_dark { "HUD_WINDOW" } else { "POPOVER" };
        logging::log(
            "VIBRANCY",
            &format!(
                "NSVisualEffectView config: mat {} -> {} ({}), state {} -> {}, blend {} -> {}, emph {} -> {}, mode={}, appearance={}",
                old_material, new_material, material_name,
                old_state, new_state,
                old_blending, new_blending,
                old_emphasized, new_emphasized,
                if is_dark { "dark" } else { "light" },
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
            configure_visual_effect_views_recursive(child, count, is_dark);
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_window_vibrancy_material() {
    // No-op on non-macOS platforms
}
