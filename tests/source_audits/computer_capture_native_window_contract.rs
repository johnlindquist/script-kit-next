use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn extract_section<'a>(src: &'a str, needle: &str, len: usize) -> &'a str {
    let start = src
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} not found"));
    let mut end = src.len().min(start + len);
    while end < src.len() && !src.is_char_boundary(end) {
        end += 1;
    }
    &src[start..end]
}

#[test]
fn mcp_exposes_capture_native_window_with_closed_schema() {
    let src = read("src/mcp_computer_use_tools.rs");

    assert!(src.contains("COMPUTER_CAPTURE_NATIVE_WINDOW_TOOL"));
    assert!(src.contains("\"computer/capture_native_window\""));
    assert!(src.contains("computer_capture_native_window_input_schema"));
    assert!(src.contains("\"additionalProperties\": false"));
    assert!(src.contains("\"nativeWindowId\""));
    assert!(src.contains("\"expectedBundleId\""));
    assert!(src.contains("\"includeImage\""));
    assert!(src.contains("\"hiDpi\""));
}

#[test]
fn capture_native_window_handler_routes_through_runtime_bridge_with_correlation_id() {
    let src = read("src/mcp_computer_use_tools.rs");
    let section = extract_section(&src, "fn handle_capture_native_window", 3500);

    assert!(section.contains("ComputerUseCaptureNativeWindowArgs"));
    assert!(section.contains("ComputerUseCaptureNativeWindowRequest"));
    assert!(section.contains("runtime.capture_native_window"));
    assert!(section.contains("correlation_id"));
    assert!(section.contains("computer.capture_native_window.request"));
    assert!(section.contains("computer.capture_native_window.result"));
}

#[test]
fn mcp_server_forces_accepted_connections_to_blocking_io() {
    let src = read("src/mcp_server/mod.rs");
    let section = extract_section(&src, "fn handle_connection", 900);

    assert!(
        section.contains(".set_nonblocking(false)"),
        "accepted MCP sockets must be restored to blocking mode before large JSON/image receipts are written"
    );
    assert!(
        section.contains("Failed to set accepted MCP connection to blocking mode"),
        "blocking-mode failures should keep enough context to diagnose transport resets"
    );
}

#[test]
fn capture_native_window_handler_does_not_mutate_window_or_app_state() {
    let src = read("src/mcp_computer_use_tools.rs");
    let section = extract_section(&src, "fn handle_capture_native_window", 3500);

    for forbidden in [
        "activate",
        "focus",
        "launch",
        "quit",
        "hide",
        "move_window",
        "resize",
        "click",
        "send_input",
        "simulate",
        "request_permission",
        "open_system_settings",
    ] {
        assert!(
            !section.contains(forbidden),
            "capture_native_window handler must not contain forbidden action: {forbidden}"
        );
    }
}

#[test]
fn platform_capture_matches_only_exact_native_window_id() {
    let src = read("src/platform/screenshots_window_open.rs");
    let section = extract_section(&src, "capture_native_window_id_screenshot", 2200);

    assert!(section.contains("Window::all()"));
    assert!(section.contains("window.id().ok() == Some(native_window_id)"));
    assert!(section.contains("expected_owner_pid"));
    assert!(section.contains("core_graphics_owner_pid_for_native_window(native_window_id)"));
    assert!(section.contains("OwnershipMismatch"));
    assert!(section.contains("automation.native_window_capture.final_owner_mismatch"));
    assert!(section.contains("AmbiguousNativeWindowId"));
    assert!(section.contains("NativeWindowNotFound"));
    assert!(!section.contains("title.contains"));
    assert!(!section.contains("app_name.contains"));
}

#[test]
fn gpui_capture_revalidates_inventory_before_platform_capture() {
    let src = read("src/computer_use/gpui_runtime_bridge.rs");
    let section = extract_section(&src, "capture_native_window_on_gpui_thread", 13000);

    assert!(section.contains("list_app_windows_on_gpui_thread"));
    assert!(section.contains("expected_bundle_id"));
    assert!(section.contains("select_capture_candidate_for_native_window"));
    assert!(section.contains("request.pid"));
    assert!(section.contains("capture_native_window_id_screenshot"));
    assert!(section.contains("automation.native_window_capture.target_resolved"));
    assert!(section.contains("automation.native_window_capture.image_captured"));
    assert!(section.contains("automation.native_window_capture.inventory_failed"));
    assert!(section.contains("AmbiguousNativeWindowId"));
    assert!(section.contains("ambiguous_native_window_id"));
}

#[test]
fn capture_selection_gate_refuses_missing_ambiguous_and_non_candidate_rows() {
    let src = read("src/computer_use/native_window_capture.rs");

    assert!(src.contains("WindowNotFound"));
    assert!(src.contains("AmbiguousNativeWindowRows"));
    assert!(src.contains("NotCaptureCandidate"));
    assert!(src.contains("MissingObservation"));
    assert!(src.contains("MissingCaptureSelectionCandidate"));
    assert!(src.contains("WindowCaptureSelectionCandidateStatus::Candidate"));
}

#[test]
fn preflight_server_is_not_globally_retained_without_runtime_bridge() {
    let src = read("src/main_entry/preflight.rs");
    let section = extract_section(&src, "McpServer::with_defaults()", 900);

    assert!(
        !section.contains("retain_server_handle"),
        "preflight must not globally retain a no-runtime MCP server ahead of the runtime-backed app server"
    );
}

#[test]
fn proof_script_checks_large_image_schema_and_candidate_gate() {
    let src = read("scripts/agentic/prove-native-window-capture.ts");

    assert!(src.contains("Capture receipt omitted pngBase64"));
    assert!(src.contains("89504e470d0a1a0a"));
    assert!(src.contains("createHash(\"sha256\")"));
    assert!(src.contains("unexpectedFieldForProof"));
    assert!(src.contains("invalid_arguments"));
    assert!(src.contains("notCaptureCandidate"));
    assert!(src.contains("non-candidate-receipt.json"));
}
