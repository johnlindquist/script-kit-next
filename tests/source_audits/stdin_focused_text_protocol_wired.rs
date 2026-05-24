const SYSTEM_CONTROL: &str = include_str!("../../src/protocol/message/variants/system_control.rs");
const GENERAL_CONSTRUCTORS: &str =
    include_str!("../../src/protocol/message/constructors/general.rs");

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
