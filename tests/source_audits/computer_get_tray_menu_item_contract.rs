// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_tray_menu_item_is_read_only_tray_model_lookup() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");

    assert!(
        mcp_tools.contains(
            "pub const COMPUTER_GET_TRAY_MENU_ITEM_TOOL: &str = \"computer/get_tray_menu_item\";"
        ),
        "computer/get_tray_menu_item must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools
            .contains("COMPUTER_GET_TRAY_MENU_ITEM_TOOL => handle_get_tray_menu_item(arguments),"),
        "computer/get_tray_menu_item must route through a runtime-free handler"
    );
    assert!(
        mcp_tools.contains("computer_get_tray_menu_item_input_schema"),
        "computer/get_tray_menu_item must expose a dedicated input schema"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetTrayMenuItemArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\nstruct ComputerUseGetTrayMenuItemArgs"
        ),
        "computer/get_tray_menu_item args must reject unknown fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["section_index: usize,", "item_index: usize,"],
        "computer/get_tray_menu_item args must expose exactly sectionIndex and itemIndex"
    );

    let schema_body =
        extract_function_body(&mcp_tools, "fn computer_get_tray_menu_item_input_schema()");
    for needle in [
        "\"additionalProperties\": false",
        "\"sectionIndex\"",
        "\"itemIndex\"",
        "\"type\": \"integer\"",
        "\"minimum\": 0",
        "\"required\": [\"sectionIndex\", \"itemIndex\"]",
    ] {
        assert!(
            schema_body.contains(needle),
            "computer/get_tray_menu_item schema missing {needle}"
        );
    }
    assert_eq!(
        extract_json_object_block(schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        2,
        "computer/get_tray_menu_item schema properties must contain exactly two fields"
    );
    for needle in [
        "\"click\"",
        "\"press\"",
        "\"execute\"",
        "\"open\"",
        "\"refresh\"",
        "\"focus\"",
        "\"activate\"",
    ] {
        assert!(
            !schema_body.contains(needle),
            "computer/get_tray_menu_item input must stay index-only; found {needle}"
        );
    }

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetTrayMenuItemResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "owner: crate::tray::TrayMenuOwnerObservation,",
            "section_index: usize,",
            "item_index: usize,",
            "section: Option<ComputerUseTrayMenuSectionSummary>,",
            "item: Option<crate::tray::TrayMenuItemObservation>,",
            "warnings: Vec<String>,",
        ],
        "computer/get_tray_menu_item result must stay a read-only tray item observation"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_tray_menu_item(");
    for needle in [
        "ComputerUseGetTrayMenuItemArgs",
        "crate::tray::current_tray_menu_observation_snapshot()",
        "snapshot.sections.get(args.section_index)",
        "section.items.get(args.item_index).cloned()",
        "computer_use_tray_menu_section_summary",
        "source: \"scriptKitTrayMenuModel\"",
        "scope: \"ownTrayMenuSectionItemIndex\"",
        "\"found\"",
        "\"sectionNotFound\"",
        "\"itemNotFound\"",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/get_tray_menu_item handler must contain {needle}"
        );
    }
    for needle in [
        "ComputerUseRuntimeBridge",
        "runtime.",
        "Option<&dyn",
        "runtime_unavailable",
        "MenuEvent::receiver",
        "action_from_event",
        "handle_action",
        "Command::new(\"open\")",
        "popUpContextMenu",
        "AXUIElement",
        "AXUIElementPerformAction",
        "CGEvent",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "get_menu_bar_for_pid",
        "clipboard",
        "pasteboard",
        "click",
        "press",
        "execute",
        "focus",
        "activate",
        "refresh",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/get_tray_menu_item handler must not open, click, execute, prompt, or enumerate native menus; found {needle}"
        );
    }

    let summary_body =
        extract_function_body(&mcp_tools, "fn computer_use_tray_menu_section_summary(");
    for needle in [
        "id: section.id",
        "label: section.label",
        "item_count: section.items.len()",
    ] {
        assert!(
            summary_body.contains(needle),
            "tray section summary must expose stable section metadata; missing {needle}"
        );
    }
    for needle in [
        "click",
        "press",
        "execute",
        "action_from_event",
        "handle_action",
    ] {
        assert!(
            !summary_body.contains(needle),
            "tray section summary must not expose executable behavior; found {needle}"
        );
    }

    for needle in [
        "GetTrayMenuItem",
        "get_tray_menu_item",
        "ComputerUseGetTrayMenuItemRequest",
        "ComputerUseGetTrayMenuItemSnapshot",
        "ListTrayMenu",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/get_tray_menu_item must not add runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/get_tray_menu_item must not add GPUI bridge surface; found {needle}"
        );
    }

    assert!(
        protocol.contains(
            "`computer/get_tray_menu_item` accepts a closed `{sectionIndex:number,itemIndex:number}` input"
        ),
        "protocol docs must describe the get_tray_menu_item input contract"
    );
    for needle in [
        "source:\"scriptKitTrayMenuModel\"",
        "scope:\"ownTrayMenuSectionItemIndex\"",
        "status:\"found\"|\"sectionNotFound\"|\"itemNotFound\"",
        "same tray menu model",
        "does not require the runtime bridge",
        "open the tray menu",
        "click status items",
        "execute tray actions",
        "enumerate global menu extras",
        "refresh native menu state",
        "request permissions",
        "ComputerUseRuntimeBridge::get_tray_menu_item",
    ] {
        assert!(
            protocol.contains(needle),
            "protocol docs must pin get_tray_menu_item non-goal: {needle}"
        );
    }
}

fn extract_struct_block<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("struct signature");
    let open = source[start..].find('{').expect("struct open brace") + start;
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

    panic!("struct block for {signature} did not close")
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

    panic!("function body for {signature} did not close")
}

fn extract_json_object_block<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source.find(marker).expect("json marker");
    let open = source[start..].find('{').expect("json object open") + start;
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

    panic!("json object for {marker} did not close")
}

fn field_declarations(block: &str) -> Vec<String> {
    block
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(','))
        .map(ToString::to_string)
        .collect()
}
