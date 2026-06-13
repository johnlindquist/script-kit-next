const AGENT_CHAT_VIEW: &str = include_str!("../src/ai/agent_chat/ui/view.rs");
const COMPOSER_STATE: &str = include_str!("../src/ai/agent_chat/ui/composer_state.rs");

fn function_body_after(source: &str, signature: &str, next_marker: &str) -> String {
    source
        .split(signature)
        .nth(1)
        .and_then(|rest| rest.split(next_marker).next())
        .unwrap_or_else(|| panic!("{signature} source should exist"))
        .to_string()
}

#[test]
fn agent_chat_composer_picker_has_explicit_transition_owner() {
    assert!(COMPOSER_STATE.contains("enum AgentChatComposerPickerState"));
    assert!(COMPOSER_STATE.contains("enum AgentChatComposerPickerEvent"));
    assert!(COMPOSER_STATE.contains("fn reduce_agent_chat_composer_picker"));
    assert!(COMPOSER_STATE.contains("struct AgentChatComposerPickerRefreshInput"));
}

#[test]
fn refresh_composer_picker_session_delegates_to_state_machine() {
    let refresh = function_body_after(
        AGENT_CHAT_VIEW,
        "pub(super) fn refresh_composer_picker_session",
        "fn log_composer_picker_visible_range",
    );

    assert!(
        refresh.contains("reduce_agent_chat_composer_picker")
            && refresh.contains("AgentChatComposerPickerEvent::Refresh")
            && refresh.contains("apply_composer_picker_transition"),
        "refresh_composer_picker_session must delegate lifecycle decisions to the composer picker state machine"
    );
    assert!(
        !refresh.contains("self.composer_picker_session = next_session"),
        "refresh must not assign the derived session directly"
    );
}

#[test]
fn key_handler_routes_picker_navigation_and_dismissal_through_state_machine() {
    let handle_key_down =
        function_body_after(AGENT_CHAT_VIEW, "fn handle_key_down", "impl Focusable");

    for event in [
        "AgentChatComposerPickerEvent::SlashToggle",
        "AgentChatComposerPickerEvent::NavigatePrevious",
        "AgentChatComposerPickerEvent::NavigateNext",
        "AgentChatComposerPickerEvent::SubmitStarted",
    ] {
        assert!(
            handle_key_down.contains(event),
            "handle_key_down must route {event} through the picker state machine"
        );
    }

    assert!(
        !handle_key_down.contains("self.composer_picker_session = None"),
        "key handling must not close the picker by direct field assignment"
    );
}

#[test]
fn accept_handler_uses_state_machine_for_close_and_inert_keep_open() {
    let accept = function_body_after(
        AGENT_CHAT_VIEW,
        "fn accept_composer_picker_selection_impl",
        "fn should_claim_inline_mention_ownership",
    );

    assert!(accept.contains("AgentChatComposerPickerEvent::Accept"));
    assert!(accept.contains("AgentChatComposerPickerEvent::AcceptIgnoredKeepOpen"));
    assert!(
        !accept.contains("self.composer_picker_session.take()")
            && !accept.contains("self.composer_picker_session = Some(session)"),
        "accept handler should ask the transition owner to close or restore picker state"
    );
}
