// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_native_windows_is_composition_only_read_only_inventory() {
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
            "pub const COMPUTER_LIST_NATIVE_WINDOWS_TOOL: &str = \"computer/list_native_windows\";"
        ),
        "computer/list_native_windows must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_LIST_NATIVE_WINDOWS_TOOL => handle_list_native_windows(arguments, runtime),"
        ),
        "computer/list_native_windows must route through the runtime-composition handler"
    );
    assert!(
        mcp_tools.contains("name: COMPUTER_LIST_NATIVE_WINDOWS_TOOL.to_string()"),
        "computer/list_native_windows must be registered in the static tool catalog"
    );
    assert!(
        mcp_protocol.contains("tool_names.contains(&\"computer/list_native_windows\")"),
        "tools/list tests must expect computer/list_native_windows"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseListNativeWindowsArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\nstruct ComputerUseListNativeWindowsArgs"
        ),
        "computer/list_native_windows args must reject unknown camelCase fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["include_hidden: bool,", "include_background: bool,"],
        "computer/list_native_windows args must expose exactly includeHidden/includeBackground"
    );

    let schema_body =
        extract_function_body(&mcp_tools, "fn computer_list_native_windows_input_schema()");
    for needle in [
        "\"additionalProperties\": false",
        "\"includeHidden\"",
        "\"includeBackground\"",
        "\"type\": \"boolean\"",
        "\"default\": false",
    ] {
        assert!(
            schema_body.contains(needle),
            "computer/list_native_windows schema missing {needle}"
        );
    }
    assert_eq!(
        extract_json_object_block(schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        2,
        "computer/list_native_windows schema properties must contain exactly two fields"
    );
    for needle in [
        "\"pid\"",
        "\"app\"",
        "\"bundleId\"",
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
            "computer/list_native_windows input must stay filter-only; found {needle}"
        );
    }

    let result_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseListNativeWindowsResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "frontmost_pid: Option<i32>,",
            "app_count: usize,",
            "window_count: usize,",
            "apps: Vec<ComputerUseNativeWindowsForApp>,",
            "warnings: Vec<String>,",
        ],
        "computer/list_native_windows result must expose exactly the grouped inventory fields"
    );

    let group_struct = extract_struct_block(&mcp_tools, "struct ComputerUseNativeWindowsForApp");
    assert_eq!(
        field_declarations(group_struct),
        vec![
            "app: ComputerUseRunningAppInfo,",
            "status: &'static str,",
            "windows: Vec<ComputerUseAppWindowInfo>,",
            "warnings: Vec<String>,",
        ],
        "per-app native-window group must expose exactly app/status/windows/warnings"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_list_native_windows(");
    for needle in [
        "ComputerUseListNativeWindowsArgs",
        "let Some(runtime) = runtime",
        "runtime.list_running_apps(ComputerUseListAppsRequest",
        "include_hidden: args.include_hidden",
        "include_background: args.include_background",
        "runtime.list_app_windows(ComputerUseListAppWindowsRequest { pid: app.pid })",
        "source: \"nsWorkspaceRunningApplications+coreGraphicsWindowList\"",
        "scope: \"runningGuiApps\"",
        "\"listed\"",
        "\"partial\"",
        "\"appNotFound\"",
        "\"windowListFailed\"",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/list_native_windows handler must contain {needle}"
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
            "computer/list_native_windows handler must stay read-only runtime composition; found {needle}"
        );
    }

    for needle in [
        "list_native_windows",
        "ListNativeWindows",
        "ComputerUseListNativeWindowsRequest",
        "ComputerUseListNativeWindowsSnapshot",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/list_native_windows must not add a dedicated runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/list_native_windows must not add a dedicated GPUI bridge surface; found {needle}"
        );
    }
    assert!(
        runtime.contains("fn list_running_apps(") && runtime.contains("fn list_app_windows("),
        "runtime bridge must continue to provide the composed read-only primitives"
    );

    assert!(
        protocol.contains("`computer/list_native_windows` accepts a closed `{includeHidden?:boolean,includeBackground?:boolean}` input"),
        "protocol docs must describe the list_native_windows input contract"
    );
    for needle in [
        "source:\"nsWorkspaceRunningApplications+coreGraphicsWindowList\"",
        "scope:\"runningGuiApps\"",
        "status:\"listed\"|\"partial\"",
        "composes the existing `computer/list_apps` and `computer/list_app_windows` runtime bridge calls",
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
            "protocol docs must pin list_native_windows boundary: {needle}"
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
