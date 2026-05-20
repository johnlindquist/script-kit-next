use script_kit_gpui::mcp_protocol::{
    handle_request_with_runtime_context, JsonRpcRequest, JsonRpcResponse, McpRuntimeContext,
};
use serde_json::{json, Value};

fn handle_request(request: JsonRpcRequest, context: Option<&McpRuntimeContext>) -> JsonRpcResponse {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("build test runtime");
    runtime.block_on(handle_request_with_runtime_context(
        request,
        &[],
        &[],
        None,
        context,
    ))
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
    let response = handle_request(request, context);
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
    let response = handle_request(request, None);
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

    let create_schema = tools
        .iter()
        .find(|tool| tool["name"] == "kit/notes_create")
        .and_then(|tool| tool["inputSchema"].as_object())
        .expect("kit/notes_create should expose an input schema");
    let any_of = create_schema
        .get("anyOf")
        .and_then(|value| value.as_array())
        .expect("notes_create schema should accept body or content");
    assert!(
        any_of
            .iter()
            .any(|entry| entry["required"] == json!(["body"])),
        "notes_create schema should require body in one branch"
    );
    assert!(
        any_of
            .iter()
            .any(|entry| entry["required"] == json!(["content"])),
        "notes_create schema should require content in one branch"
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
fn notes_create_accepts_content_alias_before_runtime_dispatch() {
    let result = call_tool("kit/notes_create", json!({ "content": "alias body" }), None);
    let payload = first_tool_text(&result);
    assert_eq!(result["isError"], true);
    assert_eq!(payload["error"]["code"], "missing_runtime");
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
    assert_eq!(payload["action"], "kit/notes_create");
    assert_eq!(payload["error"]["code"], "scope_denied");
    assert!(
        payload["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("notes:write"),
        "scope denial should name the missing notes:write scope"
    );
}
