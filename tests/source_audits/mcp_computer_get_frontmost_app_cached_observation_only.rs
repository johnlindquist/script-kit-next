// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_frontmost_app_reads_cached_tracker_state_only() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let tracker = std::fs::read_to_string("src/frontmost_app_tracker/mod.rs")
        .expect("read frontmost_app_tracker.rs");

    assert!(
        mcp_tools.contains("COMPUTER_GET_FRONTMOST_APP_TOOL"),
        "computer/get_frontmost_app must be registered through the computer-use MCP tool module"
    );
    assert!(
        mcp_tools.contains("fn handle_get_frontmost_app("),
        "computer/get_frontmost_app must have a dedicated handler"
    );
    assert!(
        mcp_tools.contains("computer_get_frontmost_app_input_schema"),
        "computer/get_frontmost_app must expose a closed input schema"
    );
    assert!(
        mcp_tools.contains("get_last_real_app()"),
        "computer/get_frontmost_app must read the frontmost app tracker cache"
    );
    assert!(
        mcp_tools.contains("frontmostAppTrackerCache"),
        "computer/get_frontmost_app must identify the cache as its observation source"
    );
    assert!(
        mcp_tools.contains("lastNonScriptKitApp"),
        "computer/get_frontmost_app must expose the last tracked non-Script-Kit scope"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_frontmost_app(");
    for needle in [
        "NSWorkspace",
        "runningApplications",
        "menuBarOwningApplication",
        "frontmostApplication",
        "AXUIElementCreateApplication",
        "AXUIElementCopyAttributeValue",
        "AXUIElementPerformAction",
        "get_menu_bar_for_pid",
        "get_frontmost_menu_bar",
        "activateWithOptions",
        "activateIgnoringOtherApps",
        "launchApplication",
        "openApplicationAtURL",
        "terminate",
        "forceTerminate",
        "CGEvent",
        "simulateKey",
        "focus_window",
        "move_window",
        "resize_window",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "Command::new(\"open\")",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/get_frontmost_app handler must not refresh, prompt, activate, click, or mutate native state; found {}",
            needle
        );
    }

    let helper_body = extract_function_body(&tracker, "pub fn get_last_real_app()");
    for needle in [
        "capture_current_frontmost_app",
        "fetch_menu_items_async",
        "get_menu_bar_for_pid",
        "NSWorkspace",
        "AXUIElementCreateApplication",
        "AXUIElementCopyAttributeValue",
        "menuBarOwningApplication",
        "frontmostApplication",
        "request_accessibility_permission",
    ] {
        assert!(
            !helper_body.contains(needle),
            "get_last_real_app must remain a pure state read for computer/get_frontmost_app; found {}",
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
