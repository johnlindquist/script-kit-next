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

    let content =
        script_kit_gpui::mcp_resources::read_resource("kit://context", &scripts, &scriptlets, None)
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

#[test]
fn minimal_profile_omits_selected_text_and_menu_bar_from_json() {
    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://context?profile=minimal",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("minimal profile should resolve");

    let parsed: serde_json::Value =
        serde_json::from_str(&content.text).expect("must be valid JSON");

    // Minimal profile disables selectedText and menuBar — these fields must be
    // absent from the serialized JSON, not present with placeholder values.
    assert!(
        parsed.get("selectedText").is_none(),
        "selectedText must be absent in minimal profile JSON"
    );
    assert!(
        parsed.get("menuBarItems").is_none()
            || parsed["menuBarItems"]
                .as_array()
                .is_some_and(|a| a.is_empty()),
        "menuBarItems must be absent or empty in minimal profile JSON"
    );

    // schemaVersion must always be present
    assert_eq!(
        parsed["schemaVersion"],
        script_kit_gpui::context_snapshot::AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION
    );
}

#[test]
fn inspect_current_context_builtin_is_registered() {
    let entries = script_kit_gpui::builtins::get_builtin_entries(
        &script_kit_gpui::config::BuiltInConfig::default(),
    );

    let entry = entries
        .iter()
        .find(|entry| entry.id == "builtin-inspect-current-context")
        .expect("builtin-inspect-current-context must be in the registry");

    assert_eq!(
        entry.feature,
        script_kit_gpui::builtins::BuiltInFeature::UtilityCommand(
            script_kit_gpui::builtins::UtilityCommandType::InspectCurrentContext,
        )
    );

    assert!(
        entry.keywords.iter().any(|keyword| keyword == "json"),
        "Inspect Current Context must be discoverable by 'json'"
    );
    assert!(
        entry.keywords.iter().any(|keyword| keyword == "inspect"),
        "Inspect Current Context must be discoverable by 'inspect'"
    );
    assert!(
        entry.keywords.iter().any(|keyword| keyword == "clipboard"),
        "Inspect Current Context must be discoverable by 'clipboard'"
    );
    assert!(
        entry.keywords.iter().any(|keyword| keyword == "context"),
        "Inspect Current Context must be discoverable by 'context'"
    );
}

#[test]
fn context_snapshot_inspection_receipt_is_stable() {
    let snapshot = script_kit_gpui::context_snapshot::AiContextSnapshot::default();
    let receipt = script_kit_gpui::context_snapshot::build_inspection_receipt(&snapshot, 64);

    assert_eq!(
        receipt.schema_version,
        script_kit_gpui::context_snapshot::AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION
    );
    assert_eq!(receipt.warning_count, 0);
    assert_eq!(receipt.status, "ok");
    assert_eq!(receipt.json_bytes, 64);
    assert!(!receipt.has_selected_text);
    assert!(!receipt.has_frontmost_app);
    assert!(!receipt.has_browser);
    assert!(!receipt.has_focused_window);
    assert_eq!(receipt.top_level_menu_count, 0);
}
