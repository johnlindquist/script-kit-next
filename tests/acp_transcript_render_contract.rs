const TRANSCRIPT_SOURCE: &str = include_str!("../src/ai/acp/components/transcript.rs");
const VIEW_SOURCE: &str = include_str!("../src/ai/acp/view.rs");
const BUILD_LAYOUT_INFO_SOURCE: &str = include_str!("../src/app_layout/build_layout_info.rs");

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("missing start marker: {}", start));
    let source = &source[start_index..];
    let end_index = source
        .find(end)
        .unwrap_or_else(|| panic!("missing end marker after {}: {}", start, end));
    &source[..end_index]
}

#[test]
fn transcript_list_state_starts_with_existing_messages() {
    let body = source_between(TRANSCRIPT_SOURCE, "pub fn new(", "\n    pub fn list_state(");

    assert!(
        body.contains("let total = messages.len();"),
        "AcpTranscript::new must size the virtual list from existing messages"
    );
    assert!(
        body.contains("ListState::new(total, ListAlignment::Bottom"),
        "AcpTranscript::new must not mount an already-populated thread with a zero-row list"
    );
    assert!(
        !body.contains("ListState::new(0, ListAlignment::Bottom"),
        "zero-row transcript list initialization hides pre-existing ACP messages"
    );
}

#[test]
fn streaming_activity_status_stays_out_of_transcript_rows() {
    assert!(
        !TRANSCRIPT_SOURCE.contains("acp-assistant-activity-row")
            && !TRANSCRIPT_SOURCE.contains("Working...")
            && !TRANSCRIPT_SOURCE.contains("render_assistant_activity_row_static"),
        "Agent Chat streaming/loading status must not be rendered as an inline transcript row"
    );

    let setter_body = source_between(
        TRANSCRIPT_SOURCE,
        "pub fn set_show_activity_row(",
        "\n    pub fn toggle_collapsed(",
    );
    assert!(
        !setter_body.contains("self.list_state.reset(")
            && !setter_body.contains("usize::from(self.show_activity_row)"),
        "the legacy activity-row setter must not add synthetic transcript rows"
    );
}

#[test]
fn footer_snapshot_carries_streaming_status_next_to_model_name() {
    assert!(
        VIEW_SOURCE.contains("pub(crate) status_text: Option<&'static str>")
            && VIEW_SOURCE.contains("pub(crate) fn model_status_label(&self) -> String")
            && VIEW_SOURCE.contains("format!(\"{} · {}\", self.model_display, status)")
            && VIEW_SOURCE.contains("AcpThreadStatus::Streaming => Some(\"Working...\")"),
        "Agent Chat footer snapshot must carry status text for the footer model label"
    );
}

#[test]
fn transcript_render_does_not_reset_list_state_each_frame() {
    let body = source_between(TRANSCRIPT_SOURCE, "impl Render for AcpTranscript", "\n}");

    assert!(
        !body.contains("self.list_state.reset("),
        "AcpTranscript render must not mutate the virtual list row count every frame"
    );
    assert!(
        body.contains(".relative()")
            && body.contains(".flex_1()")
            && body.contains(".overflow_hidden()"),
        "AcpTranscript render must preserve the virtual-list viewport wrapper"
    );
    assert!(
        body.contains(".size_full()")
            && body.contains(".with_sizing_behavior(ListSizingBehavior::Auto)")
            && body.contains(".vertical_scrollbar(&self.list_state)"),
        "AcpTranscript render must size the virtualized list and keep transcript scrolling wired"
    );
}

#[test]
fn transcript_message_sync_is_idempotent() {
    let helper_body = source_between(
        TRANSCRIPT_SOURCE,
        "fn messages_match_current(",
        "\n    pub fn set_messages(",
    );
    let setter_body = source_between(
        TRANSCRIPT_SOURCE,
        "pub fn set_messages(",
        "\n    pub fn set_show_activity_row(",
    );

    assert!(
        helper_body.contains("current.id == incoming.id")
            && helper_body.contains("current.role == incoming.role")
            && helper_body.contains("current.body == incoming.body")
            && helper_body.contains("current.tool_call_id == incoming.tool_call_id"),
        "AcpTranscript message sync must compare the rendered message signature"
    );
    assert!(
        setter_body.contains("if self.messages_match_current(&messages)")
            && setter_body.contains("return;"),
        "AcpTranscript::set_messages must avoid notify/reset churn when messages are unchanged"
    );
}

#[test]
fn agent_chat_mounts_transcript_from_existing_thread_messages() {
    let ensure_body = source_between(
        VIEW_SOURCE,
        "fn ensure_transcript(",
        "\n    fn confirm_setup_agent_selection(",
    );
    let middle_area_body = source_between(
        VIEW_SOURCE,
        "fn render_acp_middle_area(",
        "\n    pub(crate) fn open_profile_picker(",
    );
    let render_body = source_between(VIEW_SOURCE, "impl Render for AcpChatView", "\n#[cfg(test)]");

    assert!(
        ensure_body.contains("thread_ref.messages.clone()"),
        "Agent Chat must seed the transcript from already-available thread messages"
    );
    assert!(
        ensure_body.contains("AcpTranscript::new(messages, cx)"),
        "Agent Chat must pass existing messages into the transcript entity"
    );
    assert!(
        ensure_body.contains("transcript.set_messages(messages, cx)"),
        "Agent Chat must keep an existing transcript entity synced with live thread messages"
    );
    assert!(
        middle_area_body.contains(".child(self.ensure_transcript(cx).into_any_element())")
            && render_body.contains("self.render_acp_middle_area("),
        "Agent Chat must mount the transcript through the middle-area render path even when assistant text already exists"
    );
}

#[test]
fn acp_layout_measure_gates_empty_guidance_on_live_acp_state() {
    let branch = source_between(
        BUILD_LAYOUT_INFO_SOURCE,
        "if let AppView::AcpChatView { entity }",
        "\n        } else {\n            // Script list",
    );

    assert!(
        branch.contains("collect_acp_state_snapshot"),
        "layout.measure must read live AcpChatView state"
    );
    assert!(
        branch.contains("acp_state.message_count == 0")
            && branch.contains("!acp_state.awaiting_first_assistant_text"),
        "layout.measure must mirror AcpChatView render's empty-state predicate"
    );
    assert!(
        branch.contains("if acp_is_empty"),
        "AcpEmptyGuidance must be guarded by the live empty predicate"
    );
    assert!(
        branch.contains("LayoutComponentInfo::new(\"AcpTranscript\""),
        "non-empty ACP layout.measure receipts must expose the transcript region"
    );

    let guard = branch
        .find("if acp_is_empty")
        .expect("missing acp_is_empty guard");
    let empty = branch
        .find("LayoutComponentInfo::new(\"AcpEmptyGuidance\"")
        .expect("missing empty guidance component");
    assert!(
        guard < empty,
        "AcpEmptyGuidance must not be emitted unconditionally"
    );
}
