// doc-anchor-removed: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_see_mcp_tool_does_not_reimplement_inspection() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let mcp_protocol =
        std::fs::read_to_string("src/mcp_protocol/mod.rs").expect("read mcp_protocol/mod.rs");

    assert!(
        mcp_tools.contains("runtime.inspect_automation_window(request)"),
        "computer/see must delegate observation through ComputerUseRuntimeBridge"
    );
    assert!(
        mcp_protocol.contains("handle_computer_use_tool_call(")
            && mcp_protocol.contains("computer_runtime"),
        "MCP protocol routing must pass the optional computer runtime through to computer/* tools"
    );

    let forbidden = [
        "capture_targeted_rgba_image",
        "collect_visible",
        "resolve_automation_window",
        "collect_surface_snapshot",
        "resolve_targeted_os_window_id",
        "target_bounds_in_screenshot",
        "default_surface_hit_point",
        "default_suggested_hit_points",
        "captureScreenshot",
        "getElements",
        "simulateGpuiEvent",
    ];

    for (path, source) in [
        ("src/mcp_computer_use_tools.rs", mcp_tools.as_str()),
        ("src/mcp_protocol/mod.rs", mcp_protocol.as_str()),
    ] {
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        for needle in forbidden {
            assert!(
                !production_source.contains(needle),
                "{} must stay a thin adapter over inspectAutomationWindow; found {}",
                path,
                needle,
            );
        }
    }
}
