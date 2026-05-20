use script_kit_gpui::mcp_kit_tools::{KitRuntimeCommandResult, KitToolError, McpKitRuntimeBridge};
use script_kit_gpui::mcp_protocol::{
    handle_request_with_runtime_context, JsonRpcRequest, McpRuntimeContext,
};
use script_kit_gpui::stdin_commands::ExternalCommand;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct RecordingKitBridge {
    commands: Mutex<Vec<String>>,
}

impl McpKitRuntimeBridge for RecordingKitBridge {
    fn dispatch_external_command(
        &self,
        command: ExternalCommand,
        _correlation_id: String,
    ) -> Result<KitRuntimeCommandResult, KitToolError> {
        let command_type = command.command_type().to_string();
        self.commands
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(command_type.clone());
        Ok(KitRuntimeCommandResult {
            accepted: true,
            command_type,
            request_id: command.request_id().map(ToString::to_string),
        })
    }
}

fn call_tool(name: &str, arguments: Value, context: Option<&McpRuntimeContext>) -> Value {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("tool-call"),
        method: "tools/call".to_string(),
        params: json!({
            "name": name,
            "arguments": arguments,
        }),
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("build test runtime");
    let response = runtime.block_on(handle_request_with_runtime_context(
        request,
        &[],
        &[],
        None,
        context,
    ));
    response
        .result
        .expect("tools/call should return a JSON-RPC success envelope")
}

fn first_tool_text(result: &Value) -> Value {
    let text = result["content"][0]["text"]
        .as_str()
        .expect("tool result should include text content");
    serde_json::from_str(text).expect("tool text should be structured JSON")
}

#[test]
fn kit_show_and_trigger_builtin_dispatch_through_runtime_bridge() {
    let bridge = Arc::new(RecordingKitBridge::default());
    let context = McpRuntimeContext {
        kit_runtime_bridge: Some(bridge.clone()),
        token_scopes: vec!["mcp:read".to_string(), "ui:control".to_string()],
        ..McpRuntimeContext::default()
    };

    let show = call_tool("kit/show", json!({ "requestId": "show-1" }), Some(&context));
    let show_payload = first_tool_text(&show);
    assert_eq!(show_payload["ok"], true);
    assert_eq!(show_payload["result"]["commandType"], "show");
    assert_eq!(show_payload["result"]["requestId"], "show-1");

    let trigger = call_tool(
        "kit/trigger_builtin",
        json!({ "builtinId": "builtin/clipboard-history", "requestId": "trigger-1" }),
        Some(&context),
    );
    let trigger_payload = first_tool_text(&trigger);
    assert_eq!(trigger_payload["ok"], true);
    assert_eq!(trigger_payload["result"]["commandType"], "triggerBuiltin");

    let commands = bridge
        .commands
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    assert_eq!(commands, vec!["show", "triggerBuiltin"]);
}

#[test]
fn kit_runtime_tools_require_ui_control_scope() {
    let context = McpRuntimeContext {
        token_scopes: vec!["mcp:read".to_string()],
        ..McpRuntimeContext::default()
    };

    let result = call_tool("kit/hide", json!({}), Some(&context));
    let payload = first_tool_text(&result);
    assert_eq!(result["isError"], true);
    assert_eq!(payload["error"]["code"], "scope_denied");
    assert!(payload["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("ui:control"));
}

#[test]
fn kit_trigger_builtin_rejects_ambiguous_identifier_payload() {
    let context = McpRuntimeContext {
        kit_runtime_bridge: Some(Arc::new(RecordingKitBridge::default())),
        token_scopes: vec!["mcp:read".to_string(), "ui:control".to_string()],
        ..McpRuntimeContext::default()
    };

    let result = call_tool(
        "kit/trigger_builtin",
        json!({ "builtinId": "builtin/clipboard-history", "name": "clipboardHistory" }),
        Some(&context),
    );
    let payload = first_tool_text(&result);
    assert_eq!(result["isError"], true);
    assert_eq!(payload["error"]["code"], "invalid_params");
}
