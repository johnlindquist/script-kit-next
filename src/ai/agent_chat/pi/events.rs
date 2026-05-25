use serde_json::Value;

use crate::ai::acp::config::AcpModelEntry;
use crate::ai::agent_chat::events::AgentChatEvent;

use super::protocol::{PiRpcLine, PiRpcResponse};

pub(crate) fn map_rpc_line_to_events(line: PiRpcLine) -> Vec<AgentChatEvent> {
    match line {
        PiRpcLine::Response(response) => map_rpc_response_to_events(&response),
        PiRpcLine::Event(event) => map_rpc_event_to_events(&event),
    }
}

pub(crate) fn map_rpc_response_to_events(response: &PiRpcResponse) -> Vec<AgentChatEvent> {
    if !response.success {
        return vec![AgentChatEvent::Failed {
            error: response
                .error
                .clone()
                .unwrap_or_else(|| "Pi RPC command failed".to_string()),
        }];
    }

    if response.command.as_deref() == Some("get_available_models") {
        return vec![AgentChatEvent::ModelsAvailable {
            current_model_id: None,
            models: models_from_response_data(response.data.as_ref()),
        }];
    }

    Vec::new()
}

pub(crate) fn map_rpc_event_to_events(event: &Value) -> Vec<AgentChatEvent> {
    match get_str(event, "type") {
        Some("message_update") => map_message_update(event),
        Some("tool_call_end") => vec![AgentChatEvent::ToolCallStarted {
            tool_call_id: tool_call_id(event).unwrap_or_else(|| "tool-call".to_string()),
            title: get_str(event, "toolName")
                .or_else(|| get_str(event, "name"))
                .unwrap_or("Tool")
                .to_string(),
            status: "pending".to_string(),
        }],
        Some("tool_execution_start") => vec![AgentChatEvent::ToolCallStarted {
            tool_call_id: tool_call_id(event).unwrap_or_else(|| "tool-call".to_string()),
            title: get_str(event, "toolName")
                .or_else(|| get_str(event, "name"))
                .unwrap_or("Tool")
                .to_string(),
            status: "running".to_string(),
        }],
        Some("tool_execution_update") => vec![AgentChatEvent::ToolCallUpdated {
            tool_call_id: tool_call_id(event).unwrap_or_else(|| "tool-call".to_string()),
            title: get_str(event, "toolName")
                .or_else(|| get_str(event, "name"))
                .map(str::to_string),
            status: Some("running".to_string()),
            body: body_from_event(event),
        }],
        Some("tool_execution_end") => {
            let failed = get_str(event, "status") == Some("failed")
                || event.get("error").and_then(Value::as_str).is_some();
            vec![AgentChatEvent::ToolCallUpdated {
                tool_call_id: tool_call_id(event).unwrap_or_else(|| "tool-call".to_string()),
                title: get_str(event, "toolName")
                    .or_else(|| get_str(event, "name"))
                    .map(str::to_string),
                status: Some(if failed { "failed" } else { "complete" }.to_string()),
                body: body_from_event(event)
                    .or_else(|| get_str(event, "error").map(str::to_string)),
            }]
        }
        Some("agent_end") => {
            if let Some(error) = get_str(event, "error").filter(|error| !error.trim().is_empty()) {
                vec![AgentChatEvent::Failed {
                    error: error.to_string(),
                }]
            } else {
                vec![AgentChatEvent::TurnFinished {
                    stop_reason: "stop".to_string(),
                }]
            }
        }
        Some("event_serialize_error") | Some("extension_error") => vec![AgentChatEvent::Failed {
            error: get_str(event, "error")
                .or_else(|| get_str(event, "message"))
                .unwrap_or("Pi RPC event error")
                .to_string(),
        }],
        _ => Vec::new(),
    }
}

fn map_message_update(event: &Value) -> Vec<AgentChatEvent> {
    let update = event
        .get("assistantMessageEvent")
        .or_else(|| event.get("messageEvent"))
        .unwrap_or(event);
    let delta = get_str(update, "delta")
        .or_else(|| get_str(update, "text"))
        .unwrap_or_default();

    match get_str(update, "type") {
        Some("text_delta") if !delta.is_empty() => split_text_delta_for_reveal(delta)
            .into_iter()
            .map(AgentChatEvent::AgentMessageDelta)
            .collect(),
        Some("thinking_delta") if !delta.is_empty() => split_text_delta_for_reveal(delta)
            .into_iter()
            .map(AgentChatEvent::AgentThoughtDelta)
            .collect(),
        Some("tool_call_delta") if !delta.is_empty() => vec![AgentChatEvent::ToolCallUpdated {
            tool_call_id: tool_call_id(update)
                .or_else(|| tool_call_id(event))
                .unwrap_or_else(|| "tool-call".to_string()),
            title: None,
            status: None,
            body: Some(delta.to_string()),
        }],
        _ => Vec::new(),
    }
}

pub(crate) fn split_text_delta_for_reveal(delta: &str) -> Vec<String> {
    if delta.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut buf = String::new();
    let mut chars = delta.chars().peekable();

    while let Some(ch) = chars.next() {
        buf.push(ch);
        let flush = if ch.is_whitespace() {
            chars.peek().is_some_and(|next| !next.is_whitespace())
        } else {
            chars.peek().is_none()
        };

        if flush {
            chunks.push(std::mem::take(&mut buf));
        }
    }

    if !buf.is_empty() {
        chunks.push(buf);
    }

    chunks
}

fn models_from_response_data(data: Option<&Value>) -> Vec<AcpModelEntry> {
    let models = data
        .and_then(|data| data.get("models").or(Some(data)))
        .and_then(Value::as_array)
        .into_iter()
        .flatten();

    models
        .filter_map(|model| {
            let raw_id = get_str(model, "id").or_else(|| get_str(model, "modelId"))?;
            let provider = get_str(model, "provider");
            let id = provider
                .filter(|_| !raw_id.contains('/'))
                .map(|provider| format!("{provider}/{raw_id}"))
                .unwrap_or_else(|| raw_id.to_string());
            let context_window = model
                .get("contextWindow")
                .and_then(Value::as_u64)
                .and_then(|value| u32::try_from(value).ok());

            Some(AcpModelEntry {
                id,
                display_name: get_str(model, "name")
                    .or_else(|| get_str(model, "displayName"))
                    .map(str::to_string),
                context_window,
            })
        })
        .collect()
}

fn get_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn get_value<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.get(key)
}

fn text_from_content_array(value: &Value) -> Option<String> {
    let parts: Vec<String> = value
        .as_array()?
        .iter()
        .filter_map(|item| {
            if item.get("type").and_then(Value::as_str) == Some("text") {
                return get_str(item, "text").map(str::to_string);
            }
            get_str(item, "content")
                .or_else(|| item.as_str())
                .map(str::to_string)
        })
        .filter(|text| !text.trim().is_empty())
        .collect();

    (!parts.is_empty()).then(|| parts.join("\n"))
}

fn tool_call_id(value: &Value) -> Option<String> {
    get_str(value, "toolCallId")
        .or_else(|| get_str(value, "tool_call_id"))
        .or_else(|| get_str(value, "id"))
        .map(str::to_string)
}

fn body_from_event(event: &Value) -> Option<String> {
    for key in ["partialResult", "result"] {
        let Some(value) = get_value(event, key) else {
            continue;
        };
        if let Some(text) = value.get("content").and_then(text_from_content_array) {
            return Some(text);
        }
        if let Some(text) = get_str(value, "text").filter(|text| !text.trim().is_empty()) {
            return Some(text.to_string());
        }
        if !value.is_null() {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn pi_rpc_text_delta_maps_to_agent_message_delta() {
        let events = map_rpc_event_to_events(&json!({
            "type": "message_update",
            "assistantMessageEvent": {"type": "text_delta", "delta": "hi"}
        }));

        assert!(matches!(
            events.as_slice(),
            [AgentChatEvent::AgentMessageDelta(delta)] if delta == "hi"
        ));
    }

    #[test]
    fn pi_rpc_text_delta_splits_for_reveal_without_losing_whitespace() {
        let delta = "Hello world\n\nNext step";
        let chunks = split_text_delta_for_reveal(delta);

        assert_eq!(chunks, vec!["Hello ", "world\n\n", "Next ", "step"]);
        assert_eq!(chunks.concat(), delta);
    }

    #[test]
    fn pi_rpc_thinking_delta_maps_to_agent_thought_delta() {
        let events = map_rpc_event_to_events(&json!({
            "type": "message_update",
            "assistantMessageEvent": {"type": "thinking_delta", "delta": "hmm"}
        }));

        assert!(matches!(
            events.as_slice(),
            [AgentChatEvent::AgentThoughtDelta(delta)] if delta == "hmm"
        ));
    }

    #[test]
    fn pi_rpc_tool_execution_start_maps_to_tool_call_started() {
        let events = map_rpc_event_to_events(&json!({
            "type": "tool_execution_start",
            "toolCallId": "tool-1",
            "toolName": "bash"
        }));

        assert!(matches!(
            events.as_slice(),
            [AgentChatEvent::ToolCallStarted { tool_call_id, title, status }]
                if tool_call_id == "tool-1" && title == "bash" && status == "running"
        ));
    }

    #[test]
    fn pi_rpc_tool_execution_update_maps_text_body() {
        let events = map_rpc_event_to_events(&json!({
            "type": "tool_execution_update",
            "toolCallId": "tool-1",
            "partialResult": {"content": [{"type": "text", "text": "line"}]}
        }));

        assert!(matches!(
            events.as_slice(),
            [AgentChatEvent::ToolCallUpdated { body: Some(body), status: Some(status), .. }]
                if body == "line" && status == "running"
        ));
    }

    #[test]
    fn pi_rpc_tool_execution_end_maps_complete_status() {
        let events = map_rpc_event_to_events(&json!({
            "type": "tool_execution_end",
            "toolCallId": "tool-1",
            "result": {"content": [{"type": "text", "text": "done"}]}
        }));

        assert!(matches!(
            events.as_slice(),
            [AgentChatEvent::ToolCallUpdated { body: Some(body), status: Some(status), .. }]
                if body == "done" && status == "complete"
        ));
    }

    #[test]
    fn pi_rpc_failed_response_maps_to_failed_event() {
        let response = PiRpcResponse {
            id: Some("1".to_string()),
            command: Some("prompt".to_string()),
            success: false,
            data: None,
            error: Some("nope".to_string()),
            raw: json!({}),
        };

        let events = map_rpc_response_to_events(&response);
        assert!(matches!(
            events.as_slice(),
            [AgentChatEvent::Failed { error }] if error == "nope"
        ));
    }

    #[test]
    fn pi_rpc_agent_end_error_maps_to_failed_event() {
        let events = map_rpc_event_to_events(&json!({
            "type": "agent_end",
            "error": "failed"
        }));

        assert!(matches!(
            events.as_slice(),
            [AgentChatEvent::Failed { error }] if error == "failed"
        ));
    }

    #[test]
    fn pi_rpc_agent_end_success_maps_to_turn_finished() {
        let events = map_rpc_event_to_events(&json!({"type": "agent_end"}));

        assert!(matches!(
            events.as_slice(),
            [AgentChatEvent::TurnFinished { stop_reason }] if stop_reason == "stop"
        ));
    }

    #[test]
    fn pi_rpc_get_available_models_response_maps_to_models_available() {
        let response = PiRpcResponse {
            id: Some("models-1".to_string()),
            command: Some("get_available_models".to_string()),
            success: true,
            data: Some(json!({
                "models": [{"provider": "openai", "id": "gpt-5.4", "name": "GPT 5.4", "contextWindow": 256000}]
            })),
            error: None,
            raw: json!({}),
        };

        let events = map_rpc_response_to_events(&response);
        assert!(matches!(
            events.as_slice(),
            [AgentChatEvent::ModelsAvailable { models, .. }]
                if models.len() == 1
                    && models[0].id == "openai/gpt-5.4"
                    && models[0].display_name.as_deref() == Some("GPT 5.4")
                    && models[0].context_window == Some(256000)
        ));
    }
}
