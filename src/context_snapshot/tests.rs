use super::capture::{capture_context_snapshot_from_seed, CaptureContextSeed};
use super::*;

#[test]
fn default_capture_options_enable_all_sections() {
    let options = CaptureContextOptions::default();
    assert!(options.include_selected_text);
    assert!(options.include_frontmost_app);
    assert!(options.include_menu_bar);
    assert!(options.include_browser_url);
    assert!(options.include_focused_window);
    assert!(!options.include_screenshot, "screenshot off by default");
    assert!(!options.include_panel_screenshot, "panel screenshot off by default");
}

#[test]
fn snapshot_default_has_schema_version_4() {
    let snapshot = AiContextSnapshot::default();
    assert_eq!(snapshot.schema_version, 4);
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
        schema_version: AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
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
        focused_window_image: None,
        script_kit_panel_image: None,
        warnings: vec![],
    };

    let json = serde_json::to_string(&snapshot).expect("snapshot should serialize");
    assert!(json.contains("\"schemaVersion\":4"));
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
    assert!(json.contains("\"schemaVersion\":4"));
}

#[test]
fn snapshot_roundtrips_through_serde() {
    let original = AiContextSnapshot {
        schema_version: AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
        selected_text: Some("hello".to_string()),
        frontmost_app: Some(FrontmostAppContext {
            pid: 1,
            bundle_id: "com.test".to_string(),
            name: "Test".to_string(),
        }),
        menu_bar_items: vec![],
        browser: None,
        focused_window: None,
        focused_window_image: None,
        script_kit_panel_image: None,
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
        include_screenshot: false,
        include_panel_screenshot: false,
    };

    let json = serde_json::to_string(&original).expect("serialize");
    assert!(json.contains("\"includeSelectedText\":true"));
    assert!(json.contains("\"includeFrontmostApp\":false"));

    let deserialized: CaptureContextOptions = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(original, deserialized);
}

// =======================================================
// Profile constructor tests
// =======================================================

#[test]
fn all_profile_matches_default() {
    assert_eq!(
        CaptureContextOptions::all(),
        CaptureContextOptions::default()
    );
}

#[test]
fn minimal_profile_disables_selected_text_and_menu_bar() {
    let minimal = CaptureContextOptions::minimal();
    assert!(!minimal.include_selected_text);
    assert!(!minimal.include_menu_bar);
    assert!(minimal.include_frontmost_app);
    assert!(minimal.include_browser_url);
    assert!(minimal.include_focused_window);
}

#[test]
fn default_snapshot_uses_schema_constant() {
    let snapshot = AiContextSnapshot::default();
    assert_eq!(snapshot.schema_version, AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION);
}

// =======================================================
// Seed-based capture tests
// =======================================================

fn full_seed() -> CaptureContextSeed {
    CaptureContextSeed {
        selected_text: Ok(Some("secret code".into())),
        frontmost_app: Ok(Some(FrontmostAppContext {
            pid: 7,
            bundle_id: "com.test.editor".into(),
            name: "Editor".into(),
        })),
        menu_bar_items: Ok(vec![MenuBarItemSummary {
            title: "File".into(),
            enabled: true,
            shortcut: None,
            children: vec![],
        }]),
        browser: Ok(Some(BrowserContext {
            url: "https://example.com".into(),
        })),
        focused_window: Ok(Some(FocusedWindowContext {
            title: "Main".into(),
            width: 1200,
            height: 800,
            used_fallback: false,
        })),
        focused_window_image: Ok(None),
        script_kit_panel_image: Ok(None),
    }
}

#[test]
fn capture_from_seed_respects_minimal_profile() {
    let snapshot =
        capture_context_snapshot_from_seed(&CaptureContextOptions::minimal(), full_seed());

    assert_eq!(snapshot.schema_version, AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION);
    // minimal disables selected_text and menu_bar
    assert_eq!(snapshot.selected_text, None);
    assert!(snapshot.menu_bar_items.is_empty());
    // but keeps the rest
    assert_eq!(
        snapshot.frontmost_app.as_ref().map(|a| a.name.as_str()),
        Some("Editor")
    );
    assert_eq!(
        snapshot.browser.as_ref().map(|b| b.url.as_str()),
        Some("https://example.com")
    );
    assert_eq!(
        snapshot.focused_window.as_ref().map(|w| w.title.as_str()),
        Some("Main")
    );
    assert!(snapshot.warnings.is_empty());
}

#[test]
fn capture_from_seed_all_profile_includes_everything() {
    let snapshot = capture_context_snapshot_from_seed(&CaptureContextOptions::all(), full_seed());

    assert_eq!(snapshot.selected_text, Some("secret code".into()));
    assert!(snapshot.frontmost_app.is_some());
    assert_eq!(snapshot.menu_bar_items.len(), 1);
    assert!(snapshot.browser.is_some());
    assert!(snapshot.focused_window.is_some());
    assert!(snapshot.warnings.is_empty());
}

#[test]
fn capture_from_seed_keeps_partial_success_and_records_warnings() {
    let seed = CaptureContextSeed {
        selected_text: Err("permission denied".into()),
        frontmost_app: Ok(None),
        menu_bar_items: Err("menu not ready".into()),
        browser: Ok(Some(BrowserContext {
            url: "https://example.com".into(),
        })),
        focused_window: Err("no focused window".into()),
        focused_window_image: Ok(None),
        script_kit_panel_image: Ok(None),
    };

    let snapshot = capture_context_snapshot_from_seed(&CaptureContextOptions::all(), seed);

    assert_eq!(
        snapshot.browser.as_ref().map(|b| b.url.as_str()),
        Some("https://example.com")
    );
    assert!(snapshot
        .warnings
        .contains(&"selectedText: permission denied".to_string()));
    assert!(snapshot
        .warnings
        .contains(&"menuBar: menu not ready".to_string()));
    assert!(snapshot
        .warnings
        .contains(&"focusedWindow: no focused window".to_string()));
}

#[test]
fn capture_from_seed_skips_warnings_for_disabled_providers() {
    let seed = CaptureContextSeed {
        selected_text: Err("would fail".into()),
        frontmost_app: Ok(None),
        menu_bar_items: Err("would also fail".into()),
        browser: Ok(None),
        focused_window: Ok(None),
        focused_window_image: Ok(None),
        script_kit_panel_image: Ok(None),
    };

    // minimal disables selected_text and menu_bar, so their errors are silent
    let snapshot = capture_context_snapshot_from_seed(&CaptureContextOptions::minimal(), seed);

    assert!(
        snapshot.warnings.is_empty(),
        "disabled providers should not produce warnings"
    );
}

#[test]
fn capture_from_seed_preserves_metadata_only_focused_window() {
    let seed = CaptureContextSeed {
        selected_text: Ok(None),
        frontmost_app: Ok(None),
        menu_bar_items: Ok(Vec::new()),
        browser: Ok(None),
        focused_window: Ok(Some(FocusedWindowContext {
            title: "Cached Title".into(),
            width: 0,
            height: 0,
            used_fallback: false,
        })),
        focused_window_image: Ok(None),
        script_kit_panel_image: Ok(None),
    };

    let snapshot = capture_context_snapshot_from_seed(&CaptureContextOptions::minimal(), seed);
    let focused_window = snapshot
        .focused_window
        .expect("focused window metadata should be preserved");

    assert_eq!(focused_window.title, "Cached Title");
    assert_eq!(focused_window.width, 0);
    assert_eq!(focused_window.height, 0);
    assert!(!focused_window.used_fallback);
}

#[test]
fn recommendation_profile_includes_browser_excludes_menu_and_window() {
    let rec = CaptureContextOptions::recommendation();
    assert!(rec.include_selected_text);
    assert!(rec.include_frontmost_app);
    assert!(rec.include_browser_url);
    assert!(!rec.include_menu_bar);
    assert!(!rec.include_focused_window);
    assert!(!rec.include_screenshot);
}

#[test]
fn tab_ai_profile_enables_all_including_screenshot() {
    let tab_ai = CaptureContextOptions::tab_ai();
    assert!(tab_ai.include_selected_text);
    assert!(tab_ai.include_frontmost_app);
    assert!(tab_ai.include_browser_url);
    assert!(tab_ai.include_menu_bar);
    assert!(tab_ai.include_focused_window);
    assert!(tab_ai.include_screenshot);
    assert!(tab_ai.include_panel_screenshot);
}

#[test]
fn capture_from_seed_recommendation_profile_includes_browser_skips_window() {
    let snapshot =
        capture_context_snapshot_from_seed(&CaptureContextOptions::recommendation(), full_seed());

    assert_eq!(snapshot.schema_version, AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION);
    assert_eq!(snapshot.selected_text, Some("secret code".into()));
    assert!(snapshot.frontmost_app.is_some());
    assert_eq!(
        snapshot.browser.as_ref().map(|b| b.url.as_str()),
        Some("https://example.com")
    );
    // recommendation() excludes focused_window and menu_bar
    assert!(snapshot.focused_window.is_none());
    assert!(snapshot.menu_bar_items.is_empty());
}
