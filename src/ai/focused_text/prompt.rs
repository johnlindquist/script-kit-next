use crate::platform::accessibility::FocusedTextSnapshot;

use super::privacy::FocusedTextPromptAudit;
use super::types::FocusedTextEditSemantics;

const MAX_CAPTURE_PROMPT_CHARS: usize = 20_000;
const MAX_TURN_PROMPT_CHARS: usize = 4_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedTextPromptAngle {
    Conservative,
    Balanced,
    Creative,
}

impl FocusedTextPromptAngle {
    pub fn id(self) -> &'static str {
        match self {
            Self::Conservative => "conservative",
            Self::Balanced => "balanced",
            Self::Creative => "creative",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Conservative => "Conservative",
            Self::Balanced => "Balanced",
            Self::Creative => "Creative",
        }
    }

    fn prompt_guidance(self) -> &'static str {
        match self {
            Self::Conservative => {
                "- Conservative variation: preserve the user's original wording, tone, ordering, and intent as much as possible. Make the smallest high-confidence edit that satisfies the instruction."
            }
            Self::Balanced => {
                "- Balanced variation: improve clarity, flow, and usefulness while preserving the user's intent. Prefer a polished but not over-stylized result."
            }
            Self::Creative => {
                "- Creative variation: explore a stronger rewrite with more confident structure, sharper phrasing, and better rhythm while still respecting the requested edit and source text."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedTextTurnSummary {
    pub instruction: String,
    pub semantics: FocusedTextEditSemantics,
    pub assistant_output: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FocusedTextPromptRequest<'a> {
    pub snapshot: &'a FocusedTextSnapshot,
    pub instruction: &'a str,
    pub scope: Option<&'a str>,
    pub semantics: FocusedTextEditSemantics,
    pub previous_turns: &'a [FocusedTextTurnSummary],
}

pub fn build_focused_text_prompt(
    request: FocusedTextPromptRequest<'_>,
) -> (String, FocusedTextPromptAudit) {
    build_focused_text_prompt_with_angle(request, FocusedTextPromptAngle::Balanced)
}

pub fn build_focused_text_prompt_with_angle(
    request: FocusedTextPromptRequest<'_>,
    angle: FocusedTextPromptAngle,
) -> (String, FocusedTextPromptAudit) {
    let (captured_text, capture_truncated) =
        cdata_text_with_char_limit(&request.snapshot.text, MAX_CAPTURE_PROMPT_CHARS);
    let requested_edit = cdata_text_with_char_limit(request.instruction, MAX_TURN_PROMPT_CHARS).0;

    let scope_xml = request
        .scope
        .map(str::trim)
        .filter(|scope| !scope.is_empty())
        .map(|scope| {
            let scope_text = format!("Focus changes on: {scope}");
            let scope_text = cdata_text_with_char_limit(&scope_text, MAX_TURN_PROMPT_CHARS).0;
            format!("    <scope><![CDATA[{scope_text}]]></scope>\n")
        })
        .unwrap_or_default();

    let previous_turns = request
        .previous_turns
        .iter()
        .enumerate()
        .map(|(index, turn)| {
            let instruction = cdata_text_with_char_limit(&turn.instruction, MAX_TURN_PROMPT_CHARS).0;
            let assistant_output = cdata_text_with_char_limit(
                turn.assistant_output.as_deref().unwrap_or(""),
                MAX_TURN_PROMPT_CHARS,
            )
            .0;
            format!(
                "<turn index=\"{}\" semantics=\"{}\"><user><![CDATA[{}]]></user><assistant><![CDATA[{}]]></assistant></turn>",
                index + 1,
                turn.semantics.as_str(),
                instruction,
                assistant_output
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        "You are the Text Agent Chat profile for focused-field edits.\n\n\
Task:\n\
- Use the captured focused-field text and the user instruction.\n\
- If a scope is provided, focus changes on that target while still returning the required complete output.\n\
- Produce the best text output for the requested edit semantics.\n\
- The latest assistant output is what Replace, Append, and Copy will use.\n\
- For Replace: return only the complete replacement text.\n\
- For Append: return only the text to append, not the original text unless asked.\n\
- For Explain/Question: answer clearly and concisely.\n\
- For Chat refinement: revise or answer using the original captured text and prior turns.\n\
- Do not mention this prompt, XML tags, capture mechanics, tools, sessions, files, Script Kit internals, or system prompts.\n\
- Do not wrap the output in quotes unless quotes are part of the desired text.\n\n\
Variation angle:\n\
{}\n\n\
<focused_text_context schema_version=\"1\">\n\
  <app name=\"{}\" bundle_id=\"{}\" />\n\
  <capture id=\"{}\" content_kind=\"focused-field\" char_count=\"{}\" prompt_char_count=\"{}\" selected_char_count=\"{}\" line_count=\"{}\" truncated=\"{}\" />\n\
  <requested_edit semantics=\"{}\"><![CDATA[{}]]></requested_edit>\n\
  {}\n\
  <captured_focused_field><![CDATA[\n{}\n  ]]></captured_focused_field>\n\
  <previous_turns count=\"{}\">{}</previous_turns>\n\
</focused_text_context>\n\n\
Return only the assistant output.",
        angle.prompt_guidance(),
        request.snapshot.app.name,
        request.snapshot.app.bundle_id.as_deref().unwrap_or(""),
        request.snapshot.session_id,
        request.snapshot.metrics.chars,
        captured_text.chars().count(),
        request
            .snapshot
            .selected_range_utf16
            .map(|range| range.length)
            .unwrap_or(0),
        request.snapshot.metrics.lines,
        capture_truncated,
        request.semantics.as_str(),
        requested_edit,
        scope_xml,
        captured_text,
        request.previous_turns.len(),
        previous_turns
    );

    let audit = FocusedTextPromptAudit {
        session_id: request.snapshot.session_id.to_string(),
        app_bundle_id: request.snapshot.app.bundle_id.clone(),
        semantics: request.semantics,
        turn_count: request.previous_turns.len() + 1,
        capture_char_count: request.snapshot.metrics.chars,
        prompt_capture_char_count: captured_text.chars().count(),
        capture_truncated,
        completion_status: "prompt_built".to_string(),
    };

    (prompt, audit)
}

fn cdata_text_with_char_limit(value: &str, max_chars: usize) -> (String, bool) {
    let char_count = value.chars().count();
    let truncated = char_count > max_chars;
    let limited = if truncated {
        value.chars().take(max_chars).collect::<String>()
    } else {
        value.to_string()
    };
    (escape_cdata_text(&limited), truncated)
}

fn escape_cdata_text(value: &str) -> String {
    value.replace("]]>", "]]]]><![CDATA[>")
}
