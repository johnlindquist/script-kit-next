//! Source-level contract for sixth-loop agentic-testing hard scenarios.

const INDEX: &str = include_str!("../scripts/agentic/index.ts");
const SCENARIO: &str = include_str!("../scripts/agentic/scenario.ts");

#[test]
fn index_help_exposes_loop_six_recipes() {
    for name in [
        "permission-assistant-drag-preflight-stress",
        "quick-terminal-pty-apply-back-stress",
        "mcp-context-resource-attachment-identity-stress",
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
fn permission_assistant_drag_stress_pins_read_only_drag_receipts() {
    for token in [
        "permission-assistant-drag-preflight-stress",
        "missing_permission_assistant_drag_preflight_receipt",
        "permissionAssistant",
        "PassiveOverlayPanel",
        "nonactivatingPanel",
        "AppDragSourceView",
        "fileURL",
        "settings_window_snapshot",
        "openedSystemSettings: false",
        "mutatedTcc: false",
        "calledPromptingApi: false",
        "wrongPaneAccepted",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "Permission Assistant drag stress must pin {token}"
        );
    }
}

#[test]
fn quick_terminal_apply_back_stress_pins_pty_lifecycle_receipts() {
    for token in [
        "quick-terminal-pty-apply-back-stress",
        "missing_quick_terminal_apply_back_receipt",
        "quickTerminal",
        "ptyReady",
        "shellOutputReceipt",
        "applyBackTarget",
        "focusRestored",
        "selectionPreserved",
        "ptyShutdownReceipt",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "Quick Terminal apply-back stress must pin {token}"
        );
    }
}

#[test]
fn mcp_context_resource_stress_pins_resource_identity_receipts() {
    for token in [
        "mcp-context-resource-attachment-identity-stress",
        "missing_mcp_context_resource_attachment_receipt",
        "mcpContextResource",
        "kit://context/agentic-loop-six",
        "agentic-test",
        "mcp-resource",
        "acceptedContextPart",
        "openedWithoutManualPicker",
        "staleResource",
        "wrongProfileAccepted",
    ] {
        assert!(
            INDEX.contains(token) || SCENARIO.contains(token),
            "MCP context resource stress must pin {token}"
        );
    }
}
