// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_tray_menu_item_by_id_is_read_only_tray_model_lookup() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");

    assert!(
        mcp_tools.contains(
            "pub const COMPUTER_GET_TRAY_MENU_ITEM_BY_ID_TOOL: &str = \"computer/get_tray_menu_item_by_id\";"
        ),
        "computer/get_tray_menu_item_by_id must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_GET_TRAY_MENU_ITEM_BY_ID_TOOL => handle_get_tray_menu_item_by_id(arguments),"
        ),
        "computer/get_tray_menu_item_by_id must route through a runtime-free handler"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetTrayMenuItemByIdArgs");
    assert!(
        mcp_tools
            .contains("#[serde(deny_unknown_fields)]\nstruct ComputerUseGetTrayMenuItemByIdArgs"),
        "computer/get_tray_menu_item_by_id args must reject unknown fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["id: String,"],
        "computer/get_tray_menu_item_by_id args must expose exactly one id field"
    );

    let schema_body = extract_function_body(
        &mcp_tools,
        "fn computer_get_tray_menu_item_by_id_input_schema()",
    );
    for needle in [
        "\"additionalProperties\": false",
        "\"id\"",
        "\"type\": \"string\"",
        "\"minLength\": 1",
        "\"required\": [\"id\"]",
    ] {
        assert!(
            schema_body.contains(needle),
            "computer/get_tray_menu_item_by_id schema missing {needle}"
        );
    }
    assert_eq!(
        extract_json_object_block(schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        1,
        "computer/get_tray_menu_item_by_id schema properties must contain exactly one field"
    );
    for needle in [
        "\"sectionIndex\"",
        "\"itemIndex\"",
        "\"title\"",
        "\"click\"",
        "\"press\"",
        "\"execute\"",
        "\"open\"",
        "\"refresh\"",
        "\"includeGlobalStatusItems\"",
    ] {
        assert!(
            !schema_body.contains(needle),
            "computer/get_tray_menu_item_by_id input must stay id-only; found {needle}"
        );
    }

    let result_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseGetTrayMenuItemByIdResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "owner: crate::tray::TrayMenuOwnerObservation,",
            "id: String,",
            "section_index: Option<usize>,",
            "item_index: Option<usize>,",
            "section: Option<ComputerUseTrayMenuSectionSummary>,",
            "item: Option<crate::tray::TrayMenuItemObservation>,",
            "warnings: Vec<String>,",
        ],
        "computer/get_tray_menu_item_by_id result must stay a read-only tray item observation"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_tray_menu_item_by_id(");
    for needle in [
        "ComputerUseGetTrayMenuItemByIdArgs",
        "args.id.is_empty()",
        "crate::tray::current_tray_menu_observation_snapshot()",
        "snapshot.sections.iter().enumerate()",
        "section.items.iter().enumerate()",
        "item.id == args.id",
        "computer_use_tray_menu_section_summary(section)",
        "item.clone()",
        "source: \"scriptKitTrayMenuModel\"",
        "scope: \"ownTrayMenuItemId\"",
        "\"found\"",
        "\"notFound\"",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/get_tray_menu_item_by_id handler must contain {needle}"
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
            "computer/get_tray_menu_item_by_id handler must not open, click, execute, prompt, or enumerate native menus; found {needle}"
        );
    }

    for needle in [
        "GetTrayMenuItemById",
        "get_tray_menu_item_by_id",
        "ComputerUseGetTrayMenuItemByIdRequest",
        "ComputerUseGetTrayMenuItemByIdSnapshot",
        "ListTrayMenu",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/get_tray_menu_item_by_id must not add runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/get_tray_menu_item_by_id must not add GPUI bridge surface; found {needle}"
        );
    }

    assert!(
        protocol
            .contains("`computer/get_tray_menu_item_by_id` accepts a closed `{id:string}` input"),
        "protocol docs must describe the get_tray_menu_item_by_id input contract"
    );
    for needle in [
        "source:\"scriptKitTrayMenuModel\"",
        "scope:\"ownTrayMenuItemId\"",
        "status:\"found\"|\"notFound\"",
        "stable ids from `computer/list_tray_menu`",
        "same tray menu model",
        "does not require the runtime bridge",
        "open the tray menu",
        "click status items",
        "execute tray actions",
        "enumerate global menu extras",
        "refresh native menu state",
        "request permissions",
        "ComputerUseRuntimeBridge::get_tray_menu_item_by_id",
    ] {
        assert!(
            protocol.contains(needle),
            "protocol docs must pin get_tray_menu_item_by_id boundary: {needle}"
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
    let marker_start = source.find(marker).expect("json marker");
    let open = source[marker_start..].find('{').expect("json object open") + marker_start;
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
        .filter(|line| {
            !line.is_empty()
                && !matches!(
                    *line,
                    "{" | "}"
                        | "#[derive(serde::Deserialize)]"
                        | "#[derive(serde::Serialize)]"
                        | "#[serde(rename_all = \"camelCase\")]"
                        | "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]"
                        | "#[serde(deny_unknown_fields)]"
                )
                && !line.starts_with("struct ")
        })
        .map(ToOwned::to_owned)
        .collect()
}
