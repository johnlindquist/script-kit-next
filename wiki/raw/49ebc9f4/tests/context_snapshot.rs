//! Integration tests for the context_snapshot module and kit://context MCP resource.

/// Enable deterministic context capture for all tests in this file.
/// Without this, `read_resource("kit://context", ...)` triggers real Cmd+C
/// keystrokes via the `get-selected-text` crate's clipboard fallback.
fn init() {
    script_kit_gpui::context_snapshot::enable_deterministic_context_capture();
}

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
    init();
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
    let tab_ai = script_kit_gpui::context_snapshot::CaptureContextOptions::tab_ai();

    assert!(full.include_selected_text);
    assert!(full.include_menu_bar);
    assert!(
        !full.include_screenshot,
        "full profile must not include screenshot by default"
    );
    assert!(
        !full.include_panel_screenshot,
        "full profile must not include panel screenshot by default"
    );

    assert!(!minimal.include_selected_text);
    assert!(!minimal.include_menu_bar);
    assert!(minimal.include_frontmost_app);
    assert!(minimal.include_browser_url);
    assert!(minimal.include_focused_window);
    assert!(
        !minimal.include_screenshot,
        "minimal profile must not include screenshot"
    );
    assert!(
        !minimal.include_panel_screenshot,
        "minimal profile must not include panel screenshot"
    );

    assert!(tab_ai.include_selected_text);
    assert!(tab_ai.include_menu_bar);
    assert!(tab_ai.include_frontmost_app);
    assert!(tab_ai.include_browser_url);
    assert!(tab_ai.include_focused_window);
    assert!(
        tab_ai.include_screenshot,
        "tab_ai profile must include screenshot"
    );
    assert!(
        tab_ai.include_panel_screenshot,
        "tab_ai profile must include panel screenshot"
    );
}

#[test]
fn tab_ai_submit_profile_is_stable_and_screenshot_free() {
    let opts = script_kit_gpui::context_snapshot::CaptureContextOptions::tab_ai_submit();

    // Rich metadata for higher-precision actions
    assert!(opts.include_selected_text);
    assert!(opts.include_frontmost_app);
    assert!(opts.include_menu_bar);
    assert!(opts.include_browser_url);
    assert!(opts.include_focused_window);

    // No screenshots — keeps the Tab AI path fast
    assert!(
        !opts.include_screenshot,
        "tab_ai_submit must not request focused-window screenshot"
    );
    assert!(
        !opts.include_panel_screenshot,
        "tab_ai_submit must not request panel screenshot"
    );
}

#[test]
fn tab_ai_submit_snapshot_contains_metadata_without_image() {
    init();

    // Use the deterministic seed path (no live OS calls) and verify that the
    // tab_ai_submit profile yields focused-window metadata but no image data.
    let snapshot = script_kit_gpui::context_snapshot::capture_context_snapshot(
        &script_kit_gpui::context_snapshot::CaptureContextOptions::tab_ai_submit(),
    );

    // Deterministic mode returns empty fields, but crucially no image data
    assert!(
        snapshot.focused_window_image.is_none(),
        "tab_ai_submit snapshot must not contain focused_window_image"
    );
    assert!(
        snapshot.script_kit_panel_image.is_none(),
        "tab_ai_submit snapshot must not contain script_kit_panel_image"
    );
}

#[test]
fn context_resource_is_available() {
    init();
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
    init();
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
        .find(|entry| entry.id == "builtin/inspect-current-context")
        .expect("builtin/inspect-current-context must be in the registry");

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
fn context_resource_schema_lists_screenshot_parameter() {
    init();
    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://context/schema",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("schema resource should resolve");

    let parsed: serde_json::Value =
        serde_json::from_str(&content.text).expect("schema must be valid JSON");

    let params = parsed["parameters"].as_array().expect("parameters array");
    assert!(
        params.iter().any(|param| param["name"] == "screenshot"),
        "schema must list screenshot parameter"
    );
}

#[test]
fn context_schema_lists_screenshot_param_and_examples() {
    init();
    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://context/schema",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("kit://context/schema should resolve");

    let value: serde_json::Value =
        serde_json::from_str(&content.text).expect("schema must be valid JSON");

    let parameters = value["parameters"].as_array().expect("parameters array");
    assert!(
        parameters
            .iter()
            .any(|p| p["name"].as_str() == Some("screenshot")),
        "schema should advertise screenshot parameter"
    );

    let examples = value["examples"].as_array().expect("examples array");
    assert!(
        examples
            .iter()
            .any(|e| e.as_str() == Some("kit://context?screenshot=1")),
        "schema should advertise kit://context?screenshot=1 example"
    );
}

#[test]
fn context_diagnostics_reflects_screenshot_override() {
    script_kit_gpui::context_snapshot::enable_deterministic_context_capture();

    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://context?screenshot=1&diagnostics=1",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("diagnostics resource should resolve");

    let value: serde_json::Value =
        serde_json::from_str(&content.text).expect("diagnostics must be valid JSON");

    assert_eq!(value["kind"], "context_diagnostics");
    assert_eq!(value["uri"], "kit://context?screenshot=1&diagnostics=1");
    assert_eq!(value["meta"]["options"]["includeScreenshot"], true);

    let field_statuses = value["meta"]["fieldStatuses"]
        .as_array()
        .expect("fieldStatuses array");
    assert!(
        field_statuses
            .iter()
            .any(|s| s["field"].as_str() == Some("screenshot") && s["enabled"] == true),
        "screenshot field status should be enabled"
    );
}

#[test]
fn versioned_resources_are_listed_and_resolve() {
    let resources = script_kit_gpui::mcp_resources::get_resource_definitions();

    for uri in ["kit://scripts", "kit://scriptlets", "kit://sdk-reference"] {
        assert!(
            resources.iter().any(|r| r.uri == uri),
            "{uri} should be listed in resource definitions"
        );
    }

    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let scripts_doc =
        script_kit_gpui::mcp_resources::read_resource("kit://scripts", &scripts, &scriptlets, None)
            .expect("kit://scripts should resolve");
    let scripts_json: serde_json::Value =
        serde_json::from_str(&scripts_doc.text).expect("kit://scripts must be valid JSON");
    assert_eq!(scripts_json["schemaVersion"], 1);
    assert_eq!(scripts_json["count"], 0);

    let scriptlets_doc = script_kit_gpui::mcp_resources::read_resource(
        "kit://scriptlets",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("kit://scriptlets should resolve");
    let scriptlets_json: serde_json::Value =
        serde_json::from_str(&scriptlets_doc.text).expect("kit://scriptlets must be valid JSON");
    assert_eq!(scriptlets_json["schemaVersion"], 1);
    assert_eq!(scriptlets_json["count"], 0);

    let sdk_doc = script_kit_gpui::mcp_resources::read_resource(
        "kit://sdk-reference",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("kit://sdk-reference should resolve");

    // Deserialize into the shared contract type instead of hardcoding scalars.
    let sdk: script_kit_gpui::mcp_resources::SdkReferenceDocument =
        serde_json::from_str(&sdk_doc.text).expect("kit://sdk-reference must be valid JSON");
    assert_eq!(
        sdk.schema_version,
        script_kit_gpui::mcp_resources::SDK_REFERENCE_SCHEMA_VERSION,
        "schema_version must match the current SDK_REFERENCE_SCHEMA_VERSION constant"
    );
    assert_eq!(
        sdk.sdk_package, "@scriptkit/sdk",
        "sdk_package must be @scriptkit/sdk"
    );
    assert!(
        sdk.metadata_format.contains("export const metadata"),
        "metadata_format must use export const metadata"
    );
    assert!(
        !sdk.harness_workflow.run_command.is_empty(),
        "harnessWorkflow must include runCommand"
    );
    assert!(
        !sdk.harness_workflow.test_script_directory.is_empty(),
        "harnessWorkflow must include testScriptDirectory"
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

#[test]
fn context_schema_lists_panel_screenshot_parameter() {
    init();
    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://context/schema",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("schema resource should resolve");

    let parsed: serde_json::Value =
        serde_json::from_str(&content.text).expect("schema must be valid JSON");

    let params = parsed["parameters"].as_array().expect("parameters array");
    assert!(
        params
            .iter()
            .any(|param| param["name"] == "panelScreenshot"),
        "schema must list panelScreenshot parameter"
    );

    let examples = parsed["examples"].as_array().expect("examples array");
    assert!(
        examples
            .iter()
            .any(|e| e.as_str() == Some("kit://context?panelScreenshot=1")),
        "schema should advertise kit://context?panelScreenshot=1 example"
    );
}

#[test]
fn context_diagnostics_reflects_panel_screenshot_override() {
    script_kit_gpui::context_snapshot::enable_deterministic_context_capture();

    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://context?panelScreenshot=1&diagnostics=1",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("diagnostics resource should resolve");

    let value: serde_json::Value =
        serde_json::from_str(&content.text).expect("diagnostics must be valid JSON");

    assert_eq!(value["kind"], "context_diagnostics");
    assert_eq!(value["meta"]["options"]["includePanelScreenshot"], true);

    let field_statuses = value["meta"]["fieldStatuses"]
        .as_array()
        .expect("fieldStatuses array");
    assert!(
        field_statuses
            .iter()
            .any(|s| s["field"].as_str() == Some("panelScreenshot") && s["enabled"] == true),
        "panelScreenshot field status should be enabled"
    );
}

// =======================================================
// Clipboard history resource integration tests
// =======================================================

#[test]
fn clipboard_history_resource_is_listed() {
    let resources = script_kit_gpui::mcp_resources::get_resource_definitions();
    assert!(
        resources.iter().any(|r| r.uri == "kit://clipboard-history"),
        "kit://clipboard-history should be in resource definitions"
    );
}

#[test]
fn clipboard_history_resource_resolves_and_returns_valid_json() {
    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://clipboard-history",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("kit://clipboard-history should resolve");

    assert_eq!(content.uri, "kit://clipboard-history");
    assert_eq!(content.mime_type, "application/json");

    let value: serde_json::Value = serde_json::from_str(&content.text).expect("must be valid JSON");
    assert_eq!(
        value["schemaVersion"],
        script_kit_gpui::mcp_resources::CLIPBOARD_HISTORY_RESOURCE_SCHEMA_VERSION
    );
    assert!(value["count"].is_number());
}

#[test]
fn clipboard_history_resource_supports_limit_param() {
    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://clipboard-history?limit=3",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("should resolve with limit param");

    let value: serde_json::Value = serde_json::from_str(&content.text).expect("must be valid JSON");
    assert_eq!(value["schemaVersion"], 1);
}

#[test]
fn clipboard_history_diagnostics_returns_meta() {
    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://clipboard-history?diagnostics=1",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("diagnostics should resolve");

    let value: serde_json::Value = serde_json::from_str(&content.text).expect("must be valid JSON");
    assert_eq!(value["kind"], "clipboard_history_diagnostics");
    assert!(value["meta"]["durationMs"].is_number());
    assert_eq!(value["meta"]["source"], "cached_entries");
}

// =======================================================
// Focused item resource integration tests
// =======================================================

#[test]
fn focused_item_resource_is_listed() {
    let resources = script_kit_gpui::mcp_resources::get_resource_definitions();
    assert!(
        resources.iter().any(|r| r.uri == "kit://focused-item"),
        "kit://focused-item should be in resource definitions"
    );
}

#[test]
fn focused_item_resource_resolves_and_returns_valid_json() {
    script_kit_gpui::mcp_resources::clear_focused_item();

    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://focused-item",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("kit://focused-item should resolve");

    assert_eq!(content.uri, "kit://focused-item");
    assert_eq!(content.mime_type, "application/json");

    let value: serde_json::Value = serde_json::from_str(&content.text).expect("must be valid JSON");
    assert_eq!(
        value["schemaVersion"],
        script_kit_gpui::mcp_resources::FOCUSED_ITEM_RESOURCE_SCHEMA_VERSION
    );
    assert_eq!(value["hasFocusedItem"], false);
}

#[test]
fn focused_item_resource_returns_published_item() {
    script_kit_gpui::mcp_resources::publish_focused_item(
        script_kit_gpui::mcp_resources::FocusedItemInfo {
            source: "TestSurface".to_string(),
            kind: "test_entry".to_string(),
            semantic_id: "choice:0:test".to_string(),
            label: "Test Item".to_string(),
            metadata: Some(serde_json::json!({"key": "value"})),
        },
    );

    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://focused-item",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("should resolve");

    let value: serde_json::Value = serde_json::from_str(&content.text).expect("must be valid JSON");
    assert_eq!(value["hasFocusedItem"], true);
    assert_eq!(value["focusedItem"]["source"], "TestSurface");
    assert_eq!(value["focusedItem"]["semanticId"], "choice:0:test");

    // Clean up
    script_kit_gpui::mcp_resources::clear_focused_item();
}

#[test]
fn focused_item_diagnostics_returns_meta() {
    script_kit_gpui::mcp_resources::clear_focused_item();

    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://focused-item?diagnostics=1",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("diagnostics should resolve");

    let value: serde_json::Value = serde_json::from_str(&content.text).expect("must be valid JSON");
    assert_eq!(value["kind"], "focused_item_diagnostics");
    assert!(value["meta"]["durationMs"].is_number());
    assert_eq!(value["meta"]["hasFocusedItem"], false);
    assert!(value["meta"]["warningCount"].as_u64().unwrap_or(0) > 0);
}

// =======================================================
// Provider-backed resource integration tests
// =======================================================

fn restore_env(key: &str, value: Option<std::ffi::OsString>) {
    match value {
        Some(v) => unsafe { std::env::set_var(key, v) },
        None => unsafe { std::env::remove_var(key) },
    }
}

#[test]
fn provider_resources_are_listed_and_empty_fallbacks_are_stable() {
    // Save and clear env vars + slots to ensure empty-fallback path
    let prev_dictation = std::env::var_os("SCRIPT_KIT_DICTATION_JSON");
    let prev_calendar = std::env::var_os("SCRIPT_KIT_CALENDAR_JSON");
    let prev_notifications = std::env::var_os("SCRIPT_KIT_NOTIFICATIONS_JSON");
    unsafe {
        std::env::remove_var("SCRIPT_KIT_DICTATION_JSON");
        std::env::remove_var("SCRIPT_KIT_CALENDAR_JSON");
        std::env::remove_var("SCRIPT_KIT_NOTIFICATIONS_JSON");
    }
    script_kit_gpui::mcp_resources::clear_provider_json_slots();

    let resources = script_kit_gpui::mcp_resources::get_resource_definitions();
    for uri in ["kit://dictation", "kit://calendar", "kit://notifications"] {
        assert!(
            resources.iter().any(|r| r.uri == uri),
            "{uri} should be listed in resource definitions"
        );
    }

    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    for (uri, kind, env_key) in [
        ("kit://dictation", "dictation", "SCRIPT_KIT_DICTATION_JSON"),
        ("kit://calendar", "calendar", "SCRIPT_KIT_CALENDAR_JSON"),
        (
            "kit://notifications",
            "notifications",
            "SCRIPT_KIT_NOTIFICATIONS_JSON",
        ),
    ] {
        let content =
            script_kit_gpui::mcp_resources::read_resource(uri, &scripts, &scriptlets, None)
                .expect("provider resource should resolve");

        let json: serde_json::Value =
            serde_json::from_str(&content.text).expect("provider resource must be valid JSON");
        assert_eq!(json["schemaVersion"], 1, "{uri} schemaVersion");
        assert_eq!(json["type"], kind, "{uri} type");
        assert_eq!(json["ok"], true, "{uri} ok");
        assert_eq!(json["available"], false, "{uri} available");
        assert_eq!(json["source"], "empty-fallback", "{uri} source");
        assert_eq!(json["items"], serde_json::json!([]), "{uri} items");
        assert!(
            json["nextStep"]
                .as_str()
                .unwrap_or_default()
                .contains(env_key),
            "nextStep for {uri} should mention {env_key}"
        );
    }

    restore_env("SCRIPT_KIT_DICTATION_JSON", prev_dictation);
    restore_env("SCRIPT_KIT_CALENDAR_JSON", prev_calendar);
    restore_env("SCRIPT_KIT_NOTIFICATIONS_JSON", prev_notifications);
}

#[test]
fn provider_resources_report_slot_source_when_data_published() {
    // Use the slot-based API (mutex-protected) to avoid env-var races with
    // parallel tests. Slots take priority over env vars in resolution.
    script_kit_gpui::mcp_resources::clear_provider_json_slots();
    script_kit_gpui::mcp_resources::publish_calendar_json(
        r#"{"schemaVersion":1,"type":"calendar","ok":true,"available":true,"source":"slot","items":[{"title":"Demo"}]}"#,
    );

    let scripts: Vec<std::sync::Arc<script_kit_gpui::scripts::Script>> = Vec::new();
    let scriptlets: Vec<std::sync::Arc<script_kit_gpui::scripts::Scriptlet>> = Vec::new();

    let content = script_kit_gpui::mcp_resources::read_resource(
        "kit://calendar",
        &scripts,
        &scriptlets,
        None,
    )
    .expect("kit://calendar should resolve");

    let json: serde_json::Value =
        serde_json::from_str(&content.text).expect("calendar resource must be valid JSON");
    assert_eq!(json["source"], "slot");
    assert_eq!(json["available"], true);
    assert_eq!(json["items"][0]["title"], "Demo");

    // Clean up
    script_kit_gpui::mcp_resources::clear_provider_json_slots();
}
