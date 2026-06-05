#[test]
fn computer_list_permissions_reports_read_only_permission_statuses() {
    let mcp_tools = std::fs::read_to_string("src/mcp_computer_use_tools.rs")
        .expect("read mcp_computer_use_tools.rs");
    let platform = std::fs::read_to_string("src/platform/screenshots_window_open.rs")
        .expect("read screenshots_window_open.rs");
    let permissions_wizard =
        std::fs::read_to_string("src/permissions_wizard.rs").expect("read permissions_wizard.rs");

    assert!(
        mcp_tools.contains(
            "pub const COMPUTER_LIST_PERMISSIONS_TOOL: &str = \"computer/list_permissions\";"
        ),
        "computer/list_permissions must be a static computer-use MCP tool"
    );
    assert!(
        mcp_tools.contains("COMPUTER_LIST_PERMISSIONS_TOOL => handle_list_permissions(arguments),"),
        "computer/list_permissions must route through a runtime-free handler"
    );
    assert!(
        mcp_tools
            .contains("#[serde(deny_unknown_fields)]\nstruct ComputerUseListPermissionsArgs {}"),
        "computer/list_permissions args must stay closed and empty"
    );

    let result_struct = extract_struct_block(&mcp_tools, "struct ComputerUseListPermissionsResult");
    assert_eq!(
        field_declarations(result_struct),
        vec![
            "schema_version: u32,",
            "permissions: Vec<ComputerUsePermissionStatus>,",
        ],
        "computer/list_permissions result must expose only schemaVersion and permissions"
    );

    let permission_struct = extract_struct_block(&mcp_tools, "struct ComputerUsePermissionStatus");
    assert_eq!(
        field_declarations(permission_struct),
        vec![
            "id: &'static str,",
            "name: &'static str,",
            "granted: Option<bool>,",
            "status: &'static str,",
        ],
        "permission rows must stay status-only"
    );

    let handler_body = extract_function_body(&mcp_tools, "fn handle_list_permissions(");
    assert!(
        handler_body.contains("computer_use_permission_statuses()"),
        "computer/list_permissions must reuse the shared permission status helper"
    );

    let permission_helper =
        extract_function_body(&mcp_tools, "fn computer_use_permission_statuses()");
    for needle in [
        "\"accessibility\"",
        "\"screenRecording\"",
        "\"eventSynthesizing\"",
        "check_accessibility_permission()",
        "screen_capture_access_preflight()",
        "event_synthesizing_access_preflight()",
    ] {
        assert!(
            permission_helper.contains(needle),
            "shared permission helper must contain {needle}"
        );
    }
    for needle in [
        "ComputerUseRuntimeBridge",
        "runtime.inspect_automation_window",
        "list_running_apps",
        "list_app_windows",
        "AXPress",
        "CGRequestScreenCaptureAccess",
        "CGRequestPostEventAccess",
        "request_accessibility_permission",
        "AXIsProcessTrustedWithOptions",
        "openSettings",
        "System Settings",
        "click",
        "press",
        "execute",
        "focus",
        "activate",
        "hotkey",
        "type",
    ] {
        assert!(
            !handler_body.contains(needle),
            "computer/list_permissions handler must stay read-only status reporting; found {needle}"
        );
        assert!(
            !permission_helper.contains(needle),
            "shared permission helper must stay read-only status reporting; found {needle}"
        );
    }

    let event_preflight = extract_function_body(
        &platform,
        "pub(crate) fn event_synthesizing_access_preflight()",
    );
    assert!(
        event_preflight.contains("CGPreflightPostEventAccess"),
        "Event Synthesizing status must use CoreGraphics preflight"
    );
    for needle in [
        "CGRequestPostEventAccess",
        "CGEventPost",
        "AXPress",
        "request_accessibility_permission",
        "open_accessibility_settings",
        "Command::new(\"open\")",
        "x-apple.systempreferences",
    ] {
        assert!(
            !event_preflight.contains(needle),
            "Event Synthesizing preflight must not request permissions or synthesize events; found {needle}"
        );
    }

    let screen_preflight =
        extract_function_body(&platform, "pub(crate) fn screen_capture_access_preflight()");
    assert!(
        screen_preflight.contains("CGPreflightScreenCaptureAccess"),
        "Screen Recording status must use CoreGraphics preflight"
    );
    for needle in [
        "CGRequestScreenCaptureAccess",
        "capture_image",
        "capture_targeted_screenshot",
        "Command::new(\"open\")",
        "x-apple.systempreferences",
    ] {
        assert!(
            !screen_preflight.contains(needle),
            "Screen Recording preflight must not request permission, capture, or open settings; found {needle}"
        );
    }

    let accessibility_check = extract_function_body(
        &permissions_wizard,
        "pub fn check_accessibility_permission()",
    );
    assert!(
        accessibility_check.contains("accessibility::application_is_trusted()"),
        "Accessibility status must use the non-prompting trust check"
    );
    for needle in [
        "application_is_trusted_with_prompt",
        "request_accessibility_permission",
        "open_accessibility_settings",
        "Command::new(\"open\")",
        "x-apple.systempreferences",
    ] {
        assert!(
            !accessibility_check.contains(needle),
            "Accessibility status must not request permission or open settings; found {needle}"
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
        .filter(|line| line.ends_with(','))
        .map(ToString::to_string)
        .collect()
}
