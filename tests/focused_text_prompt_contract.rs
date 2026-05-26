use script_kit_gpui::ai::focused_text::{
    build_focused_text_prompt, FocusedTextEditSemantics, FocusedTextPromptRequest,
    FocusedTextTurnSummary,
};
use script_kit_gpui::platform::accessibility::focused_text::focused_text_snapshot_for_tests;

#[test]
fn focused_text_prompt_contains_capture_instruction_and_prior_turns() {
    let snapshot = focused_text_snapshot_for_tests("Original focused text");
    let previous_turns = vec![FocusedTextTurnSummary {
        instruction: "Make it shorter".to_string(),
        semantics: FocusedTextEditSemantics::Replace,
        assistant_output: Some("Short output".to_string()),
    }];

    let (prompt, audit) = build_focused_text_prompt(FocusedTextPromptRequest {
        snapshot: &snapshot,
        instruction: "Make it warmer",
        scope: None,
        semantics: FocusedTextEditSemantics::Chat,
        previous_turns: &previous_turns,
    });

    assert!(prompt.contains("You are the Text Agent Chat profile for focused-field edits."));
    assert!(prompt.contains("<focused_text_context schema_version=\"1\">"));
    assert!(prompt.contains("<captured_focused_field><![CDATA[\nOriginal focused text"));
    assert!(prompt.contains("<requested_edit semantics=\"chat\"><![CDATA[Make it warmer]]>"));
    assert!(prompt.contains("<previous_turns count=\"1\">"));
    assert!(prompt.contains("Short output"));
    assert_eq!(audit.capture_char_count, snapshot.metrics.chars);
    assert_eq!(audit.turn_count, 2);
}

#[test]
fn focused_text_prompt_audit_excludes_sensitive_text() {
    let snapshot = focused_text_snapshot_for_tests("private captured text");
    let (_prompt, audit) = build_focused_text_prompt(FocusedTextPromptRequest {
        snapshot: &snapshot,
        instruction: "rewrite private text",
        scope: None,
        semantics: FocusedTextEditSemantics::Replace,
        previous_turns: &[],
    });

    let audit_debug = format!("{audit:?}");
    assert!(!audit_debug.contains("private captured text"));
    assert!(!audit_debug.contains("rewrite private text"));
    assert_eq!(audit.completion_status, "prompt_built");
}
