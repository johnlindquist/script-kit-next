const ACP_VIEW: &str = include_str!("../../src/ai/acp/view.rs");
const ACTIONS_BUILDERS: &str = include_str!("../../src/actions/builders/script_context.rs");
const ACTIONS_TOGGLE: &str = include_str!("../../src/app_impl/actions_toggle.rs");
const ACTIONS_DIALOG_IMPL: &str = include_str!("../../src/actions/dialog.rs");
const ACTIONS_DIALOG: &str = include_str!("../../src/app_impl/actions_dialog.rs");
const HANDLE_ACTION: &str = include_str!("../../src/app_actions/handle_action/mod.rs");

fn source_between<'a>(source: &'a str, start_marker: &str, end_marker: &str) -> &'a str {
    let start = source
        .find(start_marker)
        .unwrap_or_else(|| panic!("missing start marker: {start_marker}"));
    let rest = &source[start..];
    let end = rest
        .find(end_marker)
        .unwrap_or_else(|| panic!("missing end marker: {end_marker}"));
    &rest[..end]
}

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
fn focused_text_expanded_uses_standard_agent_chat_footer_not_focused_mini_footer() {
    let footer_fn = source_between(
        ACP_VIEW,
        "fn footer_buttons_for_thread",
        "fn focused_text_visible_footer_buttons",
    );

    assert!(
        footer_fn.contains(
            "self.focused_text.is_some() && self.ui_variant == AcpChatUiVariant::FocusedTextMini"
        ),
        "focused-text footer override must only apply while FocusedTextMini is active"
    );
    assert!(
        !footer_fn.contains(
            "if self.focused_text.is_some() {\n            return self.focused_text_visible_footer_buttons(thread);"
        ),
        "expanded focused-text Agent Chat must fall through to standard Run/Actions footer"
    );
}

#[test]
fn focused_text_expanded_actions_do_not_offer_collapse() {
    let semantic_fn = source_between(
        ACP_VIEW,
        "fn focused_text_semantic_actions",
        "fn has_pastable_assistant_response",
    );

    assert!(
        semantic_fn.contains("if !expanded"),
        "Chat/expand action should be mini-only"
    );
    assert!(
        !semantic_fn.contains("\"focused-text-action-collapse\""),
        "expanded-from-mini should not expose a custom Collapse action"
    );
}

#[test]
fn focused_text_mini_result_footer_is_replace_only() {
    let footer_fn = source_between(
        ACP_VIEW,
        "fn focused_text_visible_footer_buttons",
        "fn focused_text_semantic_actions",
    );
    assert!(footer_fn.contains("focused_text_mini_result_ready_for_thread"));
    assert!(footer_fn.contains("return Vec::new()"));

    let mini_branch = source_between(
        footer_fn,
        "if self.ui_variant == AcpChatUiVariant::FocusedTextMini",
        "match thread.status",
    );

    assert!(mini_branch.contains("FooterAction::Replace"));
    assert!(!mini_branch.contains("FooterAction::Stop"));
    assert!(!mini_branch.contains("FooterAction::Actions"));
    assert!(!mini_branch.contains("FooterAction::Append"));
    assert!(!mini_branch.contains("FooterAction::Copy"));
    assert!(!mini_branch.contains("FooterAction::Expand"));
    assert!(!mini_branch.contains("FooterAction::Retry"));
}

#[test]
fn focused_text_input_only_omits_semantic_actions_and_preview() {
    for required in [
        "FocusedTextMiniPhase::InputOnly",
        "FocusedTextMiniPhase::Loading",
        "collect_focused_text_mini_elements",
        "\"focused-text-context-badge\"",
        "\"focused-text-context-status\"",
        "\"focused-text-profile-icon\"",
        "\"inputOnly\"",
        "if result_ready",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing input-only automation contract: {required}"
        );
    }
}

#[test]
fn focused_text_mini_loading_has_no_body_thinking_text() {
    let render_fn = source_between(
        ACP_VIEW,
        "fn render_focused_text_mini",
        "fn render_pending_context_chips",
    );

    assert!(!render_fn.contains("\"Thinking"));
    assert!(!render_fn.contains("focused-text-thinking"));
    assert!(!render_fn.contains(".child(\"✦\")"));
    assert!(!render_fn.contains("Edit, refine, ask"));
    assert!(ACP_VIEW.contains("FOCUSED_TEXT_MINI_PLACEHOLDER"));
    assert!(render_fn.contains("Self::render_composer_input_text"));
    assert!(render_fn.contains("focused_text_mini_input_height()"));
    assert!(render_fn.contains("crate::panel::HEADER_PADDING_X"));
    assert!(render_fn.contains("active_pending"));
    assert!(render_fn.contains("render_input_profile_icon"));
    assert!(render_fn.contains("FOCUSED_TEXT_MINI_INPUT_MAX_VISIBLE_HEIGHT"));
    assert!(render_fn.contains("focused-text-mini-preview-enter"));
    assert!(render_fn.contains("input_locked"));
    assert!(!render_fn.contains("focused-text-mini-close"));
    assert!(!render_fn.contains(".child(\"×\")"));
    assert!(!render_fn.contains("trigger_close_window_requested"));
    assert!(ACP_VIEW.contains("fn focused_text_locked_input_allows_key"));
    assert!(ACP_VIEW.contains("Self::focused_text_locked_input_allows_key(key)"));
}

#[test]
fn full_agent_chat_profile_icon_moves_to_header_focused_text_keeps_icon() {
    let composer_fn = source_between(
        ACP_VIEW,
        "fn render_composer_input_shell",
        "fn render_composer_bar",
    );
    let mini_render_fn = source_between(
        ACP_VIEW,
        "fn render_focused_text_mini",
        "fn render_pending_context_chips",
    );
    let footer_marker_fn = source_between(
        ACP_VIEW,
        "fn render_profile_status_marker_from_snapshot",
        "pub(crate) fn build_external_host_footer",
    );
    let left_info_fn = source_between(
        ACP_VIEW,
        "pub(crate) fn profile_left_info",
        "#[derive(Clone, Debug)]",
    );

    assert!(!composer_fn.contains("\"agent-chat-input-profile-icon\""));
    assert!(!composer_fn.contains("let profile_icon = Self::render_input_profile_icon"));
    assert!(composer_fn.contains("trailing: Vec::new()"));
    assert!(mini_render_fn.contains("\"focused-text-profile-icon\""));
    assert!(mini_render_fn.contains("render_input_profile_icon"));
    let input_pos = mini_render_fn
        .find(".id(\"focused-text-input\")")
        .expect("mini input should render before trailing accessories");
    let app_badge_pos = mini_render_fn
        .find("render_focused_text_app_icon_badge")
        .expect("app badge should remain in the mini input row");
    let profile_pos = mini_render_fn
        .find("\"focused-text-profile-icon\"")
        .expect("profile icon should remain semantic");
    assert!(input_pos < app_badge_pos && app_badge_pos < profile_pos);
    assert!(!mini_render_fn.contains(".child(self.render_focused_text_profile_icon"));
    assert!(!footer_marker_fn.contains("footer_icon_path_or_profile"));
    assert!(!footer_marker_fn.contains("acp-footer-profile-icon-pulse"));
    assert!(left_info_fn.contains("icon_token: None"));
    assert!(!left_info_fn.contains("icon_token: Some"));
}

#[test]
fn focused_text_mini_semantics_are_redacted_and_diagnostic() {
    for required in [
        "word_count",
        "context_status",
        "submitted_prompt_locked",
        "submitted_prompt_char_count",
        "input_redacted",
        "focused_text_context_status_label",
        "focused-text-context-status",
        "focused-text-profile-icon",
        "value: None",
        "submitted_prompt_locked",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text diagnostic/redaction contract: {required}"
        );
    }

    assert!(
        !ACP_VIEW.contains("value: Some(thread.input.text().to_string())"),
        "focused-text mini semantic input must not expose raw prompt text"
    );
}

#[test]
fn focused_text_secondary_actions_remain_semantic_cmd_k_actions() {
    for required in [
        "focused_text_semantic_actions",
        "struct FocusedTextSemanticActionSpec",
        "\"focused-text-action-append\"",
        "\"focused-text-action-copy\"",
        "\"focused-text-action-expand\"",
        "source_name: Some(\"Cmd+K\"",
    ] {
        assert!(
            ACP_VIEW.contains(required),
            "missing focused-text semantic Cmd+K action contract: {required}"
        );
    }
}

#[test]
fn focused_text_cmd_k_uses_focused_text_action_route_not_generic_acp_actions() {
    for required in [
        "pub(crate) fn get_focused_text_agent_chat_actions",
        "pub(crate) fn get_focused_text_agent_chat_root_route",
        "\"Focused Text\"",
        "\"Replace Selected Text\"",
        "\"Append to Selected Text\"",
        "\"Copy Response\"",
        "\"focused-text-action-replace\"",
        "initial_selected_action_id: Some(\"focused-text-action-replace\"",
    ] {
        assert!(
            ACTIONS_BUILDERS.contains(required),
            "missing focused-text Cmd+K action builder contract: {required}"
        );
    }

    for required in [
        "focused_text: bool",
        "focused_text_expanded: bool",
        "get_focused_text_agent_chat_root_route(context.focused_text_expanded)",
    ] {
        assert!(
            ACTIONS_DIALOG_IMPL.contains(required),
            "missing focused-text Cmd+K dialog routing contract: {required}"
        );
    }

    for required in [
        "view.has_focused_text_context()",
        "view.focused_text_actions_expanded()",
        "focused_text,",
        "focused_text_expanded,",
    ] {
        assert!(
            ACTIONS_TOGGLE.contains(required),
            "missing focused-text Cmd+K host context contract: {required}"
        );
    }
}
