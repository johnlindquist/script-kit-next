/// Main window entry. The preset is an atomic Copy snapshot; no later
/// callback of this entry reads the live selection again.
///
/// # Safety
/// `window` must be a valid NSWindow on the main thread.
#[cfg(target_os = "macos")]
unsafe fn animate_tahoe_glass_main_appearance(window: id, log_target: &str, window_name: &str) {
    animate_tahoe_glass_entry(
        window,
        log_target,
        window_name,
        crate::theme::animation::current_preset(),
        NativeGlassSurfaceRole::WindowBackdrop,
    )
}

/// The one entry every window runs: the NSWindow keeps its settled frame
/// while the glass material, GPUI content roots, floating footer capsules,
/// and window alpha rise together over `spec.duration_ms` (ease-out), with an
/// optional defocus resolving over the same span. Child windows stay attached
/// to their parent: nothing here animates a frame.
///
/// Main-only state (recent-start debounce, footer capsules, the runtime
/// receipt, hidden-park restore) is gated on the physical main NSWindow.
#[cfg(target_os = "macos")]
unsafe fn animate_tahoe_glass_entry(
    window: id,
    log_target: &str,
    window_name: &str,
    preset: crate::theme::animation::MainWindowAnimationPreset,
    backdrop_role: NativeGlassSurfaceRole,
) {
    use cocoa::foundation::NSRect;
    let spec = preset.spec();
    let is_main = crate::window_manager::get_main_window() == Some(window);
    if is_main {
        MAIN_COHERENT_ENTRY_PREPARING.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    // Every show is an exit supersession boundary. Alpha is PRESERVED here:
    // the entry installs its own start alpha below.
    cancel_ns_window_exit_dematerialize_impl(window, false);
    MAIN_COHERENT_ENTRY_PREPARING.store(false, std::sync::atomic::Ordering::Relaxed);
    if GLASS_COHERENT_ENTRY_STATES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .contains_key(&(window as usize))
    {
        if is_main {
            logging::log(
                log_target,
                &format!(
                    "event=glass_morph_skip window={} phase=enter preset={} reason=pending_hidden_restore",
                    window_name,
                    preset.id(),
                ),
            );
            return;
        }
        // Only the main window is hidden-parked, so nothing else will restore
        // a deferred (or pointer-reused stale) entry: restore it before arming.
        finish_coherent_glass_entry(window, "interrupted");
    }
    // The hide path parks the window at alpha 0, so every early exit must
    // restore alpha or a skipped entry would leave the window invisible.
    let skip = |reason: &str| {
        let _: () = msg_send![window, setAlphaValue: 1.0f64];
        logging::log(
            log_target,
            &format!(
                "event=glass_morph_skip window={} phase=enter preset={} reason={}",
                window_name,
                preset.id(),
                reason,
            ),
        );
    };
    if is_main && glass_morph_recently_started() {
        skip("recently_started_debounce");
        return;
    }
    let final_frame: NSRect = msg_send![window, frame];
    if final_frame.size.width < 40.0 || final_frame.size.height < 40.0 {
        skip("frame_too_small");
        return;
    }
    let content_view: id = msg_send![window, contentView];
    if content_view == nil {
        skip("no_content_view");
        return;
    }
    let backdrop: id = msg_send![content_view, viewWithTag: TAHOE_GLASS_BACKDROP_TAG];
    if backdrop == nil {
        skip("no_glass_view");
        return;
    }

    let native_window_number: i64 = msg_send![window, windowNumber];
    let mut state = GlassCoherentEntryState::new(backdrop, native_window_number);
    let Some(backdrop_natural_alpha) = state.capture_alpha(backdrop) else {
        skip("no_backdrop_alpha");
        return;
    };
    let backdrop_layer_captured = state.capture_layer(backdrop);
    let _: () = msg_send![window, setAlphaValue: spec.window_start_alpha];

    // Settle on the material the install applied for this surface's role
    // (popups resolve untinted: no washed-out tint snap).
    let resolved_style = resolve_native_glass_style(&crate::theme::get_cached_theme(), backdrop_role);
    let tint = native_glass_tint_color(&resolved_style);
    let style_supported: bool = msg_send![backdrop, respondsToSelector: sel!(setStyle:)];
    let tint_supported: bool = msg_send![backdrop, respondsToSelector: sel!(setTintColor:)];
    let material_supported = style_supported && tint_supported;
    let duration = f64::from(spec.duration_ms) / 1000.0;
    if material_supported {
        state.capture_style(backdrop);
        let _: () = msg_send![backdrop, setStyle: 1isize]; // Clear
        let _: () = msg_send![backdrop, setTintColor: nil];
    }
    let backdrop_start_alpha = backdrop_natural_alpha * spec.material_start_alpha;
    let _: () = msg_send![backdrop, setAlphaValue: backdrop_start_alpha];

    // The native detached footer is a separate GPUI sibling: never seed or
    // filter its gap-spanning container. Only the rounded glass capsules and
    // their own foreground labels participate in this reveal.
    let (footer_host, capsules) = if is_main {
        (
            crate::footer_popup::main_window_footer_entry_alpha_target(window),
            crate::footer_popup::main_window_footer_entry_capsules(window),
        )
    } else {
        (nil, Vec::new())
    };
    let capsule_count = capsules.len();
    let mut capsule_blur_count = 0usize;
    let mut capsule_material_count = 0usize;
    let mut content_roots = 0usize;
    let mut foreground_blur_count = 0usize;
    let mut content_targets = Vec::new();
    let mut main_start_alpha_min = f64::INFINITY;
    let mut main_start_alpha_max = 0.0f64;
    let mut capsule_label_start_min = f64::INFINITY;
    let mut capsule_label_start_max = 0.0f64;
    let mut capsule_label_count = 0usize;
    // The outer capsule already multiplies its label alpha. Seed its inner
    // foreground only for the additional content/material difference.
    let capsule_inner_start_fraction =
        (spec.content_start_alpha / spec.material_start_alpha).min(1.0);
    let subviews: id = msg_send![content_view, subviews];
    if subviews != nil {
        let count: usize = msg_send![subviews, count];
        for index in 0..count {
            let child: id = msg_send![subviews, objectAtIndex: index];
            if child == nil || child == backdrop || child == footer_host {
                continue;
            }
            let native_glass: bool = objc::runtime::Class::get("NSGlassEffectView")
                .is_some_and(|class| msg_send![child, isKindOfClass: class]);
            let native_container: bool = objc::runtime::Class::get("NSGlassEffectContainerView")
                .is_some_and(|class| msg_send![child, isKindOfClass: class]);
            let contains_footer: bool =
                footer_host != nil && msg_send![footer_host, isDescendantOf: child];
            if native_glass || native_container || contains_footer {
                continue;
            }
            let Some(alpha) = state.capture_alpha(child) else {
                continue;
            };
            state.capture_layer(child);
            if alpha <= 0.001 {
                continue;
            }
            let _: () = msg_send![child, setAlphaValue: alpha * spec.content_start_alpha];
            let measured: f64 = msg_send![child, alphaValue];
            main_start_alpha_min = main_start_alpha_min.min(measured);
            main_start_alpha_max = main_start_alpha_max.max(measured);
            content_targets.push((child, alpha));
            content_roots += 1;
            if spec.foreground_blur_radius > 0.0 && clip_entry_defocus_to_backdrop(child, backdrop)
            {
                state.mark_owned_mask(child);
                state.mark_owned_clip(child);
                let installed =
                    ramp_entry_defocus(child, duration, spec.foreground_blur_radius, log_target);
                state.mark_owned_filter(child);
                state.mark_owned_clip(child);
                if installed > 0.0 {
                    foreground_blur_count += 1;
                }
            }
        }
    }
    let mut capsule_targets = Vec::with_capacity(capsule_count);
    for &capsule in &capsules {
        let capsule_layer_captured = state.capture_layer(capsule);
        let style_ok: bool = msg_send![capsule, respondsToSelector: sel!(setStyle:)];
        let tint_ok: bool = msg_send![capsule, respondsToSelector: sel!(setTintColor:)];
        let natural_style = if style_ok && tint_ok {
            let style: isize = msg_send![capsule, style];
            let tint: id = msg_send![capsule, tintColor];
            state.capture_style(capsule);
            let _: () = msg_send![capsule, setStyle: 1isize];
            let _: () = msg_send![capsule, setTintColor: nil];
            capsule_material_count += 1;
            Some((style, tint))
        } else {
            None
        };
        let natural_alpha = state.capture_alpha(capsule);
        if let Some(alpha) = natural_alpha {
            let _: () = msg_send![capsule, setAlphaValue: alpha * spec.material_start_alpha];
        }
        capsule_targets.push((capsule, natural_style, natural_alpha));
        if spec.blur_radius > 0.0 && capsule_layer_captured {
            let installed = ramp_entry_defocus(capsule, duration, spec.blur_radius, log_target);
            state.mark_owned_filter(capsule);
            state.mark_owned_clip(capsule);
            if installed > 0.0 {
                capsule_blur_count += 1;
            }
        }
        let foreground: id = msg_send![capsule, contentView];
        let foreground_original_alpha = if capsule_inner_start_fraction < 1.0 {
            state.capture_layer(foreground);
            state.capture_alpha(foreground)
        } else if foreground != nil {
            Some(msg_send![foreground, alphaValue])
        } else {
            None
        };
        if let Some(alpha) = foreground_original_alpha {
            if alpha > 0.001 {
                if capsule_inner_start_fraction < 1.0 {
                    let _: () = msg_send![foreground,
                        setAlphaValue: alpha * capsule_inner_start_fraction];
                    content_targets.push((foreground, alpha));
                }
                let outer_actual: f64 = msg_send![capsule, alphaValue];
                let inner_actual: f64 = msg_send![foreground, alphaValue];
                let effective = outer_actual * inner_actual;
                capsule_label_start_min = capsule_label_start_min.min(effective);
                capsule_label_start_max = capsule_label_start_max.max(effective);
                capsule_label_count += 1;
            }
        }
    }
    GLASS_COHERENT_ENTRY_STATES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(window as usize, state);

    // One grouped fade: material, capsules, content, and window alpha share
    // the duration and curve, so the surface arrives as one object.
    let _: () = msg_send![class!(NSAnimationContext), beginGrouping];
    let fade_ctx: id = msg_send![class!(NSAnimationContext), currentContext];
    let _: () = msg_send![fade_ctx, setDuration: duration];
    let _: () = msg_send![fade_ctx, setAllowsImplicitAnimation: true];
    if let Some(timing_class) = objc::runtime::Class::get("CAMediaTimingFunction") {
        let name = tahoe_ns_string("easeOut");
        if name != nil {
            let curve: id = msg_send![timing_class, functionWithName: name];
            if curve != nil {
                let _: () = msg_send![fade_ctx, setTimingFunction: curve];
            }
        }
    }
    let backdrop_animator: id = msg_send![backdrop, animator];
    if material_supported {
        let _: () = msg_send![backdrop_animator, setStyle: 0isize]; // Regular
        if tint != nil {
            let _: () = msg_send![backdrop_animator, setTintColor: tint];
        }
    }
    let _: () = msg_send![backdrop_animator, setAlphaValue: backdrop_natural_alpha];
    for (capsule, natural_style, natural_alpha) in capsule_targets {
        let capsule_animator: id = msg_send![capsule, animator];
        if let Some((style, tint)) = natural_style {
            let _: () = msg_send![capsule_animator, setStyle: style];
            let _: () = msg_send![capsule_animator, setTintColor: tint];
        }
        if let Some(alpha) = natural_alpha {
            let _: () = msg_send![capsule_animator, setAlphaValue: alpha];
        }
    }
    for (view, natural_alpha) in content_targets {
        let animator: id = msg_send![view, animator];
        let _: () = msg_send![animator, setAlphaValue: natural_alpha];
    }
    let window_animator: id = msg_send![window, animator];
    let _: () = msg_send![window_animator, setAlphaValue: 1.0f64];
    let _: () = msg_send![class!(NSAnimationContext), endGrouping];

    let backdrop_blur = if spec.blur_radius > 0.0 && backdrop_layer_captured {
        ramp_entry_defocus(backdrop, duration, spec.blur_radius, log_target)
    } else {
        0.0
    };
    if let Some(state) = GLASS_COHERENT_ENTRY_STATES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_mut(&(window as usize))
    {
        if spec.blur_radius > 0.0 && backdrop_layer_captured {
            state.mark_owned_filter(backdrop);
            state.mark_owned_clip(backdrop);
        }
        state.mark_added_animations();
    }
    let _: () = msg_send![backdrop,
        performSelector: sel!(finishOwnWindowEntry)
        withObject: nil
        afterDelay: (duration + 0.020f64)
    ];
    record_native_glass_entry_span(window, duration);
    let configured_ns = crate::platform::host_clock::host_time_ns();
    let duration_ns = u64::from(spec.duration_ms) * 1_000_000;
    if is_main {
        MAIN_GLASS_ENTRY_RUNTIME_RECEIPT
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(MainGlassEntryRuntimeReceipt {
                window_key: window as usize,
                window_number: native_window_number,
                preset_id: preset.id(),
                duration_ms: spec.duration_ms,
                window_start_alpha: spec.window_start_alpha,
                material_start_alpha: spec.material_start_alpha,
                backdrop_start_alpha_effective: backdrop_start_alpha,
                content_start_alpha: spec.content_start_alpha,
                main_content_start_alpha_effective_range: (content_roots > 0)
                    .then_some([main_start_alpha_min, main_start_alpha_max]),
                capsule_label_start_alpha_effective_range: (capsule_label_count > 0)
                    .then_some([capsule_label_start_min, capsule_label_start_max]),
                capsule_label_count,
                capsule_inner_start_fraction,
                backdrop_blur_radius: backdrop_blur,
                capsule_blur_radius: if capsule_blur_count > 0 {
                    spec.blur_radius
                } else {
                    0.0
                },
                foreground_blur_radius: spec.foreground_blur_radius,
                foreground_root_count: content_roots,
                foreground_blurred_root_count: foreground_blur_count,
                capsule_count,
                capsule_blurred_count: capsule_blur_count,
                capsule_material_count,
                material_supported,
                expected_frame: [
                    final_frame.origin.x,
                    final_frame.origin.y,
                    final_frame.size.width,
                    final_frame.size.height,
                ],
                actual_frame: None,
                actual_alpha: None,
                post_hide_alpha: None,
                effects_restored: None,
                frame_unchanged: None,
                outcome: "in_flight",
                configured_at_host_time_ns: configured_ns,
                cancelled_at_host_time_ns: None,
                actual_end_host_time_ns: None,
            });
        GLASS_MORPH_LAST_DURATION_MS.store(
            u64::from(spec.duration_ms),
            std::sync::atomic::Ordering::Relaxed,
        );
    }
    if spec.foreground_blur_radius > 0.0 && content_roots > 0 && foreground_blur_count == 0 {
        logging::log(
            log_target,
            &format!(
                "event=main_glass_foreground_defocus_skip preset={} reason=no_safely_clipped_gpui_root",
                preset.id(),
            ),
        );
    }
    if content_roots > 0 && capsule_label_count > 0 {
        logging::log(log_target, &format!(
            "event=main_glass_effective_presence preset={} main_content_start_alpha_min={:.6} main_content_start_alpha_max={:.6} capsule_label_start_alpha_min={:.6} capsule_label_start_alpha_max={:.6} capsule_inner_start_fraction={:.6} capsule_label_capped_by_material={} capsule_blur_radius={:.2} main_foreground_blur_radius={:.2}",
            preset.id(), main_start_alpha_min, main_start_alpha_max,
            capsule_label_start_min, capsule_label_start_max,
            capsule_inner_start_fraction,
            spec.content_start_alpha > spec.material_start_alpha,
            if capsule_blur_count > 0 { spec.blur_radius } else { 0.0 },
            spec.foreground_blur_radius,
        ));
    }
    logging::log(log_target, &format!(
        "event=glass_morph window={} variant=fade phase=enter surface_profile={} preset={} direction=fixed_frame duration_ns={} width_scale=1 height_scale=1 window_start_alpha={:.4} material_start_alpha={:.4} content_start_alpha={:.4} entry_blur_radius={:.2} foreground_blur_radius={:.2} foreground_blurred_root_count={} foreground_root_count={} footer_capsule_count={} footer_blurred_capsule_count={} footer_material_ramp_count={} start_alpha={:.4} start_alpha_bits={:016x} settle_duration_ns={} configured_at_host_time_ns={} expected_settle_deadline_ns={} final_width_pt={:.2} final_height_pt={:.2} final_alpha=1 curve=easeOut",
        window_name, if is_main { "main" } else { "secondary" },
        preset.id(), duration_ns,
        spec.window_start_alpha, spec.material_start_alpha,
        spec.content_start_alpha, backdrop_blur,
        spec.foreground_blur_radius, foreground_blur_count, content_roots,
        capsule_count, capsule_blur_count, capsule_material_count,
        spec.window_start_alpha, spec.window_start_alpha.to_bits(),
        duration_ns, configured_ns,
        configured_ns.saturating_add(duration_ns),
        final_frame.size.width, final_frame.size.height,
    ));
}
