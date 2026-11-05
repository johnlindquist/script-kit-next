//! ACP (Agent Client Protocol) framing for the goose sidecar.
//!
//! goose speaks ACP v1: newline-delimited JSON-RPC 2.0 over stdio. Script Kit
//! is the client. This module builds the requests Script Kit sends and
//! classifies every line goose writes back, without owning any transport.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::ai::agent_chat::content::ContentBlock;

pub(crate) const ACP_PROTOCOL_VERSION: u64 = 1;
/// JSON-RPC code goose returns when the selected provider rejects the
/// credentials (`agent_client_protocol::Error::auth_required`).
pub(crate) const ACP_AUTH_REQUIRED_CODE: i64 = -32000;

/// Everything needed to start one goose ACP process for a profile.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GooseLaunchSpec {
    pub command: PathBuf,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: PathBuf,
    /// Inline goose recipe carrying instructions, tools, provider and model.
    pub recipe: Value,
    pub session_title: String,
    /// Keep the conversation out of goose's visible session history.
    pub hidden_session: bool,
    /// Reasoning level requested by the profile (`thinking_effort` option).
    pub thinking: Option<String>,
    /// Script Kit agent-tools policy registered for this session, if any.
    pub agent_policy_id: Option<String>,
    /// The dedicated launcher profile requires this exact live tool surface.
    pub launcher_author: Option<sk_personal::api::Endpoint>,
    /// Whether dropping this runtime releases the policy. Isolated turns
    /// borrow their parent session's policy and must not release it.
    pub owns_agent_policy: bool,
    pub surface: crate::ai::phase_trace::AiSurface,
    /// How much the model may do before it asks; applied to the session.
    pub approval_mode: crate::ai::agent_chat::approval::AgentApprovalMode,
}

impl GooseLaunchSpec {
    pub(crate) fn new(command: impl Into<PathBuf>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            command: command.into(),
            args: vec!["acp".to_string()],
            env: HashMap::new(),
            cwd: cwd.into(),
            recipe: json!({}),
            session_title: "Script Kit".to_string(),
            hidden_session: true,
            thinking: None,
            agent_policy_id: None,
            launcher_author: None,
            owns_agent_policy: true,
            surface: crate::ai::phase_trace::AiSurface::AgentChat,
            approval_mode: crate::ai::agent_chat::approval::AgentApprovalMode::AskBeforeChanges,
        }
    }
}

/// One line read from goose's stdout.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AcpLine {
    /// Answer to a request Script Kit sent.
    Response {
        id: u64,
        outcome: Result<Value, AcpError>,
    },
    /// Fire-and-forget message from goose (`session/update`, ...).
    Notification { method: String, params: Value },
    /// Request goose expects Script Kit to answer (`session/request_permission`).
    Request {
        id: Value,
        method: String,
        params: Value,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AcpError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

impl AcpError {
    /// The error's own words: message plus any string/object detail goose
    /// attached. Used as the opaque provider payload for classification.
    pub(crate) fn detail(&self) -> String {
        match &self.data {
            Some(Value::String(data)) if !data.trim().is_empty() => {
                format!("{}: {data}", self.message)
            }
            Some(Value::Null) | None => self.message.clone(),
            Some(data) => format!("{}: {data}", self.message),
        }
    }
}

pub(crate) fn parse_line(line: &str) -> Result<AcpLine> {
    let raw: Value = serde_json::from_str(line)?;
    let method = raw
        .get("method")
        .and_then(Value::as_str)
        .map(str::to_string);
    let id = raw.get("id").cloned();
    match (method, id) {
        (Some(method), Some(id)) if !id.is_null() => Ok(AcpLine::Request {
            id,
            method,
            params: raw.get("params").cloned().unwrap_or(Value::Null),
        }),
        (Some(method), _) => Ok(AcpLine::Notification {
            method,
            params: raw.get("params").cloned().unwrap_or(Value::Null),
        }),
        (None, Some(id)) => {
            let id = id
                .as_u64()
                .ok_or_else(|| anyhow!("ACP response id is not a client request id: {id}"))?;
            let outcome = match raw.get("error") {
                Some(error) if !error.is_null() => Err(AcpError {
                    code: error.get("code").and_then(Value::as_i64).unwrap_or(-32603),
                    message: error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("ACP request failed")
                        .to_string(),
                    data: error.get("data").cloned(),
                }),
                _ => Ok(raw.get("result").cloned().unwrap_or(Value::Null)),
            };
            Ok(AcpLine::Response { id, outcome })
        }
        (None, None) => Err(anyhow!(
            "ACP line is neither a request, response, nor notification"
        )),
    }
}

pub(crate) fn request(id: u64, method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
}

pub(crate) fn notification(method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "method": method, "params": params })
}

pub(crate) fn response(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

pub(crate) fn error_response(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

pub(crate) fn initialize_params() -> Value {
    json!({
        "protocolVersion": ACP_PROTOCOL_VERSION,
        "clientCapabilities": {
            "fs": { "readTextFile": false, "writeTextFile": false },
            "terminal": false,
        },
        "clientInfo": {
            "name": "script-kit",
            "version": env!("CARGO_PKG_VERSION"),
        },
    })
}

pub(crate) fn new_session_params(spec: &GooseLaunchSpec, recipe_deeplink: &str) -> Value {
    json!({
        "cwd": spec.cwd,
        "mcpServers": [],
        "_meta": {
            "recipeDeeplink": recipe_deeplink,
            "hidden": spec.hidden_session,
            "sessionTitle": spec.session_title,
        },
    })
}

pub(crate) fn prompt_params(session_id: &str, blocks: &[ContentBlock]) -> Result<Value> {
    let prompt: Vec<Value> = blocks
        .iter()
        .map(|block| match block {
            ContentBlock::Text(text) => json!({ "type": "text", "text": text.text }),
            ContentBlock::Image(image) => json!({
                "type": "image",
                "data": image.data,
                "mimeType": image.mime_type,
            }),
        })
        .collect();
    if prompt.is_empty() {
        return Err(anyhow!("cannot send an empty prompt"));
    }
    Ok(json!({ "sessionId": session_id, "prompt": prompt }))
}

pub(crate) fn cancel_params(session_id: &str) -> Value {
    json!({ "sessionId": session_id })
}

pub(crate) fn set_config_option_params(session_id: &str, config_id: &str, value: &str) -> Value {
    json!({ "sessionId": session_id, "configId": config_id, "value": value })
}

/// Split a `provider/model` selection. goose provider ids never contain `/`,
/// while model ids may (`openrouter`'s `anthropic/claude-...`), so only the
/// first separator counts.
pub(crate) fn split_model_selection(raw: &str) -> Option<(&str, &str)> {
    let raw = raw.trim();
    let (provider, model) = raw.split_once('/')?;
    let (provider, model) = (provider.trim(), model.trim());
    (!provider.is_empty() && !model.is_empty()).then_some((provider, model))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::agent_chat::content::{ImageContent, TextContent};

    #[test]
    fn classifies_responses_notifications_and_requests() {
        assert_eq!(
            parse_line(r#"{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}"#).unwrap(),
            AcpLine::Response {
                id: 3,
                outcome: Ok(json!({"stopReason": "end_turn"})),
            }
        );
        assert!(matches!(
            parse_line(r#"{"jsonrpc":"2.0","method":"session/update","params":{"x":1}}"#).unwrap(),
            AcpLine::Notification { method, .. } if method == "session/update"
        ));
        assert!(matches!(
            parse_line(r#"{"jsonrpc":"2.0","id":"p1","method":"session/request_permission","params":{}}"#).unwrap(),
            AcpLine::Request { method, .. } if method == "session/request_permission"
        ));
    }

    #[test]
    fn error_responses_keep_code_message_and_data() {
        let line = r#"{"jsonrpc":"2.0","id":3,"error":{"code":-32603,"message":"Internal error","data":"Error getting agent reply: Provider not set"}}"#;
        let AcpLine::Response {
            outcome: Err(error),
            ..
        } = parse_line(line).unwrap()
        else {
            panic!("expected error response");
        };
        assert_eq!(error.code, -32603);
        assert_eq!(
            error.detail(),
            "Internal error: Error getting agent reply: Provider not set"
        );
    }

    #[test]
    fn prompt_carries_text_and_images_as_acp_content_blocks() {
        let params = prompt_params(
            "s1",
            &[
                ContentBlock::Text(TextContent::new("hi")),
                ContentBlock::Image(ImageContent::new("AAAA", "image/png")),
            ],
        )
        .unwrap();
        assert_eq!(params["sessionId"], "s1");
        assert_eq!(params["prompt"][0], json!({"type": "text", "text": "hi"}));
        assert_eq!(params["prompt"][1]["mimeType"], "image/png");
        assert!(prompt_params("s1", &[]).is_err());
    }

    #[test]
    fn model_selection_splits_on_the_first_slash_only() {
        assert_eq!(
            split_model_selection("openrouter/anthropic/claude-opus-5.5"),
            Some(("openrouter", "anthropic/claude-opus-5.5"))
        );
        assert_eq!(split_model_selection("gpt-5.6-sol"), None);
        assert_eq!(split_model_selection("/model"), None);
    }
}
