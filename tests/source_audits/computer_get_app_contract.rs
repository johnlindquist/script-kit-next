// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_app_is_pid_only_list_apps_lookup() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");
    let mcp_protocol =
        std::fs::read_to_string("src/mcp_protocol/mod.rs").expect("read mcp_protocol/mod.rs");

    assert!(
        mcp_tools.contains("pub const COMPUTER_GET_APP_TOOL: &str = \"computer/get_app\";"),
        "computer/get_app must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains("COMPUTER_GET_APP_TOOL => handle_get_app(arguments, runtime),"),
        "computer/get_app must route through its runtime-bridged handler"
    );
    assert!(
        mcp_tools.contains("computer_get_app_input_schema"),
        "computer/get_app must expose a dedicated input schema"
    );
    assert!(
        mcp_tools.contains("name: COMPUTER_GET_APP_TOOL.to_string()"),
        "computer/get_app must be registered in the static tool catalog"
    );
    assert!(
        mcp_protocol.contains("tool_names.contains(&\"computer/get_app\")"),
        "tools/list tests must expect computer/get_app"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetAppArgs");
    assert!(
        mcp_tools.contains("#[serde(deny_unknown_fields)]\nstruct ComputerUseGetAppArgs"),
        "computer/get_app args must reject unknown fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["pid: i32,"],
        "computer/get_app args must expose exactly one pid field"
    );

    let input_schema_body = extract_function_body(&mcp_tools, "fn computer_get_app_input_schema()");
    assert!(
        input_schema_body.contains("\"additionalProperties\": false"),
        "computer/get_app must reject unknown input fields"
    );
    assert!(
        input_schema_body.contains("\"pid\": { \"type\": \"integer\" }"),
        "computer/get_app pid must be an integer schema property"
    );
    assert!(
        input_schema_body.contains("\"required\": [\"pid\"]"),
        "computer/get_app must require pid"
    );
    assert_eq!(
        extract_json_object_block(input_schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        1,
        "computer/get_app schema properties must contain exactly one field"
    );
    for needle in [
        "\"target\"",
        "\"bundleId\"",
        "\"name\"",
        "\"focus\"",
        "\"activate\"",
        "\"launch\"",
        "\"quit\"",
        "\"hide\"",
        "\"click\"",
        "\"move\"",
        "\"resize\"",
        "\"screenshot\"",
        "\"includeWindows\"",
        "\"includeElements\"",
    ] {
        assert!(
            !input_schema_body.contains(needle),
            "computer/get_app input must stay PID-only; found {}",
            needle
        );
    }

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetAppResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "app: Option<ComputerUseRunningAppInfo>,",
            "warnings: Vec<String>,",
        ],
        "computer/get_app result must expose exactly schemaVersion/source/scope/status/app/warnings"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_app(");
    assert!(
        handler_body.contains("ComputerUseGetAppArgs"),
        "handler must parse the dedicated PID-only args"
    );
    assert!(
        handler_body.contains("let Some(runtime) = runtime"),
        "handler must require the live runtime bridge"
    );
    assert!(
        handler_body.contains("ComputerUseListAppsRequest {\n        include_hidden: true,\n        include_background: true,\n    }"),
        "handler must reuse the list-apps runtime inventory with hidden/background apps included"
    );
    assert!(
        handler_body.contains("runtime.list_running_apps(request)"),
        "handler must delegate to the existing list_running_apps bridge"
    );
    assert!(
        handler_body.contains(".find(|app| app.pid == args.pid)"),
        "handler must filter the runtime app inventory by PID"
    );
    assert!(
        handler_body.contains("source: \"nsWorkspaceRunningApplications\""),
        "handler must identify NSWorkspace runningApplications as the source"
    );
    assert!(
        handler_body.contains("scope: \"runningAppPid\""),
        "handler must identify PID scope"
    );
    assert!(
        handler_body.contains("\"found\"") && handler_body.contains("\"notFound\""),
        "handler must expose found/notFound statuses"
    );
    for needle in [
        "ComputerUseRuntimeBridge::list_app",
        "ComputerUseGetAppRequest",
        "NSWorkspace",
        "runningApplications",
        "process_manager",
        "read_scripts",
        "load_scriptlets",
        "Command::new(\"open\")",
        "hide]",
        "hideOtherApplications",
        "unhide",
        "focus",
        "focus_window",
        "set_automation_focus",
        "activateWithOptions",
        "activateIgnoringOtherApps",
        "launchApplication",
        "openApplicationAtURL",
        "terminate",
        "forceTerminate",
        "CGEvent",
        "AXUIElementPerformAction",
        "AXPress",
        "CGWindowListCopyWindowInfo",
        "CGWindowListCreateImage",
        "capture_targeted_rgba_image",
        "inspect_automation_window",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/get_app handler must stay a read-only list_apps lookup; found {}",
            needle
        );
    }

    assert!(
        !mcp_tools.contains("fn get_app_on_gpui_thread"),
        "computer/get_app must not add a new native GPUI bridge helper"
    );
    assert!(
        !mcp_tools.contains("ComputerUseRuntimeBridge::get_app"),
        "computer/get_app must not add a new runtime bridge method"
    );
    assert!(
        protocol.contains("computer/get_app"),
        "protocol docs must mention computer/get_app"
    );
    assert!(
        protocol.contains("does not add a new native bridge method"),
        "protocol docs must state get_app reuses the list_apps bridge"
    );
    assert!(
        protocol.contains("does not add a new native bridge method, inspect windows, focus, activate, launch, quit, hide, send input, or expose action handles"),
        "protocol docs must state the no-action/no-new-native-boundary contract"
    );
}

fn extract_struct_block<'a>(source: &'a str, signature: &str) -> &'a str {
    extract_braced_block(source, signature)
}

fn extract_function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    extract_braced_block(source, signature)
}

fn field_declarations(block: &str) -> Vec<&str> {
    block
        .lines()
        .map(str::trim)
        .map(|line| line.strip_prefix("pub ").unwrap_or(line))
        .filter(|line| line.ends_with(',') && line.contains(':') && !line.starts_with("#["))
        .collect()
}

fn extract_json_object_block<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source.find(marker).expect("json object marker");
    let open = source[start..].find('{').expect("json object open brace") + start;
    let mut depth = 0usize;

    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return &source[open..=open + offset];
                }
            }
            _ => {}
        }
    }

    panic!("json object block for {} did not close", marker)
}

fn extract_braced_block<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("signature");
    let open = source[start..].find('{').expect("open brace") + start;
    let mut depth = 0usize;

    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return &source[open..=open + offset];
                }
            }
            _ => {}
        }
    }

    panic!("braced block for {} did not close", signature)
}
