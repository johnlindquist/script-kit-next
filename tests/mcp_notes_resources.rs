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
