// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_screen_is_display_id_only_read_only_lookup() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");

    assert!(
        mcp_tools.contains("pub const COMPUTER_GET_SCREEN_TOOL: &str = \"computer/get_screen\";"),
        "computer/get_screen must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains("COMPUTER_GET_SCREEN_TOOL => handle_get_screen(arguments),"),
        "computer/get_screen must route through a runtime-free handler"
    );
    assert!(
        mcp_tools.contains("computer_get_screen_input_schema"),
        "computer/get_screen must expose a dedicated input schema"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetScreenArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\nstruct ComputerUseGetScreenArgs"
        ),
        "computer/get_screen args must use camelCase and reject unknown fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["display_id: u32,"],
        "computer/get_screen args must expose exactly one displayId field"
    );

    let schema_body = extract_function_body(&mcp_tools, "fn computer_get_screen_input_schema()");
    assert!(
        schema_body.contains("\"additionalProperties\": false"),
        "computer/get_screen schema must reject unknown fields"
    );
    for needle in [
        "\"displayId\"",
        "\"type\": \"integer\"",
        "\"minimum\": 0",
        "\"maximum\": 4_294_967_295u64",
        "\"required\": [\"displayId\"]",
    ] {
        assert!(
            schema_body.contains(needle),
            "computer/get_screen schema missing {needle}"
        );
    }
    assert_eq!(
        extract_json_object_block(schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        1,
        "computer/get_screen schema properties must contain exactly one field"
    );
    for needle in [
        "\"id\"",
        "\"screen\"",
        "\"target\"",
        "\"index\"",
        "\"move\"",
        "\"resize\"",
        "\"screenshot\"",
        "\"capture\"",
        "\"click\"",
        "\"press\"",
        "\"execute\"",
    ] {
        assert!(
            !schema_body.contains(needle),
            "computer/get_screen input must stay displayId-only; found {needle}"
        );
    }

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetScreenResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "screen: Option<DisplayInfo>,",
            "warnings: Vec<String>,",
        ],
        "computer/get_screen result must stay a status-only read model"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_screen(");
    for needle in [
        "ComputerUseGetScreenArgs",
        "list_screens()",
        "screen.display_id == args.display_id",
        "source: \"coreGraphicsActiveDisplays\"",
        "scope: \"displayId\"",
        "\"found\"",
        "\"notFound\"",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/get_screen handler must contain {needle}"
        );
    }
    for needle in [
        "ComputerUseRuntimeBridge",
        "runtime.inspect_automation_window",
        "runtime.list_running_apps",
        "runtime.list_app_windows",
        "NSScreen",
        "CGWindowListCreateImage",
        "CGEvent",
        "AXUIElement",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "move",
        "resize",
        "focus",
        "click",
        "press",
        "execute",
        "openSettings",
        "System Settings",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/get_screen handler must stay read-only display lookup; found {needle}"
        );
    }

    for needle in [
        "GetScreen",
        "get_screen",
        "ComputerUseGetScreenRequest",
        "ComputerUseGetScreenSnapshot",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/get_screen must not add runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/get_screen must not add GPUI bridge surface; found {needle}"
        );
    }

    assert!(
        protocol.contains("`computer/get_screen` accepts a closed `{displayId:integer}` input"),
        "protocol docs must describe the get_screen input contract"
    );
    for needle in [
        "source:\"coreGraphicsActiveDisplays\"",
        "scope:\"displayId\"",
        "status:\"found\"|\"notFound\"",
        "does not add a native bridge method",
        "call `NSScreen`",
        "move windows",
        "change screen placement",
        "capture screenshots",
        "request permissions",
        "expose action handles",
    ] {
        assert!(
            protocol.contains(needle),
            "protocol docs must pin get_screen non-goal: {needle}"
        );
    }
}

fn extract_struct_block<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("struct signature");
    let open = source[start..].find('{').expect("struct open brace") + start;
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

    panic!("struct block for {signature} did not close")
}

fn extract_function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("function signature");
    let open = source[start..].find('{').expect("function open brace") + start;
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

    panic!("function body for {signature} did not close")
}

fn extract_json_object_block<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source.find(marker).expect("json marker");
    let open = source[start..].find('{').expect("json object open") + start;
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

    panic!("json object for {marker} did not close")
}

fn field_declarations(block: &str) -> Vec<String> {
    block
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(','))
        .map(ToString::to_string)
        .collect()
}
