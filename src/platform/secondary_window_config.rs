// ============================================================================
// Actions Popup Window Configuration
// ============================================================================

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NativeGlassEntryReceipt {
    pub(crate) window_number: i64,
    pub(crate) configured: bool,
    pub(crate) backdrop_found_or_created: bool,
    pub(crate) native_selectors_supported: bool,
    pub(crate) style_applied: bool,
    pub(crate) style_signature: NativeGlassStyleSignature,
    pub(crate) morph_started: bool,
    /// `f64::to_bits` of the visible entry start alpha the morph launched
    /// with (bits keep the receipt `Eq`). `None` when no morph ran.
    pub(crate) morph_start_alpha_bits: Option<u64>,
    /// Fade duration in ms; every material, content, and alpha channel has
    /// settled by then.
    pub(crate) settle_duration_ms: u64,
    pub(crate) configured_at_monotonic_ns: u64,
    pub(crate) configured_at_unix_ms: u64,
}

/// The main-entry receipt is native-observed state, separate from the
/// catalog's selected spec. It survives a hide/reopen until the next entry.
#[cfg(target_os = "macos")]
#[derive(Clone, Copy)]
struct MainGlassEntryRuntimeReceipt {
    window_key: usize,
    window_number: i64,
    preset_id: &'static str,
    duration_ms: u32,
    window_start_alpha: f64,
    material_start_alpha: f64,
    backdrop_start_alpha_effective: f64,
    content_start_alpha: f64,
    main_content_start_alpha_effective_range: Option<[f64; 2]>,
    capsule_label_start_alpha_effective_range: Option<[f64; 2]>,
    capsule_label_count: usize,
    capsule_inner_start_fraction: f64,
    backdrop_blur_radius: f64,
    capsule_blur_radius: f64,
    foreground_blur_radius: f64,
    foreground_root_count: usize,
    foreground_blurred_root_count: usize,
    capsule_count: usize,
    capsule_blurred_count: usize,
    capsule_material_count: usize,
    material_supported: bool,
    expected_frame: [f64; 4],
    actual_frame: Option<[f64; 4]>,
    actual_alpha: Option<f64>,
    post_hide_alpha: Option<f64>,
    effects_restored: Option<bool>,
    frame_unchanged: Option<bool>,
    outcome: &'static str,
    configured_at_host_time_ns: u64,
    cancelled_at_host_time_ns: Option<u64>,
    actual_end_host_time_ns: Option<u64>,
}

#[cfg(target_os = "macos")]
static MAIN_GLASS_ENTRY_RUNTIME_RECEIPT: std::sync::Mutex<Option<MainGlassEntryRuntimeReceipt>> =
    std::sync::Mutex::new(None);

/// Actual native main-window entry, not the live catalog preference. The
/// selected preset may change while this previously captured entry is in
/// flight; this receipt continues describing the entry that really ran.
#[cfg(target_os = "macos")]
pub fn main_glass_entry_lifecycle_receipt() -> serde_json::Value {
    let receipt = {
        let guard = MAIN_GLASS_ENTRY_RUNTIME_RECEIPT
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard
    };
    let Some(receipt) = receipt else {
        return serde_json::json!({"scope":"main_window_entry","outcome":"not_started"});
    };
    // Only the main AppKit thread may read NSWindow properties. Registry
    // lookup checks NSApp's live window inventory before returning a pointer;
    // number AND pointer must match this captured entry, not a reopen.
    let sample = if is_main_thread() {
        unsafe {
            crate::window_manager::get_main_window().and_then(|window| {
                let number: i64 = msg_send![window, windowNumber];
                if number != receipt.window_number || window as usize != receipt.window_key {
                    return None;
                }
                let at_ns = crate::platform::host_clock::host_time_ns();
                let frame: cocoa::foundation::NSRect = msg_send![window, frame];
                let alpha: f64 = msg_send![window, alphaValue];
                Some((
                    at_ns,
                    [
                        frame.origin.x,
                        frame.origin.y,
                        frame.size.width,
                        frame.size.height,
                    ],
                    alpha,
                ))
            })
        }
    } else {
        None
    };
    let sample_host_time_ns = sample.map(|(at_ns, _, _)| at_ns);
    let current_native_frame = sample.map(|(_, frame, _)| frame);
    let current_native_alpha = sample.map(|(_, _, alpha)| alpha);
    serde_json::json!({
        "scope": "main_window_entry",
        "windowNumber": receipt.window_number,
        "sampleKind": "nswindow_model_properties_not_pixels",
        "sampleHostTimeNs": sample_host_time_ns,
        "currentNativeFrame": current_native_frame,
        "currentNativeAlpha": current_native_alpha,
        "liveWindowIdentityMatched": sample.is_some(),
        "presetId": receipt.preset_id,
        "outcome": receipt.outcome,
        "durationMs": receipt.duration_ms,
        "windowStartAlpha": receipt.window_start_alpha,
        "materialStartAlpha": receipt.material_start_alpha,
        "backdropStartAlphaEffective": receipt.backdrop_start_alpha_effective,
        "contentStartAlpha": receipt.content_start_alpha,
        "presenceSampleKind": "nsview_model_alpha_and_capsule_content_product_not_pixels",
        "mainContentStartAlphaEffectiveRange": receipt.main_content_start_alpha_effective_range,
        "capsuleLabelStartAlphaEffectiveRange": receipt.capsule_label_start_alpha_effective_range,
        "capsuleLabelCount": receipt.capsule_label_count,
        "capsuleInnerStartFraction": receipt.capsule_inner_start_fraction,
        "backdropBlurRadius": receipt.backdrop_blur_radius,
        "capsuleBlurRadius": receipt.capsule_blur_radius,
        "foregroundBlurRadius": receipt.foreground_blur_radius,
        "foregroundRootCount": receipt.foreground_root_count,
        "foregroundBlurredRootCount": receipt.foreground_blurred_root_count,
        "capsuleCount": receipt.capsule_count,
        "capsuleBlurredCount": receipt.capsule_blurred_count,
        "capsuleMaterialCount": receipt.capsule_material_count,
        "materialSupported": receipt.material_supported,
        "expectedFinalFrame": receipt.expected_frame,
        "actualFinalFrame": receipt.actual_frame,
        "actualFinalAlpha": receipt.actual_alpha,
        "postHideAlpha": receipt.post_hide_alpha,
        "frameUnchanged": receipt.frame_unchanged,
        "effectsRestored": receipt.effects_restored,
        "configuredAtHostTimeNs": receipt.configured_at_host_time_ns,
        "expectedSettleDeadlineNs": receipt.configured_at_host_time_ns
            .saturating_add(u64::from(receipt.duration_ms) * 1_000_000),
        "cancelledAtHostTimeNs": receipt.cancelled_at_host_time_ns,
        "actualEndHostTimeNs": receipt.actual_end_host_time_ns,
    })
}

#[cfg(not(target_os = "macos"))]
pub fn main_glass_entry_lifecycle_receipt() -> serde_json::Value {
    serde_json::json!({"scope":"main_window_entry","outcome":"not_available"})
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug)]
struct TahoeGlassBackdropResult {
    created: bool,
    found_or_created: bool,
    native_selectors_supported: bool,
    style_applied: bool,
    style_signature: NativeGlassStyleSignature,
}

/// Alpha for TRULY HIDDEN parking only (window ordered out between shows).
/// Zero-alpha parking of a visible window is a contract violation — the
/// park helpers runtime-check `isVisible == false` before applying this.
#[cfg(target_os = "macos")]
const GLASS_HIDDEN_PARK_ALPHA: f64 = 0.0;
/// The floating footer overlay re-presents on the main entry's clock with no
/// hold before its own fade.
#[cfg(target_os = "macos")]
const GLASS_ENTRY_CONTENT_HOLD_DURATION: f64 = 0.0;
#[cfg(target_os = "macos")]
const GLASS_ENTRY_CONTENT_FADE_DURATION: f64 = 0.088;
/// Every window exits with the same fixed-frame alpha fade.
#[cfg(target_os = "macos")]
const GLASS_EXIT_DURATION: f64 = 0.12;
const GLASS_EXIT_REMOVE_DELAY_MS: u64 = 135;

/// Defocus radius, in points, the floating footer overlay resolves out of
/// when it re-presents.
#[cfg(target_os = "macos")]
const GLASS_ENTRY_BLUR_RADIUS: f64 = 8.0;

/// Live override for the footer overlay defocus radius, in points:
/// `SCRIPT_KIT_GLASS_ENTRY_BLUR=16 script-kit-gpui`. `0` disables it.
#[cfg(target_os = "macos")]
fn glass_entry_blur_radius() -> f64 {
    std::env::var("SCRIPT_KIT_GLASS_ENTRY_BLUR")
        .ok()
        .and_then(|raw| raw.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.0 && *value <= 64.0)
        .unwrap_or(GLASS_ENTRY_BLUR_RADIUS)
}

/// Footer overlay fade duration; `SCRIPT_KIT_GLASS_CONTENT_FADE=<ms>`
/// overrides it.
#[cfg(target_os = "macos")]
fn glass_entry_content_fade_duration() -> f64 {
    std::env::var("SCRIPT_KIT_GLASS_CONTENT_FADE")
        .ok()
        .and_then(|raw| raw.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.0 && *value <= 2000.0)
        .map(|ms| ms / 1000.0)
        .unwrap_or(GLASS_ENTRY_CONTENT_FADE_DURATION)
}

pub fn glass_exit_remove_delay() -> std::time::Duration {
    std::time::Duration::from_millis(GLASS_EXIT_REMOVE_DELAY_MS)
}

include!("secondary_window_glass_lifecycle.rs");

// SAFETY: Caller must pass a valid NSWindow pointer on the main thread.
// The function nil-checks all derived pointers (content view, appearance).
#[cfg(target_os = "macos")]
unsafe fn configure_window_vibrancy_common(
    window: id,
    log_target: &str,
    window_name: &str,
    is_dark: bool,
    backdrop_role: NativeGlassSurfaceRole,
) -> NativeGlassEntryReceipt {
    configure_window_vibrancy_common_impl(window, log_target, window_name, is_dark, backdrop_role)
}

include!("secondary_window_vibrancy_impl.rs");

#[cfg(target_os = "macos")]
fn current_window_material() -> crate::theme::VibrancyMaterial {
    crate::theme::get_cached_theme().get_vibrancy().material
}

#[cfg(target_os = "macos")]
fn current_window_material_name(material: crate::theme::VibrancyMaterial) -> &'static str {
    match material {
        crate::theme::VibrancyMaterial::Hud => "HUD_WINDOW",
        crate::theme::VibrancyMaterial::Popover => "POPOVER",
        crate::theme::VibrancyMaterial::Menu => "MENU",
        crate::theme::VibrancyMaterial::Sidebar => "SIDEBAR",
        crate::theme::VibrancyMaterial::Content => "CONTENT_BACKGROUND",
    }
}

#[cfg(target_os = "macos")]
fn tahoe_liquid_glass_class() -> Option<id> {
    // NSGlassEffectView is the AppKit Liquid Glass API introduced in macOS 26
    // Tahoe, so class availability is the capability gate.
    #[link(name = "Foundation", kind = "framework")]
    extern "C" {
        fn NSClassFromString(a_class_name: id) -> id;
    }

    let glass_class_name: id =
        unsafe { msg_send![class!(NSString), stringWithUTF8String: c"NSGlassEffectView".as_ptr()] };
    let glass_class = if glass_class_name.is_null() {
        cocoa::base::nil
    } else {
        unsafe { NSClassFromString(glass_class_name) }
    };
    if glass_class.is_null() {
        None
    } else {
        Some(glass_class)
    }
}

#[cfg(target_os = "macos")]
#[allow(dead_code)] // Kept for reference: the blur-era themed glass tint.
unsafe fn liquid_glass_tint_color() -> id {
    let theme = crate::theme::get_cached_theme();
    let rgba = crate::ui_foundation::main_window_matched_background_rgba(&theme);
    let red = ((rgba >> 24) & 0xff) as f64 / 255.0;
    let green = ((rgba >> 16) & 0xff) as f64 / 255.0;
    let blue = ((rgba >> 8) & 0xff) as f64 / 255.0;
    let alpha = (rgba & 0xff) as f64 / 255.0;
    msg_send![
        class!(NSColor),
        colorWithCalibratedRed: red
        green: green
        blue: blue
        alpha: alpha
    ]
}

/// True when the AppKit Liquid Glass API (macOS 26 Tahoe) is available, i.e.
/// the tagged NSGlassEffectView backdrop can actually render. Window creation
/// uses this to pick `Transparent` (the glass backdrop supplies the material)
/// over `Blurred`, whose full-window NSVisualEffectView would sit above the
/// backmost glass view and hide it entirely.
#[cfg(target_os = "macos")]
pub fn tahoe_liquid_glass_available() -> bool {
    // Cached: also consulted per-render by the veil-opacity resolver.
    static AVAILABLE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *AVAILABLE.get_or_init(|| tahoe_liquid_glass_class().is_some())
}

#[cfg(not(target_os = "macos"))]
pub fn tahoe_liquid_glass_available() -> bool {
    false
}

/// True only when the complete one-window Liquid Glass composition can be
/// installed. The debug no-glass switch is intentionally part of this gate so
/// fallback launches do not keep the detached footer inset or transparent
/// window background after the native backdrop has been disabled.
#[cfg(target_os = "macos")]
pub fn tahoe_native_glass_composition_available() -> bool {
    native_glass_composition_gate(
        tahoe_liquid_glass_available(),
        crate::platform::glass_button_host::native_glass_container_available(),
        std::env::var("SCRIPT_KIT_DEBUG_NO_GLASS").is_ok(),
    )
}

fn native_glass_composition_gate(
    effect_view_available: bool,
    container_view_available: bool,
    debug_no_glass: bool,
) -> bool {
    effect_view_available && container_view_available && !debug_no_glass
}

#[cfg(not(target_os = "macos"))]
pub fn tahoe_native_glass_composition_available() -> bool {
    false
}

/// Background appearance for a vibrancy-enabled window: `Transparent` when
/// the Tahoe glass backdrop supplies the material (a `Blurred` appearance
/// would stack the gpui fork's NSVisualEffectView above the glass and hide
/// it), `Blurred` otherwise.
pub fn vibrancy_window_background() -> gpui::WindowBackgroundAppearance {
    if tahoe_native_glass_composition_available() {
        gpui::WindowBackgroundAppearance::Transparent
    } else {
        gpui::WindowBackgroundAppearance::Blurred
    }
}

/// Resolve the NSWindow behind a live GPUI window and run the shared
/// secondary vibrancy/glass configuration on it (glass backdrop, VEV
/// handling, glass-mode window base). For overlay windows (confirm popup,
/// AI, flow manager) that have no dedicated native config path.
#[cfg(target_os = "macos")]
pub fn configure_overlay_window_glass(window: &gpui::Window, window_name: &str) {
    let Ok(handle) = raw_window_handle::HasWindowHandle::window_handle(window) else {
        return;
    };
    let raw_window_handle::RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        return;
    };
    let ns_view = appkit.ns_view.as_ptr() as id;
    let is_dark = crate::theme::get_cached_theme().should_use_dark_vibrancy();
    // SAFETY: ns_view belongs to the live GPUI window on the main thread;
    // `-[NSView window]` is standard and the result is nil-checked inside
    // configure_secondary_window_vibrancy.
    unsafe {
        let ns_window: id = msg_send![ns_view, window];
        if ns_window.is_null() {
            return;
        }
        configure_secondary_window_vibrancy(ns_window, window_name, is_dark);
    }
}

/// Configure a GPUI overlay that will be attached to a parent NSWindow.
#[cfg(target_os = "macos")]
pub fn configure_child_attached_overlay_window_glass(
    window: &gpui::Window,
    log_target: &str,
    window_name: &str,
) {
    let Ok(handle) = raw_window_handle::HasWindowHandle::window_handle(window) else {
        return;
    };
    let raw_window_handle::RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        return;
    };
    let ns_view = appkit.ns_view.as_ptr() as id;
    let is_dark = crate::theme::get_cached_theme().should_use_dark_vibrancy();
    unsafe {
        let ns_window: id = msg_send![ns_view, window];
        if ns_window.is_null() {
            return;
        }
        let backdrop_role = if window_name_uses_popup_backdrop(window_name) {
            NativeGlassSurfaceRole::PopupBackdrop
        } else {
            NativeGlassSurfaceRole::WindowBackdrop
        };
        configure_window_vibrancy_common(ns_window, log_target, window_name, is_dark, backdrop_role);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_child_attached_overlay_window_glass(
    _window: &gpui::Window,
    _log_target: &str,
    _window_name: &str,
) {
}

#[cfg(not(target_os = "macos"))]
pub fn configure_overlay_window_glass(_window: &gpui::Window, _window_name: &str) {}

include!("secondary_window_glass_backdrop.rs");
/// Pass-through hit test: the backdrop never participates in input so it can
/// never steal clicks/scrolls from GPUI content or the footer trio. Mirrors
/// the existing `ScriptKitFooterPassthroughView` principle.
#[cfg(target_os = "macos")]
extern "C" fn tahoe_glass_backdrop_hit_test(
    _this: &objc::runtime::Object,
    _: objc::runtime::Sel,
    _: cocoa::foundation::NSPoint,
) -> id {
    cocoa::base::nil
}

include!("secondary_window_glass_animation.rs");
include!("secondary_window_glass_style.rs");
/// Install (or reuse) a native macOS 26 Tahoe `NSGlassEffectView` as the
/// backmost backdrop of the window's content view.
///
/// Design (Oracle-Session tahoe-native-glass-backdrop):
/// - Gated on `NSGlassEffectView` availability; no-op on older macOS.
/// - A dedicated, tagged, pass-through subclass inserted as a BACKMOST SIBLING
///   of content via `addSubview:positioned:NSWindowBelow relativeTo:nil`. It is
///   NOT a content wrapper (`setContentView:`), so the main-window footer blur
///   trio (NSVisualEffectView + hitTest:nil + transparent hitbox) is untouched.
/// - Idempotent: repeated configure passes find the same tagged view via
///   `viewWithTag:` instead of stacking duplicates.
/// - The view is NOT an NSVisualEffectView, so the vibrancy recursion that runs
///   before this call never reconfigures it as blur material.
///
/// # Safety
/// `window` must be a valid NSWindow on the main thread (checked + null-guarded).
#[cfg(target_os = "macos")]
unsafe fn configure_tahoe_window_backdrop(window: id, log_target: &str, window_name: &str) -> bool {
    configure_tahoe_window_backdrop_with_result(
        window,
        log_target,
        window_name,
        NativeGlassSurfaceRole::WindowBackdrop,
    )
    .is_some_and(|result| result.created)
}

#[cfg(target_os = "macos")]
unsafe fn configure_tahoe_window_backdrop_with_result(
    window: id,
    log_target: &str,
    window_name: &str,
    backdrop_role: NativeGlassSurfaceRole,
) -> Option<TahoeGlassBackdropResult> {
    use cocoa::appkit::{NSViewHeightSizable, NSViewWidthSizable};
    use cocoa::foundation::NSRect;

    if window.is_null() {
        return None;
    }
    if require_main_thread("configure_tahoe_window_backdrop") {
        return None;
    }

    // Reconcile capability loss as well as installation. In particular, the
    // debug no-glass launch must remove any stale inset backdrop and restore
    // the ordinary full-window shadow/fallback material.
    if !tahoe_native_glass_composition_available()
        || !crate::theme::get_cached_theme().is_vibrancy_enabled()
    {
        remove_tahoe_window_backdrop(window, window_name);
        logging::log(
            log_target,
            &format!(
                "{}: Tahoe glass composition unavailable or disabled; restored fallback backdrop",
                window_name
            ),
        );
        return None;
    }

    let Some(glass_class) = tahoe_liquid_glass_class() else {
        logging::log(
            log_target,
            &format!(
                "{}: Tahoe NSGlassEffectView unavailable; native glass backdrop skipped",
                window_name
            ),
        );
        return None;
    };

    let content_view: id = msg_send![window, contentView];
    if content_view == nil {
        logging::log(
            log_target,
            &format!(
                "WARNING: {} has no contentView; Tahoe native glass backdrop skipped",
                window_name
            ),
        );
        return None;
    }

    let content_bounds: NSRect = msg_send![content_view, bounds];
    let mut backdrop_layout = tahoe_backdrop_layout(window_name);
    let backdrop_frame = backdrop_layout.frame(content_bounds);
    let vev_count_before =
        tahoe_count_views_kind_of_excluding(content_view, class!(NSVisualEffectView), nil);

    let mut created = false;
    let mut glass_view: id = msg_send![content_view, viewWithTag: TAHOE_GLASS_BACKDROP_TAG];
    if glass_view != nil {
        let is_glass: bool = msg_send![glass_view, isKindOfClass: glass_class];
        let superview: id = msg_send![glass_view, superview];
        if !is_glass || superview != content_view {
            logging::log(
                log_target,
                &format!(
                    "WARNING: {}: Tahoe glass backdrop tag collision (is_glass={}, direct_child={}); skipped",
                    window_name,
                    is_glass,
                    superview == content_view
                ),
            );
            return None;
        }
        // Appearance/theme refresh over an existing Notes backdrop must
        // preserve the dynamically selected inset (Agent mode reserves the
        // footer band via `set_gpui_window_backdrop_bottom_inset`); resetting
        // to the static default would silently collapse the Agent partition.
        if window_name == "Notes" {
            let existing_inset: f64 = *(*(glass_view as *const objc::runtime::Object))
                .get_ivar::<f64>("_scriptKitBottomInset");
            backdrop_layout = if existing_inset > 0.0 {
                TahoeBackdropLayout::ContentAboveDetachedFooter {
                    bottom_inset: existing_inset,
                }
            } else {
                TahoeBackdropLayout::FullWindow
            };
        }
    } else {
        let Some(backdrop_class) = tahoe_glass_backdrop_view_class(glass_class) else {
            logging::log(
                log_target,
                &format!(
                    "WARNING: {}: failed to register ScriptKitTahoeGlassBackdropView",
                    window_name
                ),
            );
            return None;
        };
        let allocated: id = msg_send![backdrop_class, alloc];
        glass_view = msg_send![allocated, initWithFrame: backdrop_frame];
        if glass_view == nil {
            logging::log(
                log_target,
                &format!(
                    "WARNING: {}: failed to allocate NSGlassEffectView backdrop",
                    window_name
                ),
            );
            return None;
        }
        let identifier = tahoe_ns_string(TAHOE_GLASS_BACKDROP_IDENTIFIER);
        if identifier != nil {
            let _: () = msg_send![glass_view, setIdentifier: identifier];
        }
        let _: () =
            msg_send![glass_view, setAutoresizingMask: NSViewWidthSizable | NSViewHeightSizable];
        let _: () = msg_send![
            content_view,
            addSubview: glass_view
            positioned: NS_WINDOW_BELOW
            relativeTo: nil
        ];
        // The superview now owns the view. Balance `alloc` so teardown via
        // removeFromSuperview can release it instead of leaking a native
        // object for every main-window lifetime.
        let _: () = msg_send![glass_view, release];
        created = true;
    }

    let _: () =
        msg_send![glass_view, setAutoresizingMask: NSViewWidthSizable | NSViewHeightSizable];

    let style = resolve_native_glass_style(&crate::theme::get_cached_theme(), backdrop_role);
    let tint_selector_supported: bool =
        msg_send![glass_view, respondsToSelector: sel!(setTintColor:)];
    let corner_selector_supported: bool =
        msg_send![glass_view, respondsToSelector: sel!(setCornerRadius:)];
    let native_selectors_supported = tint_selector_supported && corner_selector_supported;
    // A reconfigure pass over an existing backdrop is a theme/appearance
    // refresh, not a new installation.
    let tint_applied = apply_native_glass_style_with_reason(
        glass_view,
        style,
        if created {
            NativeGlassStyleApplicationReason::Install
        } else {
            NativeGlassStyleApplicationReason::ThemeRefresh
        },
    );

    let corner_radius =
        tahoe_backdrop_corner_radius_for(window_name, backdrop_layout, content_view);
    let corner_applied = {
        let responds: bool = msg_send![glass_view, respondsToSelector: sel!(setCornerRadius:)];
        if responds {
            let _: () = msg_send![glass_view, setCornerRadius: corner_radius];
            true
        } else {
            false
        }
    };

    // The physical main NSWindow is only a transparent composition host. A
    // native full-window shadow would reveal that host as one rectangular
    // slab and visually bridge the main material to the footer capsules.
    // Cast any depth from the bounded backdrop layer instead.
    update_tahoe_backdrop_geometry_and_shadow(
        window,
        content_view,
        glass_view,
        backdrop_layout,
        corner_radius,
    );

    tahoe_pin_glass_backdrop_backmost(content_view, glass_view);
    let _: () = msg_send![glass_view, setNeedsDisplay: true];

    let vev_count_after =
        tahoe_count_views_kind_of_excluding(content_view, class!(NSVisualEffectView), glass_view);
    let (glass_count, glass_index, subview_count) =
        tahoe_glass_subview_audit(content_view, glass_class);
    let backmost = tahoe_glass_backdrop_is_backmost(content_view, glass_view);
    let index_label = glass_index
        .map(|index| index.to_string())
        .unwrap_or_else(|| "none".to_string());

    logging::log(
        log_target,
        &format!(
            "{}: Tahoe NSGlassEffectView backdrop {} (glass_count={}, backmost={}, index={}, subviews={}, frame=({:.1},{:.1},{:.1},{:.1}), tint_applied={}, corner_applied={}, corner_radius={:.1}, vev_before={}, vev_after_excl_glass={})",
            window_name,
            if created { "installed" } else { "reused" },
            glass_count,
            backmost,
            index_label,
            subview_count,
            backdrop_frame.origin.x,
            backdrop_frame.origin.y,
            backdrop_frame.size.width,
            backdrop_frame.size.height,
            tint_applied,
            corner_applied,
            corner_radius,
            vev_count_before,
            vev_count_after,
        ),
    );

    if glass_count != 1 || !backmost || vev_count_before != vev_count_after {
        logging::log(
            log_target,
            &format!(
                "WARNING: {}: Tahoe glass backdrop audit FAILED (glass_count={}, backmost={}, vev_before={}, vev_after_excl_glass={})",
                window_name, glass_count, backmost, vev_count_before, vev_count_after
            ),
        );
    }

    Some(TahoeGlassBackdropResult {
        created,
        found_or_created: glass_count == 1 && backmost,
        native_selectors_supported,
        style_applied: tint_applied,
        style_signature: style.signature,
    })
}

/// Build an autoreleased NSString from a Rust `&str` (nil on interior NUL).
#[cfg(target_os = "macos")]
fn tahoe_ns_string(text: &str) -> id {
    let Ok(c_string) = std::ffi::CString::new(text) else {
        return nil;
    };
    // SAFETY: `c_string` is a valid NUL-terminated C string for this call.
    unsafe { msg_send![class!(NSString), stringWithUTF8String: c_string.as_ptr()] }
}

#[cfg(not(target_os = "macos"))]
fn configure_tahoe_window_backdrop(
    _window: *mut std::ffi::c_void,
    _log_target: &str,
    _window_name: &str,
) -> bool {
    false
}

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
unsafe fn configure_attached_popup_window(
    window: id,
    is_dark: bool,
    log_target: &str,
    window_name: &str,
    backdrop_role: NativeGlassSurfaceRole,
) {
    if window.is_null() {
        tracing::warn!(
            event = "attached_popup_configure.null_window",
            window_name,
            "Cannot configure null attached popup window"
        );
        return;
    }

    // Disable window dragging
    let _: () = msg_send![window, setMovable: false];
    let _: () = msg_send![window, setMovableByWindowBackground: false];

    // Popups are content-sized by GPUI (`set_inline_popup_window_bounds` / actions
    // resize helpers). Strip AppKit's resizable style mask so edge drags cannot
    // override the computed height.
    const NS_WINDOW_STYLE_MASK_RESIZABLE: u64 = 1 << 3;
    let style_mask: u64 = msg_send![window, styleMask];
    let non_resizable_mask = style_mask & !NS_WINDOW_STYLE_MASK_RESIZABLE;
    let _: () = msg_send![window, setStyleMask: non_resizable_mask];

    // Regression guard:
    // Detached child popups can still take mouse focus even when GPUI opens them
    // with `focus: false`. If AppKit promotes the child to the key panel on click,
    // the parent panel visually drops its active shadow even though our close/focus
    // policy keeps it open. `setBecomesKeyOnlyIfNeeded:true` keeps these popup
    // windows in the "clickable child" role instead of eagerly stealing key status.
    //
    // Keep this for Actions-style child popups unless we intentionally rework the
    // parent/child focus model and verify the shadow behavior again.
    let _: () = msg_send![window, setBecomesKeyOnlyIfNeeded: true];

    // Keep the level GPUI assigned (WindowKind::PopUp → NSPopUpMenuWindowLevel = 101).
    // Do NOT call setLevel here — any override downgrades the popup below the
    // main window which is also at 101. See CLAUDE.md "Window Level Rules".

    // NOTE: We intentionally do NOT set setHidesOnDeactivate:true here.
    // The main window is a non-activating panel (WindowKind::PopUp), so the app
    // is never "active" in the macOS sense. If we set hidesOnDeactivate, the
    // actions popup would immediately hide since the app isn't active.
    // Instead, we manage visibility ourselves via close_actions_window().

    // Disable close animation (NSWindowAnimationBehaviorNone = 2)
    // This prevents the white flash on dismiss
    let _: () = msg_send![window, setAnimationBehavior: NS_WINDOW_ANIMATION_BEHAVIOR_NONE];

    // Disable restoration
    let _: () = msg_send![window, setRestorable: false];

    // Disable frame autosave
    let empty_string: id = msg_send![class!(NSString), string];
    let _: () = msg_send![window, setFrameAutosaveName: empty_string];

    configure_window_vibrancy_common(window, log_target, window_name, is_dark, backdrop_role);

    // SAFETY: `window` is a valid, non-null NSWindow pointer (checked at function entry).
    // orderFrontRegardless brings the popup visually above the main panel without
    // activating the app — same pattern as show_main_window_without_activation.
    let _: () = msg_send![window, orderFrontRegardless];
}

/// Configure the Actions popup without activating its parent application.
///
/// # Safety
/// `window` must be a valid, live `NSWindow` pointer, and this function must
/// be called on the AppKit main thread while the window remains owned.
#[cfg(target_os = "macos")]
pub unsafe fn configure_actions_popup_window(window: id, is_dark: bool) {
    configure_attached_popup_window(
        window,
        is_dark,
        "ACTIONS",
        "Actions popup",
        NativeGlassSurfaceRole::PopupBackdrop,
    );
}

#[cfg(not(target_os = "macos"))]
pub fn configure_actions_popup_window(_window: *mut std::ffi::c_void, _is_dark: bool) {
    // No-op on non-macOS platforms
}

/// Configure an Agent Chat inline dropdown popup window with the same vibrancy path as
/// actions popups, including the detached AppKit shadow.
///
/// # Safety
/// Same invariants as `configure_actions_popup_window`.
#[cfg(target_os = "macos")]
pub unsafe fn configure_inline_dropdown_popup_window(window: id, is_dark: bool) {
    configure_attached_popup_window(
        window,
        is_dark,
        "POPUP",
        "Inline popup",
        NativeGlassSurfaceRole::WindowBackdrop,
    );

    // Inline dropdowns should read as native child popups with depth.
    let _: () = msg_send![window, setHasShadow: true];

    tracing::info!(
        target: "script_kit::popup",
        event = "inline_dropdown_popup_window_configured",
        dark = is_dark,
        "Configured inline dropdown popup window"
    );
}

#[cfg(not(target_os = "macos"))]
pub fn configure_inline_dropdown_popup_window(_window: *mut std::ffi::c_void, _is_dark: bool) {
    // No-op on non-macOS platforms
}

/// Configure the confirm popup window with the same vibrancy setup as the
/// actions popup. Reuses the shared popup vibrancy path so confirm dialogs
/// get native macOS blur.
///
/// # Safety
/// Same invariants as `configure_actions_popup_window`.
#[cfg(target_os = "macos")]
pub unsafe fn configure_confirm_popup_window(window: id, is_dark: bool) {
    configure_attached_popup_window(
        window,
        is_dark,
        "CONFIRM",
        "Confirm popup",
        NativeGlassSurfaceRole::PopupBackdrop,
    );
}

#[cfg(not(target_os = "macos"))]
pub fn configure_confirm_popup_window(_window: *mut std::ffi::c_void, _is_dark: bool) {
    // No-op on non-macOS platforms
}

/// Configure the child-attached shortcut recorder with its own morph receipt.
///
/// # Safety
/// `window` must be a valid, live `NSWindow` pointer, and this function must
/// be called on the AppKit main thread while the window remains owned.
#[cfg(target_os = "macos")]
pub unsafe fn configure_shortcut_recorder_popup_window(window: id, is_dark: bool) {
    configure_attached_popup_window(
        window,
        is_dark,
        "SHORTCUT",
        "Shortcut recorder popup",
        NativeGlassSurfaceRole::WindowBackdrop,
    );
}

#[cfg(not(target_os = "macos"))]
pub fn configure_shortcut_recorder_popup_window(_window: *mut std::ffi::c_void, _is_dark: bool) {}

/// Configure the launcher footer popup window. This uses the shared popup
/// vibrancy path, disables its shadow, and ignores mouse events so the launcher
/// content beneath it remains interactive.
///
/// # Safety
/// Same invariants as `configure_actions_popup_window`.
#[cfg(target_os = "macos")]
pub unsafe fn configure_footer_popup_window(window: id, is_dark: bool) {
    configure_attached_popup_window(
        window,
        is_dark,
        "ACTIONS",
        "Actions popup",
        NativeGlassSurfaceRole::WindowBackdrop,
    );
    let _: () = msg_send![window, setIgnoresMouseEvents: true];

    // SAFETY: `window` is a valid NSWindow. The footer popup sits flush with
    // the parent window, so remove native panel depth that is appropriate for
    // centered confirm/actions popups but wrong for the footer strip.
    let content_view: id = msg_send![window, contentView];
    if content_view != nil {
        let layer: id = msg_send![content_view, layer];
        if layer != nil {
            let _: () = msg_send![layer, setCornerRadius: 0.0_f64];
        }
        let _: () = msg_send![content_view, setWantsLayer: true];
        let layer: id = msg_send![content_view, layer];
        if layer != nil {
            let _: () = msg_send![layer, setCornerRadius: 0.0_f64];
        }
    }
    let _: () = msg_send![window, setHasShadow: false];

    let title: id = msg_send![
        class!(NSString),
        stringWithUTF8String: c"Script Kit Footer".as_ptr()
    ];
    if title != nil {
        let _: () = msg_send![window, setTitle: title];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_footer_popup_window(_window: *mut std::ffi::c_void, _is_dark: bool) {
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
    let _ = configure_secondary_window_vibrancy_with_receipt(window, window_name, is_dark);
}

#[cfg(target_os = "macos")]
pub(crate) unsafe fn configure_secondary_window_vibrancy_with_receipt(
    window: id,
    window_name: &str,
    is_dark: bool,
) -> Option<NativeGlassEntryReceipt> {
    if window.is_null() {
        logging::log(
            "PANEL",
            &format!(
                "WARNING: Cannot configure null window for {} vibrancy",
                window_name
            ),
        );
        return None;
    }

    Some(configure_window_vibrancy_common(
        window,
        "PANEL",
        window_name,
        is_dark,
        NativeGlassSurfaceRole::WindowBackdrop,
    ))
}

/// Configure the live dictation overlay with the shared native material path.
///
/// The title is intentionally stable so appearance refresh can find the overlay
/// without treating it as a main-window footer surface.
///
/// # Safety
///
/// - `window` must be a valid, non-null NSWindow pointer obtained from GPUI
///   window creation. The pointer is checked for null at entry.
/// - Must be called on the main thread because AppKit property setters are not
///   thread-safe.
#[cfg(target_os = "macos")]
pub unsafe fn configure_dictation_overlay_window(window: id, is_dark: bool) {
    if window.is_null() {
        logging::log(
            "DICTATION",
            "WARNING: Cannot configure null Dictation overlay window vibrancy",
        );
        return;
    }

    tracing::info!(
        category = "DICTATION",
        is_dark,
        "Configuring dictation overlay shared native material"
    );
    configure_window_vibrancy_common(
        window,
        "DICTATION",
        "Dictation overlay",
        is_dark,
        NativeGlassSurfaceRole::WindowBackdrop,
    );

    let title: id = msg_send![
        class!(NSString),
        stringWithUTF8String: c"Script Kit Dictation".as_ptr()
    ];
    if title != nil {
        let _: () = msg_send![window, setTitle: title];
        tracing::info!(
            category = "DICTATION",
            title = "Script Kit Dictation",
            "Set dictation overlay NSWindow title for material refresh"
        );
    } else {
        tracing::warn!(
            category = "DICTATION",
            "Failed to allocate dictation overlay NSWindow title"
        );
    }
}

/// Configure a HUD overlay with the same native background and material path as
/// the main window while preserving HUD-specific level and input behavior in the
/// caller.
///
/// # Safety
///
/// - `window` must be a valid, non-null NSWindow pointer obtained from GPUI
///   window creation. The pointer is checked for null at entry.
/// - Must be called on the main thread because AppKit property setters are not
///   thread-safe.
/// - NSAppearance and content view pointers are nil-checked before use.
#[cfg(target_os = "macos")]
pub unsafe fn configure_hud_window_vibrancy(window: id, is_dark: bool) {
    if window.is_null() {
        logging::log("HUD", "WARNING: Cannot configure null HUD window vibrancy");
        return;
    }

    configure_window_vibrancy_common(
        window,
        "HUD",
        "HUD",
        is_dark,
        NativeGlassSurfaceRole::WindowBackdrop,
    );

    let title: id = msg_send![
        class!(NSString),
        stringWithUTF8String: c"Script Kit HUD".as_ptr()
    ];
    if title != nil {
        let _: () = msg_send![window, setTitle: title];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn configure_secondary_window_vibrancy(
    _window: *mut std::ffi::c_void,
    _window_name: &str,
    _is_dark: bool,
) {
    // No-op on non-macOS platforms
}

#[cfg(not(target_os = "macos"))]
pub fn configure_dictation_overlay_window(_window: *mut std::ffi::c_void, _is_dark: bool) {
    // No-op on non-macOS platforms
}

#[cfg(not(target_os = "macos"))]
pub fn configure_hud_window_vibrancy(_window: *mut std::ffi::c_void, _is_dark: bool) {
    // No-op on non-macOS platforms
}

#[cfg(test)]
mod secondary_window_config_tests {
    include!("secondary_window_config_behavior_tests.rs");

    #[test]
    fn actions_popup_focus_shadow_contract_uses_becomes_key_only_if_needed() {
        let source = include_str!("secondary_window_config.rs");
        assert!(
            source.contains("setBecomesKeyOnlyIfNeeded: true"),
            "actions-style child popups must keep becomesKeyOnlyIfNeeded enabled so clicking them does not visually demote the parent window"
        );
    }

    #[test]
    fn actions_popup_strips_appkit_resizable_style_mask() {
        let source = include_str!("secondary_window_config.rs");
        assert!(
            source.contains("NS_WINDOW_STYLE_MASK_RESIZABLE")
                && source.contains("setStyleMask: non_resizable_mask"),
            "content-sized child popups must not keep AppKit edge-resize affordances"
        );
    }

    #[test]
    fn hud_window_vibrancy_reuses_main_window_material_source() {
        let source = include_str!("secondary_window_config.rs");
        let start = source
            .find("pub unsafe fn configure_hud_window_vibrancy")
            .expect("HUD vibrancy function exists");
        let body = &source[start..];
        let body = body
            .split("#[cfg(not(target_os = \"macos\"))]")
            .next()
            .expect("HUD vibrancy function body");

        assert!(
            body.contains("configure_window_vibrancy_common(")
                && body.contains("\"HUD\",")
                && body.contains("NativeGlassSurfaceRole::WindowBackdrop"),
            "HUD window vibrancy must reuse the shared native background/material configuration"
        );
        assert!(
            body.contains("c\"Script Kit HUD\".as_ptr()")
                && super::should_refresh_secondary_window_appearance("Script Kit HUD"),
            "HUD windows need a stable title so theme/appearance refresh can retint them with the shared material path"
        );
        assert!(
            source.contains("fn current_window_material()")
                && source.contains("get_cached_theme().get_vibrancy().material"),
            "shared native window configuration must source material from the cached theme"
        );
    }

    #[test]
    fn secondary_appearance_refresh_title_predicate_covers_current_and_legacy_secondary_titles() {
        for title in [
            "Notes",
            "Mini AI",
            "Script Kit Agent Chat",
            "Script Kit Agent Chat",
            "Script Kit Notes",
            "Actions",
            "Script Kit Footer",
            "Script Kit Dictation",
            "Script Kit HUD",
        ] {
            assert!(
                super::should_refresh_secondary_window_appearance(title),
                "appearance refresh must cover secondary window title: {title}"
            );
        }
    }

    #[test]
    fn secondary_appearance_refresh_title_predicate_rejects_generic_titles() {
        for title in [
            "Agent Chat",
            "Notes Archive",
            "Mini",
            "AI",
            "Script Kit",
            "Script Kit Main",
            "Random User Window",
        ] {
            assert!(
                !super::should_refresh_secondary_window_appearance(title),
                "appearance refresh predicate must not match generic/non-secondary title: {title}"
            );
        }
    }

    // Source-contract guards for the native Tahoe NSGlassEffectView backdrop
    // (Oracle-Session tahoe-native-glass-backdrop). These do not prove runtime
    // pixels; they prevent a later "simplification" from removing the
    // safety-critical idempotence / backmost-insertion / pass-through / footer
    // non-mutation properties.
    #[test]
    fn tahoe_glass_backdrop_source_contract_is_native_idempotent_backmost_and_passthrough() {
        let source = include_str!("secondary_window_config.rs");
        assert!(
            source.contains("tahoe_liquid_glass_class()"),
            "Tahoe glass must remain gated by NSClassFromString availability"
        );
        assert!(
            source.contains("ScriptKitTahoeGlassBackdropView"),
            "Tahoe glass backdrop must use a dedicated subclass"
        );
        assert!(
            source.contains("viewWithTag: TAHOE_GLASS_BACKDROP_TAG"),
            "Tahoe glass backdrop must be idempotent via a stable tag lookup"
        );
        assert!(
            source.contains("positioned: NS_WINDOW_BELOW") && source.contains("relativeTo: nil"),
            "Tahoe glass backdrop must be inserted below all existing contentView subviews"
        );
        assert!(
            source.contains("tahoe_glass_backdrop_hit_test") && source.contains("cocoa::base::nil"),
            "Tahoe glass backdrop must be pass-through for hit testing"
        );
        assert!(
            source.contains("sel!(setTintColor:)") && source.contains("setTintColor: tint_color"),
            "Tahoe glass backdrop must apply the theme tint through NSGlassEffectView tintColor"
        );
        assert!(
            source.contains("sel!(setCornerRadius:)")
                && source.contains("setCornerRadius: corner_radius"),
            "Tahoe glass backdrop must apply content corner radius when the selector exists"
        );
    }

    #[test]
    fn tahoe_glass_backdrop_source_contract_does_not_mutate_footer_or_wrap_content() {
        let source = include_str!("secondary_window_config.rs");
        let start = source
            .find("unsafe fn configure_tahoe_window_backdrop")
            .expect("configure_tahoe_window_backdrop exists");
        let body = &source[start
            ..source[start..]
                .find("#[cfg(not(target_os = \"macos\"))]")
                .map(|offset| start + offset)
                .unwrap_or(source.len())];
        assert!(
            !body.contains("setContentView:"),
            "Tahoe backdrop must not wrap/reparent GPUI or footer content"
        );
        assert!(
            !body.contains("setIgnoresMouseEvents:"),
            "NSGlassEffectView must be made pass-through with hitTest:nil, not NSWindow-only ignoresMouseEvents"
        );
        assert!(
            !body.contains("setTag:"),
            "NSView tag is read-only; use subclass tag override instead"
        );
        assert!(
            !body.contains("setMaterial:")
                && !body.contains("setBlendingMode:")
                && !body.contains("setState:")
                && !body.contains("setEmphasized:"),
            "NSGlassEffectView must not be configured with NSVisualEffectView material selectors"
        );
        assert!(
            !body.contains("FOOTER_") && !body.contains("script-kit-footer-effect"),
            "Tahoe backdrop configuration must not special-case or mutate footer internals"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tahoe_glass_backdrop_ordering_constant_is_backmost() {
        assert_eq!(super::NS_WINDOW_BELOW, -1);
    }
}

fn should_refresh_secondary_window_appearance(title: &str) -> bool {
    const EXACT_SECONDARY_TITLES: &[&str] = &["Notes", "Mini AI", "Script Kit Agent Chat"];
    const EXISTING_SECONDARY_TITLE_MARKERS: &[&str] = &[
        "Script Kit Agent Chat",
        "Script Kit Notes",
        "Actions",
        "Script Kit Footer",
        "Script Kit Dictation",
        "Script Kit HUD",
    ];

    EXACT_SECONDARY_TITLES.contains(&title)
        || EXISTING_SECONDARY_TITLE_MARKERS
            .iter()
            .any(|marker| title.contains(marker))
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
#[allow(dead_code)]
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
            if should_refresh_secondary_window_appearance(&title_string) {
                // Clear window appearance so GPUI can detect system appearance changes.
                // Set appearance on individual NSVisualEffectViews instead.
                let _: () = msg_send![window, setAppearance: nil];

                // Walk view hierarchy and set appearance + material on each NSVisualEffectView
                let content_view: id = msg_send![window, contentView];
                if content_view != nil {
                    let mut vev_count = 0;
                    let material = current_window_material();
                    configure_visual_effect_views_recursive(
                        content_view,
                        &mut vev_count,
                        is_dark,
                        material,
                    );
                    configure_tahoe_window_backdrop(window, "APPEARANCE", &title_string);
                    logging::log(
                        "APPEARANCE",
                        &format!(
                            "Updated window '{}': cleared window appearance, configured {} NSVisualEffectView(s) for {} using {}",
                            title_string,
                            vev_count,
                            if is_dark { "dark" } else { "light" },
                            current_window_material_name(material),
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

include!("secondary_window_resize_policy.rs");

// Re-export display/coordinate helpers from the unified display module.
pub use self::display::{
    clamp_to_visible, display_for_point, flip_y, get_active_display, get_global_mouse_position,
    get_macos_displays, get_macos_visible_displays, prefers_reduced_motion, primary_screen_height,
    VisibleDisplayBounds,
};
