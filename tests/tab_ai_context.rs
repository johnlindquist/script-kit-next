//! Integration tests for Tab AI context blob assembly and serialization.
//!
//! Validates the deterministic `TabAiContextBlob` structure that is sent to
//! the AI model alongside the user's intent. Ensures schema stability,
//! JSON field naming, and round-trip correctness.

use script_kit_gpui::ai::{
    TabAiClipboardContext, TabAiContextBlob, TabAiMemorySuggestion, TabAiUiSnapshot,
    TAB_AI_CONTEXT_SCHEMA_VERSION,
};
use script_kit_gpui::context_snapshot::{AiContextSnapshot, BrowserContext, FrontmostAppContext};
use script_kit_gpui::protocol::ElementInfo;

/// Build a fully-populated context blob for assertion.
fn full_blob() -> TabAiContextBlob {
    TabAiContextBlob::from_parts(
        TabAiUiSnapshot {
            prompt_type: "ClipboardHistory".to_string(),
            input_text: Some("search".to_string()),
            focused_semantic_id: Some("input:filter".to_string()),
            selected_semantic_id: Some("choice:1:item-b".to_string()),
            visible_elements: vec![
                ElementInfo::input("filter", Some("search"), true),
                ElementInfo::choice(0, "Item A", "item-a", false),
                ElementInfo::choice(1, "Item B", "item-b", true),
            ],
        },
        AiContextSnapshot {
            frontmost_app: Some(FrontmostAppContext {
                name: "Safari".to_string(),
                bundle_id: "com.apple.Safari".to_string(),
                pid: 42,
            }),
            selected_text: Some("selected text".to_string()),
            browser: Some(BrowserContext {
                url: "https://docs.rs".to_string(),
            }),
            ..Default::default()
        },
        vec!["recent-a".to_string(), "recent-b".to_string()],
        Some(TabAiClipboardContext {
            content_type: "text".to_string(),
            preview: "clipboard preview".to_string(),
            ocr_text: None,
        }),
        vec![],
        "2026-03-28T20:00:00Z".to_string(),
    )
}

#[test]
fn schema_version_is_current() {
    let blob = full_blob();
    assert_eq!(blob.schema_version, TAB_AI_CONTEXT_SCHEMA_VERSION);
    assert_eq!(blob.schema_version, 2, "bump tests when schema changes");
}

#[test]
fn json_field_names_are_camel_case() {
    let json = serde_json::to_string(&full_blob()).unwrap();

    // camelCase present
    for field in &[
        "schemaVersion",
        "promptType",
        "inputText",
        "focusedSemanticId",
        "selectedSemanticId",
        "visibleElements",
        "recentInputs",
        "contentType",
        "frontmostApp",
        "selectedText",
    ] {
        assert!(json.contains(field), "missing camelCase field: {field}");
    }

    // snake_case absent
    for field in &[
        "schema_version",
        "prompt_type",
        "input_text",
        "focused_semantic_id",
        "selected_semantic_id",
        "visible_elements",
        "recent_inputs",
        "content_type",
        "frontmost_app",
        "selected_text",
    ] {
        assert!(!json.contains(field), "found snake_case field: {field}");
    }
}

#[test]
fn full_blob_round_trips_through_json() {
    let original = full_blob();
    let json = serde_json::to_string_pretty(&original).unwrap();
    let parsed: TabAiContextBlob = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.schema_version, original.schema_version);
    assert_eq!(parsed.timestamp, original.timestamp);
    assert_eq!(parsed.ui.prompt_type, "ClipboardHistory");
    assert_eq!(parsed.ui.input_text.as_deref(), Some("search"));
    assert_eq!(
        parsed.ui.focused_semantic_id.as_deref(),
        Some("input:filter")
    );
    assert_eq!(
        parsed.ui.selected_semantic_id.as_deref(),
        Some("choice:1:item-b")
    );
    assert_eq!(parsed.ui.visible_elements.len(), 3);
    assert_eq!(
        parsed
            .desktop
            .frontmost_app
            .as_ref()
            .map(|a| a.name.as_str()),
        Some("Safari")
    );
    assert_eq!(
        parsed.desktop.selected_text.as_deref(),
        Some("selected text")
    );
    assert_eq!(
        parsed.desktop.browser.as_ref().map(|b| b.url.as_str()),
        Some("https://docs.rs")
    );
    assert_eq!(parsed.recent_inputs, vec!["recent-a", "recent-b"]);
    assert_eq!(
        parsed.clipboard.as_ref().map(|c| c.preview.as_str()),
        Some("clipboard preview")
    );
}

#[test]
fn empty_optional_fields_omitted_from_json() {
    let blob = TabAiContextBlob::from_parts(
        TabAiUiSnapshot {
            prompt_type: "ScriptList".to_string(),
            ..Default::default()
        },
        Default::default(),
        vec![],
        None,
        vec![],
        "2026-03-28T00:00:00Z".to_string(),
    );

    let json = serde_json::to_string(&blob).unwrap();

    assert!(!json.contains("inputText"), "None should be omitted");
    assert!(
        !json.contains("focusedSemanticId"),
        "None should be omitted"
    );
    assert!(
        !json.contains("selectedSemanticId"),
        "None should be omitted"
    );
    assert!(
        !json.contains("visibleElements"),
        "empty Vec should be omitted"
    );
    assert!(
        !json.contains("recentInputs"),
        "empty Vec should be omitted"
    );
    assert!(!json.contains("clipboard"), "None should be omitted");
    assert!(
        !json.contains("priorAutomations"),
        "empty Vec should be omitted"
    );
}

#[test]
fn from_parts_populates_all_fields() {
    let blob = full_blob();

    assert_eq!(blob.schema_version, TAB_AI_CONTEXT_SCHEMA_VERSION);
    assert!(!blob.timestamp.is_empty());
    assert_eq!(blob.ui.prompt_type, "ClipboardHistory");
    assert!(blob.ui.input_text.is_some());
    assert!(blob.ui.focused_semantic_id.is_some());
    assert!(blob.ui.selected_semantic_id.is_some());
    assert!(!blob.ui.visible_elements.is_empty());
    assert!(blob.desktop.frontmost_app.is_some());
    assert!(blob.desktop.selected_text.is_some());
    assert!(blob.desktop.browser.is_some());
    assert!(!blob.recent_inputs.is_empty());
    assert!(blob.clipboard.is_some());
}

#[test]
fn tab_ai_context_blob_serializes_clipboard_and_prior_automations() {
    let blob = TabAiContextBlob::from_parts(
        TabAiUiSnapshot {
            prompt_type: "ClipboardHistory".to_string(),
            input_text: Some("rename to kebab-case".to_string()),
            focused_semantic_id: Some("choice:0:Quarterly Report Final.png".to_string()),
            selected_semantic_id: Some("choice:0:Quarterly Report Final.png".to_string()),
            visible_elements: Vec::new(),
        },
        AiContextSnapshot {
            selected_text: Some("Quarterly Report Final".to_string()),
            frontmost_app: Some(FrontmostAppContext {
                pid: 42,
                bundle_id: "com.apple.finder".to_string(),
                name: "Finder".to_string(),
            }),
            ..Default::default()
        },
        vec!["rename file".to_string()],
        Some(TabAiClipboardContext {
            content_type: "image".to_string(),
            preview: "Quarterly Report Final.png".to_string(),
            ocr_text: Some("Quarterly Report Final.png".to_string()),
        }),
        vec![TabAiMemorySuggestion {
            slug: "rename-file-kebab".to_string(),
            app_name: "Finder".to_string(),
            bundle_id: "com.apple.finder".to_string(),
            raw_query: "rename file".to_string(),
            effective_query: "rename selected file to kebab case".to_string(),
            provider_id: "openai".to_string(),
            model_id: "gpt-5".to_string(),
            score: 0.91,
        }],
        "2026-03-28T07:27:22Z".to_string(),
    );

    let json = serde_json::to_value(&blob).expect("serialize tab ai context blob");
    assert_eq!(json["schemaVersion"], 2);
    assert_eq!(json["clipboard"]["ocrText"], "Quarterly Report Final.png");
    assert_eq!(json["priorAutomations"][0]["slug"], "rename-file-kebab");
    let score = json["priorAutomations"][0]["score"]
        .as_f64()
        .expect("score is a number");
    assert!(
        (score - 0.91).abs() < 0.001,
        "score should be approximately 0.91, got {score}"
    );
}

#[test]
fn truncate_tab_ai_text_caps_long_strings() {
    use script_kit_gpui::ai::truncate_tab_ai_text;

    assert_eq!(truncate_tab_ai_text("short", 100), "short");
    assert_eq!(truncate_tab_ai_text("", 10), "");
    assert_eq!(truncate_tab_ai_text("anything", 0), "");

    let long = "a".repeat(300);
    let truncated = truncate_tab_ai_text(&long, 10);
    assert!(truncated.ends_with('…'));
    assert_eq!(truncated.chars().count(), 10);
}

#[test]
fn truncate_tab_ai_text_handles_unicode() {
    use script_kit_gpui::ai::truncate_tab_ai_text;

    // Each emoji is one char
    let emojis = "🎉🎊🎈🎁🎂";
    let result = truncate_tab_ai_text(emojis, 3);
    assert_eq!(result, "🎉🎊…");
    assert_eq!(result.chars().count(), 3);
}

#[test]
fn tab_ai_clipboard_context_omits_none_ocr_text() {
    let clip = TabAiClipboardContext {
        content_type: "text".to_string(),
        preview: "hello".to_string(),
        ocr_text: None,
    };
    let json = serde_json::to_string(&clip).unwrap();
    assert!(!json.contains("ocrText"));
}
