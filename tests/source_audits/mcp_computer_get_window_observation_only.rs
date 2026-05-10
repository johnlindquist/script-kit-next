// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_window_reads_automation_registry_only() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let registry = std::fs::read_to_string("src/windows/automation_registry.rs")
        .expect("read automation_registry.rs");
    let windows_mod = std::fs::read_to_string("src/windows/mod.rs").expect("read windows/mod.rs");

    assert!(
        mcp_tools.contains("pub const COMPUTER_GET_WINDOW_TOOL: &str = \"computer/get_window\";"),
        "computer/get_window must be registered through the computer-use MCP tool module"
    );
    assert!(
        mcp_tools.contains("fn handle_get_window("),
        "computer/get_window must have a dedicated handler"
    );
    assert!(
        mcp_tools.contains("computer_get_window_input_schema"),
        "computer/get_window must expose a closed input schema"
    );
    assert!(
        mcp_tools.contains("COMPUTER_GET_WINDOW_TOOL => handle_get_window(arguments),"),
        "computer/get_window must route directly to the registry-only handler"
    );
    assert!(
        mcp_tools.contains("ComputerUseGetWindowArgs"),
        "computer/get_window must use a dedicated input envelope"
    );
    assert!(
        mcp_tools.contains("ComputerUseGetWindowResult"),
        "computer/get_window must use a dedicated result envelope"
    );
    assert!(
        registry.contains("pub fn automation_window_by_id("),
        "automation registry must expose a pure by-id snapshot helper"
    );
    assert!(
        windows_mod.contains("automation_window_by_id"),
        "crate::windows must re-export the by-id automation registry helper"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetWindowArgs");
    assert!(
        mcp_tools.contains("#[serde(deny_unknown_fields)]\nstruct ComputerUseGetWindowArgs"),
        "computer/get_window args must reject unknown fields"
    );
    assert!(
        args_struct.contains("id: String"),
        "computer/get_window input must be exactly a stable window id"
    );

    let input_schema_body =
        extract_function_body(&mcp_tools, "fn computer_get_window_input_schema()");
    assert!(
        input_schema_body.contains("\"additionalProperties\": false"),
        "computer/get_window must reject unknown input fields"
    );
    assert!(
        input_schema_body.contains("\"id\": { \"type\": \"string\" }"),
        "computer/get_window must require a string id"
    );
    assert!(
        input_schema_body.contains("\"required\": [\"id\"]"),
        "computer/get_window must require id"
    );
    for needle in [
        "\"target\"",
        "\"focus\"",
        "\"activate\"",
        "\"refresh\"",
        "\"click\"",
        "\"move\"",
        "\"resize\"",
        "\"includeElements\"",
        "\"elements\"",
        "\"screenshot\"",
        "\"probes\"",
    ] {
        assert!(
            !input_schema_body.contains(needle),
            "computer/get_window input schema must not expand into inspection/action scope; found {}",
            needle
        );
    }

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetWindowResult");
    assert!(
        result_struct.contains("schema_version: u32"),
        "computer/get_window result must include schemaVersion"
    );
    assert!(
        result_struct.contains("source: &'static str"),
        "computer/get_window result must include source"
    );
    assert!(
        result_struct.contains("status: &'static str"),
        "computer/get_window result must include status"
    );
    assert!(
        result_struct.contains("window: Option<AutomationWindowInfo>"),
        "computer/get_window result must include a nullable registry window"
    );
    assert!(
        result_struct.contains("warnings: Vec<String>"),
        "computer/get_window result must include warnings"
    );
    assert!(
        !result_struct.contains("skip_serializing_if"),
        "computer/get_window must serialize window as null instead of omitting it"
    );
    for needle in [
        "focused_window_id",
        "actions",
        "elements",
        "screenshot",
        "target_bounds_in_screenshot",
    ] {
        assert!(
            !result_struct.contains(needle),
            "computer/get_window result must stay a registry record lookup, not an inspection/action envelope; found {}",
            needle
        );
    }

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_window(");
    assert!(
        handler_body.contains("serde_json::from_value(arguments.clone())"),
        "computer/get_window handler must parse only its closed input envelope"
    );
    assert!(
        handler_body.contains("automation_window_by_id(&args.id)"),
        "computer/get_window handler must read the by-id automation registry helper"
    );
    assert!(
        handler_body.contains("schema_version: AUTOMATION_WINDOW_SCHEMA_VERSION"),
        "computer/get_window handler must return the automation window schema version"
    );
    assert!(
        handler_body.contains("source: \"automationWindowRegistry\""),
        "computer/get_window handler must identify the automation registry as its source"
    );
    assert!(
        handler_body.contains("\"found\""),
        "computer/get_window handler must report found status"
    );
    assert!(
        handler_body.contains("\"notFound\""),
        "computer/get_window handler must report notFound status"
    );
    assert!(
        handler_body.contains("warnings: Vec::new()"),
        "computer/get_window handler must not invent warnings on a plain registry lookup"
    );
    assert!(
        handler_body.contains("json_tool_result(&ComputerUseGetWindowResult"),
        "computer/get_window handler must serialize the dedicated result envelope"
    );
    for needle in [
        "ComputerUseRuntimeBridge",
        "runtime.",
        "Option<&dyn",
        "resolve_automation_window",
        "focused_automation_window",
        "focused_automation_window_id",
        "list_automation_windows",
        "inspect_automation_window",
        "build_automation_inspect_snapshot",
        "capture_targeted_rgba_image",
        "collect_visible_elements",
        "target_bounds_in_screenshot",
        "NSWorkspace",
        "NSScreen",
        "runningApplications",
        "frontmostApplication",
        "menuBarOwningApplication",
        "AXUIElementCreateApplication",
        "AXUIElementCopyAttributeValue",
        "AXUIElementPerformAction",
        "AXPress",
        "CGEvent",
        "simulateKey",
        "upsert_automation_window",
        "remove_automation_window",
        "register_attached_popup",
        "ensure_embedded_ai_window",
        "set_automation_focus",
        "update_automation_semantic_surface",
        "set_automation_visibility",
        "set_automation_bounds",
        "focus_window",
        "move_window",
        "resize_window",
        "request_accessibility_permission",
        "AXIsProcessTrustedWithOptions",
        "CGRequestScreenCaptureAccess",
        "Command::new(\"open\")",
        "x-apple.systempreferences",
        "activateWithOptions",
        "activateIgnoringOtherApps",
        "launchApplication",
        "openApplicationAtURL",
        "terminate",
        "forceTerminate",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/get_window handler must not inspect, prompt, focus, activate, click, or mutate native/app state; found {}",
            needle
        );
    }

    let helper_body = extract_function_body(&registry, "pub fn automation_window_by_id(");
    assert!(
        helper_body.contains("AUTOMATION_WINDOWS.lock()"),
        "automation_window_by_id must read from the automation registry lock"
    );
    assert!(
        helper_body.contains("state.windows.get(id).cloned()"),
        "automation_window_by_id must clone only the requested registered window"
    );
    for needle in [
        "resolve_automation_window",
        "tracing::",
        "anyhow!",
        "rebuild_indexes",
        "upsert_automation_window",
        "remove_automation_window",
        "register_attached_popup",
        "ensure_embedded_ai_window",
        "set_automation_focus",
        "set_automation_visibility",
        "set_automation_bounds",
        "update_automation_semantic_surface",
        "NSWorkspace",
        "NSScreen",
        "AXUIElement",
        "CGEvent",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "Command::new(\"open\")",
    ] {
        assert!(
            !helper_body.contains(needle),
            "automation_window_by_id must remain a pure lock-and-clone read; found {}",
            needle
        );
    }
}

fn extract_struct_block<'a>(source: &'a str, signature: &str) -> &'a str {
    extract_braced_block(source, signature)
}

fn extract_function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    extract_braced_block(source, signature)
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
