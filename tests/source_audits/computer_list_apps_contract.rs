// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_apps_is_closed_runtime_bridged_inventory() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let runtime =
        std::fs::read_to_string("src/computer_use/runtime_bridge.rs").expect("read runtime");
    let bridge =
        std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs").expect("read bridge");
    let app_run_setup =
        std::fs::read_to_string("src/main_entry/app_run_setup.rs").expect("read app_run_setup");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");

    assert!(
        mcp_tools.contains("pub const COMPUTER_LIST_APPS_TOOL: &str = \"computer/list_apps\";"),
        "computer/list_apps must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains("COMPUTER_LIST_APPS_TOOL => handle_list_apps(arguments, runtime),"),
        "computer/list_apps must route through the runtime-bridged handler"
    );
    assert!(
        mcp_tools.contains("computer_list_apps_input_schema"),
        "computer/list_apps must expose a dedicated input schema"
    );
    assert!(
        mcp_tools.contains("COMPUTER_APPS_SCHEMA_VERSION"),
        "computer/list_apps must expose a schema version"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseListAppsArgs");
    assert!(
        mcp_tools.contains(
            "#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\nstruct ComputerUseListAppsArgs"
        ),
        "computer/list_apps args must reject unknown fields and use camelCase"
    );
    assert!(
        args_struct.contains("#[serde(default)]\n    include_hidden: bool"),
        "includeHidden must default to false via serde"
    );
    assert!(
        args_struct.contains("#[serde(default)]\n    include_background: bool"),
        "includeBackground must default to false via serde"
    );
    let arg_fields: Vec<&str> = args_struct
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(": bool,"))
        .collect();
    assert_eq!(
        arg_fields,
        vec!["include_hidden: bool,", "include_background: bool,"],
        "computer/list_apps args must expose exactly the two include flags"
    );

    let input_schema_body =
        extract_function_body(&mcp_tools, "fn computer_list_apps_input_schema()");
    assert!(
        input_schema_body.contains("\"additionalProperties\": false"),
        "computer/list_apps must reject unknown input fields"
    );
    for field in ["\"includeHidden\"", "\"includeBackground\""] {
        assert!(
            input_schema_body.contains(field),
            "computer/list_apps schema missing {}",
            field
        );
    }
    assert_eq!(
        input_schema_body.matches("\"default\": false").count(),
        2,
        "both computer/list_apps flags must default to false"
    );
    assert_eq!(
        extract_json_object_block(input_schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        2,
        "computer/list_apps schema properties must contain exactly two fields"
    );
    for needle in [
        "\"pid\"",
        "\"bundleId\"",
        "\"name\"",
        "\"target\"",
        "\"focus\"",
        "\"activate\"",
        "\"launch\"",
        "\"quit\"",
        "\"hide\"",
        "\"click\"",
        "\"scroll\"",
        "\"hotkey\"",
        "\"move\"",
        "\"resize\"",
        "\"screenshot\"",
    ] {
        assert!(
            !input_schema_body.contains(needle),
            "computer/list_apps input must stay two-flag only; found {}",
            needle
        );
    }

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseListAppsResult");
    for field in [
        "schema_version: u32",
        "apps: Vec<ComputerUseRunningAppInfo>",
        "frontmost_pid: Option<i32>",
    ] {
        assert!(
            result_struct.contains(field),
            "computer/list_apps result missing {}",
            field
        );
    }

    let app_info_struct = extract_struct_block(&runtime, "struct ComputerUseRunningAppInfo");
    for field in [
        "pid: i32",
        "bundle_id: Option<String>",
        "name: String",
        "is_active: bool",
        "is_hidden: bool",
        "activation_policy: String",
    ] {
        assert!(
            app_info_struct.contains(field),
            "running app metadata missing {}",
            field
        );
    }
    for needle in [
        "action",
        "click",
        "press",
        "execute",
        "focus",
        "launch",
        "quit",
        "terminate",
        "force_terminate",
        "open_url",
    ] {
        assert!(
            !app_info_struct.contains(needle),
            "running app metadata must not expose executable/action fields; found {}",
            needle
        );
    }

    let handler_body = extract_function_body(&mcp_tools, "fn handle_list_apps(");
    assert!(
        handler_body.contains("ComputerUseListAppsArgs"),
        "handler must parse the dedicated two-flag args"
    );
    assert!(
        handler_body.contains("let Some(runtime) = runtime"),
        "handler must require the live runtime bridge"
    );
    assert!(
        handler_body.contains("ComputerUseListAppsRequest {\n        include_hidden: args.include_hidden,\n        include_background: args.include_background,\n    }"),
        "handler must pass only the include flags into the runtime request"
    );
    assert!(
        handler_body.contains("runtime.list_running_apps(request)"),
        "handler must delegate app enumeration to the runtime bridge"
    );
    for needle in [
        "process_manager",
        "read_scripts",
        "load_scriptlets",
        "Command::new(\"open\")",
        "NSWorkspace",
        "activateWithOptions",
        "activateIgnoringOtherApps",
        "launchApplication",
        "openApplicationAtURL",
        "terminate",
        "forceTerminate",
        "CGEvent",
        "AXUIElementPerformAction",
        "AXPress",
        "CGRequestScreenCaptureAccess",
        "request_accessibility_permission",
    ] {
        assert!(
            !handler_body.contains(needle),
            "MCP handler must not perform native/action/catalog work directly; found {}",
            needle
        );
    }

    assert!(
        runtime.contains("struct ComputerUseListAppsRequest"),
        "runtime bridge must define a list-apps request"
    );
    assert!(
        runtime.contains("struct ComputerUseListAppsSnapshot"),
        "runtime bridge must define a list-apps snapshot"
    );
    assert!(
        runtime.contains("struct ComputerUseRunningAppInfo"),
        "runtime bridge must define running app metadata"
    );
    assert!(
        runtime.contains("fn list_running_apps("),
        "runtime bridge trait must own running-app enumeration"
    );
    assert!(
        bridge.contains("ListRunningApps"),
        "GPUI bridge must carry running-app requests to the GPUI/native side"
    );
    assert!(
        app_run_setup.contains("list_running_apps_on_gpui_thread"),
        "app runtime must execute running-app enumeration on the GPUI side"
    );

    let native_wrapper = extract_function_body(&bridge, "pub fn list_running_apps_on_gpui_thread(");
    assert!(
        native_wrapper.contains("NSWorkspace"),
        "native wrapper must use NSWorkspace running-app inventory"
    );
    assert!(
        native_wrapper.contains("runningApplications"),
        "native wrapper must enumerate running applications"
    );
    assert!(
        native_wrapper.contains("activationPolicy"),
        "native wrapper must expose activation policy metadata"
    );
    for needle in [
        "process_manager",
        "read_scripts",
        "load_scriptlets",
        "Command::new(\"open\")",
        "activateWithOptions",
        "activateIgnoringOtherApps",
        "launchApplication",
        "openApplicationAtURL",
        "terminate",
        "forceTerminate",
        "CGEvent",
        "AXUIElementPerformAction",
        "AXPress",
        "CGRequestScreenCaptureAccess",
        "request_accessibility_permission",
    ] {
        assert!(
            !native_wrapper.contains(needle),
            "native wrapper must stay read-only and non-prompting; found {}",
            needle
        );
    }

    assert!(
        protocol.contains("computer/list_apps"),
        "protocol docs must mention computer/list_apps"
    );
    assert!(
        protocol.contains("not an installed-app catalog"),
        "protocol docs must keep list_apps distinct from installed app catalogs"
    );
    assert!(
        protocol.contains("does not use `process_manager`"),
        "protocol docs must keep list_apps distinct from process-manager data"
    );
    assert!(
        protocol.contains("closed two-flag input"),
        "protocol docs must cite the source-audited list_apps input contract"
    );
}

fn extract_struct_block<'a>(source: &'a str, signature: &str) -> &'a str {
    extract_braced_block(source, signature)
}

fn extract_function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    extract_braced_block(source, signature)
}

fn extract_json_object_block<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source.find(marker).expect("json object marker");
    let open = source[start..].find('{').expect("json object open brace") + start;
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

    panic!("json object block for {} did not close", marker)
}

fn extract_braced_block<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).expect("signature");
    let open = source[start..].find('{').expect("open brace") + start;
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

    panic!("braced block for {} did not close", signature)
}
