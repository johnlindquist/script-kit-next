// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_apps_by_bundle_id_is_exact_bundle_read_only_lookup() {
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
            "pub const COMPUTER_LIST_APPS_BY_BUNDLE_ID_TOOL: &str = \"computer/list_apps_by_bundle_id\";"
        ),
        "computer/list_apps_by_bundle_id must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_LIST_APPS_BY_BUNDLE_ID_TOOL => handle_list_apps_by_bundle_id(arguments, runtime),"
        ),
        "computer/list_apps_by_bundle_id must route through the runtime app-list handler"
    );
    assert!(
        mcp_tools.contains("name: COMPUTER_LIST_APPS_BY_BUNDLE_ID_TOOL.to_string()"),
        "computer/list_apps_by_bundle_id must be registered in the static tool catalog"
    );
    assert!(
        mcp_protocol.contains("tool_names.contains(&\"computer/list_apps_by_bundle_id\")"),
        "tools/list tests must expect computer/list_apps_by_bundle_id"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseListAppsByBundleIdArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\nstruct ComputerUseListAppsByBundleIdArgs"
        ),
        "computer/list_apps_by_bundle_id args must reject unknown camelCase fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["bundle_id: String,"],
        "computer/list_apps_by_bundle_id args must expose exactly bundleId"
    );

    let schema_body = extract_function_body(
        &mcp_tools,
        "fn computer_list_apps_by_bundle_id_input_schema()",
    );
    for needle in [
        "\"additionalProperties\": false",
        "\"bundleId\"",
        "\"type\": \"string\"",
        "\"minLength\": 1",
        "\"required\": [\"bundleId\"]",
    ] {
        assert!(
            schema_body.contains(needle),
            "computer/list_apps_by_bundle_id schema missing {needle}"
        );
    }
    assert_eq!(
        extract_json_object_block(schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        1,
        "computer/list_apps_by_bundle_id schema properties must contain exactly one field"
    );
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
        "\"screenshot\"",
        "\"capture\"",
        "\"click\"",
        "\"press\"",
        "\"execute\"",
        "\"input\"",
        "\"typeText\"",
        "\"key\"",
        "\"includeGlobalStatusItems\"",
    ] {
        assert!(
            !schema_body.contains(needle),
            "computer/list_apps_by_bundle_id input must stay bundleId-only; found {needle}"
        );
    }

    let result_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseListAppsByBundleIdResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "bundle_id: String,",
            "app_count: usize,",
            "apps: Vec<ComputerUseRunningAppInfo>,",
            "warnings: Vec<String>,",
        ],
        "computer/list_apps_by_bundle_id result must expose exactly the bundle app lookup fields"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_list_apps_by_bundle_id(");
    for needle in [
        "ComputerUseListAppsByBundleIdArgs",
        "if args.bundle_id.is_empty()",
        "let Some(runtime) = runtime",
        "runtime.list_running_apps(ComputerUseListAppsRequest",
        "include_hidden: true",
        "include_background: true",
        "app.bundle_id.as_deref() == Some(args.bundle_id.as_str())",
        "source: \"nsWorkspaceRunningApplications\"",
        "scope: \"runningAppBundleId\"",
        "\"listed\"",
        "\"notFound\"",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/list_apps_by_bundle_id handler must contain {needle}"
        );
    }
    for needle in [
        "list_app_windows",
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
        "click",
        "press",
        "execute",
        "listMenuExtras",
        "status item",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/list_apps_by_bundle_id handler must stay an app-list lookup; found {needle}"
        );
    }

    for needle in [
        "list_apps_by_bundle_id",
        "ListAppsByBundleId",
        "ComputerUseListAppsByBundleIdRequest",
        "ComputerUseListAppsByBundleIdSnapshot",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/list_apps_by_bundle_id must not add a dedicated runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/list_apps_by_bundle_id must not add a dedicated GPUI bridge surface; found {needle}"
        );
    }
    assert!(
        runtime.contains("fn list_running_apps("),
        "runtime bridge must continue to provide the composed running-app primitive"
    );

    assert!(
        protocol.contains(
            "`computer/list_apps_by_bundle_id` accepts a closed `{bundleId:string}` exact-bundle input"
        ),
        "protocol docs must describe the list_apps_by_bundle_id input contract"
    );
    for needle in [
        "source:\"nsWorkspaceRunningApplications\"",
        "scope:\"runningAppBundleId\"",
        "status:\"listed\"|\"notFound\"",
        "filters by exact `bundle_id`",
        "does not add a native bridge method",
        "inspect windows",
        "focus, activate, launch, quit, hide",
        "send input",
        "enumerate menu extras or status items",
        "request permissions",
        "expose action handles",
    ] {
        assert!(
            protocol.contains(needle),
            "protocol docs must pin list_apps_by_bundle_id boundary: {needle}"
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
