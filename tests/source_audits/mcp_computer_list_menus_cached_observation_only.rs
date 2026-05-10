// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_menus_reads_cached_tracker_snapshot_only() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let tracker = std::fs::read_to_string("src/frontmost_app_tracker/mod.rs")
        .expect("read frontmost_app_tracker.rs");

    assert!(
        mcp_tools.contains("COMPUTER_LIST_MENUS_TOOL"),
        "computer/list_menus must be registered through the computer-use MCP tool module"
    );
    assert!(
        mcp_tools.contains("fn handle_list_menus("),
        "computer/list_menus must have a dedicated handler"
    );
    assert!(
        mcp_tools.contains("get_cached_menu_snapshot()"),
        "computer/list_menus must read the frontmost app tracker cache snapshot"
    );
    assert!(
        mcp_tools.contains("frontmostAppTrackerCache"),
        "computer/list_menus must identify the cache as its observation source"
    );
    assert!(
        mcp_tools.contains("ComputerUseMenuItem"),
        "computer/list_menus must map cached MenuBarItem values to an action-free result type"
    );

    let production_mcp = mcp_tools
        .split("#[cfg(test)]")
        .next()
        .expect("production mcp source");
    for needle in [
        "get_menu_bar_for_pid",
        "get_frontmost_menu_bar",
        "AXUIElementCreateApplication",
        "AXUIElementCopyAttributeValue",
        "AXUIElementPerformAction",
        "AXPress",
        "perform_ax_action",
        "execute_menu_action",
        "menu_executor",
        "open_menu_at_element",
        "activateWithOptions",
        "activateIgnoringOtherApps",
        "frontmostApplication",
        "menuBarOwningApplication",
        "request_accessibility_permission",
        "application_is_trusted_with_prompt",
        "CGRequestScreenCaptureAccess",
        "Command::new(\"open\")",
        "x-apple.systempreferences",
        "click-extra",
        "click_status",
        "clickStatus",
        "status item click",
    ] {
        assert!(
            !production_mcp.contains(needle),
            "computer/list_menus must not refresh, prompt, activate, click, or execute menus from the MCP handler; found {}",
            needle
        );
    }

    let helper_body = extract_function_body(&tracker, "pub fn get_cached_menu_snapshot()");
    for needle in [
        "fetch_menu_items_async",
        "get_menu_bar_for_pid",
        "get_frontmost_menu_bar",
        "AXUIElementCreateApplication",
        "AXUIElementPerformAction",
        "NSWorkspace",
        "menuBarOwningApplication",
        "frontmostApplication",
        "request_accessibility_permission",
    ] {
        assert!(
            !helper_body.contains(needle),
            "get_cached_menu_snapshot must be a pure state read; found {}",
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
