// ============================================================================
// Panel Invariants (Oracle-Session `window-activation-invariants-guard` PR1)
// ============================================================================
//
// Runtime invariant checks for the main NSPanel configuration.
//
// The main window is an NSPanel with NonactivatingPanel style, the GPUI
// `WindowKind::PopUp` level, accessory app activation policy, a specific
// collection-behavior shape, and a three-layer cursor-in-background system.
// Any change to `src/platform/visibility_focus.rs`, `app_window_management.rs`,
// or `cursor.rs` can silently break this contract.
//
// This module exposes a cheap runtime checker that queries AppKit for the
// actual current values and produces a structured `PanelInvariantReport`.
// It hooks at every show path and at the end of `configure_as_floating_panel`.
// In debug builds a mismatch panics loudly; in release it logs a grep-friendly
// report and continues.
//
// Fragile assumptions Oracle flagged in the original question:
//
// - The main window level is 101 (GPUI PopUp / `NSPopUpMenuWindowLevel`),
//   not `NSFloatingWindowLevel = 3`. `configure_as_floating_panel` explicitly
//   does NOT override the level; GPUI owns it through `WindowKind::PopUp`.
// - Collection behavior must assert required bits plus forbidden all-spaces
//   behavior, not exact equality. `MoveToActiveSpace` is required,
//   `CanJoinAllSpaces` is rejected, and GPUI/AppKit may layer additional
//   unrelated bits.
// - `styleMask` only needs the `NonactivatingPanel` bit. Do not assert
//   equality — AppKit layers extra bits (titled, resizable, etc.).
// - `PANEL_CONFIGURED` is NOT an invariant. It is a caller-owned one-shot
//   flag that `ensure_main_panel_configured` only sets after the
//   post-configure invariant report passes.

/// Expected `[window level]` for the main panel.
///
/// GPUI's `WindowKind::PopUp` maps to `NSPopUpMenuWindowLevel = 101`. The
/// launcher deliberately does NOT override the level in
/// `configure_as_floating_panel`; GPUI owns it natively.
#[cfg(target_os = "macos")]
pub(crate) const EXPECTED_MAIN_PANEL_LEVEL: i64 = 101;

/// `NSWindowStyleMaskNonactivatingPanel` bit.
#[cfg(target_os = "macos")]
pub(crate) const NONACTIVATING_PANEL_STYLE_BIT: u64 = 1 << 7;

/// `NSApplicationActivationPolicyAccessory = 1`.
#[cfg(target_os = "macos")]
pub(crate) const ACTIVATION_POLICY_ACCESSORY: i64 = 1;

/// `NSWindowAnimationBehaviorNone = 2`.
#[cfg(target_os = "macos")]
pub(crate) const ANIMATION_BEHAVIOR_NONE: i64 = 2;

/// Which show path (or configure hook) invoked the invariant check.
///
/// Oracle-Session `window-activation-invariants-guard` — different phases
/// imply different expected post-conditions. `PostMakeKey` additionally
/// requires the panel to be the key window; `BackgroundShow` explicitly
/// does not.
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanelInvariantPhase {
    /// Before `orderFrontRegardless` in the activating show path.
    PreShow,
    /// After `makeKeyWindow` in the activating show path.
    PostMakeKey,
    /// Background show (`orderFrontRegardless` without `makeKeyWindow`).
    BackgroundShow,
    /// End of `configure_as_floating_panel` or `ensure_main_panel_configured`.
    AfterConfigure,
}

/// Single invariant check result.
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub(crate) struct Invariant {
    pub name: &'static str,
    pub getter: &'static str,
    pub expected: String,
    pub actual: String,
    // Read via `Debug` when the report is logged; not (yet) consumed
    // programmatically outside `record`, so suppress dead-code.
    #[allow(dead_code)]
    pub ok: bool,
}

/// Aggregate report for one invariant pass.
///
/// `mismatched` carries fail-loud invariants (window level, style mask,
/// activation policy, etc.) — any entry here flips `ok()` to false and
/// panics in debug. `soft_mismatched` carries invariants that may fail
/// transiently for reasons outside the launcher's control (e.g. AppKit's
/// `makeKeyWindow` is dispatched asynchronously, so `[window isKeyWindow]`
/// can still be false the instant `assert_main_panel_invariants` runs). Soft
/// failures are logged but do NOT flip `ok()` — they preserve the structured
/// log signal without turning a cold-start AppKit race into a debug-build
/// panic. Oracle-Session observation (Run 8 Pass #22, 2026-04-19): the first
/// two cold-start `{"type":"show"}` events panicked at
/// `src/platform/panel_invariants.rs:326:13` on this exact race.
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Default)]
pub(crate) struct PanelInvariantReport {
    pub checked: Vec<Invariant>,
    pub mismatched: Vec<Invariant>,
    pub soft_mismatched: Vec<Invariant>,
}

#[cfg(target_os = "macos")]
impl PanelInvariantReport {
    fn record(
        &mut self,
        ok: bool,
        name: &'static str,
        getter: &'static str,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) {
        let inv = Invariant {
            name,
            getter,
            expected: expected.into(),
            actual: actual.into(),
            ok,
        };
        if !ok {
            self.mismatched.push(inv.clone());
        }
        self.checked.push(inv);
    }

    /// Record an invariant whose failure is WARN-only.
    ///
    /// Use this for invariants that may legitimately fail under circumstances
    /// outside the launcher's control — specifically AppKit timing races
    /// where the expected state is correct but not yet observable. Soft
    /// failures are logged through the same `PANEL_INVARIANTS` tag but do
    /// NOT push into `mismatched`, so `ok()` stays true and the debug-build
    /// panic in `finish()` does not fire.
    fn record_soft(
        &mut self,
        ok: bool,
        name: &'static str,
        getter: &'static str,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) {
        let inv = Invariant {
            name,
            getter,
            expected: expected.into(),
            actual: actual.into(),
            ok,
        };
        if !ok {
            self.soft_mismatched.push(inv.clone());
        }
        self.checked.push(inv);
    }

    pub(crate) fn ok(&self) -> bool {
        self.mismatched.is_empty()
    }
}

/// Pure predicate for the collection-behavior shape the launcher requires.
///
/// Exported for table-driven unit tests that run on every platform. The
/// assertion is: `FullScreenAuxiliary`, `IgnoresCycle`, and
/// `MoveToActiveSpace` are required; `CanJoinAllSpaces` is rejected.
///
/// Oracle-Session `window-activation-invariants-guard` explicitly warned
/// against re-simplifying this back to exact equality.
pub(crate) const fn collection_behavior_ok(bits: u64) -> bool {
    let has_can_join = bits & (1 << 0) != 0; // CanJoinAllSpaces
    let has_move_to_active = bits & (1 << 1) != 0; // MoveToActiveSpace
    let has_ignores_cycle = bits & (1 << 6) != 0; // IgnoresCycle
    let has_full_screen_aux = bits & (1 << 8) != 0; // FullScreenAuxiliary

    let required = has_full_screen_aux && has_ignores_cycle;
    let spaces_ok = has_move_to_active && !has_can_join;
    required && spaces_ok
}

/// Query AppKit and return a structured report of every invariant.
///
/// In debug builds a mismatched report panics after logging (unless
/// `SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH=1`). In release it logs and
/// returns — callers can branch on `report.ok()`.
#[cfg(target_os = "macos")]
pub(crate) fn assert_main_panel_invariants(
    context: &'static str,
    phase: PanelInvariantPhase,
) -> PanelInvariantReport {
    let mut r = PanelInvariantReport::default();

    // `require_main_thread` is a sibling in the same flattened `platform` module
    // (see `src/platform/app_window_management.rs`) because all platform files
    // are `include!`-merged into `platform/mod.rs`.
    if require_main_thread("assert_main_panel_invariants") {
        r.record(
            false,
            "main_thread",
            "NSThread.isMainThread",
            "true",
            "false",
        );
        return finish(context, phase, r);
    }

    // SAFETY: main-thread verified above; all msg_send! calls target standard
    // NSPanel / NSApp getters.
    unsafe {
        let Some(window) = crate::window_manager::get_main_window() else {
            r.record(
                false,
                "main_window_registered",
                "window_manager::get_main_window",
                "Some(NSWindow)",
                "None",
            );
            return finish(context, phase, r);
        };

        let is_panel: bool = msg_send![window, isKindOfClass: class!(NSPanel)];
        r.record(
            is_panel,
            "window_class",
            "[window isKindOfClass:NSPanel]",
            "true",
            is_panel.to_string(),
        );

        let style: u64 = msg_send![window, styleMask];
        r.record(
            style & NONACTIVATING_PANEL_STYLE_BIT != 0,
            "nonactivating_style",
            "[window styleMask]",
            "has NonactivatingPanel(1<<7)",
            format!("0x{style:x}"),
        );

        let can_key: bool = msg_send![window, canBecomeKeyWindow];
        r.record(
            can_key,
            "can_become_key",
            "[window canBecomeKeyWindow]",
            "true",
            can_key.to_string(),
        );

        let level: i64 = msg_send![window, level];
        r.record(
            level == EXPECTED_MAIN_PANEL_LEVEL,
            "window_level",
            "[window level]",
            EXPECTED_MAIN_PANEL_LEVEL.to_string(),
            level.to_string(),
        );

        let behavior: u64 = msg_send![window, collectionBehavior];
        r.record(
            collection_behavior_ok(behavior),
            "collection_behavior",
            "[window collectionBehavior]",
            "FullScreenAuxiliary|IgnoresCycle plus MoveToActiveSpace and not CanJoinAllSpaces",
            format!("0x{behavior:x}"),
        );

        let app: id = cocoa::appkit::NSApp();
        let policy: i64 = msg_send![app, activationPolicy];
        r.record(
            policy == ACTIVATION_POLICY_ACCESSORY,
            "activation_policy",
            "[NSApp activationPolicy]",
            "Accessory(1)",
            policy.to_string(),
        );

        let animation: i64 = msg_send![window, animationBehavior];
        r.record(
            animation == ANIMATION_BEHAVIOR_NONE,
            "animation_behavior",
            "[window animationBehavior]",
            "None(2)",
            animation.to_string(),
        );

        let restorable: bool = msg_send![window, isRestorable];
        r.record(
            !restorable,
            "restorable",
            "[window isRestorable]",
            "false",
            restorable.to_string(),
        );

        let autosave: id = msg_send![window, frameAutosaveName];
        let autosave_len: usize = if autosave == nil {
            0
        } else {
            msg_send![autosave, length]
        };
        r.record(
            autosave_len == 0,
            "frame_autosave_name",
            "[window frameAutosaveName length]",
            "0",
            autosave_len.to_string(),
        );

        if matches!(phase, PanelInvariantPhase::PostMakeKey) {
            // `is_key_window` is soft because AppKit dispatches
            // `[window makeKeyWindow]` asynchronously — at the instant this
            // check runs on a cold-start show, the window may not yet have
            // been promoted to key. All the other PostMakeKey invariants
            // (level, style mask, collection behavior, activation policy,
            // animation behavior, restorable, autosave-name) read state
            // that is configured synchronously and cannot race; they stay
            // on `record`. Only the key-window promotion races, so only
            // this check softens. Without the softening, `finish()` would
            // panic in debug builds on the normal cold-start path. See
            // Run 8 Pass #22 log entry for the failure signature.
            let is_key: bool = msg_send![window, isKeyWindow];
            r.record_soft(
                is_key,
                "is_key_window",
                "[window isKeyWindow]",
                "true",
                is_key.to_string(),
            );
        }
    }

    finish(context, phase, r)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn assert_main_panel_invariants(
    _context: &'static str,
    _phase: PanelInvariantPhase,
) -> PanelInvariantReport {
    PanelInvariantReport::default()
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PanelInvariantPhase {
    PreShow,
    PostMakeKey,
    BackgroundShow,
    AfterConfigure,
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug, Clone, Default)]
pub(crate) struct PanelInvariantReport {
    pub checked: Vec<()>,
    pub mismatched: Vec<()>,
    pub soft_mismatched: Vec<()>,
}

#[cfg(not(target_os = "macos"))]
impl PanelInvariantReport {
    pub(crate) fn ok(&self) -> bool {
        true
    }
}

#[cfg(target_os = "macos")]
fn finish(
    context: &'static str,
    phase: PanelInvariantPhase,
    r: PanelInvariantReport,
) -> PanelInvariantReport {
    if !r.ok() {
        log_report(context, phase, &r);
        if cfg!(debug_assertions)
            && std::env::var_os("SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH").is_none()
        {
            panic!(
                "panel_invariants: FAIL context={} phase={:?} mismatches={}",
                context,
                phase,
                r.mismatched.len()
            );
        }
    }
    if !r.soft_mismatched.is_empty() {
        log_soft_report(context, phase, &r);
    }
    r
}

#[cfg(target_os = "macos")]
fn log_report(context: &'static str, phase: PanelInvariantPhase, r: &PanelInvariantReport) {
    logging::log(
        "PANEL_INVARIANTS",
        &format!(
            "FAIL context={} phase={:?} mismatches={} checked={}",
            context,
            phase,
            r.mismatched.len(),
            r.checked.len()
        ),
    );
    for inv in &r.mismatched {
        logging::log(
            "PANEL_INVARIANTS",
            &format!(
                "{} getter=\"{}\" expected={} actual={}",
                inv.name, inv.getter, inv.expected, inv.actual
            ),
        );
    }
}

#[cfg(target_os = "macos")]
fn log_soft_report(context: &'static str, phase: PanelInvariantPhase, r: &PanelInvariantReport) {
    logging::log(
        "PANEL_INVARIANTS",
        &format!(
            "SOFT context={} phase={:?} soft_mismatches={} checked={}",
            context,
            phase,
            r.soft_mismatched.len(),
            r.checked.len()
        ),
    );
    for inv in &r.soft_mismatched {
        logging::log(
            "PANEL_INVARIANTS",
            &format!(
                "SOFT {} getter=\"{}\" expected={} actual={}",
                inv.name, inv.getter, inv.expected, inv.actual
            ),
        );
    }
}

#[cfg(test)]
mod panel_invariant_unit_tests {
    use super::collection_behavior_ok;

    const CAN_JOIN: u64 = 1 << 0;
    const MOVE_TO_ACTIVE: u64 = 1 << 1;
    const IGNORES_CYCLE: u64 = 1 << 6;
    const FULL_SCREEN_AUX: u64 = 1 << 8;

    #[test]
    fn all_spaces_is_rejected_for_main_panel() {
        assert!(!collection_behavior_ok(
            CAN_JOIN | IGNORES_CYCLE | FULL_SCREEN_AUX
        ));
    }

    #[test]
    fn normal_shape_is_ok_with_move_to_active_and_required_bits() {
        assert!(collection_behavior_ok(
            MOVE_TO_ACTIVE | IGNORES_CYCLE | FULL_SCREEN_AUX
        ));
    }

    #[test]
    fn mutually_exclusive_can_join_and_move_to_active_is_rejected() {
        assert!(!collection_behavior_ok(
            CAN_JOIN | MOVE_TO_ACTIVE | IGNORES_CYCLE | FULL_SCREEN_AUX
        ));
    }

    #[test]
    fn missing_full_screen_auxiliary_is_rejected() {
        assert!(!collection_behavior_ok(CAN_JOIN | IGNORES_CYCLE));
        assert!(!collection_behavior_ok(MOVE_TO_ACTIVE | IGNORES_CYCLE));
    }

    #[test]
    fn missing_ignores_cycle_is_rejected() {
        assert!(!collection_behavior_ok(CAN_JOIN | FULL_SCREEN_AUX));
        assert!(!collection_behavior_ok(MOVE_TO_ACTIVE | FULL_SCREEN_AUX));
    }

    #[test]
    fn missing_both_spaces_bits_is_rejected() {
        assert!(!collection_behavior_ok(IGNORES_CYCLE | FULL_SCREEN_AUX));
    }

    #[test]
    fn extra_unknown_bits_do_not_break_the_shape() {
        let extra = 1 << 20;
        assert!(collection_behavior_ok(
            MOVE_TO_ACTIVE | IGNORES_CYCLE | FULL_SCREEN_AUX | extra
        ));
    }
}
