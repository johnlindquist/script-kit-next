// doc-anchor-removed: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_tray_menu_reads_script_kit_tray_model_only() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let tray = std::fs::read_to_string("src/tray/mod.rs").expect("read tray/mod.rs");
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");

    assert!(
        mcp_tools.contains("COMPUTER_LIST_TRAY_MENU_TOOL"),
        "computer/list_tray_menu must be registered through the computer-use MCP tool module"
    );
    assert!(
        mcp_tools.contains("fn handle_list_tray_menu("),
        "computer/list_tray_menu must have a dedicated handler"
    );
    assert!(
        mcp_tools.contains("computer_list_tray_menu_input_schema"),
        "computer/list_tray_menu must expose a closed input schema"
    );
    assert!(
        mcp_tools.contains("crate::tray::current_tray_menu_observation_snapshot()"),
        "computer/list_tray_menu must read the current tray menu observation snapshot directly"
    );
    assert!(
        tray.contains("pub fn current_tray_menu_observation_snapshot()"),
        "tray menu observation must expose a direct current snapshot accessor"
    );
    assert!(
        tray.contains("pub(crate) fn tray_menu_observation_snapshot("),
        "tray menu observation must be built by a pure model/state snapshot helper"
    );
    assert!(
        mcp_tools.contains("COMPUTER_LIST_TRAY_MENU_TOOL => handle_list_tray_menu(arguments),"),
        "computer/list_tray_menu must route without passing the runtime bridge"
    );
    assert!(
        mcp_tools.contains("fn handle_list_tray_menu(arguments: &Value) -> ToolResult {"),
        "computer/list_tray_menu handler must not accept a runtime bridge parameter"
    );
    for (path, source) in [
        ("src/computer_use/runtime_bridge.rs", runtime.as_str()),
        ("src/computer_use/gpui_runtime_bridge.rs", bridge.as_str()),
    ] {
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        for needle in [
            "fn list_tray_menu",
            "list_tray_menu(",
            "ListTrayMenu",
            "TrayMenuObservation",
        ] {
            assert!(
                !production_source.contains(needle),
                "{path} must not expose stale computer/list_tray_menu runtime bridge surface; found {needle}"
            );
        }
    }

    let handler_body = extract_function_body(&mcp_tools, "fn handle_list_tray_menu(");
    for needle in [
        "ComputerUseRuntimeBridge",
        "runtime.",
        "Option<&dyn",
        "runtime_unavailable",
        "list_tray_menu()",
        "MenuEvent::receiver",
        "action_from_event",
        "handle_action",
        "Command::new(\"open\")",
        "AXUIElement",
        "CGEvent",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "get_menu_bar_for_pid",
        "window_control",
        "click",
        "execute",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/list_tray_menu handler must not open, click, execute, prompt, or enumerate native windows; found {}",
            needle
        );
    }

    let current_helper_body =
        extract_function_body(&tray, "pub fn current_tray_menu_observation_snapshot()");
    for needle in [
        "set_text",
        "set_enabled",
        "set_menu",
        ".append(",
        "MenuEvent",
        "handle_action",
        "Command::new(\"open\")",
        "AXUIElement",
        "CGEvent",
        "TrayIconBuilder",
        "create_menu",
        "template_menu_items",
    ] {
        assert!(
            !current_helper_body.contains(needle),
            "current_tray_menu_observation_snapshot must remain a pure model/state read; found {}",
            needle
        );
    }

    let helper_body = extract_function_body(&tray, "pub(crate) fn tray_menu_observation_snapshot(");
    for needle in [
        "set_text",
        "set_enabled",
        "set_menu",
        ".append(",
        "MenuEvent",
        "handle_action",
        "Command::new(\"open\")",
        "AXUIElement",
        "CGEvent",
        "TrayIconBuilder",
        "create_menu",
        "template_menu_items",
    ] {
        assert!(
            !helper_body.contains(needle),
            "tray_menu_observation_snapshot must remain a pure model/state read; found {}",
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
