// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_app_window_by_bundle_id_is_exact_bundle_read_only_composition() {
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
            "pub const COMPUTER_GET_APP_WINDOW_BY_BUNDLE_ID_TOOL: &str = \"computer/get_app_window_by_bundle_id\";"
        ),
        "computer/get_app_window_by_bundle_id must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_GET_APP_WINDOW_BY_BUNDLE_ID_TOOL => {\n            handle_get_app_window_by_bundle_id(arguments, runtime)\n        }"
        ),
        "computer/get_app_window_by_bundle_id must route through the bundle-id app-window handler"
    );
    assert!(
        mcp_tools.contains("name: COMPUTER_GET_APP_WINDOW_BY_BUNDLE_ID_TOOL.to_string()"),
        "computer/get_app_window_by_bundle_id must be registered in the static tool catalog"
    );
    assert!(
        mcp_protocol.contains("tool_names.contains(&\"computer/get_app_window_by_bundle_id\")"),
        "tools/list tests must expect computer/get_app_window_by_bundle_id"
    );

    let args_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseGetAppWindowByBundleIdArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\nstruct ComputerUseGetAppWindowByBundleIdArgs"
        ),
        "computer/get_app_window_by_bundle_id args must reject unknown camelCase fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["bundle_id: String,", "native_window_id: u32,"],
        "computer/get_app_window_by_bundle_id args must expose exactly bundleId and nativeWindowId"
    );

    let schema_body = extract_function_body(
        &mcp_tools,
        "fn computer_get_app_window_by_bundle_id_input_schema()",
    );
    for needle in [
        "\"additionalProperties\": false",
        "\"bundleId\"",
        "\"type\": \"string\"",
        "\"minLength\": 1",
        "\"nativeWindowId\"",
        "\"type\": \"integer\"",
        "\"minimum\": 0",
        "\"maximum\": 4_294_967_295u64",
        "\"required\": [\"bundleId\", \"nativeWindowId\"]",
    ] {
        assert!(
            schema_body.contains(needle),
            "computer/get_app_window_by_bundle_id schema missing {needle}"
        );
    }
    assert_eq!(
        extract_json_object_block(schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        2,
        "computer/get_app_window_by_bundle_id schema properties must contain exactly two fields"
    );
    for needle in [
        "\"pid\"",
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
        "\"AXPress\"",
    ] {
        assert!(
            !schema_body.contains(needle),
            "computer/get_app_window_by_bundle_id input must stay bundleId/nativeWindowId-only; found {needle}"
        );
    }

    let result_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseGetAppWindowByBundleIdResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "bundle_id: String,",
            "native_window_id: u32,",
            "app_count: usize,",
            "app: Option<ComputerUseRunningAppInfo>,",
            "window: Option<ComputerUseAppWindowInfo>,",
            "warnings: Vec<String>,",
        ],
        "computer/get_app_window_by_bundle_id result must expose exactly the ownership lookup fields"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_app_window_by_bundle_id(");
    for needle in [
        "ComputerUseGetAppWindowByBundleIdArgs",
        "if args.bundle_id.is_empty()",
        "let Some(runtime) = runtime",
        "runtime.list_running_apps(ComputerUseListAppsRequest",
        "include_hidden: true",
        "include_background: true",
        "app.bundle_id.as_deref() == Some(args.bundle_id.as_str())",
        "runtime.list_app_windows(ComputerUseListAppWindowsRequest { pid: app.pid })",
        "snapshot_app.bundle_id.as_deref() != Some(args.bundle_id.as_str())",
        "bundleIdChanged",
        "window.native_window_id == args.native_window_id",
        "source: \"nsWorkspaceRunningApplications+coreGraphicsWindowList\"",
        "scope: \"runningAppBundleIdNativeWindowId\"",
        "\"found\"",
        "\"appNotFound\"",
        "\"windowNotFound\"",
        "\"partial\"",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/get_app_window_by_bundle_id handler must contain {needle}"
        );
    }
    let filter_position = handler_body
        .find("filter(|app| app.bundle_id.as_deref() == Some(args.bundle_id.as_str()))")
        .expect("exact bundle filter");
    let window_query_position = handler_body
        .find("runtime.list_app_windows")
        .expect("window query");
    assert!(
        filter_position < window_query_position,
        "handler must filter exact bundle ids before querying windows"
    );
    for needle in [
        "inspect_automation_window",
        "get_cached_menu_snapshot",
        "get_last_real_app",
        "list_screens",
        "computer_use_permission_statuses",
        "menu_executor",
        "AXUIElement",
        "AXUIElementPerformAction",
        "CGEvent",
        "NSWorkspace",
        "runningApplications",
        "AppKit",
        "objc::",
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
            "computer/get_app_window_by_bundle_id handler must stay an ownership lookup; found {needle}"
        );
    }

    for needle in [
        "get_app_window_by_bundle_id",
        "GetAppWindowByBundleId",
        "ComputerUseGetAppWindowByBundleIdRequest",
        "ComputerUseGetAppWindowByBundleIdSnapshot",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/get_app_window_by_bundle_id must not add a dedicated runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/get_app_window_by_bundle_id must not add a dedicated GPUI bridge surface; found {needle}"
        );
    }
    assert!(
        runtime.contains("fn list_running_apps(") && runtime.contains("fn list_app_windows("),
        "runtime bridge must continue to provide the composed running-app and app-window primitives"
    );

    for needle in [
        "`computer/get_app_window_by_bundle_id`",
        "closed `{bundleId:string,nativeWindowId:integer}` input",
        "scope:\"runningAppBundleIdNativeWindowId\"",
        "status:\"found\"|\"appNotFound\"|\"windowNotFound\"|\"partial\"",
        "filters by exact `bundle_id` before querying windows",
        "revalidates returned app metadata still has the requested bundle id",
        "top-level app-list failure as a tool error",
        "per-app window-list, stale-bundle, or disappearing-app failures as observation warnings",
        "moment-in-time CoreGraphics id",
        "does not add a native bridge method",
        "focus or activate apps",
        "capture screenshots",
        "expose action handles",
    ] {
        assert!(
            protocol.contains(needle),
            "protocol docs must pin get_app_window_by_bundle_id boundary: {needle}"
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
