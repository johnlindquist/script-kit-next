// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_permission_is_permission_id_only_read_only_lookup() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");

    assert!(
        mcp_tools.contains(
            "pub const COMPUTER_GET_PERMISSION_TOOL: &str = \"computer/get_permission\";"
        ),
        "computer/get_permission must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains("COMPUTER_GET_PERMISSION_TOOL => handle_get_permission(arguments),"),
        "computer/get_permission must route through a runtime-free handler"
    );
    assert!(
        mcp_tools.contains("computer_get_permission_input_schema"),
        "computer/get_permission must expose a dedicated input schema"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetPermissionArgs");
    assert!(
        mcp_tools.contains("#[serde(deny_unknown_fields)]\nstruct ComputerUseGetPermissionArgs"),
        "computer/get_permission args must reject unknown fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["id: String,"],
        "computer/get_permission args must expose exactly one id field"
    );

    let schema_body =
        extract_function_body(&mcp_tools, "fn computer_get_permission_input_schema()");
    assert!(
        schema_body.contains("\"additionalProperties\": false"),
        "computer/get_permission schema must reject unknown fields"
    );
    for needle in [
        "\"id\"",
        "\"type\": \"string\"",
        "\"enum\": [\"accessibility\", \"screenRecording\", \"eventSynthesizing\"]",
        "\"required\": [\"id\"]",
    ] {
        assert!(
            schema_body.contains(needle),
            "computer/get_permission schema missing {needle}"
        );
    }
    assert_eq!(
        extract_json_object_block(schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        1,
        "computer/get_permission schema properties must contain exactly one field"
    );
    for needle in [
        "\"request\"",
        "\"grant\"",
        "\"openSettings\"",
        "\"click\"",
        "\"press\"",
        "\"execute\"",
        "\"permission\"",
    ] {
        assert!(
            !schema_body.contains(needle),
            "computer/get_permission input must stay id-only; found {needle}"
        );
    }

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetPermissionResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "permission: Option<ComputerUsePermissionStatus>,",
            "warnings: Vec<String>,",
        ],
        "computer/get_permission result must stay a status-only read model"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_permission(");
    for needle in [
        "ComputerUseGetPermissionArgs",
        "computer_use_permission_statuses()",
        "permission.id == args.id",
        "source: \"macosPermissionPreflight\"",
        "scope: \"permissionId\"",
        "\"found\"",
        "\"notFound\"",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/get_permission handler must contain {needle}"
        );
    }
    for needle in [
        "ComputerUseRuntimeBridge",
        "runtime.inspect_automation_window",
        "runtime.list_running_apps",
        "runtime.list_app_windows",
        "request_accessibility_permission",
        "AXIsProcessTrustedWithOptions",
        "CGRequestScreenCaptureAccess",
        "CGRequestPostEventAccess",
        "CGEventPost",
        "capture_targeted_screenshot",
        "openSettings",
        "System Settings",
        "click",
        "press",
        "execute",
        "focus",
        "activate",
        "hotkey",
        "type",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/get_permission handler must stay read-only permission lookup; found {needle}"
        );
    }

    let helper_body = extract_function_body(&mcp_tools, "fn computer_use_permission_statuses()");
    for needle in [
        "\"accessibility\"",
        "\"screenRecording\"",
        "\"eventSynthesizing\"",
        "check_accessibility_permission()",
        "screen_capture_access_preflight()",
        "event_synthesizing_access_preflight()",
    ] {
        assert!(
            helper_body.contains(needle),
            "shared permission helper must contain {needle}"
        );
    }
    for needle in [
        "request_accessibility_permission",
        "AXIsProcessTrustedWithOptions",
        "CGRequestScreenCaptureAccess",
        "CGRequestPostEventAccess",
        "CGEventPost",
        "openSettings",
        "System Settings",
    ] {
        assert!(
            !helper_body.contains(needle),
            "shared permission helper must stay non-prompting; found {needle}"
        );
    }

    for needle in [
        "GetPermission",
        "get_permission",
        "ComputerUseGetPermissionRequest",
        "ComputerUseGetPermissionSnapshot",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/get_permission must not add runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/get_permission must not add GPUI bridge surface; found {needle}"
        );
    }

    assert!(
        protocol.contains("`computer/get_permission` accepts a closed `{id:string}` input"),
        "protocol docs must describe the get_permission input contract"
    );
    for needle in [
        "source:\"macosPermissionPreflight\"",
        "scope:\"permissionId\"",
        "status:\"found\"|\"notFound\"",
        "same non-prompting permission checks",
        "does not request permissions",
        "open System Settings",
        "synthesize events",
        "send input",
        "mutate app/window state",
        "add a runtime bridge method",
        "expose action handles",
    ] {
        assert!(
            protocol.contains(needle),
            "protocol docs must pin get_permission non-goal: {needle}"
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
