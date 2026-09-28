#[test]
fn detached_glass_requires_both_classes_and_honors_debug_fallback() {
    assert!(super::native_glass_composition_gate(true, true, false));
    assert!(!super::native_glass_composition_gate(false, true, false));
    assert!(!super::native_glass_composition_gate(true, false, false));
    assert!(!super::native_glass_composition_gate(true, true, true));
}

#[cfg(target_os = "macos")]
#[test]
fn detached_backdrop_excludes_footer_and_exact_eight_point_gutter() {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};

    let bounds = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(750.0, 501.0));
    let layout = super::TahoeBackdropLayout::ContentAboveDetachedFooter { bottom_inset: 40.0 };
    let frame = layout.frame(bounds);
    assert_eq!(frame.origin.x, 0.0);
    assert_eq!(frame.origin.y, 40.0);
    assert_eq!(frame.size.width, 750.0);
    assert_eq!(frame.size.height, 461.0);
    assert_eq!(layout.bottom_inset(), 40.0);

    let footer_top = 32.0;
    assert_eq!(frame.origin.y - footer_top, 8.0);
    assert!(footer_top <= frame.origin.y);
}

/// The main window's glass fills the whole window in glass mode (its footer
/// is in-stage) while keeping the partitioned host's shadowless, rounded
/// policy: a full-window NSWindow shadow would outline the transparent host.
#[cfg(target_os = "macos")]
#[test]
fn main_backdrop_fills_the_window_but_keeps_the_partitioned_host_policy() {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};

    if !crate::footer_popup::main_window_footer_in_stage() {
        return;
    }
    let bounds = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(750.0, 480.0));
    let layout = super::tahoe_backdrop_layout("Main window");
    let frame = layout.frame(bounds);
    assert_eq!((frame.origin.y, frame.size.height), (0.0, 480.0));
    assert_eq!(layout.bottom_inset(), 0.0);
    assert!(layout.is_detached_footer());
}

#[cfg(target_os = "macos")]
#[test]
fn backdrop_layout_can_toggle_without_changing_outer_frame() {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};

    // The same outer content bounds partition into full-window and
    // footer-inset layouts; toggling layouts never changes the outer
    // frame, only the backdrop's share of it.
    let bounds = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(350.0, 280.0));
    let full = super::TahoeBackdropLayout::FullWindow.frame(bounds);
    assert_eq!(full.origin.y, 0.0);
    assert_eq!(full.size.height, 280.0);

    let inset = 44.0;
    let partitioned = super::TahoeBackdropLayout::ContentAboveDetachedFooter {
        bottom_inset: inset,
    }
    .frame(bounds);
    assert_eq!(partitioned.origin.y, inset);
    assert_eq!(partitioned.size.height, 280.0 - inset);
    assert_eq!(partitioned.size.width, full.size.width);

    // Round-trip back to full-window restores the exact original frame.
    let restored = super::TahoeBackdropLayout::FullWindow.frame(bounds);
    assert_eq!(restored.origin.y, full.origin.y);
    assert_eq!(restored.size.height, full.size.height);
}

#[cfg(target_os = "macos")]
#[test]
fn native_glass_style_locks_tint_floor_and_preserves_requested_tint_semantics() {
    let mut inherited = crate::theme::Theme::default();
    inherited.opacity.as_mut().unwrap().glass_tint_opacity = None;
    let mut explicit_zero = inherited.clone();
    explicit_zero.opacity.as_mut().unwrap().glass_tint_opacity = Some(0.0);
    let mut below_floor = inherited.clone();
    below_floor.opacity.as_mut().unwrap().glass_tint_opacity = Some(0.34);
    let mut at_floor = inherited.clone();
    at_floor.opacity.as_mut().unwrap().glass_tint_opacity = Some(0.35);
    let mut above_floor = inherited.clone();
    above_floor.opacity.as_mut().unwrap().glass_tint_opacity = Some(0.72);
    let light = crate::theme::Theme::light_default();

    let backdrop = super::resolve_native_glass_style(
        &inherited,
        super::NativeGlassSurfaceRole::WindowBackdrop,
    );
    let capsule = super::resolve_native_glass_style(
        &inherited,
        super::NativeGlassSurfaceRole::FloatingCapsule,
    );
    let explicit = super::resolve_native_glass_style(
        &explicit_zero,
        super::NativeGlassSurfaceRole::FloatingCapsule,
    );
    let below = super::resolve_native_glass_style(
        &below_floor,
        super::NativeGlassSurfaceRole::FloatingCapsule,
    );
    let at = super::resolve_native_glass_style(
        &at_floor,
        super::NativeGlassSurfaceRole::FloatingCapsule,
    );
    let above = super::resolve_native_glass_style(
        &above_floor,
        super::NativeGlassSurfaceRole::FloatingCapsule,
    );
    let light_capsule =
        super::resolve_native_glass_style(&light, super::NativeGlassSurfaceRole::FloatingCapsule);

    assert_eq!(backdrop.signature.tint_rgb, capsule.signature.tint_rgb);
    assert_eq!(
        backdrop.signature.effective_tint_alpha_bits,
        capsule.signature.effective_tint_alpha_bits
    );
    // Material/appearance parity: both roles resolve the same appearance
    // mode from the same theme — the capsule may not diverge to its own
    // dark/light decision.
    assert_eq!(backdrop.signature.dark, capsule.signature.dark);
    let light_backdrop =
        super::resolve_native_glass_style(&light, super::NativeGlassSurfaceRole::WindowBackdrop);
    assert_eq!(
        light_backdrop.signature.tint_rgb,
        light_capsule.signature.tint_rgb
    );
    assert_eq!(
        light_backdrop.signature.effective_tint_alpha_bits,
        light_capsule.signature.effective_tint_alpha_bits
    );
    assert_eq!(light_backdrop.signature.dark, light_capsule.signature.dark);
    // The native capsule veil resolves through the chrome token. It is
    // 0.0 during the 2026-07-27 user-authorized "perfectly match the main
    // window" experiment, so the capsule material intentionally equals the
    // backdrop material; only the separation rim distinguishes them.
    assert_eq!(
        capsule.veil_alpha,
        crate::ui::chrome::LIQUID_GLASS_CAPSULE_VEIL_ALPHA
    );
    assert_eq!(
        backdrop.effective_tint_alpha,
        crate::ui::chrome::LIQUID_GLASS_STABILITY_TINT_ALPHA_FLOOR
    );
    assert_eq!(backdrop.signature.requested_tint_alpha_bits, None);
    assert_eq!(
        explicit.signature.requested_tint_alpha_bits,
        Some(0.0_f32.to_bits())
    );
    for style in [backdrop, capsule, explicit, below, at] {
        assert_eq!(style.effective_tint_alpha, 0.35);
    }
    assert_eq!(above.effective_tint_alpha, 0.72);
    assert_eq!(backdrop.veil_alpha, 0.0);
    assert_eq!(backdrop.rim_width, 0.0);
    assert_eq!(backdrop.signature.rim_rgba, 0xFFFF_FF00);
    assert_eq!(capsule.veil_alpha, 0.0);
    assert_eq!(capsule.rim_width, 1.0);
    assert_eq!(capsule.signature.rim_rgba, 0xFFFF_FF3D);
    assert_eq!(light_capsule.veil_alpha, 0.0);
    assert_eq!(light_capsule.rim_width, 1.0);
    assert_eq!(light_capsule.signature.rim_rgba, 0x0000_002E);
    assert_ne!(backdrop.signature, explicit.signature);
}

/// The material stack must be STATIC between morph start and settle: only
/// the initial installation and explicitly recorded theme refreshes may
/// apply native styles. A per-frame tint/veil/opacity animation during
/// entry would land here as an in-span `Install` re-application — the
/// ledger flags it so probes reading
/// `native_glass_style_mutation_during_entry` error events can prove a
/// healthy entry emits none.
#[cfg(target_os = "macos")]
#[test]
fn native_glass_style_ledger_flags_only_mid_entry_reapplications() {
    let theme = crate::theme::Theme::default();
    let signature =
        super::resolve_native_glass_style(&theme, super::NativeGlassSurfaceRole::FloatingCapsule)
            .signature;
    let application = |window_number: i64,
                       surface_id: usize,
                       at_ns: u64,
                       reason: super::NativeGlassStyleApplicationReason| {
        super::NativeGlassStyleApplication {
            window_number,
            surface_id,
            at_ns,
            reason,
            signature,
        }
    };
    use super::NativeGlassStyleApplicationReason::{Install, ThemeRefresh};

    let mut ledger = super::NativeGlassStyleLedger::default();
    ledger.record_entry_span(7, 1_000, 2_000);
    // Initial install before the morph span: allowed.
    assert!(!ledger.record_application(application(7, 70, 900, Install)));
    // Pooled capsule views style themselves before attaching to a window
    // (window number -1): never counted against a real window's span.
    assert!(!ledger.record_application(application(-1, 71, 1_100, Install)));
    // A distinct capsule in the same window gets its own initial install;
    // it is not a reapplication of surface 70.
    assert!(!ledger.record_application(application(7, 71, 1_100, Install)));
    // Theme refresh mid-span is the explicitly recorded exception.
    assert!(!ledger.record_application(application(7, 70, 1_200, ThemeRefresh)));
    assert!(ledger.has_identical_surface_style_during_entry(7, 70, 1_300, signature));
    assert!(!ledger.has_identical_surface_style_during_entry(7, 72, 1_300, signature));
    // A tint/veil/opacity animation during entry looks exactly like this:
    // an Install-shaped re-application of the same native surface inside
    // the span. It must be flagged — this is the assertion that fails if
    // anyone temporarily mutates capsule tint during entry.
    assert!(ledger.record_application(application(7, 70, 1_500, Install)));
    // After settle, re-applications are outside the entry contract.
    assert!(!ledger.record_application(application(7, 70, 2_500, Install)));
    // A first-ever installation landing inside its own span (window whose
    // backdrop is created mid-morph) is the initial install, not a
    // mutation.
    ledger.record_entry_span(9, 1_000, 2_000);
    assert!(!ledger.record_application(application(9, 90, 1_050, Install)));
    assert!(ledger.record_application(application(9, 90, 1_060, Install)));
}

#[cfg(target_os = "macos")]
#[test]
fn secondary_backdrop_remains_full_window() {
    use cocoa::foundation::{NSPoint, NSRect, NSSize};

    let bounds = NSRect::new(NSPoint::new(3.0, 5.0), NSSize::new(420.0, 280.0));
    let frame = super::TahoeBackdropLayout::FullWindow.frame(bounds);
    assert_eq!(frame.origin.x, bounds.origin.x);
    assert_eq!(frame.origin.y, bounds.origin.y);
    assert_eq!(frame.size.width, bounds.size.width);
    assert_eq!(frame.size.height, bounds.size.height);
}
