// doc-anchor-removed: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_windows_reads_automation_registry_only() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let registry = std::fs::read_to_string("src/windows/automation_registry.rs")
        .expect("read automation_registry.rs");

    assert!(
        mcp_tools.contains("COMPUTER_LIST_WINDOWS_TOOL"),
        "computer/list_windows must be registered through the computer-use MCP tool module"
    );
    assert!(
        mcp_tools.contains("fn handle_list_windows("),
        "computer/list_windows must have a dedicated handler"
    );
    assert!(
        mcp_tools.contains("computer_list_windows_input_schema"),
        "computer/list_windows must expose a closed input schema"
    );
    assert!(
        mcp_tools.contains("COMPUTER_LIST_WINDOWS_TOOL => handle_list_windows(arguments),"),
        "computer/list_windows must route directly to the registry-only handler"
    );
    assert!(
        mcp_tools.contains("ComputerUseListWindowsResult"),
        "computer/list_windows must use the list-windows result envelope"
    );
    assert!(
        mcp_tools.contains("windows: crate::windows::list_automation_windows()"),
        "computer/list_windows must read the automation-window registry snapshot"
    );
    assert!(
        mcp_tools.contains("focused_window_id: crate::windows::focused_automation_window_id()"),
        "computer/list_windows must read only the focused automation-window id"
    );

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseListWindowsResult");
    assert!(
        result_struct.contains("schema_version: u32"),
        "computer/list_windows result must include schemaVersion"
    );
    assert!(
        result_struct.contains("windows: Vec<AutomationWindowInfo>"),
        "computer/list_windows result must include the registry window snapshot"
    );
    assert!(
        result_struct.contains("focused_window_id: Option<String>"),
        "computer/list_windows result must include focusedWindowId, serialized as null when no window is focused"
    );
    assert!(
        !result_struct.contains("skip_serializing_if"),
        "computer/list_windows must serialize focusedWindowId as null instead of omitting it"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_list_windows(");
    assert!(
        handler_body.contains("schema_version: AUTOMATION_WINDOW_SCHEMA_VERSION"),
        "computer/list_windows handler must return the automation window schema version"
    );
    assert!(
        handler_body.contains("list_automation_windows()"),
        "computer/list_windows handler must read list_automation_windows"
    );
    assert!(
        handler_body.contains("focused_automation_window_id()"),
        "computer/list_windows handler must read focused_automation_window_id"
    );
    for needle in [
        "ComputerUseRuntimeBridge",
        "runtime.",
        "Option<&dyn",
        "resolve_automation_window",
        "focused_automation_window()",
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
            "computer/list_windows handler must not inspect, prompt, focus, activate, click, or mutate native/app state; found {}",
            needle
        );
    }

    let input_schema_body =
        extract_function_body(&mcp_tools, "fn computer_list_windows_input_schema()");
    assert!(
        input_schema_body.contains("\"additionalProperties\": false"),
        "computer/list_windows must reject unknown input fields"
    );
    assert!(
        input_schema_body.contains("\"properties\": {}"),
        "computer/list_windows must keep a closed empty input schema"
    );

    let list_body = extract_function_body(&registry, "pub fn list_automation_windows()");
    assert!(
        list_body.contains("AUTOMATION_WINDOWS.lock()"),
        "list_automation_windows must read from the automation registry lock"
    );
    assert!(
        list_body.contains("state.windows.values().cloned().collect()"),
        "list_automation_windows must clone registered automation windows"
    );
    assert!(
        list_body.contains("windows.sort_by"),
        "list_automation_windows must keep stable sorted output"
    );
    assert!(
        list_body.contains("kind_rank"),
        "list_automation_windows must sort by automation-window kind"
    );
    for needle in [
        "rebuild_indexes",
        "upsert_automation_window",
        "remove_automation_window",
        "register_attached_popup",
        "ensure_embedded_ai_window",
        "set_automation_focus",
        "update_automation_semantic_surface",
        "set_automation_visibility",
        "set_automation_bounds",
        "resolve_automation_window",
        "NSWorkspace",
        "NSScreen",
        "AXUIElement",
        "CGEvent",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "Command::new(\"open\")",
    ] {
        assert!(
            !list_body.contains(needle),
            "list_automation_windows must remain a lock/clone/sort registry snapshot; found {}",
            needle
        );
    }

    let focused_id_body = extract_function_body(&registry, "pub fn focused_automation_window_id()");
    assert!(
        focused_id_body.contains("AUTOMATION_WINDOWS.lock().focused_id.clone()"),
        "focused_automation_window_id must remain a pure focused-id clone"
    );
    for needle in [
        "set_automation_focus",
        "focused_automation_window()",
        "resolve_automation_window",
        "rebuild_indexes",
        "upsert_automation_window",
        "remove_automation_window",
        "NSWorkspace",
        "AXUIElement",
        "CGEvent",
    ] {
        assert!(
            !focused_id_body.contains(needle),
            "focused_automation_window_id must remain a pure focused-id clone; found {}",
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

    panic!("function body for {} did not close", signature)
}
