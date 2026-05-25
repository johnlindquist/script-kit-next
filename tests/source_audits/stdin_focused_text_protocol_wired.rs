const SYSTEM_CONTROL: &str = include_str!("../../src/protocol/message/variants/system_control.rs");
const GENERAL_CONSTRUCTORS: &str =
    include_str!("../../src/protocol/message/constructors/general.rs");
const PROMPT_HANDLER: &str = include_str!("../../src/prompt_handler/mod.rs");

#[test]
fn focused_text_protocol_declares_capture_and_mutation_verbs() {
    for wire_name in [
        "captureFocusedText",
        "replaceFocusedText",
        "appendFocusedText",
        "copyInlineAgentOutput",
        "focusedTextSnapshot",
        "focusedTextMutation",
    ] {
        assert!(
            SYSTEM_CONTROL.contains(wire_name),
            "missing focused text protocol wire name {wire_name}"
        );
    }
}

#[test]
fn focused_text_protocol_has_typed_constructors() {
    for constructor in [
        "capture_focused_text",
        "replace_focused_text",
        "append_focused_text",
        "copy_inline_agent_output",
        "focused_text_snapshot_response",
        "focused_text_snapshot_error",
        "focused_text_mutation_response",
        "focused_text_mutation_error",
    ] {
        assert!(
            GENERAL_CONSTRUCTORS.contains(constructor),
            "missing focused text constructor {constructor}"
        );
    }
}

#[test]
fn focused_text_protocol_verbs_are_dispatched_with_typed_responses() {
    for arm in [
        "Message::CaptureFocusedText { request_id } =>",
        "Message::ReplaceFocusedText",
        "Message::AppendFocusedText",
        "Message::CopyInlineAgentOutput",
    ] {
        assert!(PROMPT_HANDLER.contains(arm), "missing dispatcher arm {arm}");
    }
    for response in [
        "Message::focused_text_snapshot_response",
        "Message::focused_text_snapshot_error",
        "Message::focused_text_mutation_response",
        "Message::focused_text_mutation_error",
    ] {
        assert!(
            PROMPT_HANDLER.contains(response),
            "missing typed focused-text response {response}"
        );
    }
}

#[test]
fn focused_text_dispatcher_does_not_log_captured_text_or_output() {
    for event in [
        "capture_focused_text_result",
        "replace_focused_text_result",
        "append_focused_text_result",
        "copy_inline_agent_output_result",
    ] {
        let start = PROMPT_HANDLER
            .find(event)
            .unwrap_or_else(|| panic!("missing tracing event {event}"));
        let snippet = &PROMPT_HANDLER[start..PROMPT_HANDLER.len().min(start + 900)];
        assert!(
            snippet.contains("text_len"),
            "focused-text tracing event {event} must log lengths, not raw text"
        );
        assert!(
            !snippet.contains("text = %text") && !snippet.contains("text = snapshot.text"),
            "focused-text tracing event {event} must not log captured/output text"
        );
    }
}
