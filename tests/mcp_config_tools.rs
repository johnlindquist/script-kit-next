use script_kit_gpui::mcp_protocol::{
    handle_request_with_runtime_context, JsonRpcRequest, McpRuntimeContext,
};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

static CONFIG_TEST_LOCK: Mutex<()> = Mutex::new(());

fn with_temp_config<F: FnOnce(&std::path::Path)>(f: F) {
    let _lock = CONFIG_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let temp_dir = TempDir::new().expect("create temp config dir");
    let config_path = temp_dir.path().join("config.ts");
    let cli_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("scripts")
        .join("config-cli.ts");
    let previous_config_path = std::env::var_os("SCRIPT_KIT_CONFIG_PATH");
    let previous_cli_path = std::env::var_os("SCRIPT_KIT_CONFIG_CLI_PATH");
    std::env::set_var("SCRIPT_KIT_CONFIG_PATH", &config_path);
    std::env::set_var("SCRIPT_KIT_CONFIG_CLI_PATH", &cli_path);
    f(&config_path);
    if let Some(value) = previous_config_path {
        std::env::set_var("SCRIPT_KIT_CONFIG_PATH", value);
    } else {
        std::env::remove_var("SCRIPT_KIT_CONFIG_PATH");
    }
    if let Some(value) = previous_cli_path {
        std::env::set_var("SCRIPT_KIT_CONFIG_CLI_PATH", value);
    } else {
        std::env::remove_var("SCRIPT_KIT_CONFIG_CLI_PATH");
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

fn config_read_context() -> McpRuntimeContext {
    McpRuntimeContext {
        token_scopes: vec!["mcp:read".to_string(), "config:read".to_string()],
        ..McpRuntimeContext::default()
    }
}

fn config_write_context() -> McpRuntimeContext {
    McpRuntimeContext {
        token_scopes: vec![
            "mcp:read".to_string(),
            "config:read".to_string(),
            "config:write".to_string(),
        ],
        ..McpRuntimeContext::default()
    }
}

#[test]
fn config_tools_are_listed_before_generic_kit_fallback() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: json!("tools-list"),
        method: "tools/list".to_string(),
        params: json!({}),
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
        None,
    ));
    let tools = response.result.unwrap()["tools"]
        .as_array()
        .unwrap()
        .clone();
    let names: Vec<_> = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect();

    let config_get_idx = names
        .iter()
        .position(|name| *name == "kit/config_get")
        .expect("kit/config_get should be listed");
    let kit_show_idx = names
        .iter()
        .position(|name| *name == "kit/show")
        .expect("kit/show should be listed");
    assert!(
        config_get_idx < kit_show_idx,
        "config tools must be listed before generic kit tools"
    );
}

#[test]
fn config_write_tools_require_config_write_scope() {
    let context = config_read_context();
    let result = call_tool(
        "kit/config_set",
        json!({ "key": "editorFontSize", "value": 17 }),
        Some(&context),
    );
    let payload = first_tool_text(&result);
    assert_eq!(result["isError"], true);
    assert!(
        payload["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("config:write"),
        "scope denial should name the missing config:write scope"
    );
}

#[test]
fn config_set_get_validate_and_reset_round_trip_through_cli() {
    with_temp_config(|config_path| {
        let context = config_write_context();
        let set = call_tool(
            "kit/config_set",
            json!({ "key": "editorFontSize", "value": 18 }),
            Some(&context),
        );
        let set_payload = first_tool_text(&set);
        assert_eq!(set_payload["ok"], true);
        assert_eq!(set_payload["result"]["data"]["value"], 18);
        assert!(config_path.exists(), "config set should create config.ts");

        let get = call_tool(
            "kit/config_get",
            json!({ "key": "editorFontSize" }),
            Some(&context),
        );
        let get_payload = first_tool_text(&get);
        assert_eq!(get_payload["ok"], true);
        assert_eq!(get_payload["result"]["data"]["value"], 18);

        let validate = call_tool("kit/config_validate", json!({}), Some(&context));
        let validate_payload = first_tool_text(&validate);
        assert_eq!(validate_payload["ok"], true);
        assert_eq!(validate_payload["result"]["data"]["valid"], true);

        let reset_without_confirm = call_tool(
            "kit/config_reset",
            json!({ "key": "editorFontSize" }),
            Some(&context),
        );
        let reset_without_confirm_payload = first_tool_text(&reset_without_confirm);
        assert_eq!(reset_without_confirm["isError"], true);
        assert_eq!(
            reset_without_confirm_payload["error"]["code"],
            "confirm_required"
        );

        let reset = call_tool(
            "kit/config_reset",
            json!({ "key": "editorFontSize", "confirm": true }),
            Some(&context),
        );
        let reset_payload = first_tool_text(&reset);
        assert_eq!(reset_payload["ok"], true);
    });
}

#[test]
fn config_command_shortcut_tools_round_trip_and_confirm_removal() {
    with_temp_config(|_config_path| {
        let context = config_write_context();
        let set = call_tool(
            "kit/config_set_command_shortcut",
            json!({
                "commandId": "builtin/clipboard-history",
                "key": "v",
                "cmd": true,
                "shift": true
            }),
            Some(&context),
        );
        let set_payload = first_tool_text(&set);
        assert_eq!(set_payload["ok"], true);
        assert_eq!(
            set_payload["result"]["data"]["shortcut"]["key"], "KeyV",
            "shortcut key should be normalized through config-cli.ts"
        );

        let get = call_tool(
            "kit/config_get",
            json!({ "key": "commands" }),
            Some(&context),
        );
        let get_payload = first_tool_text(&get);
        assert_eq!(
            get_payload["result"]["data"]["value"]["builtin/clipboard-history"]["shortcut"]["key"],
            "KeyV"
        );

        let remove_without_confirm = call_tool(
            "kit/config_remove_command_shortcut",
            json!({ "commandId": "builtin/clipboard-history" }),
            Some(&context),
        );
        let remove_without_confirm_payload = first_tool_text(&remove_without_confirm);
        assert_eq!(remove_without_confirm["isError"], true);
        assert_eq!(
            remove_without_confirm_payload["error"]["code"],
            "confirm_required"
        );

        let remove = call_tool(
            "kit/config_remove_command_shortcut",
            json!({ "commandId": "builtin/clipboard-history", "confirm": true }),
            Some(&context),
        );
        let remove_payload = first_tool_text(&remove);
        assert_eq!(remove_payload["ok"], true);
        assert_eq!(remove_payload["result"]["data"]["removed"], true);
    });
}

#[test]
fn config_validate_change_reports_tool_error_for_invalid_value() {
    with_temp_config(|_config_path| {
        let context = config_read_context();
        let result = call_tool(
            "kit/config_validate_change",
            json!({ "key": "editorFontSize", "value": 1000 }),
            Some(&context),
        );
        let payload = first_tool_text(&result);
        assert_eq!(result["isError"], true);
        assert_eq!(payload["error"]["code"], "cli_failed");
        assert!(
            payload["error"]["message"]
                .as_str()
                .unwrap_or_default()
                .contains("Font size"),
            "invalid validation result should surface config-cli error text"
        );
    });
}
