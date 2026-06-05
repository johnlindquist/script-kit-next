#[test]
fn computer_get_focused_window_reads_automation_registry_only() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let registry = std::fs::read_to_string("src/windows/automation_registry.rs")
        .expect("read automation_registry.rs");

    assert!(
        mcp_tools.contains("COMPUTER_GET_FOCUSED_WINDOW_TOOL"),
        "computer/get_focused_window must be registered through the computer-use MCP tool module"
    );
    assert!(
        mcp_tools.contains("fn handle_get_focused_window("),
        "computer/get_focused_window must have a dedicated handler"
    );
    assert!(
        mcp_tools.contains("computer_get_focused_window_input_schema"),
        "computer/get_focused_window must expose a closed input schema"
    );
    assert!(
        mcp_tools.contains("automationWindowRegistry"),
        "computer/get_focused_window must identify the automation registry as its source"
    );
    assert!(
        mcp_tools.contains("focusedAutomationWindow"),
        "computer/get_focused_window must identify the focused automation-window scope"
    );
    assert!(
        registry.contains("pub fn focused_automation_window()"),
        "automation registry must expose a pure focused-window snapshot helper"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_GET_FOCUSED_WINDOW_TOOL => handle_get_focused_window(arguments),"
        ),
        "computer/get_focused_window must route directly to the registry-only handler"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_focused_window(");
    assert!(
        handler_body.contains("focused_automation_window()"),
        "computer/get_focused_window handler must read the focused automation-window registry helper"
    );
    for needle in [
        "ComputerUseRuntimeBridge",
        "runtime.",
        "Option<&dyn",
        "resolve_automation_window",
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
            "computer/get_focused_window handler must not inspect, prompt, focus, activate, click, or mutate native/app state; found {}",
            needle
        );
    }

    let helper_body = extract_function_body(&registry, "pub fn focused_automation_window()");
    assert!(
        helper_body.contains("AUTOMATION_WINDOWS.lock()"),
        "focused_automation_window must read from the automation registry lock"
    );
    assert!(
        helper_body.contains(".cloned()"),
        "focused_automation_window must return a cloned snapshot"
    );
    for needle in [
        "resolve_automation_window",
        "tracing::",
        "anyhow!",
        "rebuild_indexes",
        "upsert_automation_window",
        "remove_automation_window",
        "set_automation_focus",
        "set_automation_visibility",
        "set_automation_bounds",
    ] {
        assert!(
            !helper_body.contains(needle),
            "focused_automation_window must remain a pure lock-and-clone read; found {}",
            needle
        );
    }
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

    panic!("function body for {} did not close", signature)
}
