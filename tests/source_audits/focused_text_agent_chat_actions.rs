const ACP_VIEW: &str = include_str!("../../src/ai/acp/view.rs");
const ACTIONS_DIALOG: &str = include_str!("../../src/app_impl/actions_dialog.rs");
const HANDLE_ACTION: &str = include_str!("../../src/app_actions/handle_action/mod.rs");

#[test]
fn focused_text_action_ids_route_through_acp_view_dispatcher() {
    for required in [
        "\"focused-text-action-replace\"",
        "\"focused-text-action-append\"",
        "\"focused-text-action-copy\"",
        "\"focused-text-action-expand\"",
        "\"focused-text-action-collapse\"",
        "\"focused-text-action-stop\"",
        "\"focused-text-action-retry\"",
        "pub(crate) fn perform_focused_text_mini_action",
        "apply_focused_text_output",
        "set_ui_variant(AcpChatUiVariant::Standard",
        "set_ui_variant(AcpChatUiVariant::FocusedTextMini",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text mini action contract in ACP view: {required}"
        );
    }

    for required in [
        "FocusedTextMiniAction::from_action_id(action_id)",
        "view.perform_focused_text_mini_action(action, cx)",
        "focused_text_mini_action_dispatched",
    ] {
        assert!(
            HANDLE_ACTION.contains(required),
            "missing focused-text action router contract: {required}"
        );
    }

    assert!(
        ACTIONS_DIALOG.contains("ActionsDialogHost::AcpChat"),
        "TriggerAction host=acpChat must continue to route through shared actions"
    );
    assert!(
        !HANDLE_ACTION.contains("crate::inline_agent::"),
        "focused-text Agent Chat action routing must not call the legacy inline-agent window"
    );
}

#[test]
fn focused_text_action_receipts_are_redacted_protocol_state() {
    for required in [
        "AcpFocusedTextActionReceipt",
        "last_action_receipt",
        "context_present",
        "context_fingerprint",
        "output_length",
        "error_code",
    ] {
        assert!(
            ACP_VIEW.contains(required)
                || include_str!("../../src/protocol/types/acp_state.rs").contains(required),
            "missing redacted focused-text receipt/state field: {required}"
        );
    }

    for forbidden in [
        "captured_text",
        "assistant_output",
        "prompt_xml",
        "clipboard_text",
    ] {
        assert!(
            !include_str!("../../src/protocol/types/acp_state.rs").contains(forbidden),
            "focused-text action receipts must not expose raw sensitive field: {forbidden}"
        );
    }
}

#[test]
fn focused_text_footer_persists_actions_after_expand_and_can_collapse() {
    for required in [
        "if self.focused_text.is_some()",
        "return self.focused_text_footer_buttons(thread)",
        "label: if self.ui_variant == AcpChatUiVariant::FocusedTextMini",
        "\"Collapse\"",
        "\"focused-text-action-collapse\"",
        "set_on_focused_text_collapse_requested",
        "focused_text_collapse_agent_chat",
    ] {
        assert!(
            ACP_VIEW.contains(required)
                || include_str!("../../src/app_impl/tab_ai_mode/mod.rs").contains(required),
            "missing focused-text expanded/collapse footer contract: {required}"
        );
    }
}
