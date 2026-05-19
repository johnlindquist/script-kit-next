use script_kit_gpui::mcp_protocol::{
    handle_request_with_runtime_context, JsonRpcRequest, McpRuntimeContext,
};
use script_kit_gpui::setup::SK_PATH_ENV;
use serde_json::{json, Value};
use std::fs;
use tempfile::TempDir;

static SK_PATH_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn with_temp_sk_path<F: FnOnce(&std::path::Path)>(f: F) {
    let _lock = SK_PATH_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let temp_dir = TempDir::new().expect("create temp dir");
    let kit_root = temp_dir.path().join("scriptkit-test");
    std::env::set_var(SK_PATH_ENV, kit_root.to_str().unwrap());
    f(&kit_root);
    std::env::remove_var(SK_PATH_ENV);
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

fn scripts_write_context() -> McpRuntimeContext {
    McpRuntimeContext {
        token_scopes: vec!["mcp:read".to_string(), "scripts:write".to_string()],
        ..McpRuntimeContext::default()
    }
}

fn scripts_all_context() -> McpRuntimeContext {
    McpRuntimeContext {
        token_scopes: vec![
            "mcp:read".to_string(),
            "scripts:write".to_string(),
            "scripts:run".to_string(),
        ],
        ..McpRuntimeContext::default()
    }
}

#[test]
fn scripts_tools_are_listed_before_generic_kit_fallback() {
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

    let scripts_create_idx = names
        .iter()
        .position(|name| *name == "kit/scripts_create")
        .expect("kit/scripts_create should be listed");
    let scripts_run_idx = names
        .iter()
        .position(|name| *name == "kit/scripts_run")
        .expect("kit/scripts_run should be listed");
    let kit_show_idx = names
        .iter()
        .position(|name| *name == "kit/show")
        .expect("kit/show should be listed");
    assert!(
        scripts_create_idx < kit_show_idx,
        "scripts mutation tools must be listed before generic kit tools"
    );
    assert!(
        scripts_run_idx < kit_show_idx,
        "scripts run tool must be listed before generic kit tools"
    );
}

#[test]
fn scripts_create_update_delete_round_trips_inside_main_scripts_dir() {
    with_temp_sk_path(|kit_root| {
        let context = scripts_all_context();
        let create = call_tool(
            "kit/scripts_create",
            json!({
                "name": "MCP Script Round Trip",
                "body": "console.log(JSON.stringify({ phase: \"created\" }));\n"
            }),
            Some(&context),
        );
        let create_payload = first_tool_text(&create);
        assert_eq!(create_payload["ok"], true);
        let script_path = kit_root
            .join("plugins/main/scripts")
            .join("mcp-script-round-trip.ts");
        assert_eq!(
            create_payload["result"]["path"].as_str(),
            Some(script_path.to_string_lossy().as_ref())
        );
        assert!(
            script_path.exists(),
            "script create should write the script file"
        );

        let update = call_tool(
            "kit/scripts_update",
            json!({
                "name": "mcp-script-round-trip",
                "body": "const stdin = await Bun.stdin.text();\nconsole.log(JSON.stringify({ phase: \"updated\", argv: Bun.argv.slice(2), env: process.env.MCP_SCRIPT_RUN_TEST, stdin }));\n"
            }),
            Some(&context),
        );
        let update_payload = first_tool_text(&update);
        assert_eq!(update_payload["ok"], true);
        let content = fs::read_to_string(&script_path).expect("read updated script");
        assert!(content.contains("updated"));

        let run = call_tool(
            "kit/scripts_run",
            json!({
                "name": "mcp-script-round-trip",
                "args": ["one", "two"],
                "env": { "MCP_SCRIPT_RUN_TEST": "scoped" },
                "stdin": "from stdin",
                "timeoutMs": 10_000
            }),
            Some(&context),
        );
        let run_payload = first_tool_text(&run);
        assert_eq!(run_payload["ok"], true);
        assert_eq!(run_payload["result"]["exitCode"], 0);
        assert_eq!(run_payload["result"]["timedOut"], false);
        let stdout = run_payload["result"]["stdout"].as_str().unwrap_or_default();
        assert!(stdout.contains("\"phase\":\"updated\""));
        assert!(stdout.contains("\"argv\":[\"one\",\"two\"]"));
        assert!(stdout.contains("\"env\":\"scoped\""));
        assert!(stdout.contains("\"stdin\":\"from stdin\""));

        let delete_without_confirm = call_tool(
            "kit/scripts_delete",
            json!({ "name": "mcp-script-round-trip" }),
            Some(&context),
        );
        let delete_without_confirm_payload = first_tool_text(&delete_without_confirm);
        assert_eq!(delete_without_confirm["isError"], true);
        assert_eq!(
            delete_without_confirm_payload["error"]["code"],
            "confirm_required"
        );
        assert!(
            script_path.exists(),
            "delete without confirm must not remove"
        );

        let delete = call_tool(
            "kit/scripts_delete",
            json!({ "name": "mcp-script-round-trip", "confirm": true }),
            Some(&context),
        );
        let delete_payload = first_tool_text(&delete);
        assert_eq!(delete_payload["ok"], true);
        assert!(
            !script_path.exists(),
            "confirmed delete should remove script"
        );
    });
}

#[test]
fn scripts_run_requires_scripts_run_scope() {
    with_temp_sk_path(|_kit_root| {
        let write_context = scripts_write_context();
        let create = call_tool(
            "kit/scripts_create",
            json!({ "name": "run denied", "body": "console.log('denied');\n" }),
            Some(&write_context),
        );
        assert_eq!(first_tool_text(&create)["ok"], true);

        let result = call_tool(
            "kit/scripts_run",
            json!({ "name": "run denied" }),
            Some(&write_context),
        );
        let payload = first_tool_text(&result);
        assert_eq!(result["isError"], true);
        assert!(
            payload["error"]["message"]
                .as_str()
                .unwrap_or_default()
                .contains("scripts:run"),
            "scope denial should name the missing scripts:run scope"
        );
    });
}

#[test]
fn scripts_tool_scope_denial_returns_tool_error() {
    let context = McpRuntimeContext {
        token_scopes: vec!["mcp:read".to_string()],
        ..McpRuntimeContext::default()
    };
    let result = call_tool(
        "kit/scripts_create",
        json!({ "name": "denied", "body": "import \"@scriptkit/sdk\";\n" }),
        Some(&context),
    );
    let payload = first_tool_text(&result);
    assert_eq!(result["isError"], true);
    assert!(
        payload["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("scripts:write"),
        "scope denial should name the missing scripts:write scope"
    );
}
