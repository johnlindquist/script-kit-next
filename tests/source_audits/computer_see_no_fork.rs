// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_see_mcp_tool_does_not_reimplement_inspection() {
    let source = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let forbidden = [
        "capture_targeted",
        "collect_visible",
        "resolve_automation_window",
        "captureScreenshot",
        "getElements",
        "simulateGpuiEvent",
    ];

    for needle in forbidden {
        assert!(
            !source.contains(needle),
            "computer/see must stay a thin adapter over inspectAutomationWindow; found {}",
            needle
        );
    }
}
