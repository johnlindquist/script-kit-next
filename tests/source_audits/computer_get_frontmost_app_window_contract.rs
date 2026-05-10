// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_frontmost_app_window_is_composition_only_read_only_lookup() {
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
        mcp_tools.contains("pub const COMPUTER_GET_FRONTMOST_APP_WINDOW_TOOL: &str")
            && mcp_tools.contains("\"computer/get_frontmost_app_window\""),
        "computer/get_frontmost_app_window must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_GET_FRONTMOST_APP_WINDOW_TOOL => {\n            handle_get_frontmost_app_window(arguments, runtime)\n        }"
        ),
        "computer/get_frontmost_app_window must route through the runtime-composition handler"
    );
    assert!(
        mcp_tools.contains("name: COMPUTER_GET_FRONTMOST_APP_WINDOW_TOOL.to_string()"),
        "computer/get_frontmost_app_window must be registered in the static tool catalog"
    );
    assert!(
        mcp_protocol.contains("tool_names.contains(&\"computer/get_frontmost_app_window\")"),
        "tools/list tests must expect computer/get_frontmost_app_window"
    );

    let args_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseGetFrontmostAppWindowArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\nstruct ComputerUseGetFrontmostAppWindowArgs"
        ),
        "computer/get_frontmost_app_window args must reject unknown fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["native_window_id: u32,"],
        "computer/get_frontmost_app_window args must expose exactly native_window_id"
    );

    let schema_body = extract_function_body(
        &mcp_tools,
        "fn computer_get_frontmost_app_window_input_schema()",
    );
    for needle in [
        "\"additionalProperties\": false",
        "\"nativeWindowId\"",
        "\"minimum\": 0",
        "\"maximum\": u32::MAX as u64",
        "\"required\": [\"nativeWindowId\"]",
    ] {
        assert!(
            schema_body.contains(needle),
            "computer/get_frontmost_app_window schema missing {needle}"
        );
    }
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
        "\"includeGlobalStatusItems\"",
    ] {
        assert!(
            !schema_body.contains(needle),
            "computer/get_frontmost_app_window input must stay scoped to nativeWindowId; found {needle}"
        );
    }

    let result_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseGetFrontmostAppWindowResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "native_window_id: u32,",
            "frontmost_pid: Option<i32>,",
            "app: Option<ComputerUseRunningAppInfo>,",
            "window: Option<ComputerUseAppWindowInfo>,",
            "window_count: usize,",
            "warnings: Vec<String>,",
        ],
        "computer/get_frontmost_app_window result must expose exactly frontmost-window lookup fields"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_frontmost_app_window(");
    for needle in [
        "ComputerUseGetFrontmostAppWindowArgs",
        "let Some(runtime) = runtime",
        "runtime.list_running_apps(ComputerUseListAppsRequest",
        "include_hidden: true",
        "include_background: true",
        "apps_snapshot.frontmost_pid",
        "status: \"noFrontmostApp\"",
        "runtime.list_app_windows(ComputerUseListAppWindowsRequest { pid: frontmost_pid })",
        "let app = snapshot.app;",
        "let window_count = snapshot.windows.len();",
        ".find(|window| window.native_window_id == args.native_window_id)",
        "source: \"nsWorkspaceRunningApplications+coreGraphicsWindowList\"",
        "scope: \"frontmostAppNativeWindowId\"",
        "\"appNotFound\"",
        "\"noWindows\"",
        "\"found\"",
        "\"windowNotFound\"",
        "native_window_id: args.native_window_id",
        "window,",
        "warnings: snapshot.warnings",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/get_frontmost_app_window handler must contain {needle}"
        );
    }
    assert!(
        !handler_body.contains(".or(app_from_list)"),
        "appNotFound must be driven by list_app_windows snapshot.app, not masked by list_running_apps fallback metadata"
    );
    for needle in [
        "inspect_automation_window",
        "get_cached_menu_snapshot",
        "get_last_real_app",
        "crate::windows::",
        "crate::tray::",
        "list_screens",
        "computer_use_permission_statuses",
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
            "computer/get_frontmost_app_window handler must stay read-only runtime composition; found {needle}"
        );
    }

    for needle in [
        "get_frontmost_app_window",
        "GetFrontmostAppWindow",
        "ComputerUseGetFrontmostAppWindowRequest",
        "ComputerUseGetFrontmostAppWindowSnapshot",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/get_frontmost_app_window must not add a dedicated runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/get_frontmost_app_window must not add a dedicated GPUI bridge surface; found {needle}"
        );
    }

    assert!(
        protocol.contains(
            "`computer/get_frontmost_app_window` accepts a closed `{nativeWindowId:integer}` input"
        ),
        "protocol docs must describe the get_frontmost_app_window input contract"
    );
    for needle in [
        "scope:\"frontmostAppNativeWindowId\"",
        "status:\"found\"|\"noFrontmostApp\"|\"appNotFound\"|\"noWindows\"|\"windowNotFound\"",
        "nativeWindowId, frontmostPid, app, window, windowCount, warnings",
        "does not add a native bridge method",
        "expose action handles",
    ] {
        assert!(
            protocol.contains(needle),
            "protocol docs must pin get_frontmost_app_window boundary: {needle}"
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
        .filter(|line| {
            line.ends_with(',')
                && !line.starts_with("#[")
                && !line.starts_with("//")
                && line.contains(':')
        })
        .collect()
}

fn extract_braced_block<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("signature exists");
    let brace_start = source[start..].find('{').expect("opening brace") + start;
    let mut depth = 0i32;
    for (offset, ch) in source[brace_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[brace_start..=brace_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated block for {signature}");
}
