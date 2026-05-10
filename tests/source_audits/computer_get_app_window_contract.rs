// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_app_window_is_read_only_window_lookup() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");
    let mcp_protocol =
        std::fs::read_to_string("src/mcp_protocol/mod.rs").expect("read mcp_protocol/mod.rs");

    assert!(
        mcp_tools.contains(
            "pub const COMPUTER_GET_APP_WINDOW_TOOL: &str = \"computer/get_app_window\";"
        ),
        "computer/get_app_window must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools
            .contains("COMPUTER_GET_APP_WINDOW_TOOL => handle_get_app_window(arguments, runtime),"),
        "computer/get_app_window must route through its runtime-bridged handler"
    );
    assert!(
        mcp_tools.contains("computer_get_app_window_input_schema"),
        "computer/get_app_window must expose a dedicated input schema"
    );
    assert!(
        mcp_tools.contains("name: COMPUTER_GET_APP_WINDOW_TOOL.to_string()"),
        "computer/get_app_window must be registered in the static tool catalog"
    );
    assert!(
        mcp_protocol.contains("tool_names.contains(&\"computer/get_app_window\")"),
        "tools/list tests must expect computer/get_app_window"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetAppWindowArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\nstruct ComputerUseGetAppWindowArgs"
        ),
        "computer/get_app_window args must reject unknown camelCase fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["pid: i32,", "native_window_id: u32,"],
        "computer/get_app_window args must expose exactly pid and nativeWindowId"
    );

    let input_schema_body =
        extract_function_body(&mcp_tools, "fn computer_get_app_window_input_schema()");
    assert!(
        input_schema_body.contains("\"additionalProperties\": false"),
        "computer/get_app_window must reject unknown input fields"
    );
    assert!(
        input_schema_body.contains("\"pid\": { \"type\": \"integer\" }"),
        "computer/get_app_window pid must be an integer schema property"
    );
    assert!(
        input_schema_body.contains(
            "\"nativeWindowId\": { \"type\": \"integer\", \"minimum\": 0, \"maximum\": 4294967295u64 }"
        ),
        "computer/get_app_window nativeWindowId must match the non-negative u32 schema boundary"
    );
    assert!(
        input_schema_body.contains("\"required\": [\"pid\", \"nativeWindowId\"]"),
        "computer/get_app_window must require pid and nativeWindowId"
    );
    assert_eq!(
        extract_json_object_block(input_schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        2,
        "computer/get_app_window schema properties must contain exactly two fields"
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
        "\"setBounds\"",
        "\"screenshot\"",
        "\"includeElements\"",
        "\"action\"",
    ] {
        assert!(
            !input_schema_body.contains(needle),
            "computer/get_app_window input must stay pid/nativeWindowId only; found {}",
            needle
        );
    }

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetAppWindowResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "app: Option<ComputerUseRunningAppInfo>,",
            "window: Option<ComputerUseAppWindowInfo>,",
            "warnings: Vec<String>,",
        ],
        "computer/get_app_window result must expose exactly schemaVersion/source/scope/status/app/window/warnings"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_app_window(");
    assert!(
        handler_body.contains("ComputerUseGetAppWindowArgs"),
        "handler must parse the dedicated two-field args"
    );
    assert!(
        handler_body.contains("let Some(runtime) = runtime"),
        "handler must require the live runtime bridge"
    );
    assert!(
        handler_body.contains("ComputerUseListAppWindowsRequest { pid: args.pid }"),
        "handler must pass only pid into the app-window runtime request"
    );
    assert!(
        handler_body.contains("runtime.list_app_windows(request)"),
        "handler must reuse the existing list_app_windows runtime bridge"
    );
    assert!(
        handler_body.contains("let window = if app.is_some()"),
        "handler must not search windows when the app is absent"
    );
    assert!(
        handler_body.contains(".find(|window| window.native_window_id == args.native_window_id)"),
        "handler must filter the app-window inventory by nativeWindowId"
    );
    assert!(
        handler_body.contains("source: \"coreGraphicsWindowList\""),
        "handler must identify CoreGraphics window metadata as the source"
    );
    assert!(
        handler_body.contains("scope: \"runningAppPidNativeWindowId\""),
        "handler must identify PID plus native-window-id scope"
    );
    for status in ["\"found\"", "\"windowNotFound\"", "\"appNotFound\""] {
        assert!(
            handler_body.contains(status),
            "handler must expose {} status",
            status
        );
    }
    for needle in [
        "ComputerUseRuntimeBridge::get_app_window",
        "ComputerUseGetAppWindowRequest",
        "list_running_apps",
        "inspect_automation_window",
        "build_automation_inspect_snapshot",
        "capture_targeted_rgba_image",
        "collect_visible_elements",
        "CGWindowListCopyWindowInfo",
        "AXUIElement",
        "CGEvent",
        "focus_window",
        "move_window",
        "resize_window",
        "set_automation_focus",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "Command::new(\"open\")",
        "activateWithOptions",
        "activateIgnoringOtherApps",
        "launchApplication",
        "openApplicationAtURL",
        "terminate",
        "forceTerminate",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/get_app_window handler must stay a read-only list_app_windows lookup; found {}",
            needle
        );
    }

    for (label, source) in [
        ("runtime bridge", runtime.as_str()),
        ("GPUI runtime bridge", bridge.as_str()),
    ] {
        for needle in [
            "GetAppWindow",
            "fn get_app_window",
            "get_app_window_on_gpui_thread",
            "ComputerUseGetAppWindowRequest",
            "ComputerUseGetAppWindowSnapshot",
        ] {
            assert!(
                !source.contains(needle),
                "computer/get_app_window must not add a dedicated {label} surface; found {needle}"
            );
        }
    }

    assert!(
        runtime.contains("fn list_app_windows("),
        "runtime bridge must continue to provide list_app_windows for this lookup"
    );
    assert!(
        bridge.contains("ListAppWindows"),
        "GPUI bridge must continue to carry app-window requests to the GPUI/native side"
    );
    assert!(
        protocol.contains("computer/get_app_window"),
        "protocol docs must mention computer/get_app_window"
    );
    assert!(
        protocol.contains("status:\"found\"|\"appNotFound\"|\"windowNotFound\""),
        "protocol docs must document get_app_window status variants"
    );
    assert!(
        protocol.contains("does not add a new native bridge method, focus, activate, launch, quit, move, resize, capture screenshots, inspect AX elements, request permissions, send input, or expose action handles"),
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
