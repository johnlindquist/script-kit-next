use super::*;

#[test]
fn default_capture_options_enable_all_sections() {
    let options = CaptureContextOptions::default();
    assert!(options.include_selected_text);
    assert!(options.include_frontmost_app);
    assert!(options.include_menu_bar);
    assert!(options.include_browser_url);
    assert!(options.include_focused_window);
}

#[test]
fn snapshot_default_has_schema_version_1() {
    let snapshot = AiContextSnapshot::default();
    assert_eq!(snapshot.schema_version, 1);
    assert!(snapshot.selected_text.is_none());
    assert!(snapshot.frontmost_app.is_none());
    assert!(snapshot.menu_bar_items.is_empty());
    assert!(snapshot.browser.is_none());
    assert!(snapshot.focused_window.is_none());
    assert!(snapshot.warnings.is_empty());
}

#[test]
fn snapshot_serializes_to_stable_camel_case_json() {
    let snapshot = AiContextSnapshot {
        schema_version: 1,
        selected_text: Some("fn render(...) -> impl IntoElement".to_string()),
        frontmost_app: Some(FrontmostAppContext {
            pid: 4242,
            bundle_id: "com.microsoft.VSCode".to_string(),
            name: "Visual Studio Code".to_string(),
        }),
        menu_bar_items: vec![MenuBarItemSummary {
            title: "File".to_string(),
            enabled: true,
            shortcut: None,
            children: vec![MenuBarItemSummary {
                title: "Save".to_string(),
                enabled: true,
                shortcut: Some("\u{2318}S".to_string()),
                children: vec![],
            }],
        }],
        browser: Some(BrowserContext {
            url: "https://docs.rs/gpui/latest/gpui/".to_string(),
        }),
        focused_window: Some(FocusedWindowContext {
            title: "README.md \u{2014} script-kit-gpui".to_string(),
            width: 1440,
            height: 900,
            used_fallback: false,
        }),
        warnings: vec![],
    };

    let json = serde_json::to_string(&snapshot).expect("snapshot should serialize");
    assert!(json.contains("\"schemaVersion\":1"));
    assert!(
        json.contains("\"selectedText\":\"fn render(...) -> impl IntoElement\""),
        "should use camelCase for selectedText"
    );
    assert!(
        json.contains("\"bundleId\":\"com.microsoft.VSCode\""),
        "should use camelCase for bundleId"
    );
    assert!(
        json.contains("\"usedFallback\":false"),
        "should use camelCase for usedFallback"
    );
    // warnings is empty so should be omitted
    assert!(
        !json.contains("\"warnings\""),
        "empty warnings should be skipped"
    );
}

#[test]
fn snapshot_omits_none_fields_in_json() {
    let snapshot = AiContextSnapshot::default();
    let json = serde_json::to_string(&snapshot).expect("should serialize");
    assert!(
        !json.contains("\"selectedText\""),
        "None fields should be omitted"
    );
    assert!(
        !json.contains("\"frontmostApp\""),
        "None fields should be omitted"
    );
    assert!(
        !json.contains("\"browser\""),
        "None fields should be omitted"
    );
    assert!(
        !json.contains("\"focusedWindow\""),
        "None fields should be omitted"
    );
    // Only schemaVersion should remain
    assert!(json.contains("\"schemaVersion\":1"));
}

#[test]
fn snapshot_roundtrips_through_serde() {
    let original = AiContextSnapshot {
        schema_version: 1,
        selected_text: Some("hello".to_string()),
        frontmost_app: Some(FrontmostAppContext {
            pid: 1,
            bundle_id: "com.test".to_string(),
            name: "Test".to_string(),
        }),
        menu_bar_items: vec![],
        browser: None,
        focused_window: None,
        warnings: vec!["test warning".to_string()],
    };

    let json = serde_json::to_string(&original).expect("serialize");
    let deserialized: AiContextSnapshot = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

#[test]
fn capture_options_roundtrip_through_serde() {
    let original = CaptureContextOptions {
        include_selected_text: true,
        include_frontmost_app: false,
        include_menu_bar: true,
        include_browser_url: false,
        include_focused_window: true,
    };

    let json = serde_json::to_string(&original).expect("serialize");
    assert!(json.contains("\"includeSelectedText\":true"));
    assert!(json.contains("\"includeFrontmostApp\":false"));

    let deserialized: CaptureContextOptions = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}
