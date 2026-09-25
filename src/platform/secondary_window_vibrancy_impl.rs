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
    let entry_spec = preset.spec();
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
    let style_signature = backdrop
        .map(|result| result.style_signature)
        .unwrap_or_else(|| {
            resolve_native_glass_style(&crate::theme::get_cached_theme(), backdrop_role)
            .signature
        });
    let configured_at_monotonic_ns = crate::platform::host_clock::host_time_ns();
    let configured_at_unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let backdrop_found_or_created = backdrop.is_some_and(|result| result.found_or_created);
    let native_selectors_supported =
        backdrop.is_some_and(|result| result.native_selectors_supported);
    let style_applied = backdrop.is_some_and(|result| result.style_applied);
    NativeGlassEntryReceipt {
        window_number,
        configured: window_number > 0
            && content_view != nil
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
