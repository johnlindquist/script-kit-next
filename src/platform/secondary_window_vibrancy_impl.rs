#[cfg(target_os = "macos")]
unsafe fn configure_window_vibrancy_common_impl(
    window: id,
    log_target: &str,
    window_name: &str,
    is_dark: bool,
    backdrop_role: NativeGlassSurfaceRole,
) -> NativeGlassEntryReceipt {
    // Clear window appearance so GPUI can detect system appearance changes.
    // Appearance is set on individual NSVisualEffectViews instead.
    let _: () = msg_send![window, setAppearance: nil];
    logging::log(
        log_target,
        &format!(
            "{}: Cleared window appearance (nil) for {} mode; appearance set on views",
            window_name,
            if is_dark { "dark" } else { "light" }
        ),
    );

    // Use windowBackgroundColor for semi-opaque background — except in glass
    // mode, where that base renders UNDER the NSGlassEffectView backdrop and
    // dims the whole material; use the near-clear base instead (0.0001 alpha
    // keeps the window shadow machinery alive, unlike clearColor).
    let glass_mode = tahoe_native_glass_composition_available()
        && crate::theme::get_cached_theme().is_vibrancy_enabled();
    let window_bg_color: id = if glass_mode {
        msg_send![
            class!(NSColor),
            colorWithSRGBRed: 0.0f64 green: 0.0f64 blue: 0.0f64 alpha: 0.0001f64
        ]
    } else {
        msg_send![class!(NSColor), windowBackgroundColor]
    };
    let _: () = msg_send![window, setBackgroundColor: window_bg_color];
    logging::log(
        log_target,
        &format!(
            "{}: Set backgroundColor ({} base)",
            window_name,
            if glass_mode {
                "glass near-clear"
            } else {
                "windowBackgroundColor semi-opaque"
            }
        ),
    );

    // Mark window as non-opaque to allow transparency/vibrancy.
    let _: () = msg_send![window, setOpaque: false];

    // Enable shadow for native depth perception.
    let _: () = msg_send![window, setHasShadow: true];

    // Configure NSVisualEffectViews in the window hierarchy.
    let content_view: id = msg_send![window, contentView];
    if !content_view.is_null() {
        let mut count = 0;
        let material = current_window_material();
        configure_visual_effect_views_recursive(content_view, &mut count, is_dark, material);
        let material_name = current_window_material_name(material);
        logging::log(
            log_target,
            &format!(
                "{}: Configured {} NSVisualEffectView(s) with {} material",
                window_name, count, material_name
            ),
        );
    }

    let backdrop =
        configure_tahoe_window_backdrop_with_result(window, log_target, window_name, backdrop_role);
    let glass_created = backdrop.is_some_and(|result| result.created);
    // Secondary/overlay windows (notes, dictation, confirm, actions, AI,
    // flow manager, inline popups) are created per appearance, so a freshly
    // created backdrop means the window just appeared: fade it in.
    let preset = crate::theme::animation::current_preset();
    if glass_created {
        animate_tahoe_glass_entry(window, log_target, window_name, preset, backdrop_role);
    }

    let appearance_name = if is_dark {
        "VibrantDark"
    } else {
        "VibrantLight"
    };
    let material_name = current_window_material_name(current_window_material());
    logging::log(
        log_target,
        &format!(
            "{} vibrancy configured ({} + {} + blur)",
            window_name, appearance_name, material_name
        ),
    );
    let window_number: i64 = msg_send![window, windowNumber];
    let configured_at_monotonic_ns = crate::platform::host_clock::host_time_ns();
    let configured_at_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    native_glass_entry_receipt(
        window_number,
        content_view != nil,
        backdrop,
        || resolve_native_glass_style(&crate::theme::get_cached_theme(), backdrop_role).signature,
        preset.spec(),
        configured_at_monotonic_ns,
        configured_at_unix_ms,
    )
}

/// A window's glass is configured only when the Tahoe backdrop hook found or created its
/// backdrop and styled it; a backdrop the hook just created is what starts the entry fade.
#[cfg(target_os = "macos")]
fn native_glass_entry_receipt(
    window_number: i64,
    has_content_view: bool,
    backdrop: Option<TahoeGlassBackdropResult>,
    unstyled_signature: impl FnOnce() -> NativeGlassStyleSignature,
    entry_spec: crate::theme::animation::MainWindowAnimationSpec,
    configured_at_monotonic_ns: u64,
    configured_at_unix_ms: u64,
) -> NativeGlassEntryReceipt {
    let glass_created = backdrop.is_some_and(|result| result.created);
    let style_signature = backdrop
        .map(|result| result.style_signature)
        .unwrap_or_else(unstyled_signature);
    let backdrop_found_or_created = backdrop.is_some_and(|result| result.found_or_created);
    let native_selectors_supported =
        backdrop.is_some_and(|result| result.native_selectors_supported);
    let style_applied = backdrop.is_some_and(|result| result.style_applied);
    NativeGlassEntryReceipt {
        window_number,
        configured: window_number > 0
            && has_content_view
            && backdrop_found_or_created
            && native_selectors_supported
            && style_applied,
        backdrop_found_or_created,
        native_selectors_supported,
        style_applied,
        style_signature,
        morph_started: glass_created,
        morph_start_alpha_bits: glass_created.then(|| entry_spec.window_start_alpha.to_bits()),
        settle_duration_ms: if glass_created {
            u64::from(entry_spec.duration_ms)
        } else {
            0
        },
        configured_at_monotonic_ns,
        configured_at_unix_ms,
    }
}

#[cfg(all(test, target_os = "macos"))]
mod native_glass_entry_receipt_tests {
    use super::{
        native_glass_entry_receipt, resolve_native_glass_style, NativeGlassEntryReceipt,
        NativeGlassSurfaceRole, TahoeGlassBackdropResult,
    };
    use crate::theme::animation::MainWindowAnimationPreset;
    use crate::theme::Theme;

    fn signature(role: NativeGlassSurfaceRole) -> super::NativeGlassStyleSignature {
        resolve_native_glass_style(&Theme::dark_default(), role).signature
    }

    fn backdrop(created: bool, style_applied: bool) -> TahoeGlassBackdropResult {
        TahoeGlassBackdropResult {
            created,
            found_or_created: true,
            native_selectors_supported: true,
            style_applied,
            style_signature: signature(NativeGlassSurfaceRole::WindowBackdrop),
        }
    }

    fn receipt(
        window_number: i64,
        backdrop: Option<TahoeGlassBackdropResult>,
    ) -> NativeGlassEntryReceipt {
        native_glass_entry_receipt(
            window_number,
            true,
            backdrop,
            || signature(NativeGlassSurfaceRole::FloatingCapsule),
            MainWindowAnimationPreset::SharpPresence.spec(),
            11,
            22,
        )
    }

    #[test]
    fn a_backdrop_the_hook_just_created_is_configured_and_fades_in_on_the_preset() {
        let spec = MainWindowAnimationPreset::SharpPresence.spec();
        let receipt = receipt(7, Some(backdrop(true, true)));
        assert!(receipt.configured);
        assert!(receipt.morph_started);
        assert_eq!(
            receipt.morph_start_alpha_bits,
            Some(spec.window_start_alpha.to_bits())
        );
        assert_eq!(receipt.settle_duration_ms, u64::from(spec.duration_ms));
        assert_eq!(
            receipt.style_signature,
            signature(NativeGlassSurfaceRole::WindowBackdrop),
            "the receipt carries the style the hook applied"
        );
    }

    #[test]
    fn a_backdrop_the_hook_found_again_is_configured_without_a_second_fade() {
        let receipt = receipt(7, Some(backdrop(false, true)));
        assert!(receipt.configured);
        assert!(!receipt.morph_started);
        assert_eq!(receipt.morph_start_alpha_bits, None);
        assert_eq!(receipt.settle_duration_ms, 0);
    }

    #[test]
    fn without_a_styled_backdrop_from_the_hook_the_window_is_not_configured() {
        let none = receipt(7, None);
        assert!(!none.configured && !none.backdrop_found_or_created && !none.morph_started);
        assert_eq!(
            none.style_signature,
            signature(NativeGlassSurfaceRole::FloatingCapsule),
            "with no backdrop the receipt reports the unstyled role's signature"
        );
        assert!(!receipt(7, Some(backdrop(true, false))).configured);
        assert!(!receipt(0, Some(backdrop(true, true))).configured);
        assert!(
            !native_glass_entry_receipt(
                7,
                false,
                Some(backdrop(true, true)),
                || signature(NativeGlassSurfaceRole::WindowBackdrop),
                MainWindowAnimationPreset::SharpPresence.spec(),
                0,
                0,
            )
            .configured
        );
    }
}
