//! Source-level contract for attached-popup image-library navigation.
//!
//! Attached popup coverage spans Actions Dialog and the promoted ACP slash
//! Prompt Popup proof.

const NAVIGATOR: &str = include_str!("../scripts/agentic/surface-navigator.ts");
const MATRIX: &str = include_str!("../scripts/agentic/attached-popup-surface-matrix.ts");

#[test]
fn attached_popup_matrix_declares_actions_dialog_active_cases() {
    assert!(
        MATRIX.contains("actions-dialog-attached-popup"),
        "attached popup matrix must keep the base Actions Dialog screenshot case"
    );
    for case_id in [
        "actions-dialog-on-clipboard-history",
        "actions-dialog-on-emoji-picker",
        "actions-dialog-on-app-launcher",
    ] {
        assert!(
            MATRIX.contains(case_id),
            "attached popup matrix must include hosted Actions Dialog case {case_id}"
        );
    }
    assert!(
        MATRIX.contains("windowKind: \"ActionsDialog\""),
        "hosted attached popup cases must still declare ActionsDialog"
    );
    assert!(
        MATRIX.contains("parent_capture_with_crop"),
        "attached popup proofs must expect parent capture with popup crop bounds"
    );
}

#[test]
fn attached_popup_matrix_declares_prompt_popup_active_case() {
    assert!(
        MATRIX.contains("id: \"prompt-popup-on-acp-chat-slash\"")
            && MATRIX.contains("windowKind: \"PromptPopup\"")
            && MATRIX.contains("targetKind: \"promptPopup\"")
            && MATRIX.contains("expectedAutomationWindowId: \"acp-mention-popup\"")
            && MATRIX.contains("hostFixture: { kind: \"acp-chat\", trigger: \"slash\" }")
            && MATRIX.contains("parent_capture_with_crop"),
        "attached popup matrix must include the promoted ACP slash Prompt Popup case"
    );
    assert!(
        !MATRIX.contains("prompt-popup-on-acp-chat-slash-candidate")
            && !MATRIX.contains("PROMPT_POPUP_FIXTURE_MATRIX"),
        "active attached-popup matrix must use durable Prompt Popup case names"
    );
}

#[test]
fn attached_popup_promotion_can_require_exact_automation_id() {
    let promote_start = NAVIGATOR
        .find("async function promoteAttachedPopupTarget")
        .expect("navigator must define attached popup target promotion");
    let promote = &NAVIGATOR[promote_start..];
    let id_check = promote
        .find("automationWindowId !== entry.expectedAutomationWindowId")
        .expect("navigator must compare promoted popup id with expected matrix id");
    let return_resolved = promote
        .find("resolvedTarget: {")
        .expect("navigator must return a promoted resolved target");
    assert!(
        id_check < return_resolved,
        "expected automation id check must happen before returning a promoted popup target"
    );
    assert!(
        MATRIX.contains("expectedAutomationWindowId?: string")
            && NAVIGATOR.contains("entry.expectedAutomationWindowId")
            && NAVIGATOR.contains("expected automation window id"),
        "attached-popup entries must support fail-closed expected automation id checks"
    );
}

#[test]
fn attached_popup_cases_use_filterable_main_host_fixtures() {
    assert!(
        MATRIX.contains("hostFixture")
            && MATRIX.contains("kind: \"filterable-main\"")
            && MATRIX.contains("caseId: \"clipboard-history-visible-rows\"")
            && MATRIX.contains("caseId: \"emoji-picker-visible-rows\"")
            && MATRIX.contains("caseId: \"app-launcher-visible-rows\""),
        "hosted Actions Dialog cases must reuse stable filterable-main matrix fixtures"
    );
}

#[test]
fn navigator_supports_explicit_attached_popup_group() {
    assert!(
        NAVIGATOR.contains("\"filterable-main\" | \"attached-popup\"")
            && NAVIGATOR.contains("| \"all-active\""),
        "navigator must expose an explicit attached-popup group without changing the default group"
    );
    assert!(
        NAVIGATOR.contains("argValue(\"--group\", \"filterable-main\")"),
        "filterable-main must remain the default group"
    );
    assert!(
        NAVIGATOR.contains("selectedAttachedPopupCases"),
        "attached-popup group must select from the attached popup matrix"
    );
}

#[test]
fn actions_dialog_setup_uses_protocol_cmd_k_not_scenario_helpers() {
    assert!(
        NAVIGATOR.contains("type: \"simulateKey\", key: \"k\", modifiers: [\"cmd\"]"),
        "Actions Dialog setup must open the dialog through the Cmd+K protocol path"
    );
    assert!(
        !NAVIGATOR.contains("runActionsDialogExactIdScenario")
            && !NAVIGATOR.contains("runPromptPopupExactIdScenario"),
        "surface navigator must not use scenario helpers as attached-popup setup sources"
    );
}

#[test]
fn attached_popup_host_setup_is_state_first_before_cmd_k() {
    let host_setup = NAVIGATOR
        .find("enterAttachedPopupHostFixture")
        .expect("navigator must define attached popup host setup");
    let cmd_k = NAVIGATOR
        .find("type: \"simulateKey\", key: \"k\", modifiers: [\"cmd\"]")
        .expect("navigator must still open actions dialog through stdin Cmd+K");
    assert!(
        host_setup < cmd_k,
        "host fixture setup must be defined and used before Cmd+K popup open"
    );
    assert!(
        NAVIGATOR.contains("enterFilterableSurface(")
            && NAVIGATOR.contains("waitForPromptType(")
            && NAVIGATOR.contains("getStateAndElements("),
        "attached popup host setup must reuse filterable state/elements receipts"
    );
}

#[test]
fn attached_popup_manifest_carries_host_fixture_receipts() {
    for field in [
        "hostFixture",
        "hostSetup",
        "hostObservation",
        "hostResolvedTarget",
    ] {
        assert!(
            NAVIGATOR.contains(field),
            "hosted attached-popup manifest must carry {field}"
        );
    }
}

#[test]
fn attached_popup_capture_requires_strict_identity_and_popup_crop() {
    assert!(
        NAVIGATOR.contains("\"--target-json\"")
            && NAVIGATOR.contains("\"--capture-window-id\"")
            && NAVIGATOR.contains("\"--strict-window\""),
        "attached popup capture must use the same strict target/window identity as main surfaces"
    );
    assert!(
        NAVIGATOR.contains("popupCapture?.strategy")
            && NAVIGATOR.contains("popupCapture.targetBounds"),
        "attached popup capture must validate popupCapture strategy and crop bounds"
    );
    assert!(
        NAVIGATOR.contains("preCaptureInspection") && NAVIGATOR.contains("preCaptureElements"),
        "attached popup sidecars and manifest must preserve pre-capture semantic receipts"
    );
}

#[test]
fn attached_popup_navigator_does_not_use_macos_input() {
    assert!(
        !NAVIGATOR.contains("macos-input.ts"),
        "attached popup navigator should stay on protocol-level setup and receipts"
    );
}
