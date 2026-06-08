use script_kit_gpui::protocol::Message;

const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const RUNTIME_STDIN_TAIL: &str = include_str!("../src/main_entry/runtime_stdin_match_tail.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const SIMULATE_KEY: &str = include_str!("../src/main_entry/runtime_stdin_match_simulate_key.rs");

#[test]
fn inline_agent_fixture_open_result_is_redacted_and_correlatable() {
    let message = Message::inline_agent_fixture_open_result(
        "fixture-1".to_string(),
        "pi".to_string(),
        false,
        false,
        11,
        19,
        Some("gated_off".to_string()),
        Some("Inline Agent real Pi fixture is gated off".to_string()),
    );

    assert_eq!(message.request_id(), Some("fixture-1"));

    let json = serde_json::to_value(&message).expect("serialize fixture result");
    assert_eq!(json["type"], "inlineAgentFixtureOpenResult");
    assert_eq!(json["requestId"], "fixture-1");
    assert_eq!(json["mode"], "pi");
    assert_eq!(json["ok"], false);
    assert_eq!(json["submitted"], false);
    assert_eq!(json["targetId"], "agent_chat-chat");
    assert_eq!(json["targetKind"], "main");
    assert_eq!(json["textLength"], 11);
    assert_eq!(json["instructionLength"], 19);
    assert_eq!(json["errorCode"], "gated_off");

    let raw = json.to_string();
    for forbidden in [
        "Hello world",
        "Translate to French",
        "<focused_focused_field>",
        "assistantOutput",
        "capturedText",
        "promptXml",
    ] {
        assert!(
            !raw.contains(forbidden),
            "fixture result must stay redacted and omit `{forbidden}`"
        );
    }
}

#[test]
fn inline_agent_fixture_dispatchers_emit_deterministic_receipts() {
    for (label, source) in [
        ("runtime_stdin.rs", RUNTIME_STDIN),
        ("runtime_stdin_match_tail.rs", RUNTIME_STDIN_TAIL),
        ("app_run_setup.rs", APP_RUN_SETUP),
    ] {
        for required in [
            "ExternalCommand::OpenInlineAgentWithMockData { text, instruction, request_id }",
            "ExternalCommand::OpenInlineAgentWithPiData { text, instruction, request_id }",
            "Message::inline_agent_fixture_open_result",
            "\"mock\".to_string()",
            "\"pi\".to_string()",
            "Some(\"gated_off\".to_string())",
            "text_length",
            "instruction_length",
            "ok && requested_submit",
            "view.response_sender",
        ] {
            assert!(
                source.contains(required),
                "{label} must include fixture receipt contract fragment `{required}`"
            );
        }
    }
}

#[test]
fn simulate_key_dispatch_remains_fire_and_forget() {
    assert!(
        !SIMULATE_KEY.contains("inline_agent_fixture_open_result"),
        "simulateKey must not reuse fixture-open receipts"
    );
    assert!(
        !SIMULATE_KEY.contains("SimulateKeyResult")
            && !SIMULATE_KEY.contains(r#""simulateKeyResult""#),
        "simulateKey remains a no-response-envelope command"
    );
}
