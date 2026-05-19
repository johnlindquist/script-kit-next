use script_kit_gpui::mcp_protocol::{error_codes, handle_request, JsonRpcRequest};
use serde_json::json;

#[test]
fn notes_resource_is_listed() {
    let resources = script_kit_gpui::mcp_resources::get_resource_definitions();
    assert!(
        resources
            .iter()
            .any(|resource| resource.uri == "kit://notes"),
        "resources/list should expose kit://notes"
    );
    assert!(
        resources
            .iter()
            .any(|resource| resource.uri == "kit://audit"),
        "resources/list should expose kit://audit so MCP mutations are auditable"
    );
}

#[test]
fn notes_resource_rejects_invalid_note_id() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("bad-note-id"),
        method: "resources/read".to_string(),
        params: json!({ "uri": "kit://notes/not-a-uuid" }),
    };

    let response = handle_request(request);
    let error = response
        .error
        .expect("invalid note URI should be a JSON-RPC error");
    assert_eq!(error.code, error_codes::INVALID_PARAMS);
}

#[test]
fn audit_resource_returns_bounded_json_envelope() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("audit-read"),
        method: "resources/read".to_string(),
        params: json!({ "uri": "kit://audit?limit=2&traceId=missing-test-trace" }),
    };

    let response = handle_request(request);
    let result = response
        .result
        .expect("kit://audit should return a resource envelope even when the log is empty");
    let text = result["contents"][0]["text"]
        .as_str()
        .expect("audit resource should be text JSON");
    let payload: serde_json::Value =
        serde_json::from_str(text).expect("audit resource text should be valid JSON");
    assert_eq!(payload["schemaVersion"], 1);
    assert_eq!(
        payload["uri"],
        "kit://audit?limit=2&traceId=missing-test-trace"
    );
    assert!(payload["events"].is_array());
    assert!(
        payload["count"].as_u64().unwrap_or_default() <= 2,
        "audit resource must honor the requested limit"
    );
}
