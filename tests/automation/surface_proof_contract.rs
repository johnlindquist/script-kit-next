//! Source-audit tests for the seconds-first `surface-proof` contract.

use script_kit_gpui::test_utils::read_source;

#[test]
fn scenario_cli_lists_all_surface_proof_scenarios() {
    let source = read_source("scripts/agentic/scenario.ts");

    for expected in [
        "\"main-window-exact-id\"",
        "\"actions-dialog-exact-id\"",
        "\"prompt-popup-exact-id\"",
        "\"detached-acp-exact-id\"",
    ] {
        assert!(
            source.contains(expected),
            "scenario.ts must advertise {expected}"
        );
    }
}

#[test]
fn index_help_exposes_surface_proof_as_default_entrypoint() {
    let source = read_source("scripts/agentic/index.ts");

    assert!(
        source.contains("\"surface-proof\""),
        "index.ts help JSON must expose surface-proof"
    );
    assert!(
        source.contains("surface-proof --session default --kind main"),
        "help must show main proof example"
    );
    assert!(
        source.contains("surface-proof --session default --kind promptPopup"),
        "help must show attached popup proof example"
    );
    assert!(
        source.contains("surface-proof --session default --kind acpDetached"),
        "help must show detached proof example"
    );
}

#[test]
fn surface_proof_keeps_non_main_no_focus_routing() {
    let source = read_source("scripts/agentic/index.ts");
    assert!(
        source.contains("acpDetached, actionsDialog, promptPopup → batch/simulateGpuiEvent"),
        "help text must preserve non-main no-focus routing contract"
    );
}

#[test]
fn attached_popup_proof_depends_on_popup_semantics_receipt() {
    let source = read_source("scripts/agentic/scenario.ts");
    assert!(
        source.contains("batch_unavailable"),
        "attached popup scenario must fail or warn when popup semantics degrade"
    );
}
