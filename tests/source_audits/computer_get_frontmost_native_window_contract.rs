// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_frontmost_native_window_is_composition_only_read_only_lookup() {
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
        mcp_tools.contains("pub const COMPUTER_GET_FRONTMOST_NATIVE_WINDOW_TOOL: &str")
            && mcp_tools.contains("\"computer/get_frontmost_native_window\""),
        "computer/get_frontmost_native_window must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_GET_FRONTMOST_NATIVE_WINDOW_TOOL => {\n            handle_get_frontmost_native_window(arguments, runtime)\n        }"
        ),
        "computer/get_frontmost_native_window must route through the runtime-composition handler"
    );
    assert!(
        mcp_tools.contains("name: COMPUTER_GET_FRONTMOST_NATIVE_WINDOW_TOOL.to_string()"),
        "computer/get_frontmost_native_window must be registered in the static tool catalog"
    );
    assert!(
        mcp_protocol.contains("tool_names.contains(&\"computer/get_frontmost_native_window\")"),
        "tools/list tests must expect computer/get_frontmost_native_window"
    );

    let args_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseGetFrontmostNativeWindowArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(deny_unknown_fields)]\nstruct ComputerUseGetFrontmostNativeWindowArgs"
        ),
        "computer/get_frontmost_native_window args must reject unknown fields"
    );
    assert!(
        field_declarations(args_struct).is_empty(),
        "computer/get_frontmost_native_window args must expose no fields"
    );

    let schema_body = extract_function_body(
        &mcp_tools,
        "fn computer_get_frontmost_native_window_input_schema()",
    );
    for needle in ["\"additionalProperties\": false", "\"properties\": {}"] {
        assert!(
            schema_body.contains(needle),
            "computer/get_frontmost_native_window schema missing {needle}"
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
            "computer/get_frontmost_native_window input must stay empty; found {needle}"
        );
    }

    let result_struct = extract_struct_block(
        &mcp_tools,
        "struct ComputerUseGetFrontmostNativeWindowResult",
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
            "window: Option<ComputerUseAppWindowInfo>,",
            "window_count: usize,",
            "warnings: Vec<String>,",
        ],
        "computer/get_frontmost_native_window result must expose exactly frontmost-window lookup fields"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_frontmost_native_window(");
    for needle in [
        "ComputerUseGetFrontmostNativeWindowArgs",
        "let Some(runtime) = runtime",
        "runtime.list_running_apps(ComputerUseListAppsRequest",
        "include_hidden: true",
        "include_background: true",
        "apps_snapshot.frontmost_pid",
        "status: \"noFrontmostApp\"",
        "runtime.list_app_windows(ComputerUseListAppWindowsRequest { pid: frontmost_pid })",
        "let app = snapshot.app;",
        "choose_frontmost_native_window(snapshot.windows)",
        "source: \"nsWorkspaceRunningApplications+coreGraphicsWindowList\"",
        "scope: \"frontmostNativeWindow\"",
        "\"appNotFound\"",
        "\"found\"",
        "\"noWindows\"",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/get_frontmost_native_window handler must contain {needle}"
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
            "computer/get_frontmost_native_window handler must stay read-only runtime composition; found {needle}"
        );
    }

    let chooser_body = extract_function_body(&mcp_tools, "fn choose_frontmost_native_window(");
    for needle in [".min_by_key(|window| (window.z_order, window.native_window_id))"] {
        assert!(
            chooser_body.contains(needle),
            "frontmost native-window chooser must pin deterministic z-order selection: {needle}"
        );
    }
    assert!(
        mcp_tools.contains("windows: Vec<ComputerUseAppWindowInfo>"),
        "frontmost native-window chooser must consume the existing native-window observation type"
    );

    for needle in [
        "get_frontmost_native_window",
        "GetFrontmostNativeWindow",
        "ComputerUseGetFrontmostNativeWindowRequest",
        "ComputerUseGetFrontmostNativeWindowSnapshot",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/get_frontmost_native_window must not add a dedicated runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/get_frontmost_native_window must not add a dedicated GPUI bridge surface; found {needle}"
        );
    }

    assert!(
        protocol.contains("`computer/get_frontmost_native_window` accepts no arguments"),
        "protocol docs must describe the get_frontmost_native_window input contract"
    );
    for needle in [
        "scope:\"frontmostNativeWindow\"",
        "status:\"found\"|\"noFrontmostApp\"|\"appNotFound\"|\"noWindows\"",
        "lowest `zOrder`, then lowest `nativeWindowId`",
        "composes the existing runtime-bridged running-app inventory and per-app native-window inventory",
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
            "protocol docs must pin get_frontmost_native_window boundary: {needle}"
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
