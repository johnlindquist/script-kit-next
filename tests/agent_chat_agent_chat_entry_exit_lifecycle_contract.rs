const AGENT_CHAT_SURFACE_TRANSITIONS_SOURCE: &str =
    include_str!("../src/app_impl/agent_chat_surface_transitions.rs");
const TAB_AI_MODE_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode/mod.rs");

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source
        .find(signature)
        .unwrap_or_else(|| panic!("missing function signature: {signature}"));
    let after_start = &source[start..];
    let open = after_start
        .find('{')
        .unwrap_or_else(|| panic!("missing function body for: {signature}"));
    let mut depth = 0usize;
    for (offset, ch) in after_start[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &after_start[..open + offset + 1];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated function body: {signature}");
}

fn assert_before(source: &str, first: &str, second: &str) {
    let first_pos = source
        .find(first)
        .unwrap_or_else(|| panic!("missing first marker: {first}"));
    let second_pos = source
        .find(second)
        .unwrap_or_else(|| panic!("missing second marker: {second}"));
    assert!(
        first_pos < second_pos,
        "expected `{first}` before `{second}`"
    );
}

#[test]
fn embedded_entry_emits_lifecycle_receipt_contract() {
    for needle in [
        "pub(crate) struct AgentChatSurfaceLifecycleReceipt",
        "schema_version",
        "target_automation_id",
        "target_kind",
        "surface_kind",
        "app_view_variant",
        "return_view",
        "return_focus_target",
        "main_rekeyed",
        "embedded_ai_window_visible",
        "actions_popup_cleared",
        "event = \"agent_chat_surface_lifecycle_receipt\"",
        "debug_assert_agent_chat_surface_consistent",
    ] {
        assert!(
            AGENT_CHAT_SURFACE_TRANSITIONS_SOURCE.contains(needle),
            "missing lifecycle receipt marker: {needle}"
        );
    }

    let body = function_body(
        AGENT_CHAT_SURFACE_TRANSITIONS_SOURCE,
        "pub(crate) fn enter_embedded_agent_chat_surface",
    );
    assert_before(
        body,
        "crate::windows::ensure_embedded_ai_window(true)",
        "self.rekey_main_automation_surface_from_current_view()",
    );
    assert_before(
        body,
        "self.rekey_main_automation_surface_from_current_view()",
        "self.transition_agent_chat_surface(AgentChatSurfaceEvent::EmbeddedOpened)",
    );
    assert_before(
        body,
        "self.transition_agent_chat_surface(AgentChatSurfaceEvent::EmbeddedOpened)",
        "self.clear_actions_popup_state()",
    );
    assert_before(
        body,
        "self.clear_actions_popup_state()",
        "FocusRequest::agent_chat",
    );
}

#[test]
fn embedded_exit_is_single_restore_rekey_teardown_actor() {
    let close_body = function_body(TAB_AI_MODE_SOURCE, "fn close_tab_ai_harness_terminal_impl");
    assert_before(
        close_body,
        "view.prepare_for_host_hide(cx);",
        "self.exit_embedded_agent_chat_surface(",
    );
    assert!(
        !close_body.contains("self.transition_agent_chat_surface(\n                crate::ai::agent_chat::ui::surface_state::AgentChatSurfaceEvent::EmbeddedClosed"),
        "embedded close must delegate the surface close transition to exit_embedded_agent_chat_surface"
    );

    let exit_body = function_body(
        AGENT_CHAT_SURFACE_TRANSITIONS_SOURCE,
        "pub(crate) fn exit_embedded_agent_chat_surface",
    );
    for needle in [
        "self.restore_current_view_with_focus(return_view.clone(), return_focus_target)",
        "self.rekey_main_automation_surface_from_current_view()",
        "crate::windows::ensure_embedded_ai_window(false)",
        "self.embedded_agent_chat_focus_handle = None",
        "self.clear_actions_popup_state()",
        "self.transition_agent_chat_surface(AgentChatSurfaceEvent::EmbeddedClosed)",
    ] {
        assert!(
            exit_body.contains(needle),
            "missing exit actor step: {needle}"
        );
    }
}
