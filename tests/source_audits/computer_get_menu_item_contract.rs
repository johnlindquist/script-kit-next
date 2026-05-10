// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_get_menu_item_is_cached_read_only_path_lookup() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");
    let mcp_protocol =
        std::fs::read_to_string("src/mcp_protocol/mod.rs").expect("read mcp_protocol/mod.rs");

    assert!(
        mcp_tools
            .contains("pub const COMPUTER_GET_MENU_ITEM_TOOL: &str = \"computer/get_menu_item\";"),
        "computer/get_menu_item must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains("COMPUTER_GET_MENU_ITEM_TOOL => handle_get_menu_item(arguments),"),
        "computer/get_menu_item must route through its cache-only handler"
    );
    assert!(
        mcp_tools.contains("name: COMPUTER_GET_MENU_ITEM_TOOL.to_string()"),
        "computer/get_menu_item must be registered in the static tool catalog"
    );
    assert!(
        mcp_protocol.contains("tool_names.contains(&\"computer/get_menu_item\")"),
        "tools/list tests must expect computer/get_menu_item"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetMenuItemArgs");
    assert!(
        mcp_tools.contains("#[serde(deny_unknown_fields)]\nstruct ComputerUseGetMenuItemArgs"),
        "computer/get_menu_item args must reject unknown fields"
    );
    assert_eq!(
        field_declarations(args_struct),
        vec!["path: Vec<String>,"],
        "computer/get_menu_item args must expose exactly one exact-title path field"
    );

    let input_schema_body =
        extract_function_body(&mcp_tools, "fn computer_get_menu_item_input_schema()");
    assert!(
        input_schema_body.contains("\"additionalProperties\": false"),
        "computer/get_menu_item must reject unknown input fields"
    );
    assert!(
        input_schema_body.contains("\"path\""),
        "computer/get_menu_item schema must expose path"
    );
    assert!(
        input_schema_body.contains("\"type\": \"array\""),
        "computer/get_menu_item path must be an array"
    );
    assert!(
        input_schema_body.contains("\"minItems\": 1"),
        "computer/get_menu_item path must require at least one segment"
    );
    assert!(
        input_schema_body.contains("\"type\": \"string\""),
        "computer/get_menu_item path items must be strings"
    );
    assert!(
        input_schema_body.contains("\"minLength\": 1"),
        "computer/get_menu_item path segments must be non-empty"
    );
    assert!(
        input_schema_body.contains("\"required\": [\"path\"]"),
        "computer/get_menu_item must require path"
    );
    for needle in [
        "\"pid\"",
        "\"bundleId\"",
        "\"target\"",
        "\"refresh\"",
        "\"focus\"",
        "\"activate\"",
        "\"click\"",
        "\"execute\"",
        "\"includeDisabled\"",
    ] {
        assert!(
            !input_schema_body.contains(needle),
            "computer/get_menu_item input must stay exact-path only; found {}",
            needle
        );
    }

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseGetMenuItemResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "source: &'static str,",
            "scope: &'static str,",
            "status: &'static str,",
            "app: Option<ComputerUseMenuApp>,",
            "cache: ComputerUseMenuCache,",
            "path: Vec<String>,",
            "item: Option<ComputerUseMenuItem>,",
            "warnings: Vec<String>,",
        ],
        "computer/get_menu_item result must expose exactly schemaVersion/source/scope/status/app/cache/path/item/warnings"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_get_menu_item(");
    assert!(
        handler_body.contains("ComputerUseGetMenuItemArgs"),
        "handler must parse the dedicated path args"
    );
    assert!(
        handler_body.contains("args.path.is_empty()"),
        "handler must reject empty paths beyond serde parsing"
    );
    assert!(
        handler_body.contains("args.path.iter().any(|segment| segment.is_empty())"),
        "handler must reject empty path segments beyond serde parsing"
    );
    assert!(
        handler_body.contains("get_cached_menu_snapshot()"),
        "handler must read the cached menu snapshot"
    );
    assert!(
        handler_body.contains("find_cached_menu_item_by_path(&snapshot.items, &args.path)"),
        "handler must perform exact title-path lookup over the cached tree"
    );
    assert!(
        handler_body.contains("source: \"frontmostAppTrackerCache\""),
        "handler must identify the frontmost app tracker cache as source"
    );
    assert!(
        handler_body.contains("scope: \"cachedMenuPath\""),
        "handler must identify exact cached-menu path scope"
    );
    for status in [
        "\"found\"",
        "\"notFound\"",
        "\"noTrackedApp\"",
        "\"noCachedMenus\"",
    ] {
        assert!(
            handler_body.contains(status),
            "handler must expose {} status",
            status
        );
    }
    for needle in [
        "ComputerUseRuntimeBridge",
        "ComputerUseGetMenuItemRequest",
        "runtime.inspect_automation_window",
        "list_running_apps",
        "list_app_windows",
        "AXUIElement",
        "AXPress",
        "menu_executor",
        "click",
        "press",
        "execute",
        "focus",
        "activate",
        "refresh",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "listMenuExtras",
        "listMenuBarItems",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/get_menu_item handler must stay a cache-only read; found {}",
            needle
        );
    }

    let lookup_body = extract_function_body(&mcp_tools, "fn find_cached_menu_item_by_path");
    assert!(
        lookup_body.contains("path.split_first()"),
        "lookup helper must split the exact path into current segment and tail"
    );
    assert!(
        lookup_body.contains("items.iter().find(|item| item.title == *head)"),
        "lookup helper must match exact cached titles"
    );
    assert!(
        lookup_body.contains("find_cached_menu_item_by_path(&item.children, tail)"),
        "lookup helper must recurse into cached children"
    );
    for needle in ["to_lowercase", "contains(", "starts_with", "fuzzy"] {
        assert!(
            !lookup_body.contains(needle),
            "lookup helper must not perform fuzzy or partial matching; found {}",
            needle
        );
    }

    for (label, source) in [
        ("runtime bridge", runtime.as_str()),
        ("GPUI runtime bridge", bridge.as_str()),
    ] {
        for needle in [
            "GetMenuItem",
            "get_menu_item",
            "ComputerUseGetMenuItemRequest",
            "ComputerUseGetMenuItemSnapshot",
            "ListMenus",
            "RefreshMenus",
        ] {
            assert!(
                !source.contains(needle),
                "computer/get_menu_item must not add a dedicated {label} surface; found {needle}"
            );
        }
    }

    assert!(
        protocol.contains("computer/get_menu_item"),
        "protocol docs must mention computer/get_menu_item"
    );
    assert!(
        protocol.contains("status:\"found\"|\"notFound\"|\"noTrackedApp\"|\"noCachedMenus\""),
        "protocol docs must pin computer/get_menu_item statuses"
    );
    assert!(
        protocol.contains("scope:\"cachedMenuPath\""),
        "protocol docs must pin computer/get_menu_item scope"
    );
    assert!(
        protocol.contains("does not refresh AX state, focus or activate apps, request permissions, press or click menu items, expose menu action handles, enumerate status/menu extras, add a runtime bridge method, or use `menu_executor`"),
        "protocol docs must pin the no-action/no-refresh/no-new-bridge contract"
    );
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

    panic!("struct block for {} did not close", signature)
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

fn field_declarations(block: &str) -> Vec<String> {
    block
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(','))
        .map(ToString::to_string)
        .collect()
}
