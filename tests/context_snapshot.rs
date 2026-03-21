//! Integration tests for the context_snapshot module and kit://context MCP resource.

#[test]
fn context_resource_is_listed() {
    let resources = script_kit_gpui::mcp_resources::get_resource_definitions();
    assert!(
        resources.iter().any(|r| r.uri == "kit://context"),
        "kit://context should be in resource definitions"
    );
}

#[test]
fn context_resource_returns_valid_json_with_schema_version() {
    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://context",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("kit://context should resolve");

    assert_eq!(content.uri, "kit://context");
    assert_eq!(content.mime_type, "application/json");

    let value: serde_json::Value =
        serde_json::from_str(&content.text).expect("resource text should be valid JSON");

    assert_eq!(
        value["schemaVersion"],
        script_kit_gpui::context_snapshot::AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION
    );
}

#[test]
fn context_snapshot_types_are_public() {
    // Verify the public API surface exists and is accessible
    let _options = script_kit_gpui::context_snapshot::CaptureContextOptions::default();
    let _snapshot = script_kit_gpui::context_snapshot::AiContextSnapshot::default();
    assert_eq!(
        _snapshot.schema_version,
        script_kit_gpui::context_snapshot::AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION
    );
}

#[test]
fn context_option_profiles_are_stable() {
    let full = script_kit_gpui::context_snapshot::CaptureContextOptions::default();
    let minimal = script_kit_gpui::context_snapshot::CaptureContextOptions::minimal();

    assert!(full.include_selected_text);
    assert!(full.include_menu_bar);

    assert!(!minimal.include_selected_text);
    assert!(!minimal.include_menu_bar);
    assert!(minimal.include_frontmost_app);
    assert!(minimal.include_browser_url);
    assert!(minimal.include_focused_window);
}

#[test]
fn context_resource_is_available() {
    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://context?profile=minimal",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("minimal profile should resolve");

    assert_eq!(content.uri, "kit://context?profile=minimal");
    assert_eq!(content.mime_type, "application/json");

    let wrapped = script_kit_gpui::mcp_resources::resource_content_to_value(content);
    assert_eq!(
        wrapped["contents"][0]["uri"],
        "kit://context?profile=minimal"
    );
    assert_eq!(wrapped["contents"][0]["mimeType"], "application/json");

    let text = wrapped["contents"][0]["text"]
        .as_str()
        .expect("wrapped text must be a string");

    let parsed: serde_json::Value =
        serde_json::from_str(text).expect("wrapped text must be valid JSON");

    assert_eq!(
        parsed["schemaVersion"],
        script_kit_gpui::context_snapshot::AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION
    );
}
