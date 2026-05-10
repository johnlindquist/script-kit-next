// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_native_window_is_composition_only_read_only_lookup() {
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
            "pub const COMPUTER_GET_NATIVE_WINDOW_TOOL: &str = \"computer/get_native_window\";"
        ),
        "computer/get_native_window must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_GET_NATIVE_WINDOW_TOOL => handle_get_native_window(arguments, runtime),"
        ),
        "computer/get_native_window must route through the runtime-composition handler"
    );
    assert!(
        mcp_tools.contains("name: COMPUTER_GET_NATIVE_WINDOW_TOOL.to_string()"),
        "computer/get_native_window must be registered in the static tool catalog"
    );
    assert!(
        mcp_protocol.contains("tool_names.contains(&\"computer/get_native_window\")"),
        "tools/list tests must expect computer/get_native_window"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetNativeWindowArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\nstruct ComputerUseGetNativeWindowArgs"
        ),
        "computer/get_native_window args must reject unknown camelCase fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["native_window_id: u32,"],
        "computer/get_native_window args must expose exactly nativeWindowId"
    );

    let schema_body =
        extract_function_body(&mcp_tools, "fn computer_get_native_window_input_schema()");
    for needle in [
        "\"additionalProperties\": false",
        "\"nativeWindowId\"",
        "\"type\": \"integer\"",
        "\"minimum\": 0",
        "\"maximum\": 4_294_967_295u64",
        "\"required\": [\"nativeWindowId\"]",
    ] {
        assert!(
            schema_body.contains(needle),
            "computer/get_native_window schema missing {needle}"
        );
    }
    assert_eq!(
        extract_json_object_block(schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        1,
        "computer/get_native_window schema properties must contain exactly one field"
    );
    for needle in [
        "\"pid\"",
        "\"app\"",
        "\"bundleId\"",
        "\"includeHidden\"",
        "\"includeBackground\"",
        "\"focus\"",
        "\"activate\"",
        "\"launch\"",
        "\"quit\"",
        "\"hide\"",
        "\"move\"",
        "\"resize\"",
        "\"setBounds\"",
        "\"screenshot\"",
        "\"capture\"",
        "\"click\"",
        "\"press\"",
        "\"execute\"",
        "\"includeGlobalStatusItems\"",
    ] {
        assert!(
            !schema_body.contains(needle),
            "computer/get_native_window input must stay nativeWindowId-only; found {needle}"
        );
    }

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetNativeWindowResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "native_window_id: u32,",
            "app: Option<ComputerUseRunningAppInfo>,",
            "window: Option<ComputerUseAppWindowInfo>,",
            "warnings: Vec<String>,",
        ],
        "computer/get_native_window result must expose exactly one-window lookup fields"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_native_window(");
    for needle in [
        "ComputerUseGetNativeWindowArgs",
        "let Some(runtime) = runtime",
        "runtime.list_running_apps(ComputerUseListAppsRequest",
        "include_hidden: true",
        "include_background: true",
        "runtime.list_app_windows(ComputerUseListAppWindowsRequest { pid: app.pid })",
        "window.native_window_id == args.native_window_id",
        "source: \"nsWorkspaceRunningApplications+coreGraphicsWindowList\"",
        "scope: \"nativeWindowId\"",
        "\"found\"",
        "\"notFound\"",
        "\"partial\"",
        "windowListFailed for pid",
        "appNotFound for pid",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/get_native_window handler must contain {needle}"
        );
    }
    for needle in [
        "inspect_automation_window",
        "list_screens",
        "get_cached_menu_snapshot",
        "menu_executor",
        "AXUIElement",
        "AXUIElementPerformAction",
        "CGEvent",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "capture_targeted_screenshot",
        "screenshot",
        "focus",
        "activate",
        "launch",
        "quit",
        "hide",
        "move",
        "resize",
        "setBounds",
        "click",
        "press",
        "execute",
        "listMenuExtras",
        "status item",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/get_native_window handler must stay read-only runtime composition; found {needle}"
        );
    }

    for needle in [
        "get_native_window",
        "GetNativeWindow",
        "ComputerUseGetNativeWindowRequest",
        "ComputerUseGetNativeWindowSnapshot",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/get_native_window must not add a dedicated runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/get_native_window must not add a dedicated GPUI bridge surface; found {needle}"
        );
    }

    assert!(
        protocol.contains(
            "`computer/get_native_window` accepts a closed `{nativeWindowId:integer}` input"
        ),
        "protocol docs must describe the get_native_window input contract"
    );
    for needle in [
        "source:\"nsWorkspaceRunningApplications+coreGraphicsWindowList\"",
        "scope:\"nativeWindowId\"",
        "status:\"found\"|\"notFound\"|\"partial\"",
        "hidden/background apps included",
        "does not add a native bridge method",
        "focus or activate apps",
        "launch, quit, or hide apps",
        "move or resize windows",
        "capture screenshots",
        "inspect AX elements",
        "request permissions",
        "send input",
        "enumerate menu extras or status items",
        "expose action handles",
    ] {
        assert!(
            protocol.contains(needle),
            "protocol docs must pin get_native_window boundary: {needle}"
        );
    }
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
    extract_block_from_open_brace(source, open)
}

fn extract_braced_block<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("signature");
    let open = source[start..].find('{').expect("open brace") + start;
    extract_block_from_open_brace(source, open)
}

fn extract_block_from_open_brace(source: &str, open: usize) -> &str {
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

    panic!("braced block did not close")
}
