const ACP_VIEW: &str = include_str!("../src/ai/acp/view.rs");
const COMPOSER_STATE: &str = include_str!("../src/ai/acp/composer_state.rs");

fn function_body_after(source: &str, signature: &str, next_marker: &str) -> String {
    source
        .split(signature)
        .nth(1)
        .and_then(|rest| rest.split(next_marker).next())
        .unwrap_or_else(|| panic!("{signature} source should exist"))
        .to_string()
}

// @lat: [[lat.md/acp-chat#ACP Chat#ACP composer]]
#[test]
fn acp_composer_picker_has_explicit_transition_owner() {
    assert!(COMPOSER_STATE.contains("enum AcpComposerPickerState"));
    assert!(COMPOSER_STATE.contains("enum AcpComposerPickerEvent"));
    assert!(COMPOSER_STATE.contains("fn reduce_acp_composer_picker"));
    assert!(COMPOSER_STATE.contains("struct AcpComposerPickerRefreshInput"));
}

// @lat: [[lat.md/acp-chat#ACP Chat#ACP composer]]
#[test]
fn refresh_mention_session_delegates_to_state_machine() {
    let refresh = function_body_after(
        ACP_VIEW,
        "pub(super) fn refresh_mention_session",
        "fn log_mention_visible_range",
    );

    assert!(
        refresh.contains("reduce_acp_composer_picker")
            && refresh.contains("AcpComposerPickerEvent::Refresh")
            && refresh.contains("apply_composer_picker_transition"),
        "refresh_mention_session must delegate lifecycle decisions to the composer picker state machine"
    );
    assert!(
        !refresh.contains("self.mention_session = next_session"),
        "refresh must not assign the derived session directly"
    );
}

// @lat: [[lat.md/acp-chat#ACP Chat#ACP composer]]
#[test]
fn key_handler_routes_picker_navigation_and_dismissal_through_state_machine() {
    let handle_key_down = function_body_after(ACP_VIEW, "fn handle_key_down", "impl Focusable");

    for event in [
        "AcpComposerPickerEvent::SlashToggle",
        "AcpComposerPickerEvent::NavigatePrevious",
        "AcpComposerPickerEvent::NavigateNext",
        "AcpComposerPickerEvent::SubmitStarted",
    ] {
        assert!(
            handle_key_down.contains(event),
            "handle_key_down must route {event} through the picker state machine"
        );
    }

    assert!(
        !handle_key_down.contains("self.mention_session = None"),
        "key handling must not close the picker by direct field assignment"
    );
}

// @lat: [[lat.md/acp-chat#ACP Chat#ACP composer]]
#[test]
fn accept_handler_uses_state_machine_for_close_and_inert_keep_open() {
    let accept = function_body_after(
        ACP_VIEW,
        "fn accept_mention_selection_impl",
        "fn should_claim_inline_mention_ownership",
    );

    assert!(accept.contains("AcpComposerPickerEvent::Accept"));
    assert!(accept.contains("AcpComposerPickerEvent::AcceptIgnoredKeepOpen"));
    assert!(
        !accept.contains("self.mention_session.take()")
            && !accept.contains("self.mention_session = Some(session)"),
        "accept handler should ask the transition owner to close or restore picker state"
    );
}
