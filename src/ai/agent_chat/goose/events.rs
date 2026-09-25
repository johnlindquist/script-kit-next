//! Maps goose's ACP session updates onto Agent Chat events.

use serde_json::Value;

use crate::ai::agent_chat::events::AgentChatEvent;
use crate::ai::agent_chat::reveal::split_text_delta_for_reveal;
use crate::ai::agent_chat::ui::config::AgentChatModelEntry;
use sk_protocol::ai_reliability::{CancellationKind, PartialOutputState, ProtocolComponent};

use super::protocol::{AcpError, ACP_AUTH_REQUIRED_CODE};

/// Translate one `session/update` payload (`params.update`) into events.
pub(crate) fn map_session_update(update: &Value) -> Vec<AgentChatEvent> {
    match get_str(update, "sessionUpdate") {
        Some("agent_message_chunk") => content_text(update)
            .map(|text| {
                split_text_delta_for_reveal(&text)
                    .into_iter()
                    .map(AgentChatEvent::AgentMessageDelta)
                    .collect()
            })
            .unwrap_or_default(),
        Some("agent_thought_chunk") => content_text(update)
            .map(|text| {
                split_text_delta_for_reveal(&text)
                    .into_iter()
                    .map(AgentChatEvent::AgentThoughtDelta)
                    .collect()
            })
            .unwrap_or_default(),
        Some("tool_call") => vec![AgentChatEvent::ToolCallStarted {
            tool_call_id: tool_call_id(update),
            title: get_str(update, "title").unwrap_or("Tool").to_string(),
            status: get_str(update, "status")
                .map(tool_status)
                .unwrap_or("pending")
                .to_string(),
            tool_name: tool_name(update),
            raw_input: update.get("rawInput").filter(|v| !v.is_null()).cloned(),
        }],
        Some("tool_call_update") => {
            let status = get_str(update, "status").map(tool_status);
            vec![AgentChatEvent::ToolCallUpdated {
                tool_call_id: tool_call_id(update),
                title: get_str(update, "title").map(str::to_string),
                status: status.map(str::to_string),
                body: tool_content_text(update),
                raw_input: update.get("rawInput").filter(|v| !v.is_null()).cloned(),
                diff: tool_content_diff(update),
                is_error: status == Some("failed"),
            }]
        }
        Some("plan") => vec![AgentChatEvent::PlanUpdated {
            entries: update
                .get("entries")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|entry| get_str(entry, "content").map(str::to_string))
                .collect(),
        }],
        Some("usage_update") => {
            let cost_usd = update
                .get("cost")
                .filter(|cost| get_str(cost, "currency").is_none_or(|c| c == "USD"))
                .and_then(|cost| cost.get("amount"))
                .and_then(Value::as_f64);
            vec![AgentChatEvent::UsageUpdated {
                used_tokens: update.get("used").and_then(Value::as_u64).unwrap_or(0),
                context_size: update.get("size").and_then(Value::as_u64).unwrap_or(0),
                cost_usd,
            }]
        }
        Some("available_commands_update") => vec![AgentChatEvent::AvailableCommandsUpdated {
            command_names: update
                .get("availableCommands")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|command| get_str(command, "name").map(str::to_string))
                .collect(),
        }],
        Some("current_mode_update") => get_str(update, "currentModeId")
            .map(|mode_id| {
                vec![AgentChatEvent::ModeChanged {
                    mode_id: mode_id.to_string(),
                }]
            })
            .unwrap_or_default(),
        Some("config_option_update") => update
            .get("configOptions")
            .map(|options| vec![models_available(options)])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// The model picker's view of goose's session config options: every model of
/// the session's current provider, namespaced `provider/model`.
pub(crate) fn models_available(config_options: &Value) -> AgentChatEvent {
    let options = config_options.as_array().map(Vec::as_slice).unwrap_or(&[]);
    let find = |id: &str| {
        options
            .iter()
            .find(|option| get_str(option, "id") == Some(id))
    };
    let provider = find("provider").and_then(|option| get_str(option, "currentValue"));
    let model_option = find("model");
    let current_model = model_option.and_then(|option| get_str(option, "currentValue"));
    let namespaced = |model: &str| match provider {
        Some(provider) => format!("{provider}/{model}"),
        None => model.to_string(),
    };
    let models = model_option
        .map(select_values)
        .unwrap_or_default()
        .into_iter()
        .map(|(value, name)| AgentChatModelEntry {
            id: namespaced(&value),
            display_name: Some(name),
            context_window: None,
        })
        .collect();
    AgentChatEvent::ModelsAvailable {
        current_model_id: current_model.map(namespaced),
        models,
    }
}

/// `(value, name)` pairs of a select option, flattening grouped options.
pub(crate) fn select_values(option: &Value) -> Vec<(String, String)> {
    let mut values = Vec::new();
    for entry in option
        .get("options")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(group) = entry.get("options").and_then(Value::as_array) {
            for value in group {
                push_select_value(value, &mut values);
            }
        } else {
            push_select_value(entry, &mut values);
        }
    }
    values
}

fn push_select_value(value: &Value, values: &mut Vec<(String, String)>) {
    if let Some(id) = get_str(value, "value") {
        let name = get_str(value, "name").unwrap_or(id);
        values.push((id.to_string(), name.to_string()));
    }
}

/// Terminal event for a finished `session/prompt`.
pub(crate) fn prompt_outcome(result: Result<&Value, &AcpError>) -> AgentChatEvent {
    match result {
        Ok(result) => match get_str(result, "stopReason") {
            Some("cancelled") => AgentChatEvent::TurnCompleted {
                outcome: crate::ai::reliability::AiTurnRuntimeOutcome::Cancelled {
                    kind: CancellationKind::UserCancelled,
                    partial: PartialOutputState::None,
                },
            },
            Some(reason) => AgentChatEvent::completed(reason),
            None => AgentChatEvent::completed("end_turn"),
        },
        Err(error) => AgentChatEvent::TurnFailed {
            failure: failure_for_acp_error(error),
        },
    }
}

/// Classify an ACP error from its facts (rules/AI_RELIABILITY.md Rule 1).
///
/// goose reports rejected credentials with the ACP `auth_required` code, and a
/// provider it could not construct (no key or sign-in yet) with the stable
/// `Provider not set` detail. Both are setup facts that need Sign In. Every
/// other error is an opaque provider payload.
pub(crate) fn failure_for_acp_error(error: &AcpError) -> crate::ai::reliability::AppFailureRecord {
    if error.code == ACP_AUTH_REQUIRED_CODE {
        return crate::ai::reliability::setup_required_failure(
            ProtocolComponent::Goose,
            "authentication required",
            &[],
        );
    }
    let detail = error.detail();
    if detail.contains("Provider not set") {
        return crate::ai::reliability::setup_required_failure(
            ProtocolComponent::Goose,
            "provider credentials missing",
            &[],
        );
    }
    crate::ai::reliability::provider_failure(ProtocolComponent::Goose, detail)
}

/// OpenAI models reached through the ChatGPT (Codex) backend can emit
/// ChatGPT's hidden citation markup — private-use characters wrapping
/// `cite`/`turn0search0` references — that only ChatGPT's own UI renders.
/// Drop it from assistant text. A marker can span chunks, so `in_citation`
/// carries the open state between calls. Returns the cleaned update when the
/// text changed.
pub(crate) fn strip_citation_markers(update: &Value, in_citation: &mut bool) -> Option<Value> {
    const OPEN: char = '\u{e200}';
    const SEPARATOR: char = '\u{e202}';
    const CLOSE: char = '\u{e201}';
    if get_str(update, "sessionUpdate") != Some("agent_message_chunk") {
        return None;
    }
    let text = update
        .get("content")
        .and_then(|content| get_str(content, "text"))?;
    if !*in_citation && !text.contains([OPEN, SEPARATOR, CLOSE]) {
        return None;
    }
    let mut cleaned = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            OPEN => *in_citation = true,
            CLOSE => *in_citation = false,
            SEPARATOR => {}
            _ if *in_citation => {}
            other => cleaned.push(other),
        }
    }
    let mut update = update.clone();
    update["content"]["text"] = Value::String(cleaned);
    Some(update)
}

/// The fixed wording goose (pinned v1.52.0,
/// `Message::from_provider_error`) uses when a provider request fails in a way
/// it has no ACP error for: it sends the failure as an ordinary assistant
/// text chunk and then ends the turn normally.
const GOOSE_PROVIDER_ERROR_PREFIX: &str = "Ran into this error: ";
const GOOSE_PROVIDER_ERROR_SUFFIX: &str =
    ".\n\nPlease retry if you think this is a transient or recoverable error.";

/// If this `session/update` is goose reporting a failed provider request as
/// assistant text, the classified failure it stands for. The provider's own
/// words go only to classification and the diagnostic vault, never to the
/// transcript (rules/AI_RELIABILITY.md).
pub(crate) fn provider_error_in_update(
    update: &Value,
) -> Option<crate::ai::reliability::AppFailureRecord> {
    if get_str(update, "sessionUpdate") != Some("agent_message_chunk") {
        return None;
    }
    let text = content_text(update)?;
    let detail = text.strip_prefix(GOOSE_PROVIDER_ERROR_PREFIX)?;
    let detail = detail
        .strip_suffix(GOOSE_PROVIDER_ERROR_SUFFIX)
        .unwrap_or(detail);
    Some(crate::ai::reliability::provider_failure(
        ProtocolComponent::Goose,
        detail,
    ))
}

fn tool_status(status: &str) -> &'static str {
    match status {
        "in_progress" => "running",
        "completed" => "complete",
        "failed" => "failed",
        _ => "pending",
    }
}

fn tool_call_id(update: &Value) -> String {
    get_str(update, "toolCallId")
        .unwrap_or("tool-call")
        .to_string()
}

/// The profile tool id of a tool call. goose reports `<extension>__<tool>` in
/// `_meta.goose.toolCall`; the ACP `name` field is the fallback. Mapping back
/// to profile ids keeps tool policies and tool cards on one vocabulary.
pub(crate) fn tool_name(tool_call: &Value) -> Option<String> {
    tool_call
        .get("_meta")
        .and_then(|meta| meta.get("goose"))
        .and_then(|goose| goose.get("toolCall"))
        .and_then(|tool| get_str(tool, "toolName"))
        .or_else(|| get_str(tool_call, "name"))
        .map(super::recipe::profile_tool_id)
}

fn content_text(update: &Value) -> Option<String> {
    let content = update.get("content")?;
    match get_str(content, "type") {
        Some("text") => get_str(content, "text")
            .filter(|text| !text.is_empty())
            .map(str::to_string),
        _ => None,
    }
}

fn tool_content_text(update: &Value) -> Option<String> {
    let parts: Vec<String> = update
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| get_str(item, "type") == Some("content"))
        .filter_map(|item| item.get("content"))
        .filter_map(|content| get_str(content, "text"))
        .filter(|text| !text.trim().is_empty())
        .map(str::to_string)
        .collect();
    (!parts.is_empty()).then(|| parts.join("\n"))
}

fn tool_content_diff(update: &Value) -> Option<String> {
    let diffs: Vec<String> = update
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| get_str(item, "type") == Some("diff"))
        .map(|item| {
            render_text_diff(
                get_str(item, "oldText").unwrap_or_default(),
                get_str(item, "newText").unwrap_or_default(),
            )
        })
        .filter(|diff| !diff.is_empty())
        .collect();
    (!diffs.is_empty()).then(|| diffs.join("\n"))
}

/// Render a replacement as `-`/`+` lines, keeping unchanged leading and
/// trailing lines out of the hunk so the card shows only what changed.
pub(crate) fn render_text_diff(old: &str, new: &str) -> String {
    let old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();
    let prefix = old_lines
        .iter()
        .zip(&new_lines)
        .take_while(|(a, b)| a == b)
        .count();
    let suffix = old_lines[prefix..]
        .iter()
        .rev()
        .zip(new_lines[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let removed = &old_lines[prefix..old_lines.len() - suffix];
    let added = &new_lines[prefix..new_lines.len() - suffix];
    removed
        .iter()
        .enumerate()
        .map(|(i, line)| format!("-{:>4} {line}", prefix + i + 1))
        .chain(
            added
                .iter()
                .enumerate()
                .map(|(i, line)| format!("+{:>4} {line}", prefix + i + 1)),
        )
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn message_and_thought_chunks_become_reveal_deltas() {
        let events = map_session_update(&json!({
            "sessionUpdate": "agent_message_chunk",
            "content": {"type": "text", "text": "Hello world"}
        }));
        assert!(matches!(
            events.as_slice(),
            [AgentChatEvent::AgentMessageDelta(a), AgentChatEvent::AgentMessageDelta(b)]
                if a == "Hello " && b == "world"
        ));
        let events = map_session_update(&json!({
            "sessionUpdate": "agent_thought_chunk",
            "content": {"type": "text", "text": "hmm"}
        }));
        assert!(matches!(events.as_slice(), [AgentChatEvent::AgentThoughtDelta(t)] if t == "hmm"));
    }

    #[test]
    fn goose_tool_calls_carry_the_profile_tool_id_and_result_text() {
        let started = map_session_update(&json!({
            "sessionUpdate": "tool_call",
            "toolCallId": "call_1",
            "title": "shell · echo hi",
            "rawInput": {"command": "echo hi"},
            "_meta": {"goose": {"toolCall": {"toolName": "developer__shell", "extensionName": "developer"}}}
        }));
        assert!(matches!(
            started.as_slice(),
            [AgentChatEvent::ToolCallStarted { tool_call_id, status, tool_name: Some(name), raw_input: Some(_), .. }]
                if tool_call_id == "call_1" && status == "pending" && name == "bash"
        ));
        let search = map_session_update(&json!({
            "sessionUpdate": "tool_call",
            "toolCallId": "call_2",
            "_meta": {"goose": {"toolCall": {"toolName": "scriptkit__web_search", "extensionName": "scriptkit"}}}
        }));
        assert!(matches!(
            search.as_slice(),
            [AgentChatEvent::ToolCallStarted { tool_name: Some(name), .. }] if name == "web_search"
        ));
        let updated = map_session_update(&json!({
            "sessionUpdate": "tool_call_update",
            "toolCallId": "call_1",
            "status": "completed",
            "content": [{"type": "content", "content": {"type": "text", "text": "hi"}}]
        }));
        assert!(matches!(
            updated.as_slice(),
            [AgentChatEvent::ToolCallUpdated { status: Some(status), body: Some(body), is_error: false, .. }]
                if status == "complete" && body == "hi"
        ));
    }

    #[test]
    fn failed_tool_updates_are_errors() {
        let updated = map_session_update(&json!({
            "sessionUpdate": "tool_call_update",
            "toolCallId": "call_1",
            "status": "failed"
        }));
        assert!(matches!(
            updated.as_slice(),
            [AgentChatEvent::ToolCallUpdated { is_error: true, .. }]
        ));
    }

    #[test]
    fn config_options_list_the_current_provider_models() {
        let event = models_available(&json!([
            {"id": "provider", "currentValue": "ollama", "options": []},
            {"id": "model", "category": "model", "currentValue": "qwen3:4b", "options": [
                {"value": "qwen3:4b", "name": "Qwen3 4B"},
                {"group": "more", "name": "More", "options": [{"value": "qwen3-coder:30b", "name": "Coder"}]}
            ]}
        ]));
        let AgentChatEvent::ModelsAvailable {
            current_model_id,
            models,
        } = event
        else {
            panic!("expected models");
        };
        assert_eq!(current_model_id.as_deref(), Some("ollama/qwen3:4b"));
        assert_eq!(
            models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
            vec!["ollama/qwen3:4b", "ollama/qwen3-coder:30b"]
        );
    }

    #[test]
    fn prompt_stop_reasons_map_to_terminal_outcomes() {
        assert!(matches!(
            prompt_outcome(Ok(&json!({"stopReason": "end_turn"}))),
            AgentChatEvent::TurnCompleted { outcome: crate::ai::reliability::AiTurnRuntimeOutcome::Completed { stop_reason: Some(reason) } }
                if reason == "end_turn"
        ));
        assert!(matches!(
            prompt_outcome(Ok(&json!({"stopReason": "cancelled"}))),
            AgentChatEvent::TurnCompleted {
                outcome: crate::ai::reliability::AiTurnRuntimeOutcome::Cancelled { .. }
            }
        ));
    }
    #[test]
    fn chatgpt_citation_markup_is_dropped_even_across_chunks() {
        let chunk = |text: &str| json!({"sessionUpdate": "agent_message_chunk", "content": {"type": "text", "text": text}});
        let mut in_citation = false;
        let text = |update: Option<Value>| {
            update.unwrap()["content"]["text"]
                .as_str()
                .unwrap()
                .to_string()
        };
        assert_eq!(
            text(strip_citation_markers(
                &chunk("Seahawks won. \u{e200}cite\u{e202}turn0search0\u{e201} Done"),
                &mut in_citation
            )),
            "Seahawks won.  Done"
        );
        assert_eq!(
            text(strip_citation_markers(
                &chunk("A \u{e200}cite\u{e202}turn0"),
                &mut in_citation
            )),
            "A "
        );
        assert_eq!(
            text(strip_citation_markers(
                &chunk("search2\u{e201}B"),
                &mut in_citation
            )),
            "B"
        );
        assert!(strip_citation_markers(&chunk("plain"), &mut in_citation).is_none());
    }

    #[test]
    fn goose_provider_errors_sent_as_text_become_failures_not_answers() {
        let update = json!({
            "sessionUpdate": "agent_message_chunk",
            "content": {"type": "text", "text": "Ran into this error: Request failed: Bad request (400): {\"detail\":\"The 'gpt-5.3-codex-spark' model is not supported when using Codex with a ChatGPT account.\"}.\n\nPlease retry if you think this is a transient or recoverable error."}
        });
        let failure = provider_error_in_update(&update).expect("goose error chunk");
        assert!(
            !failure.primary_message().contains("gpt-5.3-codex-spark"),
            "raw provider text must stay out of the visible message"
        );
        // An ordinary answer that merely mentions an error is still an answer.
        let answer = json!({
            "sessionUpdate": "agent_message_chunk",
            "content": {"type": "text", "text": "I ran into this error once too."}
        });
        assert!(provider_error_in_update(&answer).is_none());
    }

    #[test]
    fn credential_errors_classify_as_setup_required_not_unknown() {
        use sk_protocol::ai_reliability::AiFailureCode;
        let auth = failure_for_acp_error(&AcpError {
            code: ACP_AUTH_REQUIRED_CODE,
            message: "Authentication required".into(),
            data: None,
        });
        let missing = failure_for_acp_error(&AcpError {
            code: -32603,
            message: "Internal error".into(),
            data: Some(json!("Error getting agent reply: Provider not set")),
        });
        for record in [auth, missing] {
            assert_ne!(record.failure.code, AiFailureCode::Unknown, "{record:?}");
        }
    }

    #[test]
    fn diffs_show_only_changed_lines() {
        assert_eq!(
            render_text_diff("a\nb\nc\n", "a\nB\nc\n"),
            "-   2 b\n+   2 B"
        );
        assert_eq!(render_text_diff("same", "same"), "");
    }
}
