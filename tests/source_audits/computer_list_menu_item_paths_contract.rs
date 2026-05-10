// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_menu_item_paths_is_cached_read_only_flat_lookup() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");

    assert!(
        mcp_tools.contains(
            "pub const COMPUTER_LIST_MENU_ITEM_PATHS_TOOL: &str = \"computer/list_menu_item_paths\";"
        ),
        "computer/list_menu_item_paths must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_LIST_MENU_ITEM_PATHS_TOOL => handle_list_menu_item_paths(arguments),"
        ),
        "computer/list_menu_item_paths must route through a runtime-free handler"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseListMenuItemPathsArgs");
    assert!(
        mcp_tools.contains(
            "#[derive(serde::Deserialize)]\n#[serde(deny_unknown_fields)]\nstruct ComputerUseListMenuItemPathsArgs"
        ),
        "computer/list_menu_item_paths args must reject unknown fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        Vec::<String>::new(),
        "computer/list_menu_item_paths args must expose no fields"
    );

    let schema_body = extract_function_body(
        &mcp_tools,
        "fn computer_list_menu_item_paths_input_schema()",
    );
    for needle in [
        "\"type\": \"object\"",
        "\"additionalProperties\": false",
        "\"properties\": {}",
    ] {
        assert!(
            schema_body.contains(needle),
            "computer/list_menu_item_paths schema missing {needle}"
        );
    }
    for needle in [
        "\"path\"",
        "\"indexPath\"",
        "\"pid\"",
        "\"bundleId\"",
        "\"click\"",
        "\"press\"",
        "\"execute\"",
        "\"refresh\"",
        "\"focus\"",
        "\"activate\"",
        "\"includeDisabled\"",
        "\"includeGlobalStatusItems\"",
    ] {
        assert!(
            !schema_body.contains(needle),
            "computer/list_menu_item_paths input must stay empty; found {needle}"
        );
    }

    let result_struct =
        extract_struct_block(&mcp_tools, "struct ComputerUseListMenuItemPathsResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "app: Option<ComputerUseMenuApp>,",
            "cache: ComputerUseMenuCache,",
            "items: Vec<ComputerUseMenuItemPath>,",
            "warnings: Vec<String>,",
        ],
        "computer/list_menu_item_paths result must stay a cached menu observation"
    );
    let item_struct = extract_struct_block(&mcp_tools, "struct ComputerUseMenuItemPath");
    assert_eq!(
        field_declarations(item_struct),
        vec![
            "index_path: Vec<usize>,",
            "path: Vec<String>,",
            "title: String,",
            "enabled: bool,",
            "shortcut: Option<String>,",
            "child_count: usize,",
        ],
        "computer/list_menu_item_paths items must stay path references"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_list_menu_item_paths(");
    for needle in [
        "ComputerUseListMenuItemPathsArgs",
        "get_cached_menu_snapshot()",
        "flatten_cached_menu_item_paths(",
        "source: \"frontmostAppTrackerCache\"",
        "scope: \"cachedMenuItemPaths\"",
        "\"listed\"",
        "\"noTrackedApp\"",
        "\"noCachedMenus\"",
    ] {
        assert!(
            handler_body.contains(needle),
            "computer/list_menu_item_paths handler must contain {needle}"
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
            "computer/list_menu_item_paths handler must stay cached read-only; found {needle}"
        );
    }

    let helper_body = extract_function_body(&mcp_tools, "fn flatten_cached_menu_item_paths(");
    for needle in [
        "for (index, item) in items.iter().enumerate()",
        "title_prefix.push(item.title.clone())",
        "index_prefix.push(index)",
        "index_path: index_prefix.clone()",
        "path: title_prefix.clone()",
        "child_count: item.children.len()",
        "flatten_cached_menu_item_paths(&item.children",
        "index_prefix.pop()",
        "title_prefix.pop()",
    ] {
        assert!(
            helper_body.contains(needle),
            "flattened menu path helper must contain {needle}"
        );
    }
    for needle in [
        "ax_element_path",
        "AXUIElement",
        "menu_executor",
        "click",
        "press",
    ] {
        assert!(
            !helper_body.contains(needle),
            "flattened menu path helper must stay observational; found {needle}"
        );
    }

    for needle in [
        "ListMenuItemPaths",
        "list_menu_item_paths",
        "ComputerUseListMenuItemPathsRequest",
        "ComputerUseListMenuItemPathsSnapshot",
    ] {
        assert!(
            !runtime.contains(needle),
            "computer/list_menu_item_paths must not add runtime bridge surface; found {needle}"
        );
        assert!(
            !bridge.contains(needle),
            "computer/list_menu_item_paths must not add GPUI bridge surface; found {needle}"
        );
    }

    assert!(
        protocol.contains(
            "`computer/list_menu_item_paths` accepts no arguments and returns flattened cached menu references"
        ),
        "protocol docs must describe the list_menu_item_paths input contract"
    );
    for needle in [
        "source:\"frontmostAppTrackerCache\"",
        "scope:\"cachedMenuItemPaths\"",
        "status:\"listed\"|\"noTrackedApp\"|\"noCachedMenus\"",
        "zero-based recursive `indexPath` values usable with `computer/get_menu_item_by_index_path`",
        "depth-first preorder",
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
            "protocol docs must pin list_menu_item_paths boundary: {needle}"
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
                        | "{}"
                        | "#[derive(serde::Deserialize)]"
                        | "#[derive(serde::Serialize)]"
                        | "#[serde(rename_all = \"camelCase\")]"
                        | "#[serde(deny_unknown_fields)]"
                        | "#[serde(skip_serializing_if = \"Option::is_none\")]"
                )
                && !line.starts_with("struct ")
        })
        .map(ToOwned::to_owned)
        .collect()
}
