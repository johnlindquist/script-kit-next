// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_menu_item_by_index_path_is_cached_read_only_lookup() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");

    assert!(
        mcp_tools.contains(
            "pub const COMPUTER_GET_MENU_ITEM_BY_INDEX_PATH_TOOL: &str = \"computer/get_menu_item_by_index_path\";"
        ),
        "computer/get_menu_item_by_index_path must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_GET_MENU_ITEM_BY_INDEX_PATH_TOOL => handle_get_menu_item_by_index_path(arguments),"
        ),
        "computer/get_menu_item_by_index_path must route through a runtime-free handler"
    );

    let args_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseGetMenuItemByIndexPathArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\nstruct ComputerUseGetMenuItemByIndexPathArgs"
        ),
        "computer/get_menu_item_by_index_path args must reject unknown fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["index_path: Vec<usize>,"],
        "computer/get_menu_item_by_index_path args must expose exactly one indexPath field"
    );

    let schema_body = extract_function_body(
        &mcp_tools,
        "fn computer_get_menu_item_by_index_path_input_schema()",
    );
    for needle in [
        "\"additionalProperties\": false",
        "\"indexPath\"",
        "\"type\": \"array\"",
        "\"minItems\": 1",
        "\"type\": \"integer\"",
        "\"minimum\": 0",
        "\"required\": [\"indexPath\"]",
    ] {
        assert!(
            schema_body.contains(needle),
            "computer/get_menu_item_by_index_path schema missing {needle}"
        );
    }
    for needle in [
        "\"path\"",
        "\"pid\"",
        "\"bundleId\"",
        "\"click\"",
        "\"press\"",
        "\"execute\"",
        "\"refresh\"",
        "\"focus\"",
        "\"activate\"",
    ] {
        assert!(
            !schema_body.contains(needle),
            "computer/get_menu_item_by_index_path input must stay indexPath-only; found {needle}"
        );
    }

    let result_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseGetMenuItemByIndexPathResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "app: Option<ComputerUseMenuApp>,",
            "cache: ComputerUseMenuCache,",
            "index_path: Vec<usize>,",
            "resolved_path: Option<Vec<String>>,",
            "item: Option<ComputerUseMenuItem>,",
            "warnings: Vec<String>,",
        ],
        "computer/get_menu_item_by_index_path result must stay a cached menu observation"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_menu_item_by_index_path(");
    for needle in [
        "ComputerUseGetMenuItemByIndexPathArgs",
        "args.index_path.is_empty()",
        "get_cached_menu_snapshot()",
        "find_cached_menu_item_by_index_path(&snapshot.items, &args.index_path)",
        "computer_use_menu_item(item)",
        "source: \"frontmostAppTrackerCache\"",
        "scope: \"cachedMenuIndexPath\"",
        "\"found\"",
        "\"notFound\"",
        "\"noTrackedApp\"",
        "\"noCachedMenus\"",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/get_menu_item_by_index_path handler must contain {needle}"
        );
    }
    for needle in [
        "ComputerUseRuntimeBridge",
        "runtime.",
        "Option<&dyn",
        "runtime_unavailable",
        "get_menu_bar_for_pid",
        "menu_executor",
        "AXUIElement",
        "AXUIElementPerformAction",
        "CGEvent",
        "request_accessibility_permission",
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
            "computer/get_menu_item_by_index_path handler must stay cached read-only; found {needle}"
        );
    }

    let helper_body = extract_function_body(&mcp_tools, "fn find_cached_menu_item_by_index_path");
    for needle in [
        "index_path.split_first()",
        "items.get(*head)",
        "item.children",
        "item.title.clone()",
    ] {
        assert!(
            helper_body.contains(needle),
            "index-path helper must contain {needle}"
        );
    }

    for needle in [
        "GetMenuItemByIndexPath",
        "get_menu_item_by_index_path",
        "ComputerUseGetMenuItemByIndexPathRequest",
        "ComputerUseGetMenuItemByIndexPathSnapshot",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/get_menu_item_by_index_path must not add runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/get_menu_item_by_index_path must not add GPUI bridge surface; found {needle}"
        );
    }

    assert!(
        protocol.contains(
            "`computer/get_menu_item_by_index_path` accepts a closed `{indexPath:number[]}` input"
        ),
        "protocol docs must describe the get_menu_item_by_index_path input contract"
    );
    for needle in [
        "source:\"frontmostAppTrackerCache\"",
        "scope:\"cachedMenuIndexPath\"",
        "status:\"found\"|\"notFound\"|\"noTrackedApp\"|\"noCachedMenus\"",
        "zero-based recursive positions from `computer/list_menus`",
        "does not refresh AX state",
        "focus or activate apps",
        "request permissions",
        "press or click menu items",
        "expose action handles",
        "enumerate status items",
        "add a runtime bridge method",
        "use `menu_executor`",
    ] {
        assert!(
            protocol.contains(needle),
            "protocol docs must pin get_menu_item_by_index_path boundary: {needle}"
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
