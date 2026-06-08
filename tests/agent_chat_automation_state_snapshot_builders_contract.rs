//! Source-level contract for Agent Chat automation state snapshot builders.
//!
//! `getAgentChatState` should read named builder methods instead of assembling
//! setup, live, picker, layout, and context state in one long function.

const AGENT_CHAT_VIEW: &str = include_str!("../src/ai/agent_chat/ui/view.rs");

fn source_between<'a>(source: &'a str, start_pat: &str, end_pat: &str) -> &'a str {
    let start = source
        .find(start_pat)
        .unwrap_or_else(|| panic!("missing source start: {start_pat}"));
    let end_rel = source[start..]
        .find(end_pat)
        .unwrap_or_else(|| panic!("missing source end: {end_pat}"));
    &source[start..start + end_rel]
}

#[test]
fn collect_agent_chat_state_snapshot_delegates_to_named_builders() {
    let body = source_between(
        AGENT_CHAT_VIEW,
        "pub(crate) fn collect_agent_chat_state_snapshot(",
        "\n    fn agent_chat_thread_status_label(",
    );

    assert!(body.contains("let setup_snapshot = self.build_setup_protocol_snapshot(cx);"));
    assert!(body.contains("return self.build_agent_chat_setup_state_snapshot(setup_snapshot);"));
    assert!(body.contains("self.build_agent_chat_live_state_snapshot(thread, setup_snapshot)"));
    assert!(
        !body.contains("AgentChatInputLayoutMetrics"),
        "collect_agent_chat_state_snapshot must not own input layout assembly directly"
    );
    assert!(
        !body.contains("pending_parts\n                    .iter()"),
        "collect_agent_chat_state_snapshot must not own context-summary assembly directly"
    );
}

#[test]
fn live_snapshot_builder_names_all_state_parts() {
    let body = source_between(
        AGENT_CHAT_VIEW,
        "fn build_agent_chat_live_state_snapshot(",
        "\n    fn build_agent_chat_picker_state_snapshot(",
    );

    for required in [
        "Self::agent_chat_thread_status_label(thread.status)",
        "picker: self.build_agent_chat_picker_state_snapshot()",
        "spine: self.build_agent_chat_spine_state_snapshot()",
        "Self::build_agent_chat_context_summary(pending_parts)",
        "Self::build_agent_chat_input_layout_metrics(",
        "Self::build_agent_chat_live_setup_snapshot(thread, setup_snapshot)",
        "crate::protocol::AGENT_CHAT_STATE_SCHEMA_VERSION",
        "thread.pending_permission.is_some()",
    ] {
        assert!(
            body.contains(required),
            "live snapshot builder must contain: {required}"
        );
    }
}

#[test]
fn picker_layout_and_context_builders_preserve_snapshot_fields() {
    let picker_body = source_between(
        AGENT_CHAT_VIEW,
        "fn build_agent_chat_picker_state_snapshot(",
        "\n    fn build_agent_chat_input_layout_metrics(",
    );
    for required in [
        "crate::protocol::AgentChatPickerState",
        "ContextPickerTrigger::Mention => \"@\"",
        "ContextPickerTrigger::Slash => \"/\"",
        "open: true",
        "item_count: session.items.len()",
        "selected_index: session.selected_index",
        "selected_label",
    ] {
        assert!(
            picker_body.contains(required),
            "picker builder must preserve: {required}"
        );
    }

    let layout_body = source_between(
        AGENT_CHAT_VIEW,
        "fn build_agent_chat_input_layout_metrics(",
        "\n    fn build_agent_chat_context_summary(",
    );
    for required in [
        "input_text.chars().count()",
        "thread.input.visible_window_range(60)",
        "cursor_index.saturating_sub(visible_start)",
    ] {
        assert!(
            layout_body.contains(required),
            "layout builder must preserve: {required}"
        );
    }

    let context_body = source_between(
        AGENT_CHAT_VIEW,
        "fn build_agent_chat_context_summary(",
        "\n    fn build_agent_chat_live_setup_snapshot(",
    );
    assert!(context_body.contains("pending_parts.is_empty()"));
    assert!(context_body.contains(".map(|part| part.label())"));
    assert!(context_body.contains(".join(\", \")"));

    let setup_body = source_between(
        AGENT_CHAT_VIEW,
        "fn build_agent_chat_live_setup_snapshot(",
        "\n    /// Build a protocol-layer setup snapshot",
    );
    assert!(setup_body.contains("thread.setup_state().is_some()"));
    assert!(setup_body.contains("setup_snapshot"));
}

#[test]
fn spine_snapshot_builder_is_redacted_and_structural() {
    let body = source_between(
        AGENT_CHAT_VIEW,
        "fn build_agent_chat_spine_state_snapshot(",
        "\n    fn build_agent_chat_input_layout_metrics(",
    );

    for required in [
        "fn build_agent_chat_spine_state_snapshot(",
        "self.agent_chat_spine_owns_list()",
        "self.agent_chat_spine_rows()",
        "self.composer_spine",
        "selected_index",
        "row_count",
        "selectable_row_count",
        "selected_row_fingerprint",
        "row_fingerprint",
        "refresh_elapsed_ms",
        "active_segment_kind",
        "subsearch_source",
    ] {
        assert!(
            body.contains(required),
            "spine snapshot builder must contain: {required}"
        );
    }

    for forbidden in [
        "row.title.to_string()",
        "row.subtitle",
        "thread.input.text().to_string()",
        "format!(\"{:?}\", projection.active_segment_kind)",
    ] {
        assert!(
            !body.contains(forbidden),
            "spine snapshot builder must not expose raw data via: {forbidden}"
        );
    }
}

#[test]
fn test_probe_snapshot_embeds_the_same_state_builder() {
    let body = source_between(
        AGENT_CHAT_VIEW,
        "pub(crate) fn test_probe_snapshot(",
        "\n    /// Emit structured key-routing telemetry",
    );

    assert!(
        body.contains("state: self.collect_agent_chat_state_snapshot(cx)"),
        "getAgentChatTestProbe must embed the same Agent Chat state builder used by getAgentChatState"
    );
}
