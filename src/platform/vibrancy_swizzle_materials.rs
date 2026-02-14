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

/// NSWindowCollectionBehaviorIgnoresCycle constant value (1 << 6 = 64)
/// Window is excluded from Cmd+Tab app switcher cycling.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub const NS_WINDOW_COLLECTION_BEHAVIOR_IGNORES_CYCLE: u64 = 1 << 6;

/// NSWindowCollectionBehaviorParticipatesInCycle constant value (1 << 7 = 128)
/// Window explicitly participates in Cmd+Tab app switcher cycling.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
pub const NS_WINDOW_COLLECTION_BEHAVIOR_PARTICIPATES_IN_CYCLE: u64 = 1 << 7;

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
/// Default: 11 = HudWindow (best for light mode vibrancy)
#[cfg(target_os = "macos")]
static CURRENT_MATERIAL_INDEX: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(11); // HudWindow

/// Current blending mode (0 = behindWindow, 1 = withinWindow)
#[cfg(target_os = "macos")]
static CURRENT_BLENDING_MODE: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// Current appearance index for cycling
/// Default: 3 = VibrantLight (for light mode), will be set to 1 (VibrantDark) for dark mode
#[cfg(target_os = "macos")]
static CURRENT_APPEARANCE_INDEX: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(3); // VibrantLight

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

    // SAFETY: Uses Objective-C runtime to look up a class by name and replace
    // a method implementation. Both the class pointer and method pointer are
    // checked for null before use. The swizzle is guarded by SWIZZLE_DONE
    // to ensure it only runs once. transmute converts a Rust fn pointer to
    // an Objective-C IMP, which is ABI-compatible for extern "C" functions.
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

    // SAFETY: `this` is a valid BlurredView instance (subclass of NSVisualEffectView)
    // provided by the Objective-C runtime as the receiver of the swizzled method.
    // super() dispatch calls the parent class (NSVisualEffectView) implementation.
    // Layer pointers are nil-checked before recursion in dump_layer_hierarchy.
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

/// Recursively dump layer hierarchy to find CAChameleonLayer.
///
/// # Safety
///
/// `layer` must be a valid CALayer pointer or nil (nil is checked at entry).
/// Caller must be on the main thread. UTF8String pointers are nil-checked
/// before CStr::from_ptr. Recursion depth is bounded to 5 levels.
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
