//! Source-level contract for targeted `getState`.
//!
//! `getState` keeps its optional target wire field for compatibility, but its
//! `stateResult` payload is the main-window state contract. Secondary surfaces
//! are inspected through `getElements(target)`, `inspectAutomationWindow(target)`,
//! and target-specific state APIs.

const QUERY_VARIANTS: &str = include_str!("../src/protocol/message/variants/query_ops.rs");
const QUERY_CONSTRUCTORS: &str = include_str!("../src/protocol/message/constructors/query_ops.rs");
const PROMPT_HANDLER: &str = include_str!("../src/prompt_handler/mod.rs");
const PROTOCOL_DOCS: &str = include_str!("../lat.md/protocol.md");
const AUTOMATION_DOCS: &str = include_str!("../lat.md/automation.md");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

// @lat: [[lat.md/protocol#Protocol#Query and introspection]]
#[test]
fn get_state_wire_variant_keeps_optional_target() {
    let get_state_variant = source_between(
        QUERY_VARIANTS,
        "#[serde(rename = \"getState\")]",
        "/// Response with current UI state.",
    );

    assert!(get_state_variant.contains("GetState {"));
    assert!(get_state_variant.contains("target: Option<AutomationWindowTarget>"));
    assert!(
        get_state_variant.contains("#[serde(default, skip_serializing_if = \"Option::is_none\")]")
    );
}

#[test]
fn get_state_constructor_still_defaults_to_untargeted_main() {
    let constructor = source_between(
        QUERY_CONSTRUCTORS,
        "pub fn get_state(request_id: String) -> Self",
        "/// Create a state result response",
    );

    assert!(constructor.contains("Message::GetState"));
    assert!(constructor.contains("target: None"));
}

#[test]
fn get_state_target_resolution_is_named_main_only_contract() {
    for required in [
        "enum GetStateTargetResolution",
        "fn resolve_get_state_target(",
        "getState is a main-window state contract",
        "getElements(target), inspectAutomationWindow(target)",
        "getAcpState(target)",
    ] {
        assert!(
            PROMPT_HANDLER.contains(required),
            "prompt_handler must name the getState target contract: {required}"
        );
    }
}

#[test]
fn get_state_main_compatible_targets_are_preserved() {
    let helper = source_between(
        PROMPT_HANDLER,
        "fn resolve_get_state_target(",
        "\n/// Which window an ACP read should target.",
    );

    for required in [
        "None",
        "AutomationWindowTarget::Main",
        "AutomationWindowTarget::Focused",
        "GetStateTargetResolution::MainCompatible",
    ] {
        assert!(
            helper.contains(required),
            "main-compatible target handling must include {required}"
        );
    }
}

#[test]
fn get_state_non_main_targets_return_unsupported_diagnostic() {
    let helper = source_between(
        PROMPT_HANDLER,
        "fn resolve_get_state_target(",
        "\n/// Which window an ACP read should target.",
    );
    let get_state_arm = source_between(
        PROMPT_HANDLER,
        "PromptMessage::GetState { request_id, target } =>",
        "// Collect current UI state",
    );

    assert!(get_state_arm.contains("match resolve_get_state_target(&request_id, target.as_ref())"));
    assert!(helper.contains("crate::windows::resolve_automation_window(Some(t))"));
    assert!(helper.contains("resolved.kind == crate::protocol::AutomationWindowKind::Main"));
    assert!(helper.contains("getState: secondary window state not yet routed"));
    assert!(helper.contains("GetStateTargetResolution::UnsupportedNonMain { resolved }"));

    for required in [
        "\"unsupported\".to_string()",
        "target_unsupported:{:?}",
        "resolved.visible",
        "String::new()",
    ] {
        assert!(
            get_state_arm.contains(required),
            "unsupported getState diagnostic must preserve {required}"
        );
    }
}

#[test]
fn get_state_resolution_failure_stays_distinct_from_unsupported_target() {
    let helper = source_between(
        PROMPT_HANDLER,
        "fn resolve_get_state_target(",
        "\n/// Which window an ACP read should target.",
    );
    let get_state_arm = source_between(
        PROMPT_HANDLER,
        "PromptMessage::GetState { request_id, target } =>",
        "// Collect current UI state",
    );

    assert!(helper.contains("GetStateTargetResolution::ResolutionFailed"));
    assert!(helper.contains("getState: target resolution failed"));
    assert!(get_state_arm.contains("\"target_resolution_failed\".to_string()"));
    assert!(get_state_arm.contains("target_error:{}"));
}

#[test]
fn protocol_docs_name_supported_secondary_inspection_path() {
    for required in [
        "main-window state contract",
        "target_unsupported",
        "target_resolution_failed",
        "target_error:",
        "getElements(target)",
        "inspectAutomationWindow(target)",
        "getAcpState(target)",
    ] {
        assert!(
            PROTOCOL_DOCS.contains(required),
            "protocol docs must name getState target boundary: {required}"
        );
    }

    assert!(
        !PROTOCOL_DOCS.contains("getState and getElements both support explicit window targets"),
        "docs must not imply targeted getState supports secondary state"
    );

    assert!(
        AUTOMATION_DOCS.contains("instead of relying on targeted `getState`"),
        "automation docs must direct secondary proof away from targeted getState"
    );
}

#[test]
fn get_elements_remains_the_secondary_semantic_inspection_path() {
    let get_elements_arm = source_between(
        PROMPT_HANDLER,
        "PromptMessage::GetElements {",
        "\n            PromptMessage::GetLayoutInfo",
    );

    for required in [
        "crate::windows::resolve_automation_window(Some(t))",
        "collect_surface_snapshot(",
        "target_unsupported_non_main: getElements has no collector",
        "target_resolution_failed: {}",
    ] {
        assert!(
            get_elements_arm.contains(required),
            "getElements must remain the secondary inspection path: {required}"
        );
    }
}
