// doc-anchor-removed: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_apps_is_runtime_bridged_running_gui_inventory() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");

    assert!(
        mcp_tools.contains("COMPUTER_LIST_APPS_TOOL"),
        "computer/list_apps must be registered through the computer-use MCP tool module"
    );
    assert!(
        mcp_tools.contains("fn handle_list_apps("),
        "computer/list_apps must have a dedicated handler"
    );
    assert!(
        mcp_tools.contains("include_hidden") && mcp_tools.contains("include_background"),
        "computer/list_apps must expose explicit hidden/background inventory options"
    );
    assert!(
        runtime.contains("list_running_apps"),
        "the runtime bridge trait must own running-app enumeration"
    );
    assert!(
        bridge.contains("ListRunningApps"),
        "the GPUI bridge must carry running-app requests to the AppKit side"
    );
    assert!(
        bridge.contains("runningApplications"),
        "running app inventory should use NSWorkspace runningApplications on the GPUI side"
    );

    for needle in ["NSWorkspace", "runningApplications"] {
        assert!(
            !mcp_tools.contains(needle),
            "the MCP handler must not call AppKit directly; found {}",
            needle
        );
    }

    for needle in [
        "scan_applications",
        "scan_applications_fresh",
        "APP_CACHE",
        "scan_all_directories",
        "PlistBuddy",
        "APP_DIRECTORIES",
    ] {
        assert!(
            !mcp_tools.contains(needle),
            "computer/list_apps must not use installed app scan/cache; found {}",
            needle
        );
    }

    for needle in [
        "PROCESS_MANAGER",
        "get_active_processes",
        "active_processes",
        "script_path",
    ] {
        assert!(
            !mcp_tools.contains(needle),
            "computer/list_apps must not mean Script Kit child processes; found {}",
            needle
        );
    }

    let combined = format!("{mcp_tools}\n{bridge}");
    for needle in [
        "activateWithOptions",
        "activateIgnoringOtherApps",
        "terminate",
        "forceTerminate",
        " hide]",
        " unhide]",
        "launchApplication",
        "openApplicationAtURL",
        "AXUIElementPerformAction",
        "CGEvent",
        "simulateKey",
        "focus_window",
        "move_window",
        "resize_window",
        "Command::new(\"open\")",
    ] {
        assert!(
            !combined.contains(needle),
            "computer/list_apps must remain observation-only; found {}",
            needle
        );
    }
}
