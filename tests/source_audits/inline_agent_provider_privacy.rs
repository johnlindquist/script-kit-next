use std::fs;
use std::path::Path;

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.as_ref().display()))
}

#[test]
fn inline_agent_provider_logs_do_not_emit_sensitive_prompt_or_output_fields() {
    for path in [
        "src/ai/inline_agent/agent_chat_adapter.rs",
        "src/ai/inline_agent/session.rs",
        "src/inline_agent/window.rs",
    ] {
        let source = read(path);
        for forbidden in [
            "%request.prompt",
            "?request.prompt",
            "prompt =",
            "%request.instruction",
            "?request.instruction",
            "%text",
            "?text",
            "%output",
            "?output",
            "clipboard",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} must not log sensitive inline-agent data pattern {forbidden}"
            );
        }
    }
}
