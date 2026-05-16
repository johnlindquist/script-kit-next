//! Source-level contract test for the `tool-hide-rpc-surface-reset`
//! user story (Run 2 Pass #19 side finding, closed in Pass #21).
//!
//! Pass #19 added a re-key to `"scriptList"` inside `hide_main_window_helper`
//! — but hides issued via the stdin `ExternalCommand::Hide` RPC path take a
//! different code path that does NOT route through the helper. That left
//! `AutomationWindowInfo.semanticSurface` stuck at whatever the last
//! subview was (e.g. `"browserTabs"`) after a hide RPC, diverging from
//! the live `getState.promptType`.
//!
//! The fix adds two lines inside the `ExternalCommand::Hide` arm in each
//! of the three triple-embedded stdin dispatchers (memories 6330/6331):
//!
//! 1. `view.reset_to_script_list(ctx);` — returns the inner view to
//!    `AppView::ScriptList` so the next show starts from a clean state.
//! 2. `crate::windows::update_automation_semantic_surface("main",
//!    Some("scriptList".to_string()));` — re-keys the introspection
//!    channel so `listAutomationWindows.windows[0].semanticSurface`
//!    reports the truth.
//!
//! If any dispatcher silently drops either call, the hide-RPC path for
//! that entry point regresses. This contract pins the fix in source.

const STDIN_CORE_SOURCE: &str = include_str!("../src/main_entry/runtime_stdin_match_core.rs");
const STDIN_SOURCE: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const APP_RUN_SETUP_SOURCE: &str = include_str!("../src/main_entry/app_run_setup.rs");

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn hide_rpc_dispatchers_reset_to_script_list() {
    for (name, source) in [
        (
            "src/main_entry/runtime_stdin_match_core.rs",
            STDIN_CORE_SOURCE,
        ),
        ("src/main_entry/runtime_stdin.rs", STDIN_SOURCE),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP_SOURCE),
    ] {
        let Some(hide_start) = source.find("ExternalCommand::Hide {") else {
            panic!(
                "{name} must define an `ExternalCommand::Hide` arm — the \
                 three-dispatcher pattern requires it"
            );
        };
        let next_arm = source[hide_start..]
            .find("ExternalCommand::SetFilter")
            .expect(
                "Hide arm must be followed by SetFilter arm — the \
                 dispatcher layout has shifted, update this test",
            );
        let hide_body = &source[hide_start..hide_start + next_arm];
        assert!(
            hide_body.contains("view.reset_to_script_list(ctx);"),
            "{name} `ExternalCommand::Hide` arm must call \
             `view.reset_to_script_list(ctx);` — without it, the inner \
             view stays on whatever subview was active when hide fired \
             (e.g. `AppView::FileSearchView`), and the next show starts \
             with a stale view state. This mirrors what \
             `hide_main_window_helper` does for the non-RPC hide path."
        );
    }
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn hide_rpc_dispatchers_rekey_semantic_surface_to_script_list() {
    let expected = "crate::windows::update_automation_semantic_surface(\n                                    \"main\",\n                                    Some(\"scriptList\".to_string()),\n                                );";
    for (name, source) in [
        (
            "src/main_entry/runtime_stdin_match_core.rs",
            STDIN_CORE_SOURCE,
        ),
        ("src/main_entry/runtime_stdin.rs", STDIN_SOURCE),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP_SOURCE),
    ] {
        let Some(hide_start) = source.find("ExternalCommand::Hide {") else {
            panic!(
                "{name} must define an `ExternalCommand::Hide` arm — the \
                 three-dispatcher pattern requires it"
            );
        };
        let next_arm_offset = source[hide_start..]
            .find("ExternalCommand::SetFilter")
            .expect("Hide arm must be followed by SetFilter arm");
        let hide_body = &source[hide_start..hide_start + next_arm_offset];
        assert!(
            hide_body.contains(expected),
            "{name} `ExternalCommand::Hide` arm must re-key the automation \
             semantic surface back to `\"scriptList\"` via \
             `update_automation_semantic_surface(\"main\", \
             Some(\"scriptList\".to_string()))`. Without this call, a \
             hide issued while in e.g. `FileSearchView` leaves the \
             introspection channel stuck at `\"fileSearch\"` across the \
             next show — diverging from `getState.promptType` and \
             breaking `automation-semantic-surface-reflects-active-appview` \
             (Pass #19)."
        );
    }
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn hide_rpc_dispatchers_sequence_reset_then_rekey() {
    // Order matters: `reset_to_script_list` flips `view.current_view`
    // back to `AppView::ScriptList`, and only then does it make sense
    // to re-key the automation surface. If the re-key ran first, a
    // racing `sync_main_automation_window` triggered by an intervening
    // bounds event (observed at `runtime_window.rs:81` during Pass #19
    // live verification) could silently preserve the stale surface
    // before `reset_to_script_list` mutated the view — defeating the
    // whole fix. `hide_main_window_helper` follows the same
    // reset-then-rekey ordering (see window_visibility.rs:389 + :397).
    for (name, source) in [
        (
            "src/main_entry/runtime_stdin_match_core.rs",
            STDIN_CORE_SOURCE,
        ),
        ("src/main_entry/runtime_stdin.rs", STDIN_SOURCE),
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP_SOURCE),
    ] {
        let hide_start = source
            .find("ExternalCommand::Hide {")
            .unwrap_or_else(|| panic!("{name} missing ExternalCommand::Hide arm"));
        let next_arm = source[hide_start..]
            .find("ExternalCommand::SetFilter")
            .expect("Hide arm must be followed by SetFilter arm");
        let hide_body = &source[hide_start..hide_start + next_arm];

        let reset_idx = hide_body
            .find("view.reset_to_script_list(ctx);")
            .unwrap_or_else(|| panic!("{name} Hide arm missing reset_to_script_list"));
        let rekey_idx = hide_body
            .find("update_automation_semantic_surface(")
            .unwrap_or_else(|| {
                panic!("{name} Hide arm missing update_automation_semantic_surface")
            });

        assert!(
            reset_idx < rekey_idx,
            "{name} `ExternalCommand::Hide` arm must call \
             `view.reset_to_script_list(ctx);` BEFORE \
             `update_automation_semantic_surface(...)`. Reversing the \
             order opens a race where a window-bounds observer could \
             clobber the re-key while the view still reports the stale \
             subview."
        );
    }
}

// doc-anchor-removed: [[removed-docs metadata]]
#[test]
fn hide_main_window_helper_still_exists_with_same_pattern() {
    // Pass #19 added the re-key + reset to `hide_main_window_helper`
    // too. The RPC path in this pass mirrors that pattern rather than
    // introducing a divergent one — so the non-RPC hide helper MUST
    // still do the same thing. If someone deletes the helper's re-key
    // later, this test (and the live regression) catches it.
    const HELPER_SOURCE: &str = include_str!("../src/main_sections/window_visibility.rs");
    assert!(
        HELPER_SOURCE.contains("fn hide_main_window_helper("),
        "src/main_sections/window_visibility.rs must still define \
         `hide_main_window_helper` — this is the non-RPC hide entry \
         point (menu dismissal, Esc, etc.). If you are refactoring \
         this away, migrate the RPC hide path in tandem."
    );
    assert!(
        HELPER_SOURCE.contains("view.reset_to_script_list(ctx);"),
        "`hide_main_window_helper` must still call \
         `view.reset_to_script_list(ctx);` — the RPC hide path and \
         the helper are intentionally parallel to stay consistent."
    );
    assert!(
        HELPER_SOURCE.contains(
            "crate::windows::update_automation_semantic_surface(\"main\", Some(\"scriptList\".to_string()));"
        ),
        "`hide_main_window_helper` must still re-key via \
         `update_automation_semantic_surface(\"main\", \
         Some(\"scriptList\".to_string()))` — the RPC hide path mirrors \
         this exact call; they must not diverge."
    );
}
