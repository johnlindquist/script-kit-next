// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_permissions_is_status_only() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let screenshots = std::fs::read_to_string("src/platform/screenshots_window_open.rs")
        .expect("read screenshots_window_open.rs");

    assert!(
        mcp_tools.contains("COMPUTER_LIST_PERMISSIONS_TOOL"),
        "computer/list_permissions must be registered through the computer-use MCP tool module"
    );
    assert!(
        mcp_tools.contains("fn handle_list_permissions(arguments: &Value) -> ToolResult"),
        "computer/list_permissions must have a dedicated read-only handler"
    );
    assert!(
        mcp_tools.contains("check_accessibility_permission()"),
        "computer/list_permissions must reuse the existing non-prompting Accessibility probe"
    );
    assert!(
        mcp_tools.contains("screen_capture_access_preflight()"),
        "computer/list_permissions must reuse the existing non-prompting Screen Recording preflight"
    );
    assert!(
        mcp_tools.contains("event_synthesizing_access_preflight()"),
        "computer/list_permissions must reuse the non-prompting Event Synthesizing preflight"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_list_permissions(");
    let forbidden = [
        "request_accessibility_permission",
        "open_accessibility_settings",
        "CGRequestScreenCaptureAccess",
        "CGRequestPostEventAccess",
        "Command::new(\"open\")",
        "x-apple.systempreferences",
    ];

    for needle in forbidden {
        assert!(
            !handler_body.contains(needle),
            "computer/list_permissions must not request permissions or open settings; found {}",
            needle
        );
    }

    assert!(
        screenshots.contains("CGPreflightScreenCaptureAccess"),
        "Screen Recording status must continue to use CoreGraphics preflight"
    );
    assert!(
        !screenshots.contains("CGRequestScreenCaptureAccess"),
        "Screen Recording status must not request permission as a side effect"
    );
    assert!(
        screenshots.contains("CGPreflightPostEventAccess"),
        "Event Synthesizing status must continue to use CoreGraphics preflight"
    );
    assert!(
        !screenshots.contains("CGRequestPostEventAccess"),
        "Event Synthesizing status must not request permission as a side effect"
    );
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
