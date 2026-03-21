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

    assert_eq!(value["schemaVersion"], 1);
    assert!(
        value.get("warnings").is_some()
            || value.get("schemaVersion").is_some(),
        "snapshot should have at least schemaVersion"
    );
}

#[test]
fn context_snapshot_types_are_public() {
    // Verify the public API surface exists and is accessible
    let _options = script_kit_gpui::context_snapshot::CaptureContextOptions::default();
    let _snapshot = script_kit_gpui::context_snapshot::AiContextSnapshot::default();
    assert_eq!(_snapshot.schema_version, 1);
}
