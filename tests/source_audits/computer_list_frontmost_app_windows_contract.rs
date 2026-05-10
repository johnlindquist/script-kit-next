// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_frontmost_app_windows_is_composition_only_read_only_lookup() {
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
        mcp_tools.contains("pub const COMPUTER_LIST_FRONTMOST_APP_WINDOWS_TOOL: &str")
            && mcp_tools.contains("\"computer/list_frontmost_app_windows\""),
        "computer/list_frontmost_app_windows must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_LIST_FRONTMOST_APP_WINDOWS_TOOL => {\n            handle_list_frontmost_app_windows(arguments, runtime)\n        }"
        ),
        "computer/list_frontmost_app_windows must route through the runtime-composition handler"
    );
    assert!(
        mcp_tools.contains("name: COMPUTER_LIST_FRONTMOST_APP_WINDOWS_TOOL.to_string()"),
        "computer/list_frontmost_app_windows must be registered in the static tool catalog"
    );
    assert!(
        mcp_protocol.contains("tool_names.contains(&\"computer/list_frontmost_app_windows\")"),
        "tools/list tests must expect computer/list_frontmost_app_windows"
    );

    let args_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseListFrontmostAppWindowsArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(deny_unknown_fields)]\nstruct ComputerUseListFrontmostAppWindowsArgs"
        ),
        "computer/list_frontmost_app_windows args must reject unknown fields"
    );
    assert!(
        field_declarations(args_struct).is_empty(),
        "computer/list_frontmost_app_windows args must expose no fields"
    );

    let schema_body = extract_function_body(
        &mcp_tools,
        "fn computer_list_frontmost_app_windows_input_schema()",
    );
    for needle in ["\"additionalProperties\": false", "\"properties\": {}"] {
        assert!(
            schema_body.contains(needle),
            "computer/list_frontmost_app_windows schema missing {needle}"
        );
    }
    for needle in [
        "\"pid\"",
        "\"nativeWindowId\"",
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
            "computer/list_frontmost_app_windows input must stay empty; found {needle}"
        );
    }

    let result_struct = extract_struct_block(
        &mcp_tools,
        "struct ComputerUseListFrontmostAppWindowsResult",
    );
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "frontmost_pid: Option<i32>,",
            "app: Option<ComputerUseRunningAppInfo>,",
            "window_count: usize,",
            "windows: Vec<ComputerUseAppWindowInfo>,",
            "warnings: Vec<String>,",
        ],
        "computer/list_frontmost_app_windows result must expose exactly frontmost-window-list fields"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_list_frontmost_app_windows(");
    for needle in [
        "ComputerUseListFrontmostAppWindowsArgs",
        "let Some(runtime) = runtime",
        "runtime.list_running_apps(ComputerUseListAppsRequest",
        "include_hidden: true",
        "include_background: true",
        "apps_snapshot.frontmost_pid",
        "status: \"noFrontmostApp\"",
        "runtime.list_app_windows(ComputerUseListAppWindowsRequest { pid: frontmost_pid })",
        "let app = snapshot.app;",
        "source: \"nsWorkspaceRunningApplications+coreGraphicsWindowList\"",
        "scope: \"frontmostAppWindows\"",
        "\"appNotFound\"",
        "\"listed\"",
        "\"noWindows\"",
        "windows: snapshot.windows",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/list_frontmost_app_windows handler must contain {needle}"
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
            "computer/list_frontmost_app_windows handler must stay read-only runtime composition; found {needle}"
        );
    }

    for needle in [
        "list_frontmost_app_windows",
        "ListFrontmostAppWindows",
        "ComputerUseListFrontmostAppWindowsRequest",
        "ComputerUseListFrontmostAppWindowsSnapshot",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/list_frontmost_app_windows must not add a dedicated runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/list_frontmost_app_windows must not add a dedicated GPUI bridge surface; found {needle}"
        );
    }

    assert!(
        protocol.contains("`computer/list_frontmost_app_windows` accepts no arguments"),
        "protocol docs must describe the list_frontmost_app_windows input contract"
    );
    for needle in [
        "scope:\"frontmostAppWindows\"",
        "status:\"listed\"|\"noFrontmostApp\"|\"appNotFound\"|\"noWindows\"",
        "windowCount, windows, warnings",
        "same runtime-composition and read-only boundaries",
    ] {
        assert!(
            protocol.contains(needle),
            "protocol docs must pin list_frontmost_app_windows boundary: {needle}"
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
