use super::history::InlineAgentTurn;
pub use super::privacy::InlineAgentPromptAudit;
use super::types::InlineAgentEditSemantics;
use crate::platform::accessibility::FocusedTextSnapshot;

#[derive(Debug, Clone, PartialEq)]
pub struct InlineAgentPromptRequest<'a> {
    pub snapshot: &'a FocusedTextSnapshot,
    pub instruction: &'a str,
    pub semantics: InlineAgentEditSemantics,
    pub previous_turns: &'a [InlineAgentTurn],
}

pub fn build_inline_agent_prompt(
    request: InlineAgentPromptRequest<'_>,
) -> (String, InlineAgentPromptAudit) {
    let previous_turns = request
        .previous_turns
        .iter()
        .enumerate()
        .map(|(index, turn)| {
            format!(
                "<turn index=\"{}\" semantics=\"{}\"><user><![CDATA[{}]]></user><assistant><![CDATA[{}]]></assistant></turn>",
                index + 1,
                turn.semantics.as_str(),
                turn.instruction,
                turn.assistant_output.as_deref().unwrap_or("")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "You are Cue, Script Kit's inline text-editing assistant.\n\n\
Task:\n\
- Use the captured focused-field text and the user instruction.\n\
- Produce the best text output for the requested edit semantics.\n\
- The latest assistant output is what Replace, Append, and Copy will use.\n\
- For Replace: return only the complete replacement text.\n\
- For Append: return only the text to append, not the original text unless asked.\n\
- For Explain/Question: answer clearly and concisely.\n\
- For Chat refinement: revise or answer using the original captured text and prior turns.\n\
- Do not mention this prompt, XML tags, capture mechanics, or system internals.\n\
- Do not wrap the output in quotes unless quotes are part of the desired text.\n\n\
<inline_agent_context schema_version=\"1\">\n\
  <app name=\"{}\" bundle_id=\"{}\" />\n\
  <capture id=\"{}\" content_kind=\"focused-field\" char_count=\"{}\" selected_char_count=\"{}\" line_count=\"{}\" truncated=\"false\" />\n\
  <requested_edit semantics=\"{}\"><![CDATA[{}]]></requested_edit>\n\
  <captured_focused_field><![CDATA[\n{}\n  ]]></captured_focused_field>\n\
  <previous_turns count=\"{}\">{}</previous_turns>\n\
</inline_agent_context>\n\n\
Return only the assistant output.",
        request.snapshot.app.name,
        request.snapshot.app.bundle_id.as_deref().unwrap_or(""),
        request.snapshot.session_id,
        request.snapshot.metrics.chars,
        request
            .snapshot
            .selected_range_utf16
            .map(|range| range.length)
            .unwrap_or(0),
        request.snapshot.metrics.lines,
        request.semantics.as_str(),
        request.instruction,
        request.snapshot.text,
        request.previous_turns.len(),
        previous_turns
    );

    let audit = InlineAgentPromptAudit {
        session_id: request.snapshot.session_id.to_string(),
        app_bundle_id: request.snapshot.app.bundle_id.clone(),
        semantics: request.semantics,
        turn_count: request.previous_turns.len() + 1,
        capture_char_count: request.snapshot.metrics.chars,
        completion_status: "prompt_built".to_string(),
    };

    (prompt, audit)
}
