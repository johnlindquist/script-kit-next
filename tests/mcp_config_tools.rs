use script_kit_gpui::mcp_protocol::{
    handle_request_with_runtime_context, JsonRpcRequest, McpRuntimeContext,
};
use serde_json::{json, Value};
use tempfile::TempDir;

// Each scenario runs in its own process: changing HOME, SK_PATH or cwd in a
// parallel test harness could otherwise redirect another writer or audit log.
fn with_temp_config(test_name: &'static str, f: impl FnOnce(&std::path::Path)) {
    const CHILD: &str = "SCRIPT_KIT_INSTALLED_CONFIG_TEST";
    if std::env::var(CHILD).as_deref() == Ok(test_name) {
        let setup = script_kit_gpui::setup::ensure_kit_setup();
        assert!(setup.is_fresh_install);
        assert!(setup.kit_path.join("sdk/config-schema.ts").is_file());
        f(&setup.kit_path.join("config.ts"));
        std::fs::write("completed", test_name).unwrap();
        return;
    }
    let dir = TempDir::new().expect("scratch install");
    let sk = dir.path().join("personal kit");
    let cwd = dir.path().join("outside-checkout");
    std::fs::create_dir_all(cwd.join("scripts")).unwrap();
    std::fs::write(
        cwd.join("scripts/config-cli.ts"),
        "throw Error('wrong helper')",
    )
    .unwrap();
    std::fs::write(cwd.join("bunfig.toml"), r#"preload = ["./trap.ts"]"#).unwrap();
    std::fs::write(cwd.join("trap.ts"), "throw Error('wrong project')").unwrap();
    let bun =
        script_kit_gpui::setup::resolve_bun().expect("Bun required for the installed-tool fixture");
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name, "--nocapture"])
        .current_dir(&cwd)
        .env(CHILD, test_name)
        .env("HOME", dir.path())
        .env("SK_PATH", &sk)
        .env("PATH", bun.parent().unwrap())
        .env_remove("SCRIPT_KIT_CONFIG_CLI_PATH")
        .env_remove("SCRIPT_KIT_CONFIG_PATH")
        .env("NODE_OPTIONS", "--require ./trap.ts")
        .env("BUN_OPTIONS", "--preload ./trap.ts")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(cwd.join("completed")).unwrap(),
        test_name
    );
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
    with_temp_config(
        "config_tools_are_listed_before_generic_kit_fallback",
        |_| {
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
        },
    );
}

#[test]
fn config_write_tools_require_config_write_scope() {
    with_temp_config("config_write_tools_require_config_write_scope", |_| {
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
    });
}

#[test]
fn config_set_get_validate_and_reset_round_trip_through_cli() {
    with_temp_config(
        "config_set_get_validate_and_reset_round_trip_through_cli",
        |config_path| {
            let context = config_write_context();
            let set = call_tool(
                "kit/config_set",
                json!({ "key": "editorFontSize", "value": 18 }),
                Some(&context),
            );
            let set_payload = first_tool_text(&set);
            assert_eq!(set_payload["ok"], true);
            assert_eq!(set_payload["result"]["data"]["value"], 18);
            assert!(
                config_path.exists(),
                "set must preserve the first-run config"
            );

            let get = call_tool(
                "kit/config_get",
                json!({ "key": "editorFontSize" }),
                Some(&context),
            );
            let get_payload = first_tool_text(&get);
            assert_eq!(get_payload["ok"], true);
            assert_eq!(get_payload["result"]["data"]["value"], 18);

            let rev = &get_payload["result"]["data"]["revision"];
            let before = std::fs::read_to_string(config_path).unwrap() + "// changed by hand\n";
            std::fs::write(config_path, &before).unwrap();
            for args in [
                json!({"key":"editorFontSize","value":22,"expectedRevision":rev}),
                json!({"key":"editorFontSize","value":22,"expectedRevision":null}),
                json!({"key":"dictation.startInKit","value":"false"}),
                json!({"key":"imaginary.setting","value":true}),
                json!({"key":"bun_path","value":"/untrusted/bun"}),
            ] {
                let denied = call_tool("kit/config_set", args, Some(&context));
                assert_eq!(denied["isError"], true);
                assert_eq!(std::fs::read_to_string(config_path).unwrap(), before);
            }
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
            let dynamic = "throw Error('must not run'); export default { editorFontSize: 16 }";
            std::fs::write(config_path, dynamic).unwrap();
            for (name, args) in [
                ("kit/config_validate", json!({})),
                ("kit/config_set", json!({"key":"editorFontSize","value":18})),
            ] {
                assert_eq!(call_tool(name, args, Some(&context))["isError"], true);
                assert_eq!(std::fs::read_to_string(config_path).unwrap(), dynamic);
            }
            let schema = config_path.parent().unwrap().join("sdk/config-schema.ts");
            std::fs::write(schema, "throw new Error('mixed install')").unwrap();
            let failed = call_tool("kit/config_get", json!({}), Some(&context));
            assert_eq!(first_tool_text(&failed)["error"]["code"], "cli_unavailable");
        },
    );
}

#[test]
fn config_command_shortcut_tools_round_trip_and_confirm_removal() {
    with_temp_config(
        "config_command_shortcut_tools_round_trip_and_confirm_removal",
        |_config_path| {
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
                get_payload["result"]["data"]["value"]["builtin/clipboard-history"]["shortcut"]
                    ["key"],
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
        },
    );
}

#[test]
fn config_validate_change_reports_tool_error_for_invalid_value() {
    with_temp_config(
        "config_validate_change_reports_tool_error_for_invalid_value",
        |_config_path| {
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
        },
    );
}
