//! Source-level contract test for the `main-menu-cmd-enter-ai` user story
//! (Run 2 Pass #24) updated for unified simulateKey dispatcher.

const CANONICAL_SIMULATEKEY: &str = include_str!("../src/app_impl/simulate_key_dispatch.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const RUNTIME_MATCH_SIMULATEKEY: &str =
    include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");

#[test]
fn simulate_key_dispatchers_route_cmd_enter_in_scriptlist_to_acp_context_capture() {
    // 1. Verify the canonical simulateKey dispatcher contains the correct Cmd+Enter routing inside AppView::ScriptList.
    let scriptlist_anchor = "AppView::ScriptList => {";
    let Some(anchor_idx) = CANONICAL_SIMULATEKEY.find(scriptlist_anchor) else {
        panic!(
            "src/app_impl/simulate_key_dispatch.rs is missing the `AppView::ScriptList` anchor. \
             Layout has shifted; update this test."
        );
    };

    let arm_body: String = CANONICAL_SIMULATEKEY[anchor_idx..]
        .lines()
        .take(60)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        arm_body.contains("has_cmd")
            && arm_body.contains("key_lower == \"enter\"")
            && arm_body.contains("!has_shift")
            && arm_body.contains("!_has_alt")
            && arm_body.contains("!_has_ctrl"),
        "simulate_key_dispatch.rs `AppView::ScriptList` must gate the Cmd+Enter routing on modifiers"
    );

    assert!(
        arm_body.contains("view.try_route_global_cmd_enter_to_acp_context_capture(ctx);"),
        "simulate_key_dispatch.rs `AppView::ScriptList` must call try_route_global_cmd_enter_to_acp_context_capture"
    );

    // 2. Verify that the entry points delegate to the unified helper.
    for (name, source) in [
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP),
        (
            "src/main_entry/runtime_stdin_match_simulate_key.rs",
            RUNTIME_MATCH_SIMULATEKEY,
        ),
    ] {
        assert!(
            source.contains("dispatch_simulate_key"),
            "{name} must delegate to dispatch_simulate_key helper"
        );
    }
}

#[test]
fn simulate_key_cmd_enter_arm_precedes_plain_enter_in_scriptlist() {
    // Ordering contract: the Cmd+Enter arm MUST appear before the plain-enter arm.
    let cmd_enter_idx = CANONICAL_SIMULATEKEY
        .find("SimulateKey: Cmd+Enter - route to ACP context capture")
        .expect("missing Cmd+Enter log line in simulate_key_dispatch.rs");

    let plain_enter_idx = CANONICAL_SIMULATEKEY
        .find("SimulateKey: Enter - execute selected")
        .expect("missing plain-Enter log line in simulate_key_dispatch.rs");

    assert!(
        cmd_enter_idx < plain_enter_idx,
        "Cmd+Enter arm must appear BEFORE the plain-Enter arm inside the ScriptList match block"
    );
}

#[test]
fn try_route_global_cmd_enter_to_acp_context_capture_still_defined() {
    const TAB_AI_MODE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");
    assert!(
        TAB_AI_MODE.contains("pub(crate) fn try_route_global_cmd_enter_to_acp_context_capture("),
        "try_route_global_cmd_enter_to_acp_context_capture must still be defined"
    );
}
