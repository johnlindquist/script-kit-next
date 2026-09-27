//! Source contracts for Agent Chat prompt handoff policy.
//!
//! These keep the handoff action aligned with the normal Agent Chat prompt-builder
//! submit gate while the broader equivalence matrix is built out.

#[test]
fn handoff_blocks_non_submittable_prompt_builder_plans() {
    let source = include_str!("../src/ai/agent_prompt_handoff.rs");
    let method = source
        .split("pub(crate) fn compile_handoff_payload_from_spine_plan")
        .nth(1)
        .and_then(|tail| tail.split("fn launch_cmux_codex").next())
        .expect("compile_handoff_payload_from_spine_plan body");

    for expected in [
        "plan.prompt_builder_segment_count > 0 && !plan.should_submit_to_chat()",
        "AgentPromptHandoffError::UnsupportedPrompt",
        "prompt builder input is not submittable",
    ] {
        assert!(
            method.contains(expected),
            "handoff must block prompt-builder drafts that normal Agent Chat prompt-builder submit would not submit: {expected}"
        );
    }
}

#[test]
fn handoff_preserves_plain_text_composer_fallback() {
    let source = include_str!("../src/ai/agent_prompt_handoff.rs");
    let method = source
        .split("pub(crate) fn compile_handoff_payload_from_spine_plan")
        .nth(1)
        .and_then(|tail| tail.split("fn launch_cmux_codex").next())
        .expect("compile_handoff_payload_from_spine_plan body");

    assert!(
        method.contains("if plan.prompt_builder_segment_count > 0")
            && method.contains("raw_input.trim().to_string()"),
        "plain Agent Chat composer text has no prompt-builder segments and must still hand off as raw prompt text"
    );
}
