// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn mcp_server_routes_runtime_without_reimplementing_ui_inspection() {
    let source = std::fs::read_to_string("src/mcp_server/mod.rs").expect("read mcp_server/mod.rs");
    let production_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    assert!(
        production_source.contains("handle_request_with_runtime_context"),
        "MCP server must call the runtime-aware protocol entrypoint"
    );
    assert!(
        production_source.contains("computer_runtime.as_deref()"),
        "MCP server must pass the live computer runtime into protocol routing"
    );

    for needle in [
        "build_automation_inspect_snapshot",
        "capture_targeted_rgba_image",
        "collect_visible_elements",
        "resolve_targeted_os_window_id",
        "target_bounds_in_screenshot",
        "resolve_automation_window",
    ] {
        assert!(
            !production_source.contains(needle),
            "MCP server must not directly call UI inspection helper {}",
            needle
        );
    }
}
