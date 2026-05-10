// @lat: [[protocol#Protocol#Tool exposure]]
#[test]
fn computer_list_app_windows_is_pid_only_read_only_inventory() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let runtime = std::fs::read_to_string("src/computer_use/runtime_bridge.rs")
        .expect("read runtime_bridge.rs");
    let bridge = std::fs::read_to_string("src/computer_use/gpui_runtime_bridge.rs")
        .expect("read gpui_runtime_bridge.rs");
    let app_run_setup =
        std::fs::read_to_string("src/main_entry/app_run_setup.rs").expect("read app_run_setup.rs");
    let protocol = std::fs::read_to_string("lat.md/protocol.md").expect("read protocol docs");

    assert!(
        mcp_tools.contains(
            "pub const COMPUTER_LIST_APP_WINDOWS_TOOL: &str = \"computer/list_app_windows\";"
        ),
        "computer/list_app_windows must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains(
            "COMPUTER_LIST_APP_WINDOWS_TOOL => handle_list_app_windows(arguments, runtime),"
        ),
        "computer/list_app_windows must route through the runtime-bridged handler"
    );
    assert!(
        mcp_tools.contains("computer_list_app_windows_input_schema"),
        "computer/list_app_windows must expose a dedicated input schema"
    );
    assert!(
        mcp_tools.contains("COMPUTER_APP_WINDOWS_SCHEMA_VERSION"),
        "computer/list_app_windows must have a dedicated schema version"
    );
    assert!(
        runtime.contains("struct ComputerUseListAppWindowsRequest"),
        "runtime bridge must define a list-app-windows request"
    );
    assert!(
        runtime.contains("struct ComputerUseListAppWindowsSnapshot"),
        "runtime bridge must define a list-app-windows snapshot"
    );
    assert!(
        runtime.contains("struct ComputerUseAppWindowInfo"),
        "runtime bridge must define native window metadata"
    );
    assert!(
        runtime.contains("fn list_app_windows("),
        "runtime bridge trait must own app-window enumeration"
    );
    assert!(
        bridge.contains("ListAppWindows"),
        "GPUI bridge must carry app-window requests to the GPUI/native side"
    );
    assert!(
        app_run_setup.contains("list_app_windows_on_gpui_thread"),
        "app runtime must execute app-window enumeration on the GPUI side"
    );

    let args_struct = extract_struct_block(&mcp_tools, "struct ComputerUseListAppWindowsArgs");
    assert!(
        mcp_tools.contains("#[serde(deny_unknown_fields)]\nstruct ComputerUseListAppWindowsArgs"),
        "computer/list_app_windows args must reject unknown fields"
    );
    assert!(
        args_struct.contains("pid: i32"),
        "computer/list_app_windows input must be exactly a running app PID"
    );
    let arg_fields: Vec<&str> = args_struct
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with(',') || *line == "pid: i32")
        .collect();
    assert_eq!(
        arg_fields,
        vec!["pid: i32,"],
        "computer/list_app_windows args must expose exactly one pid field"
    );

    let input_schema_body =
        extract_function_body(&mcp_tools, "fn computer_list_app_windows_input_schema()");
    assert!(
        input_schema_body.contains("\"additionalProperties\": false"),
        "computer/list_app_windows must reject unknown input fields"
    );
    assert!(
        input_schema_body.contains("\"pid\": { \"type\": \"integer\" }"),
        "computer/list_app_windows must require an integer pid"
    );
    assert!(
        input_schema_body.contains("\"required\": [\"pid\"]"),
        "computer/list_app_windows must require pid"
    );
    assert_eq!(
        extract_json_object_block(input_schema_body, "\"properties\":")
            .matches("\": {")
            .count(),
        1,
        "computer/list_app_windows schema properties must contain exactly one field"
    );
    for needle in [
        "\"target\"",
        "\"app\"",
        "\"bundleId\"",
        "\"name\"",
        "\"focus\"",
        "\"activate\"",
        "\"launch\"",
        "\"quit\"",
        "\"click\"",
        "\"move\"",
        "\"resize\"",
        "\"setBounds\"",
        "\"screenshot\"",
        "\"includeElements\"",
    ] {
        assert!(
            !input_schema_body.contains(needle),
            "computer/list_app_windows input must stay PID-only; found {}",
            needle
        );
    }

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseListAppWindowsResult");
    for field in [
        "schema_version: u32",
        "source: &'static str",
        "scope: &'static str",
        "status: &'static str",
        "app: Option<ComputerUseRunningAppInfo>",
        "windows: Vec<ComputerUseAppWindowInfo>",
        "warnings: Vec<String>",
    ] {
        assert!(
            result_struct.contains(field),
            "computer/list_app_windows result missing {}",
            field
        );
    }

    let window_info_struct = extract_struct_block(&runtime, "struct ComputerUseAppWindowInfo");
    for field in [
        "native_window_id: u32",
        "title: Option<String>",
        "bounds: TargetWindowBounds",
        "is_on_screen: bool",
        "layer: i64",
        "z_order: u32",
    ] {
        assert!(
            window_info_struct.contains(field),
            "native window metadata missing {}",
            field
        );
    }
    for needle in [
        "action",
        "click",
        "press",
        "execute",
        "ax_element_path",
        "focus",
        "activate",
        "launch",
        "quit",
        "move",
        "resize",
        "set_bounds",
    ] {
        assert!(
            !window_info_struct.contains(needle),
            "native window metadata must not expose executable/action fields; found {}",
            needle
        );
    }

    let handler_body = extract_function_body(&mcp_tools, "fn handle_list_app_windows(");
    assert!(
        handler_body.contains("ComputerUseListAppWindowsArgs"),
        "handler must parse the dedicated PID-only args"
    );
    assert!(
        handler_body.contains("let Some(runtime) = runtime"),
        "handler must require the live runtime bridge"
    );
    assert!(
        handler_body.contains("ComputerUseListAppWindowsRequest { pid: args.pid }"),
        "handler must pass only pid into the runtime request"
    );
    assert!(
        handler_body.contains("runtime.list_app_windows(request)"),
        "handler must delegate app-window enumeration to the runtime bridge"
    );
    assert!(
        handler_body.contains("source: \"coreGraphicsWindowList\""),
        "handler must identify CoreGraphics window metadata as the source"
    );
    assert!(
        handler_body.contains("scope: \"runningAppPid\""),
        "handler must identify PID scope"
    );
    assert!(
        handler_body.contains("\"found\"") && handler_body.contains("\"notFound\""),
        "handler must expose found/notFound statuses"
    );
    for needle in [
        "CGWindowListCopyWindowInfo",
        "NSWorkspace",
        "AXUIElement",
        "CGEvent",
        "inspect_automation_window",
        "build_automation_inspect_snapshot",
        "capture_targeted_rgba_image",
        "collect_visible_elements",
        "focus_window",
        "move_window",
        "resize_window",
        "set_automation_focus",
        "upsert_automation_window",
        "remove_automation_window",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "Command::new(\"open\")",
        "activateWithOptions",
        "activateIgnoringOtherApps",
        "launchApplication",
        "openApplicationAtURL",
        "terminate",
        "forceTerminate",
    ] {
        assert!(
            !handler_body.contains(needle),
            "MCP handler must not perform native/action work directly; found {}",
            needle
        );
    }

    let gpui_wrapper = extract_function_body(&bridge, "pub fn list_app_windows_on_gpui_thread(");
    assert!(
        gpui_wrapper.contains("list_running_apps_on_gpui_thread"),
        "GPUI wrapper must validate the PID against the running-app inventory"
    );
    assert!(
        gpui_wrapper.contains("screen_capture_access_preflight"),
        "GPUI wrapper may warn about redacted titles when screen recording is unavailable"
    );
    assert!(
        gpui_wrapper.contains("core_graphics_windows_for_pid(request.pid)"),
        "GPUI wrapper must delegate PID-scoped native window metadata to the CoreGraphics helper"
    );
    for needle in [
        "CGRequestScreenCaptureAccess",
        "request_accessibility_permission",
        "AXUIElementCreateApplication",
        "AXUIElementPerformAction",
        "AXPress",
        "CGEvent",
        "CGWindowListCreateImage",
        "capture_targeted_rgba_image",
        "focus_window",
        "move_window",
        "resize_window",
        "set_automation_focus",
        "Command::new(\"open\")",
        "activateWithOptions",
        "activateIgnoringOtherApps",
        "launchApplication",
        "openApplicationAtURL",
        "terminate",
        "forceTerminate",
    ] {
        assert!(
            !gpui_wrapper.contains(needle),
            "GPUI wrapper must stay read-only and non-prompting; found {}",
            needle
        );
    }

    let native_helper = extract_function_body(&bridge, "fn core_graphics_windows_for_pid(");
    assert!(
        native_helper.contains("CGWindowListCopyWindowInfo"),
        "native helper must read CoreGraphics window-list metadata"
    );
    assert!(
        native_helper.contains("kCGWindowOwnerPID"),
        "native helper must filter by owner PID"
    );
    assert!(
        native_helper.contains("kCGWindowNumber"),
        "native helper must expose native window ids"
    );
    assert!(
        native_helper.contains("kCGWindowBounds"),
        "native helper must expose window bounds"
    );
    for needle in [
        "CGWindowListCreateImage",
        "CGEvent",
        "AXUIElementCreateApplication",
        "AXUIElementPerformAction",
        "AXPress",
        "inspect_automation_window",
        "capture_targeted_rgba_image",
        "request_accessibility_permission",
        "CGRequestScreenCaptureAccess",
        "Command::new(\"open\")",
        "activateWithOptions",
        "activateIgnoringOtherApps",
        "launchApplication",
        "openApplicationAtURL",
        "terminate",
        "forceTerminate",
    ] {
        assert!(
            !native_helper.contains(needle),
            "native helper must stay read-only and non-prompting; found {}",
            needle
        );
    }

    assert!(
        protocol.contains("computer/list_app_windows"),
        "protocol docs must mention computer/list_app_windows"
    );
    assert!(
        protocol.contains("PID-only") || protocol.contains("caller-supplied PID"),
        "protocol docs must state the PID-only scope"
    );
    assert!(
        protocol.contains("does not focus, activate, launch, quit, move, resize, capture screenshots, inspect AX elements, prompt for permissions, or expose action handles"),
        "protocol docs must state the no-action/no-prompt contract"
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
