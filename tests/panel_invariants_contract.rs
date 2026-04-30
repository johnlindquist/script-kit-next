//! Source-level contract for Oracle-Session `window-activation-invariants-guard`.
//!
//! Background: six copies of the `PANEL_CONFIGURED.load → configure_as_floating_panel
//! → swizzle_gpui_blurred_view → configure_window_vibrancy_material_for_appearance
//! → PANEL_CONFIGURED.store(true)` block lived in `app_run_setup.rs`,
//! `runtime_stdin_match_core.rs`, `runtime_stdin.rs`, and
//! `window_visibility.rs`. If any panel knob drifted in just one copy — or if
//! an early return inside the configure sequence flipped the one-shot before
//! the NSPanel was actually correctly configured — the launcher would ship a
//! broken activation posture in the affected path and never retry.
//!
//! PR1 centralizes the sequence in `platform::ensure_main_panel_configured`
//! and adds `platform::assert_main_panel_invariants` as a runtime check that
//! panics in debug and grep-logs in release. These source-level tests pin the
//! structural guarantees the unit tests on `collection_behavior_ok` cannot
//! pin:
//!   1. The invariant names / phase tags exist — silent renames fail here.
//!   2. Only `ensure_main_panel_configured` writes to `PANEL_CONFIGURED` —
//!      no other `*.store(true` on it from any file.
//!   3. Every former duplicated call site now goes through the helper, and
//!      no copy of the old inline sequence remains.

const PANEL_INVARIANTS: &str = include_str!("../src/platform/panel_invariants.rs");
const APP_WINDOW_MGMT: &str = include_str!("../src/platform/app_window_management.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const RUNTIME_STDIN_MATCH_CORE: &str =
    include_str!("../src/main_entry/runtime_stdin_match_core.rs");
const WINDOW_VISIBILITY: &str = include_str!("../src/main_sections/window_visibility.rs");
const WINDOW_ORCHESTRATOR_EXECUTOR: &str = include_str!("../src/window_orchestrator/executor.rs");
const BUILTIN_EXECUTION: &str = include_str!("../src/app_execute/builtin_execution.rs");

#[test]
fn invariant_checklist_names_and_phase_tags_are_pinned() {
    // If one of these string literals is renamed or dropped, structured logs
    // and alerting queries silently break. The checklist itself is the
    // documentation of record.
    for name in [
        "\"main_thread\"",
        "\"main_window_registered\"",
        "\"window_class\"",
        "\"nonactivating_style\"",
        "\"can_become_key\"",
        "\"window_level\"",
        "\"collection_behavior\"",
        "\"activation_policy\"",
        "\"animation_behavior\"",
        "\"restorable\"",
        "\"frame_autosave_name\"",
        "\"is_key_window\"",
    ] {
        assert!(
            PANEL_INVARIANTS.contains(name),
            "panel_invariants.rs must keep invariant name {name}; rename it \
             everywhere or this audit is stale"
        );
    }

    assert!(
        PANEL_INVARIANTS.contains("pub(crate) enum PanelInvariantPhase"),
        "panel_invariants.rs must expose `pub(crate) enum PanelInvariantPhase`"
    );
    for variant in ["PreShow", "PostMakeKey", "BackgroundShow", "AfterConfigure"] {
        assert!(
            PANEL_INVARIANTS.contains(variant),
            "PanelInvariantPhase must keep variant {variant}"
        );
    }
}

#[test]
fn expected_panel_constants_match_raycast_floating_panel_posture() {
    // Oracle identified that the live launcher actually runs at
    // NSPopUpMenuWindowLevel (101), not NSFloatingWindowLevel (3). If this
    // ever changes back to 3 silently, the panel will sit under other
    // floating UI and the regression will be invisible in screenshots. Pin
    // the constants in source.
    assert!(
        PANEL_INVARIANTS.contains("EXPECTED_MAIN_PANEL_LEVEL: i64 = 101"),
        "main panel window level must stay 101 (NSPopUpMenuWindowLevel); see \
         Oracle-Session window-activation-invariants-guard"
    );
    assert!(
        PANEL_INVARIANTS.contains("NONACTIVATING_PANEL_STYLE_BIT: u64 = 1 << 7"),
        "NonactivatingPanel style bit must stay (1 << 7)"
    );
    assert!(
        PANEL_INVARIANTS.contains("ACTIVATION_POLICY_ACCESSORY: i64 = 1"),
        "activation policy must stay NSApplicationActivationPolicyAccessory (1)"
    );
    assert!(
        PANEL_INVARIANTS.contains("ANIMATION_BEHAVIOR_NONE: i64 = 2"),
        "window animation behavior must stay NSWindowAnimationBehaviorNone (2)"
    );
}

#[test]
fn only_ensure_main_panel_configured_writes_panel_configured() {
    // PANEL_CONFIGURED is the one-shot guard. Only the centralizing helper
    // in platform/app_window_management.rs is allowed to store `true`;
    // anything else re-introduces the early-flip bug.
    for (label, source) in [
        ("main_entry/app_run_setup.rs", APP_RUN_SETUP),
        ("main_entry/runtime_stdin.rs", RUNTIME_STDIN),
        (
            "main_entry/runtime_stdin_match_core.rs",
            RUNTIME_STDIN_MATCH_CORE,
        ),
        ("main_sections/window_visibility.rs", WINDOW_VISIBILITY),
    ] {
        assert!(
            !source.contains("PANEL_CONFIGURED.store"),
            "{label} must not write to PANEL_CONFIGURED directly; only \
             platform::ensure_main_panel_configured is allowed to flip it"
        );
        assert!(
            !source.contains("PANEL_CONFIGURED.compare_exchange"),
            "{label} must not CAS PANEL_CONFIGURED directly"
        );
    }

    // The helper itself must contain exactly the writes we expect.
    let store_count = APP_WINDOW_MGMT
        .matches("PANEL_CONFIGURED.store(true")
        .count();
    assert_eq!(
        store_count, 2,
        "app_window_management.rs should store PANEL_CONFIGURED=true in \
         exactly two arms of ensure_main_panel_configured (already-configured \
         fast path + successful-configure path); found {store_count}"
    );
}

#[test]
fn former_inline_configure_sequence_is_gone() {
    // The load-bearing signature of the old inline sequence is the
    // `configure_as_floating_panel` call — that is what the PANEL_CONFIGURED
    // one-shot was guarding, and where the early-return bug lived. The
    // startup-time `swizzle_gpui_blurred_view` in `app_run_setup.rs` is an
    // unrelated one-time hack run before any show, so we deliberately don't
    // gate on it here.
    for (label, source) in [
        ("main_entry/app_run_setup.rs", APP_RUN_SETUP),
        ("main_entry/runtime_stdin.rs", RUNTIME_STDIN),
        (
            "main_entry/runtime_stdin_match_core.rs",
            RUNTIME_STDIN_MATCH_CORE,
        ),
        ("main_sections/window_visibility.rs", WINDOW_VISIBILITY),
    ] {
        // Match the call, not mentions in comments. The old inline site
        // looked like `platform::configure_as_floating_panel(` or
        // `configure_as_floating_panel();` — both end in `(`.
        assert!(
            !source.contains("configure_as_floating_panel("),
            "{label} must not call configure_as_floating_panel directly; \
             use platform::ensure_main_panel_configured(<context>) instead. \
             This is the load-bearing marker of the old inline sequence."
        );
    }
}

#[test]
fn every_show_path_routes_through_ensure_main_panel_configured() {
    // Each of the files that used to carry an inline configure sequence must
    // now explicitly call the helper at least once, with a grep-friendly
    // context tag. This catches a refactor that silently drops the guard on
    // one show path.
    for (label, source, tag) in [
        (
            "main_entry/app_run_setup.rs",
            APP_RUN_SETUP,
            "app_run_setup",
        ),
        (
            "main_entry/runtime_stdin.rs",
            RUNTIME_STDIN,
            "runtime_stdin",
        ),
        (
            "main_entry/runtime_stdin_match_core.rs",
            RUNTIME_STDIN_MATCH_CORE,
            "runtime_stdin_match_core",
        ),
        (
            "main_sections/window_visibility.rs",
            WINDOW_VISIBILITY,
            "window_visibility",
        ),
    ] {
        assert!(
            source.contains("platform::ensure_main_panel_configured("),
            "{label} must call platform::ensure_main_panel_configured(<context>) \
             on its show path"
        );
        assert!(
            source.contains(tag),
            "{label}'s call to ensure_main_panel_configured should include a \
             context tag derived from its module path (looking for {tag:?}) \
             so log triage can pinpoint the show path"
        );
    }
}

#[test]
fn deferred_dictation_reveal_paths_configure_main_panel_before_showing() {
    for (label, source, context) in [
        (
            "window_orchestrator/executor.rs",
            WINDOW_ORCHESTRATOR_EXECUTOR,
            "window_orchestrator::executor::RevealMain/make_key",
        ),
        (
            "window_orchestrator/executor.rs",
            WINDOW_ORCHESTRATOR_EXECUTOR,
            "window_orchestrator::executor::RevealMain/background",
        ),
        (
            "window_orchestrator/executor.rs",
            WINDOW_ORCHESTRATOR_EXECUTOR,
            "window_orchestrator::executor::FocusSurface/Main",
        ),
        (
            "app_execute/builtin_execution.rs",
            BUILTIN_EXECUTION,
            "builtin_execution::dictation_main_filter_delivery",
        ),
    ] {
        assert!(
            source.contains("platform::ensure_main_panel_configured(")
                || source.contains("crate::platform::ensure_main_panel_configured("),
            "{label} must configure the main panel before revealing it"
        );
        assert!(
            source.contains(context),
            "{label} must keep grep-friendly panel configure context {context:?}"
        );
    }
}

#[test]
fn assert_main_panel_invariants_is_callable_from_platform() {
    // Silent renames of the runtime check would turn the instrumentation off
    // everywhere without any test failing. Pin the symbol name + signature
    // shape at source level.
    assert!(
        PANEL_INVARIANTS.contains(
            "pub(crate) fn assert_main_panel_invariants(\n    context: &'static str,\n    phase: PanelInvariantPhase,\n) -> PanelInvariantReport"
        ),
        "assert_main_panel_invariants must keep its (context, phase) -> \
         PanelInvariantReport signature; callers in visibility_focus.rs and \
         app_window_management.rs depend on this shape"
    );
    assert!(
        PANEL_INVARIANTS.contains("pub(crate) fn ok(&self) -> bool"),
        "PanelInvariantReport must expose ok() so callers can branch on \
         success"
    );
}
