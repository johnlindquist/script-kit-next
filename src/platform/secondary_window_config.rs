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
}

/// Alpha for TRULY HIDDEN parking only (window ordered out between shows).
/// Zero-alpha parking of a visible window is a contract violation — the
/// park helpers runtime-check `isVisible == false` before applying this.
#[cfg(target_os = "macos")]
const GLASS_HIDDEN_PARK_ALPHA: f64 = 0.0;
/// Every window exits with the same fixed-frame alpha fade.
#[cfg(target_os = "macos")]
const GLASS_EXIT_DURATION: f64 = 0.12;
const GLASS_EXIT_REMOVE_DELAY_MS: u64 = 135;

pub fn glass_exit_remove_delay() -> std::time::Duration {
    std::time::Duration::from_millis(GLASS_EXIT_REMOVE_DELAY_MS)
}

include!("secondary_window_glass_lifecycle.rs");

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

    Some(TahoeGlassBackdropResult { created })
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

#[cfg(not(target_os = "macos"))]
#[cfg(test)]
mod secondary_window_config_tests {
    include!("secondary_window_config_behavior_tests.rs");

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

include!("secondary_window_resize_policy.rs");

// Re-export display/coordinate helpers from the unified display module.
pub use self::display::{
    clamp_to_visible, display_for_point, flip_y, get_active_display, get_global_mouse_position,
    get_macos_displays, get_macos_visible_displays, prefers_reduced_motion, primary_screen_height,
    VisibleDisplayBounds,
};
