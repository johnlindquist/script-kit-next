use script_kit_gpui::mcp_protocol::{
    handle_request_with_runtime_context, JsonRpcRequest, McpRuntimeContext,
};
use serde_json::{json, Value};

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
    let response = handle_request_with_runtime_context(request, &[], &[], None, context);
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
fn notes_tools_are_listed_before_generic_kit_fallback() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("tools-list"),
        method: "tools/list".to_string(),
        params: json!({}),
    };
    let response = handle_request_with_runtime_context(request, &[], &[], None, None);
    let tools = response.result.unwrap()["tools"]
        .as_array()
        .unwrap()
        .clone();
    let names: Vec<_> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();

    let notes_create_idx = names
        .iter()
        .position(|name| *name == "kit/notes_create")
        .expect("kit/notes_create should be listed");
    let kit_show_idx = names
        .iter()
        .position(|name| *name == "kit/show")
        .expect("kit/show should be listed");
    assert!(
        notes_create_idx < kit_show_idx,
        "notes mutation tools must be listed before generic kit tools"
    );

    let result = call_tool(
        "kit/notes_create",
        json!({ "body": "route before generic kit fallback" }),
        None,
    );
    let payload = first_tool_text(&result);
    assert_eq!(result["isError"], true);
    assert_eq!(payload["error"]["code"], "missing_runtime");
    assert!(
        !payload["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("Unknown kit tool"),
        "kit/notes_create should not be swallowed by generic kit/* fallback"
    );
}

#[test]
fn malformed_notes_payload_returns_tool_level_invalid_params() {
    let result = call_tool("kit/notes_create", json!({ "body": 123 }), None);
    let payload = first_tool_text(&result);
    assert_eq!(result["isError"], true);
    assert_eq!(payload["error"]["code"], "invalid_params");
}

#[test]
fn notes_tool_scope_denial_returns_tool_error() {
    let context = McpRuntimeContext {
        token_scopes: vec!["mcp:read".to_string()],
        ..McpRuntimeContext::default()
    };
    let result = call_tool(
        "kit/notes_create",
        json!({ "body": "scope should deny this mutation" }),
        Some(&context),
    );
    let payload = first_tool_text(&result);
    assert_eq!(result["isError"], true);
    assert_eq!(payload["error"]["code"], "invalid_params");
    assert!(
        payload["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("notes:write"),
        "scope denial should name the missing notes:write scope"
    );
}
