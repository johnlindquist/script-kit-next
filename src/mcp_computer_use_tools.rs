//! MCP computer-use tools.
//!
//! Iteration 1 exposes `computer/see` as the agent-facing name for Script Kit's
//! existing `inspectAutomationWindow` snapshot contract. Native input actions
//! remain deferred until they can cite stable inspection receipts.

use crate::computer_use::runtime_bridge::{
    ComputerUseInspectRequest, ComputerUseRuntimeBridge, ComputerUseRuntimeError,
};
use crate::computer_use::types::ComputerUseSeeArgs;
use crate::mcp_kit_tools::{ToolContent, ToolDefinition, ToolResult};
use serde_json::Value;

pub const COMPUTER_USE_NAMESPACE: &str = "computer/";
pub const COMPUTER_SEE_TOOL: &str = "computer/see";

pub fn get_computer_use_tool_definitions() -> Vec<ToolDefinition> {
    vec![ToolDefinition {
        name: COMPUTER_SEE_TOOL.to_string(),
        description:
            "Inspect a Script Kit automation window and return a state-first computer-use observation."
                .to_string(),
        input_schema: computer_see_input_schema(),
    }]
}

pub fn is_computer_use_tool(name: &str) -> bool {
    name.starts_with(COMPUTER_USE_NAMESPACE)
}

pub fn handle_computer_use_tool_call(
    name: &str,
    arguments: &Value,
    runtime: Option<&dyn ComputerUseRuntimeBridge>,
) -> ToolResult {
    match name {
        COMPUTER_SEE_TOOL => handle_see(arguments, runtime),
        _ => error_result(
            "unknown_tool",
            &format!("Unknown computer-use tool: {name}"),
        ),
    }
}

fn handle_see(arguments: &Value, runtime: Option<&dyn ComputerUseRuntimeBridge>) -> ToolResult {
    let args: ComputerUseSeeArgs = match serde_json::from_value(arguments.clone()) {
        Ok(args) => args,
        Err(error) => return error_result("invalid_arguments", &error.to_string()),
    };

    let Some(runtime) = runtime else {
        return runtime_error_result(&args, ComputerUseRuntimeError::Unavailable);
    };

    let request = ComputerUseInspectRequest {
        target: args.target,
        hi_dpi: args.hi_dpi,
        probes: args.probes,
    };

    match runtime.inspect_automation_window(request) {
        Ok(snapshot) => json_tool_result(&snapshot),
        Err(error) => runtime_error_result(&args, error),
    }
}

fn runtime_error_result(args: &ComputerUseSeeArgs, error: ComputerUseRuntimeError) -> ToolResult {
    let target = args
        .target
        .as_ref()
        .map(|target| serde_json::to_value(target).unwrap_or(Value::Null));

    ToolResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: serde_json::json!({
                "schemaVersion": 1,
                "errorCode": error.error_code(),
                "message": error.message(),
                "target": target,
            })
            .to_string(),
        }],
        is_error: Some(true),
    }
}

fn json_tool_result<T: serde::Serialize>(value: &T) -> ToolResult {
    match serde_json::to_string(value) {
        Ok(text) => ToolResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text,
            }],
            is_error: None,
        },
        Err(error) => error_result("serialization_failed", &error.to_string()),
    }
}

fn error_result(code: &str, message: &str) -> ToolResult {
    ToolResult {
        content: vec![ToolContent {
            content_type: "text".to_string(),
            text: serde_json::json!({
                "schemaVersion": 1,
                "errorCode": code,
                "message": message,
            })
            .to_string(),
        }],
        is_error: Some(true),
    }
}

fn computer_see_input_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "target": {
                "description": "AutomationWindowTarget. Omit to use the focused automation window.",
                "oneOf": [
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": { "type": { "const": "main" } },
                        "required": ["type"]
                    },
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": { "type": { "const": "focused" } },
                        "required": ["type"]
                    },
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "type": { "const": "id" },
                            "id": { "type": "string" }
                        },
                        "required": ["type", "id"]
                    },
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "type": { "const": "kind" },
                            "kind": {
                                "type": "string",
                                "enum": ["main", "notes", "ai", "miniAi", "acpDetached", "actionsDialog", "promptPopup"]
                            },
                            "index": { "type": "integer", "minimum": 0 }
                        },
                        "required": ["type", "kind"]
                    },
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "properties": {
                            "type": { "const": "titleContains" },
                            "text": { "type": "string" }
                        },
                        "required": ["type", "text"]
                    }
                ]
            },
            "hiDpi": { "type": "boolean", "default": false },
            "probes": {
                "type": "array",
                "default": [],
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "x": { "type": "integer", "minimum": 0 },
                        "y": { "type": "integer", "minimum": 0 }
                    },
                    "required": ["x", "y"]
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{
        AutomationInspectSnapshot, AutomationWindowTarget, SemanticQuality,
        AUTOMATION_INSPECT_SCHEMA_VERSION,
    };

    struct FakeComputerUseRuntime;

    impl ComputerUseRuntimeBridge for FakeComputerUseRuntime {
        fn inspect_automation_window(
            &self,
            request: ComputerUseInspectRequest,
        ) -> Result<AutomationInspectSnapshot, ComputerUseRuntimeError> {
            assert_eq!(request.target, Some(AutomationWindowTarget::Focused));
            assert_eq!(request.hi_dpi, Some(true));
            assert_eq!(
                request.probes,
                vec![
                    crate::protocol::PixelProbe { x: 10, y: 20 },
                    crate::protocol::PixelProbe { x: 30, y: 40 },
                ]
            );

            Ok(AutomationInspectSnapshot {
                schema_version: AUTOMATION_INSPECT_SCHEMA_VERSION,
                window_id: "main:0".to_string(),
                window_kind: "Main".to_string(),
                title: Some("Script Kit".to_string()),
                resolved_bounds: None,
                target_bounds_in_screenshot: None,
                surface_hit_point: None,
                suggested_hit_points: Vec::new(),
                elements: Vec::new(),
                total_count: 0,
                focused_semantic_id: None,
                selected_semantic_id: None,
                screenshot_width: Some(800),
                screenshot_height: Some(600),
                pixel_probes: Vec::new(),
                os_window_id: Some(123),
                semantic_quality: Some(SemanticQuality::Full),
                warnings: Vec::new(),
            })
        }
    }

    #[test]
    fn computer_see_tool_definition_is_registered() {
        let names: Vec<String> = get_computer_use_tool_definitions()
            .into_iter()
            .map(|tool| tool.name)
            .collect();

        assert_eq!(names, vec![COMPUTER_SEE_TOOL.to_string()]);
    }

    #[test]
    fn computer_see_tool_definition_has_closed_schema() {
        let tool = get_computer_use_tool_definitions()
            .into_iter()
            .find(|tool| tool.name == COMPUTER_SEE_TOOL)
            .expect("computer/see tool");

        assert_eq!(
            tool.input_schema
                .get("additionalProperties")
                .and_then(Value::as_bool),
            Some(false)
        );
    }

    #[test]
    fn is_computer_use_tool_matches_only_computer_namespace() {
        assert!(is_computer_use_tool("computer/see"));
        assert!(!is_computer_use_tool("computer-use/see"));
        assert!(!is_computer_use_tool("kit/state"));
    }

    #[test]
    fn computer_see_without_runtime_returns_tool_error() {
        let result = handle_computer_use_tool_call(COMPUTER_SEE_TOOL, &serde_json::json!({}), None);

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("runtime_unavailable"));
    }

    #[test]
    fn computer_see_with_runtime_returns_raw_snapshot() {
        let runtime = FakeComputerUseRuntime;
        let result = handle_computer_use_tool_call(
            COMPUTER_SEE_TOOL,
            &serde_json::json!({
                "target": { "type": "focused" },
                "hiDpi": true,
                "probes": [
                    { "x": 10, "y": 20 },
                    { "x": 30, "y": 40 }
                ]
            }),
            Some(&runtime),
        );

        assert_eq!(result.is_error, None);

        let snapshot: AutomationInspectSnapshot =
            serde_json::from_str(&result.content[0].text).expect("automation inspect snapshot");
        assert_eq!(snapshot.schema_version, AUTOMATION_INSPECT_SCHEMA_VERSION);
        assert_eq!(snapshot.window_id, "main:0");
        assert!(!result.content[0].text.contains("\"action\""));
    }

    #[test]
    fn computer_see_rejects_max_elements_instead_of_truncating() {
        let result = handle_computer_use_tool_call(
            COMPUTER_SEE_TOOL,
            &serde_json::json!({ "maxElements": 1 }),
            None,
        );

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("invalid_arguments"));
    }

    #[test]
    fn computer_see_rejects_bad_arguments() {
        let result = handle_computer_use_tool_call(
            COMPUTER_SEE_TOOL,
            &serde_json::json!({ "unknown": true }),
            None,
        );

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("invalid_arguments"));
    }
}
