use script_kit_gpui::ai::inline_agent::history::InlineAgentTurn;
use script_kit_gpui::ai::inline_agent::prompt::{
    build_inline_agent_prompt, InlineAgentPromptRequest,
};
use script_kit_gpui::ai::inline_agent::InlineAgentEditSemantics;
use script_kit_gpui::platform::accessibility::focused_text::focused_text_snapshot_for_tests;

#[test]
fn prompt_includes_original_focused_field_on_chat_refinements() {
    let snapshot = focused_text_snapshot_for_tests("Original field text with emoji 🧪");
    let previous_turns = vec![InlineAgentTurn {
        instruction: "Make it shorter".to_string(),
        semantics: InlineAgentEditSemantics::Replace,
        assistant_output: Some("Short output".to_string()),
    }];

    let (prompt, audit) = build_inline_agent_prompt(InlineAgentPromptRequest {
        snapshot: &snapshot,
        instruction: "Now make it warmer",
        semantics: InlineAgentEditSemantics::Chat,
        previous_turns: &previous_turns,
    });

    assert!(prompt.contains("You are Cue, Script Kit's inline text-editing assistant."));
    assert!(prompt.contains("<inline_agent_context schema_version=\"1\">"));
    assert!(prompt.contains("<captured_focused_field><![CDATA[\nOriginal field text with emoji 🧪"));
    assert!(prompt.contains("<previous_turns count=\"1\">"));
    assert!(prompt.contains("Now make it warmer"));
    assert_eq!(audit.capture_char_count, snapshot.metrics.chars);
    assert_eq!(audit.turn_count, 2);
}

#[test]
fn prompt_audit_excludes_sensitive_text() {
    let snapshot = focused_text_snapshot_for_tests("secret focused field");
    let (_prompt, audit) = build_inline_agent_prompt(InlineAgentPromptRequest {
        snapshot: &snapshot,
        instruction: "rewrite this secret",
        semantics: InlineAgentEditSemantics::Replace,
        previous_turns: &[],
    });
    let audit_debug = format!("{audit:?}");

    assert!(!audit_debug.contains("secret focused field"));
    assert!(!audit_debug.contains("rewrite this secret"));
    assert_eq!(audit.completion_status, "prompt_built");
}

#[test]
fn prompt_escapes_cdata_terminators_in_user_controlled_text() {
    let snapshot = focused_text_snapshot_for_tests("field ]]> text");
    let previous_turns = vec![InlineAgentTurn {
        instruction: "previous ]]> instruction".to_string(),
        semantics: InlineAgentEditSemantics::Replace,
        assistant_output: Some("assistant ]]> output".to_string()),
    }];

    let (prompt, _audit) = build_inline_agent_prompt(InlineAgentPromptRequest {
        snapshot: &snapshot,
        instruction: "rewrite ]]> now",
        semantics: InlineAgentEditSemantics::Chat,
        previous_turns: &previous_turns,
    });

    assert!(prompt.contains("field ]]]]><![CDATA[> text"));
    assert!(prompt.contains("rewrite ]]]]><![CDATA[> now"));
    assert!(prompt.contains("previous ]]]]><![CDATA[> instruction"));
    assert!(prompt.contains("assistant ]]]]><![CDATA[> output"));
    assert!(!prompt.contains("field ]]> text"));
}

#[test]
fn prompt_caps_large_focused_field_and_records_redacted_truncation_metadata() {
    let large_text = "x".repeat(20_050);
    let snapshot = focused_text_snapshot_for_tests(&large_text);

    let (prompt, audit) = build_inline_agent_prompt(InlineAgentPromptRequest {
        snapshot: &snapshot,
        instruction: "summarize",
        semantics: InlineAgentEditSemantics::Replace,
        previous_turns: &[],
    });

    assert!(prompt.contains("truncated=\"true\""));
    assert!(prompt.contains("prompt_char_count=\"20000\""));
    assert_eq!(audit.capture_char_count, 20_050);
    assert_eq!(audit.prompt_capture_char_count, 20_000);
    assert!(audit.capture_truncated);
    assert!(!format!("{audit:?}").contains(&large_text));
}
