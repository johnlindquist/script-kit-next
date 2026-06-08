//! Source-level contract for advanced agentic-testing hard-scenario recipes.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");
const TARGET_THREAD: &str = include_str!("../scripts/agentic/target-thread.ts");

#[test]
fn index_help_exposes_hard_scenario_recipes() {
    for name in [
        "agent_chat-detached-target-threading-stress",
        "agent_chat-prompt-popup-parity",
        "notes-agent_chat-delayed-action-origin-stress",
    ] {
        assert!(
            INDEX.contains(&format!("name: \"{name}\"")),
            "help --json must advertise {name}"
        );
        assert!(
            INDEX.contains(&format!("case \"{name}\"")),
            "index.ts must route {name}"
        );
    }
}

#[test]
fn detached_stress_recipe_requires_native_input_and_capture_identity() {
    assert!(
        INDEX.contains("force-native"),
        "detached Agent Chat target-threading stress must force native input"
    );
    for token in [
        "agent_chat-detached-target-threading-stress",
        "targetThread",
        "peerWindows",
        "captureTarget",
        "usedNativeInput",
        "requestedWindowId",
        "actualWindowId",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "detached stress receipt must include {token}"
        );
    }
}

#[test]
fn prompt_popup_parity_receipt_pins_family_and_rows() {
    for token in [
        "agent_chat-prompt-popup-parity",
        "popupCases",
        "popupFamily",
        "popupId",
        "rowAware",
        "rowCount",
        "wrong_popup_family",
        "wrong_popup_id",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token) || TARGET_THREAD.contains(token),
            "PromptPopup parity receipt must include {token}"
        );
    }
}

#[test]
fn notes_origin_stress_fails_closed_until_origin_generation_receipts_exist() {
    for token in [
        "notes-agent_chat-delayed-action-origin-stress",
        "origin",
        "agent_chatGeneration",
        "delayedAction",
        "missingOriginGeneration",
        "missing_origin_generation",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "Notes Agent Chat origin stress must expose fail-closed token {token}"
        );
    }
}

#[test]
fn target_thread_helper_promotes_kind_targets_to_exact_id() {
    for token in [
        "promoteExactTarget",
        "assertTargetStable",
        "targetedRpc",
        "untargeted_rpc_forbidden",
        "target_identity_drift",
        "targetJson: { type: \"id\"",
        "originAgentChatViewId",
        "originAgentChatGeneration",
        "portalId",
        "portalFamily",
        "recorderId",
        "recorderGeneration",
    ] {
        assert!(
            TARGET_THREAD.contains(token),
            "target-thread helper must pin {token}"
        );
    }
}
