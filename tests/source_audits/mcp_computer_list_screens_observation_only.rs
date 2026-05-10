// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_screens_is_observation_only_and_background_safe() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");

    assert!(
        mcp_tools.contains("COMPUTER_LIST_SCREENS_TOOL"),
        "computer/list_screens must be registered through the computer-use MCP tool module"
    );
    assert!(
        mcp_tools.contains("fn handle_list_screens(arguments: &Value) -> ToolResult"),
        "computer/list_screens must have a dedicated read-only handler"
    );
    assert!(
        mcp_tools.contains("CGDisplay::active_displays()"),
        "computer/list_screens should use background-safe CoreGraphics enumeration"
    );

    let forbidden = [
        "NSScreen",
        "get_macos_visible_displays()",
        "WindowAction",
        "window_control::",
        "move_window",
        "resize_window",
        "focus_window",
        "set_automation_bounds",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "Command::new(\"open\")",
        "x-apple.systempreferences",
    ];

    for needle in forbidden {
        assert!(
            !mcp_tools.contains(needle),
            "computer/list_screens must remain observation-only/background-safe; found {}",
            needle
        );
    }
}
