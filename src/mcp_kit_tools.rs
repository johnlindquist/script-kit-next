//! MCP Kit Namespace Tools
//!
//! Implements the kit/* namespace MCP tools for Script Kit:
//! - kit/show: Show the Script Kit window
//! - kit/hide: Hide the Script Kit window
//! - kit/state: Get current app state

use crate::stdin_commands::{ExternalCommand, ExternalCommandRequestId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

pub const KIT_SHOW_TOOL: &str = "kit/show";
pub const KIT_HIDE_TOOL: &str = "kit/hide";
pub const KIT_STATE_TOOL: &str = "kit/state";
pub const KIT_TRIGGER_BUILTIN_TOOL: &str = "kit/trigger_builtin";

const KIT_RUNTIME_BRIDGE_TIMEOUT: Duration = Duration::from_secs(10);

/// Kit tool definitions for MCP tools/list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Result of a kit tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ToolContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content item in tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KitRuntimeCommandResult {
    pub accepted: bool,
    pub command_type: String,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KitToolEnvelope {
    pub ok: bool,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<KitRuntimeCommandResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<KitToolError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KitToolErrorCode {
    InvalidParams,
    MissingRuntime,
    ScopeDenied,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KitToolError {
    pub code: KitToolErrorCode,
    pub message: String,
}

impl KitToolError {
    fn new(code: KitToolErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub struct McpKitRuntimeCommand {
    pub command: ExternalCommand,
    pub correlation_id: String,
    pub response_tx: std::sync::mpsc::SyncSender<Result<KitRuntimeCommandResult, KitToolError>>,
}

pub trait McpKitRuntimeBridge: Send + Sync {
    fn dispatch_external_command(
        &self,
        command: ExternalCommand,
        correlation_id: String,
    ) -> Result<KitRuntimeCommandResult, KitToolError>;
}

#[derive(Clone)]
pub struct GpuiKitRuntimeBridge {
    tx: async_channel::Sender<McpKitRuntimeCommand>,
    timeout: Duration,
}

impl GpuiKitRuntimeBridge {
    pub fn with_default_timeout(tx: async_channel::Sender<McpKitRuntimeCommand>) -> Self {
        Self {
            tx,
            timeout: KIT_RUNTIME_BRIDGE_TIMEOUT,
        }
    }
}

impl McpKitRuntimeBridge for GpuiKitRuntimeBridge {
    fn dispatch_external_command(
        &self,
        command: ExternalCommand,
        correlation_id: String,
    ) -> Result<KitRuntimeCommandResult, KitToolError> {
        let (response_tx, response_rx) = std::sync::mpsc::sync_channel(1);
        self.tx
            .send_blocking(McpKitRuntimeCommand {
                command,
                correlation_id,
                response_tx,
            })
            .map_err(|_| {
                KitToolError::new(
                    KitToolErrorCode::MissingRuntime,
                    "MCP runtime command bridge is disconnected",
                )
            })?;

        response_rx.recv_timeout(self.timeout).map_err(|_| {
            KitToolError::new(
                KitToolErrorCode::Internal,
                "Timed out waiting for MCP runtime command bridge",
            )
        })?
    }
}

/// App state returned by kit/state tool
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppState {
    pub visible: bool,
    pub focused: bool,
    #[serde(rename = "activePrompt")]
    pub active_prompt: Option<String>,
}

/// Returns the tool definitions for kit/* namespace tools
pub fn get_kit_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: KIT_SHOW_TOOL.to_string(),
            description: "Show the Script Kit window".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: KIT_HIDE_TOOL.to_string(),
            description: "Hide the Script Kit window".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: KIT_STATE_TOOL.to_string(),
            description: "Get current Script Kit app state".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: KIT_TRIGGER_BUILTIN_TOOL.to_string(),
            description: "Trigger a Script Kit built-in command by canonical builtinId or deprecated legacy name using the same runtime dispatcher as stdin triggerBuiltin.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "builtinId": { "type": "string", "description": "Canonical builtin id, e.g. builtin/clipboard-history." },
                    "name": { "type": "string", "description": "Deprecated legacy builtin alias. Prefer builtinId." },
                    "requestId": { "type": "string" }
                }
            }),
        },
    ]
}

/// Check if a tool name is in the kit/* namespace
pub fn is_kit_tool(name: &str) -> bool {
    name.starts_with("kit/")
}

/// Handle a kit/* namespace tool call
///
/// Note: This returns a result that the caller should use to actually perform
/// the window operations. The actual show/hide operations require GPUI context
/// which is not available in this module.
pub fn handle_kit_tool_call(name: &str, _arguments: &Value) -> ToolResult {
    match name {
        KIT_SHOW_TOOL => ToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: "Window show requested".to_string(),
            }],
            is_error: None,
        },
        KIT_HIDE_TOOL => ToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: "Window hide requested".to_string(),
            }],
            is_error: None,
        },
        KIT_STATE_TOOL => {
            // Return default state - actual state will be injected by caller
            let state = AppState::default();
            ToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: serde_json::to_string(&state).unwrap_or_default(),
                }],
                is_error: None,
            }
        }
        _ => ToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: format!("Unknown kit tool: {}", name),
            }],
            is_error: Some(true),
        },
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct KitShowHideArgs {
    #[serde(default)]
    request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct KitTriggerBuiltinArgs {
    #[serde(default)]
    builtin_id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    request_id: Option<String>,
}

pub fn requires_ui_control_scope(name: &str) -> bool {
    matches!(
        name,
        KIT_SHOW_TOOL | KIT_HIDE_TOOL | KIT_TRIGGER_BUILTIN_TOOL
    )
}

pub fn handle_kit_tool_call_with_runtime(
    name: &str,
    arguments: &Value,
    runtime: Option<&(dyn McpKitRuntimeBridge + Send + Sync)>,
    trace_id: &str,
) -> ToolResult {
    match name {
        KIT_SHOW_TOOL => {
            let args: KitShowHideArgs = match serde_json::from_value(arguments.clone()) {
                Ok(args) => args,
                Err(error) => {
                    return error_tool_result(
                        "kit_show",
                        KitToolError::new(
                            KitToolErrorCode::InvalidParams,
                            format!("Invalid kit/show arguments: {error}"),
                        ),
                    );
                }
            };
            dispatch_runtime_command(
                "kit_show",
                runtime,
                trace_id,
                ExternalCommand::Show {
                    request_id: args.request_id.map(ExternalCommandRequestId::from),
                },
            )
        }
        KIT_HIDE_TOOL => {
            let args: KitShowHideArgs = match serde_json::from_value(arguments.clone()) {
                Ok(args) => args,
                Err(error) => {
                    return error_tool_result(
                        "kit_hide",
                        KitToolError::new(
                            KitToolErrorCode::InvalidParams,
                            format!("Invalid kit/hide arguments: {error}"),
                        ),
                    );
                }
            };
            dispatch_runtime_command(
                "kit_hide",
                runtime,
                trace_id,
                ExternalCommand::Hide {
                    request_id: args.request_id.map(ExternalCommandRequestId::from),
                },
            )
        }
        KIT_TRIGGER_BUILTIN_TOOL => {
            let args: KitTriggerBuiltinArgs = match serde_json::from_value(arguments.clone()) {
                Ok(args) => args,
                Err(error) => {
                    return error_tool_result(
                        "kit_trigger_builtin",
                        KitToolError::new(
                            KitToolErrorCode::InvalidParams,
                            format!("Invalid kit/trigger_builtin arguments: {error}"),
                        ),
                    );
                }
            };
            if args.builtin_id.is_some() == args.name.is_some() {
                return error_tool_result(
                    "kit_trigger_builtin",
                    KitToolError::new(
                        KitToolErrorCode::InvalidParams,
                        "kit/trigger_builtin requires exactly one of builtinId or name",
                    ),
                );
            }
            dispatch_runtime_command(
                "kit_trigger_builtin",
                runtime,
                trace_id,
                ExternalCommand::TriggerBuiltin {
                    builtin_id: args.builtin_id,
                    name: args.name,
                    request_id: args.request_id.map(ExternalCommandRequestId::from),
                },
            )
        }
        _ => handle_kit_tool_call(name, arguments),
    }
}

pub fn scope_denied_tool_result(name: &str, required_scope: &str) -> ToolResult {
    error_tool_result(
        name,
        KitToolError::new(
            KitToolErrorCode::ScopeDenied,
            format!("Missing required MCP scope: {required_scope}"),
        ),
    )
}

fn dispatch_runtime_command(
    action: &str,
    runtime: Option<&(dyn McpKitRuntimeBridge + Send + Sync)>,
    trace_id: &str,
    command: ExternalCommand,
) -> ToolResult {
    let Some(runtime) = runtime else {
        return error_tool_result(
            action,
            KitToolError::new(
                KitToolErrorCode::MissingRuntime,
                "MCP runtime command bridge is unavailable",
            ),
        );
    };

    match runtime.dispatch_external_command(command, trace_id.to_string()) {
        Ok(result) => envelope_tool_result(KitToolEnvelope {
            ok: true,
            action: action.to_string(),
            result: Some(result),
            error: None,
        }),
        Err(error) => error_tool_result(action, error),
    }
}

fn error_tool_result(action: &str, error: KitToolError) -> ToolResult {
    let mut result = envelope_tool_result(KitToolEnvelope {
        ok: false,
        action: action.to_string(),
        result: None,
        error: Some(error),
    });
    result.is_error = Some(true);
    result
}

fn envelope_tool_result(envelope: KitToolEnvelope) -> ToolResult {
    ToolResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: serde_json::to_string(&envelope).unwrap_or_else(|error| {
                format!(
                    r#"{{"ok":false,"action":"kit_internal","error":{{"code":"internal","message":"Failed to serialize kit result: {error}"}}}}"#
                )
            }),
        }],
        is_error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =======================================================
    // TDD Tests - Written FIRST per spec requirements
    // =======================================================

    #[test]
    fn test_kit_show_tool_definition() {
        let tools = get_kit_tool_definitions();
        let show_tool = tools.iter().find(|t| t.name == "kit/show");

        assert!(show_tool.is_some(), "kit/show tool should be defined");
        let tool = show_tool.unwrap();
        assert_eq!(tool.name, "kit/show");
        assert_eq!(tool.description, "Show the Script Kit window");
        assert!(tool.input_schema.get("type").is_some());
        assert_eq!(tool.input_schema.get("type").unwrap(), "object");
    }

    #[test]
    fn test_kit_hide_tool_definition() {
        let tools = get_kit_tool_definitions();
        let hide_tool = tools.iter().find(|t| t.name == "kit/hide");

        assert!(hide_tool.is_some(), "kit/hide tool should be defined");
        let tool = hide_tool.unwrap();
        assert_eq!(tool.name, "kit/hide");
        assert_eq!(tool.description, "Hide the Script Kit window");
        assert!(tool.input_schema.get("type").is_some());
        assert_eq!(tool.input_schema.get("type").unwrap(), "object");
    }

    #[test]
    fn test_kit_state_tool_definition() {
        let tools = get_kit_tool_definitions();
        let state_tool = tools.iter().find(|t| t.name == "kit/state");

        assert!(state_tool.is_some(), "kit/state tool should be defined");
        let tool = state_tool.unwrap();
        assert_eq!(tool.name, "kit/state");
        assert_eq!(tool.description, "Get current Script Kit app state");
        assert!(tool.input_schema.get("type").is_some());
        assert_eq!(tool.input_schema.get("type").unwrap(), "object");
    }

    #[test]
    fn test_tools_list_includes_kit_tools() {
        let tools = get_kit_tool_definitions();

        assert_eq!(tools.len(), 4, "Should have exactly 4 kit tools");

        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"kit/show"));
        assert!(tool_names.contains(&"kit/hide"));
        assert!(tool_names.contains(&"kit/state"));
        assert!(tool_names.contains(&"kit/trigger_builtin"));
    }

    #[test]
    fn test_kit_show_call_succeeds() {
        let result = handle_kit_tool_call("kit/show", &serde_json::json!({}));

        assert!(result.is_error.is_none() || result.is_error == Some(false));
        assert!(!result.content.is_empty());
        assert_eq!(result.content[0].content_type, "text");
        assert!(result.content[0].text.contains("show"));
    }

    #[test]
    fn test_kit_hide_call_succeeds() {
        let result = handle_kit_tool_call("kit/hide", &serde_json::json!({}));

        assert!(result.is_error.is_none() || result.is_error == Some(false));
        assert!(!result.content.is_empty());
        assert_eq!(result.content[0].content_type, "text");
        assert!(result.content[0].text.contains("hide"));
    }

    #[test]
    fn test_kit_state_returns_json() {
        let result = handle_kit_tool_call("kit/state", &serde_json::json!({}));

        assert!(result.is_error.is_none() || result.is_error == Some(false));
        assert!(!result.content.is_empty());
        assert_eq!(result.content[0].content_type, "text");

        // Verify the result is valid JSON with expected fields
        let state: Result<AppState, _> = serde_json::from_str(&result.content[0].text);
        assert!(state.is_ok(), "kit/state should return valid JSON");

        let state = state.unwrap();
        // Default state should have visible=false, focused=false
        assert!(!state.visible);
        assert!(!state.focused);
    }

    #[test]
    fn test_is_kit_tool() {
        assert!(is_kit_tool("kit/show"));
        assert!(is_kit_tool("kit/hide"));
        assert!(is_kit_tool("kit/state"));
        assert!(is_kit_tool("kit/custom"));

        assert!(!is_kit_tool("scripts/run"));
        assert!(!is_kit_tool("resources/list"));
        assert!(!is_kit_tool("kitshow")); // No slash
    }

    #[test]
    fn test_unknown_kit_tool_returns_error() {
        let result = handle_kit_tool_call("kit/unknown", &serde_json::json!({}));

        assert_eq!(result.is_error, Some(true));
        assert!(!result.content.is_empty());
        assert!(result.content[0].text.contains("Unknown kit tool"));
    }

    #[test]
    fn test_tool_definition_serialization() {
        let tools = get_kit_tool_definitions();
        let json = serde_json::to_value(&tools);

        assert!(json.is_ok(), "Tool definitions should serialize to JSON");

        let json = json.unwrap();
        assert!(json.is_array());

        // Check the first tool has expected structure
        let first_tool = &json[0];
        assert!(first_tool.get("name").is_some());
        assert!(first_tool.get("description").is_some());
        assert!(first_tool.get("inputSchema").is_some());
    }

    #[test]
    fn test_app_state_serialization() {
        let state = AppState {
            visible: true,
            focused: true,
            active_prompt: Some("arg".to_string()),
        };

        let json = serde_json::to_value(&state);
        assert!(json.is_ok());

        let json = json.unwrap();
        assert_eq!(json.get("visible").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(json.get("focused").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            json.get("activePrompt").and_then(|v| v.as_str()),
            Some("arg")
        );
    }

    #[test]
    fn test_tool_result_serialization() {
        let result = ToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: "test message".to_string(),
            }],
            is_error: None,
        };

        let json = serde_json::to_string(&result);
        assert!(json.is_ok());

        let json = json.unwrap();
        // is_error should be omitted when None
        assert!(!json.contains("isError"));
        assert!(json.contains("content"));
        assert!(json.contains("text"));
    }
}
