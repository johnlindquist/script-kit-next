//! Integration tests for Tab AI context blob assembly and serialization.
//!
//! Validates the deterministic `TabAiContextBlob` structure that is sent to
//! the AI model alongside the user's intent. Ensures schema stability,
//! JSON field naming, and round-trip correctness.

use script_kit_gpui::ai::{
    TabAiApplyBackHint, TabAiClipboardContext, TabAiClipboardHistoryEntry, TabAiContextBlob,
    TabAiMemorySuggestion, TabAiSourceType, TabAiUiSnapshot, TAB_AI_CONTEXT_SCHEMA_VERSION,
};
use script_kit_gpui::context_snapshot::{
    AiContextSnapshot, Base64PngContext, BrowserContext, FocusedWindowContext, FrontmostAppContext,
};
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
        vec![],
        "2026-03-28T20:00:00Z".to_string(),
    )
}

#[test]
fn schema_version_is_current() {
    let blob = full_blob();
    assert_eq!(blob.schema_version, TAB_AI_CONTEXT_SCHEMA_VERSION);
    assert_eq!(
        blob.schema_version, TAB_AI_CONTEXT_SCHEMA_VERSION,
        "bump tests when schema changes"
    );
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
        !json.contains("clipboardHistory"),
        "empty Vec should be omitted"
    );
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
        vec![],
        vec![TabAiMemorySuggestion {
            slug: "rename-file-kebab".to_string(),
            bundle_id: "com.apple.finder".to_string(),
            raw_query: "rename file".to_string(),
            effective_query: "rename selected file to kebab case".to_string(),
            prompt_type: "AppLauncher".to_string(),
            written_at: "2026-03-28T07:27:00Z".to_string(),
            score: 0.91,
        }],
        "2026-03-28T07:27:22Z".to_string(),
    );

    let json = serde_json::to_value(&blob).expect("serialize tab ai context blob");
    assert_eq!(json["schemaVersion"], TAB_AI_CONTEXT_SCHEMA_VERSION);
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

#[test]
fn tab_ai_context_blob_serializes_clipboard_history_and_window_image() {
    let blob = TabAiContextBlob::from_parts(
        TabAiUiSnapshot {
            prompt_type: "ClipboardHistory".to_string(),
            ..Default::default()
        },
        AiContextSnapshot {
            focused_window: Some(FocusedWindowContext {
                title: "Safari - docs.rs".to_string(),
                width: 1200,
                height: 800,
                used_fallback: false,
            }),
            focused_window_image: Some(Base64PngContext {
                mime_type: "image/png".to_string(),
                width: 1200,
                height: 800,
                base64_data: "ZmFrZS1wbmc=".to_string(),
                title: Some("Safari - docs.rs".to_string()),
            }),
            ..Default::default()
        },
        vec!["recent".to_string()],
        Some(TabAiClipboardContext {
            content_type: "text".to_string(),
            preview: "hello".to_string(),
            ocr_text: None,
        }),
        vec![TabAiClipboardHistoryEntry {
            id: "cb-1".to_string(),
            content_type: "text".to_string(),
            timestamp: 1743229000000,
            preview: "hello".to_string(),
            full_text: Some("hello".to_string()),
            ocr_text: None,
            image_width: None,
            image_height: None,
        }],
        vec![],
        "2026-03-29T06:56:43Z".to_string(),
    );

    let value = serde_json::to_value(&blob).expect("serialize tab ai context");
    assert_eq!(value["schemaVersion"], TAB_AI_CONTEXT_SCHEMA_VERSION);
    assert_eq!(value["clipboardHistory"][0]["fullText"], "hello");
    assert_eq!(
        value["desktop"]["focusedWindowImage"]["mimeType"],
        "image/png"
    );
    assert_eq!(
        value["desktop"]["focusedWindowImage"]["base64Data"],
        "ZmFrZS1wbmc="
    );
}

#[test]
fn tab_ai_clipboard_history_omits_optional_fields() {
    let entry = TabAiClipboardHistoryEntry {
        id: "cb-2".to_string(),
        content_type: "text".to_string(),
        timestamp: 1743229000000,
        preview: "hello".to_string(),
        full_text: None,
        ocr_text: None,
        image_width: None,
        image_height: None,
    };
    let json = serde_json::to_string(&entry).unwrap();
    assert!(!json.contains("fullText"));
    assert!(!json.contains("ocrText"));
    assert!(!json.contains("imageWidth"));
    assert!(!json.contains("imageHeight"));
}

#[test]
fn desktop_snapshot_omits_screenshot_by_default() {
    let snapshot = AiContextSnapshot::default();
    let json = serde_json::to_string(&snapshot).unwrap();
    assert!(!json.contains("focusedWindowImage"));
}

#[test]
fn tab_ai_context_blob_serializes_schema_v3_clipboard_history_and_targets() {
    use script_kit_gpui::ai::TabAiTargetContext;

    let blob = TabAiContextBlob::from_parts_with_targets(
        TabAiUiSnapshot {
            prompt_type: "ClipboardHistory".to_string(),
            input_text: Some("rename this".to_string()),
            focused_semantic_id: Some("choice:0:report".to_string()),
            selected_semantic_id: Some("choice:0:report".to_string()),
            visible_elements: Vec::new(),
        },
        Some(TabAiTargetContext {
            source: "ClipboardHistory".to_string(),
            kind: "clipboard_entry".to_string(),
            semantic_id: "choice:0:report".to_string(),
            label: "Quarterly Report Final.png".to_string(),
            metadata: Some(serde_json::json!({
                "id": "clip-1",
                "contentType": "image",
                "timestamp": 1_743_230_400_000i64,
                "imageWidth": 640,
                "imageHeight": 480,
                "ocrText": "Quarterly Report Final",
            })),
        }),
        vec![TabAiTargetContext {
            source: "ClipboardHistory".to_string(),
            kind: "clipboard_entry".to_string(),
            semantic_id: "choice:0:report".to_string(),
            label: "Quarterly Report Final.png".to_string(),
            metadata: None,
        }],
        AiContextSnapshot {
            frontmost_app: Some(FrontmostAppContext {
                pid: 42,
                bundle_id: "com.apple.finder".to_string(),
                name: "Finder".to_string(),
            }),
            ..Default::default()
        },
        vec!["rename selected file".to_string()],
        Some(TabAiClipboardContext {
            content_type: "image".to_string(),
            preview: "640\u{00d7}480 image".to_string(),
            ocr_text: Some("Quarterly Report Final".to_string()),
        }),
        vec![TabAiClipboardHistoryEntry {
            id: "clip-1".to_string(),
            content_type: "image".to_string(),
            timestamp: 1_743_230_400_000i64,
            preview: "640\u{00d7}480 image".to_string(),
            full_text: None,
            ocr_text: Some("Quarterly Report Final".to_string()),
            image_width: Some(640),
            image_height: Some(480),
        }],
        Vec::new(),
        "2026-03-29T07:30:00Z".to_string(),
    );

    let json = serde_json::to_value(&blob).expect("serialize context blob");
    assert_eq!(json["schemaVersion"], TAB_AI_CONTEXT_SCHEMA_VERSION);
    assert_eq!(json["focusedTarget"]["semanticId"], "choice:0:report");
    assert_eq!(json["focusedTarget"]["label"], "Quarterly Report Final.png");
    assert_eq!(json["focusedTarget"]["kind"], "clipboard_entry");
    assert_eq!(json["visibleTargets"][0]["source"], "ClipboardHistory");
    assert_eq!(json["clipboardHistory"][0]["id"], "clip-1");
    assert_eq!(json["clipboardHistory"][0]["imageWidth"], 640);
    assert_eq!(
        json["clipboardHistory"][0]["ocrText"],
        "Quarterly Report Final"
    );
    assert_eq!(json["clipboard"]["ocrText"], "Quarterly Report Final");
    assert_eq!(
        json["desktop"]["frontmostApp"]["bundleId"],
        "com.apple.finder"
    );
}

// ---------------------------------------------------------------------------
// Suggested intent tests
// ---------------------------------------------------------------------------

use script_kit_gpui::ai::{
    build_tab_ai_suggested_intents, recent_tab_ai_automations_for_bundle_from_path,
    TabAiMemoryEntry, TabAiSuggestedIntentSpec, TabAiTargetContext,
    TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
};

#[test]
fn build_tab_ai_suggested_intents_prefers_app_verbs() {
    let target = TabAiTargetContext {
        source: "AppLauncherView".to_string(),
        kind: "app".to_string(),
        semantic_id: "choice:0:notion".to_string(),
        label: "Notion".to_string(),
        metadata: None,
    };
    let suggestions = build_tab_ai_suggested_intents(Some(&target), None, &[]);
    assert_eq!(suggestions.len(), 3);
    assert_eq!(suggestions[0].intent, "focus on this app");
    assert_eq!(suggestions[1].intent, "what does this app do?");
    assert_eq!(
        suggestions[2].intent,
        "create a quick automation for this app"
    );
}

#[test]
fn build_tab_ai_suggested_intents_file_target() {
    let target = TabAiTargetContext {
        source: "FileSearch".to_string(),
        kind: "file".to_string(),
        semantic_id: "choice:0:readme".to_string(),
        label: "README.md".to_string(),
        metadata: None,
    };
    let suggestions = build_tab_ai_suggested_intents(Some(&target), None, &[]);
    assert_eq!(suggestions.len(), 3);
    assert_eq!(suggestions[0].intent, "summarize this file");
}

#[test]
fn build_tab_ai_suggested_intents_clipboard_image() {
    let clipboard = TabAiClipboardContext {
        content_type: "image".to_string(),
        preview: "screenshot.png".to_string(),
        ocr_text: None,
    };
    let suggestions = build_tab_ai_suggested_intents(None, Some(&clipboard), &[]);
    assert_eq!(suggestions.len(), 2);
    assert_eq!(suggestions[0].intent, "extract the text from this image");
}

#[test]
fn build_tab_ai_suggested_intents_no_context_fallback() {
    let suggestions = build_tab_ai_suggested_intents(None, None, &[]);
    assert_eq!(suggestions.len(), 2);
    assert_eq!(suggestions[0].label, "What Can I Do?");
}

#[test]
fn build_tab_ai_suggested_intents_prior_automation_caps_at_three() {
    let target = TabAiTargetContext {
        source: "AppLauncherView".to_string(),
        kind: "app".to_string(),
        semantic_id: "choice:0:slack".to_string(),
        label: "Slack".to_string(),
        metadata: None,
    };
    let prior = vec![script_kit_gpui::ai::TabAiMemorySuggestion {
        slug: "mute-slack".to_string(),
        bundle_id: "com.tinyspeck.slackmacgap".to_string(),
        raw_query: "mute slack".to_string(),
        effective_query: "mute slack notifications".to_string(),
        prompt_type: "QuickTerminal".to_string(),
        written_at: "2026-03-28T12:00:00Z".to_string(),
        score: 1.0,
    }];
    // 3 app suggestions + 1 prior = 4, truncated to 3
    let suggestions = build_tab_ai_suggested_intents(Some(&target), None, &prior);
    assert_eq!(suggestions.len(), 3);
    // The prior automation replaces the third app suggestion
    assert_eq!(suggestions[0].intent, "focus on this app");
    assert_eq!(suggestions[1].intent, "what does this app do?");
    assert_eq!(
        suggestions[2].intent,
        "create a quick automation for this app"
    );
}

#[test]
fn build_tab_ai_suggested_intents_prior_automation_fills_short_list() {
    let clipboard = TabAiClipboardContext {
        content_type: "text".to_string(),
        preview: "hello world".to_string(),
        ocr_text: None,
    };
    let prior = vec![script_kit_gpui::ai::TabAiMemorySuggestion {
        slug: "translate-text".to_string(),
        bundle_id: "com.apple.Safari".to_string(),
        raw_query: "translate".to_string(),
        effective_query: "translate this text".to_string(),
        prompt_type: "QuickTerminal".to_string(),
        written_at: "2026-03-28T12:00:00Z".to_string(),
        score: 1.0,
    }];
    // 2 clipboard suggestions + 1 prior = 3, exactly at cap
    let suggestions = build_tab_ai_suggested_intents(None, Some(&clipboard), &prior);
    assert_eq!(suggestions.len(), 3);
    assert_eq!(suggestions[2].intent, "translate this text");
    assert_eq!(suggestions[2].label, "Repeat translate-text");
}

// ---------------------------------------------------------------------------
// Recent automations by bundle tests
// ---------------------------------------------------------------------------

#[test]
fn recent_tab_ai_automations_for_bundle_returns_most_recent_first() {
    let dir = std::env::temp_dir().join(format!(
        "tab-ai-recent-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(".tab-ai-memory.json");

    let entries = vec![
        TabAiMemoryEntry {
            schema_version: TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
            intent: "copy current url".to_string(),
            generated_source: "await copy(url)".to_string(),
            slug: "copy-current-url".to_string(),
            prompt_type: "QuickTerminal".to_string(),
            bundle_id: Some("com.apple.Safari".to_string()),
            written_at: "2026-03-28T12:00:00Z".to_string(),
        },
        TabAiMemoryEntry {
            schema_version: TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
            intent: "summarize current tab".to_string(),
            generated_source: "await chat()".to_string(),
            slug: "summarize-current-tab".to_string(),
            prompt_type: "QuickTerminal".to_string(),
            bundle_id: Some("com.apple.Safari".to_string()),
            written_at: "2026-03-29T12:00:00Z".to_string(),
        },
    ];
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&entries).expect("serialize memory entries"),
    )
    .expect("write memory index");

    let recent = recent_tab_ai_automations_for_bundle_from_path(Some("com.apple.Safari"), 2, &path)
        .expect("read recent bundle automations");
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].slug, "summarize-current-tab");
    assert_eq!(recent[1].slug, "copy-current-url");

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recent_tab_ai_automations_for_bundle_filters_other_bundles() {
    let dir = std::env::temp_dir().join(format!(
        "tab-ai-filter-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(".tab-ai-memory.json");

    let entries = vec![
        TabAiMemoryEntry {
            schema_version: TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
            intent: "copy url".to_string(),
            generated_source: "await copy(url)".to_string(),
            slug: "copy-url".to_string(),
            prompt_type: "QuickTerminal".to_string(),
            bundle_id: Some("com.apple.Safari".to_string()),
            written_at: "2026-03-28T12:00:00Z".to_string(),
        },
        TabAiMemoryEntry {
            schema_version: TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
            intent: "open project".to_string(),
            generated_source: "await open()".to_string(),
            slug: "open-project".to_string(),
            prompt_type: "QuickTerminal".to_string(),
            bundle_id: Some("com.microsoft.VSCode".to_string()),
            written_at: "2026-03-29T12:00:00Z".to_string(),
        },
    ];
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&entries).expect("serialize"),
    )
    .expect("write");

    let safari =
        recent_tab_ai_automations_for_bundle_from_path(Some("com.apple.Safari"), 10, &path)
            .expect("read");
    assert_eq!(safari.len(), 1);
    assert_eq!(safari[0].slug, "copy-url");

    let vscode =
        recent_tab_ai_automations_for_bundle_from_path(Some("com.microsoft.VSCode"), 10, &path)
            .expect("read");
    assert_eq!(vscode.len(), 1);
    assert_eq!(vscode[0].slug, "open-project");

    // Cleanup
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn recent_tab_ai_automations_for_bundle_returns_empty_for_missing_path() {
    let path = std::path::Path::new("/tmp/nonexistent-tab-ai-memory-12345.json");
    let result = recent_tab_ai_automations_for_bundle_from_path(Some("com.apple.Safari"), 10, path)
        .expect("should succeed with empty result");
    assert!(result.is_empty());
}

#[test]
fn recent_tab_ai_automations_for_bundle_returns_empty_for_none_bundle() {
    let result = recent_tab_ai_automations_for_bundle_from_path(
        None,
        10,
        std::path::Path::new("/tmp/anything.json"),
    )
    .expect("should succeed");
    assert!(result.is_empty());
}

#[test]
fn suggested_intent_spec_new_works() {
    let spec = TabAiSuggestedIntentSpec::new("Focus", "focus on this app");
    assert_eq!(spec.label, "Focus");
    assert_eq!(spec.intent, "focus on this app");
}

// =========================================================================
// Deferred capture fields: sourceType, screenshotPath, applyBackHint
// =========================================================================

#[test]
fn context_blob_serializes_source_type_screenshot_path_and_apply_back_hint_without_schema_bump() {
    let blob = full_blob().with_deferred_capture_fields(
        Some(TabAiSourceType::DesktopSelection),
        Some("/tmp/tab-ai-screenshot-20260330T125352Z-41231.png".to_string()),
        Some(TabAiApplyBackHint {
            action: "replaceSelectedText".to_string(),
            target_label: Some("Frontmost selection".to_string()),
        }),
    );

    let json = serde_json::to_string_pretty(&blob).expect("must serialize");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("must parse");

    // Schema version must NOT have been bumped
    assert_eq!(
        parsed["schemaVersion"].as_u64().expect("schemaVersion"),
        TAB_AI_CONTEXT_SCHEMA_VERSION as u64,
        "schema version must remain unchanged with new optional fields"
    );

    // sourceType present and camelCase
    assert_eq!(
        parsed["sourceType"].as_str().expect("sourceType"),
        "desktopSelection"
    );

    // screenshotPath present
    assert!(
        parsed["screenshotPath"]
            .as_str()
            .expect("screenshotPath")
            .contains("tab-ai-screenshot"),
        "screenshotPath must contain the screenshot filename"
    );

    // applyBackHint present with expected fields
    let hint = &parsed["applyBackHint"];
    assert_eq!(
        hint["action"].as_str().expect("action"),
        "replaceSelectedText"
    );
    assert_eq!(
        hint["targetLabel"].as_str().expect("targetLabel"),
        "Frontmost selection"
    );
}

#[test]
fn context_blob_omits_deferred_fields_when_none() {
    let blob = full_blob();
    let json = serde_json::to_string(&blob).expect("must serialize");

    // When None, these fields must not appear in the JSON at all
    assert!(
        !json.contains("sourceType"),
        "sourceType must be omitted when None"
    );
    assert!(
        !json.contains("screenshotPath"),
        "screenshotPath must be omitted when None"
    );
    assert!(
        !json.contains("applyBackHint"),
        "applyBackHint must be omitted when None"
    );
}

#[test]
fn source_type_serde_round_trip() {
    let variants = vec![
        (TabAiSourceType::DesktopSelection, "desktopSelection"),
        (TabAiSourceType::ScriptListItem, "scriptListItem"),
        (TabAiSourceType::RunningCommand, "runningCommand"),
        (TabAiSourceType::ClipboardEntry, "clipboardEntry"),
        (TabAiSourceType::Desktop, "desktop"),
    ];
    for (variant, expected_json) in variants {
        let json = serde_json::to_string(&variant).expect("must serialize");
        assert_eq!(
            json,
            format!("\"{expected_json}\""),
            "serde output must be camelCase for {variant:?}"
        );
        let parsed: TabAiSourceType = serde_json::from_str(&json).expect("must parse");
        assert_eq!(parsed, variant, "round-trip must preserve variant");
    }
}

#[test]
fn apply_back_hint_serde_round_trip() {
    let hint = TabAiApplyBackHint {
        action: "pasteToPrompt".to_string(),
        target_label: Some("Active prompt".to_string()),
    };
    let json = serde_json::to_string(&hint).expect("must serialize");
    assert!(json.contains("\"action\":\"pasteToPrompt\""));
    assert!(json.contains("\"targetLabel\":\"Active prompt\""));

    let parsed: TabAiApplyBackHint = serde_json::from_str(&json).expect("must parse");
    assert_eq!(parsed, hint);
}

#[test]
fn apply_back_hint_omits_target_label_when_none() {
    let hint = TabAiApplyBackHint {
        action: "copyToClipboard".to_string(),
        target_label: None,
    };
    let json = serde_json::to_string(&hint).expect("must serialize");
    assert!(!json.contains("targetLabel"), "targetLabel must be omitted when None");
}

#[test]
fn with_deferred_capture_fields_preserves_existing_blob_data() {
    let original = full_blob();
    let enriched = original.clone().with_deferred_capture_fields(
        Some(TabAiSourceType::ClipboardEntry),
        Some("/tmp/screenshot.png".to_string()),
        None,
    );

    // Original blob fields must be preserved
    assert_eq!(enriched.schema_version, original.schema_version);
    assert_eq!(enriched.timestamp, original.timestamp);
    assert_eq!(enriched.ui.prompt_type, original.ui.prompt_type);
    assert_eq!(enriched.desktop.selected_text, original.desktop.selected_text);

    // New fields must be set
    assert_eq!(enriched.source_type, Some(TabAiSourceType::ClipboardEntry));
    assert_eq!(enriched.screenshot_path.as_deref(), Some("/tmp/screenshot.png"));
    assert!(enriched.apply_back_hint.is_none());
}

// =========================================================================
// Source-type detection priority: desktop selection wins over generic desktop
// =========================================================================

/// Validates that `detect_tab_ai_source_type` checks `selected_text` first,
/// before falling through to the `match source_view` block. This ensures that
/// desktop selection always takes priority over the generic `Desktop` fallback.
#[test]
fn source_type_detection_prefers_desktop_selection_over_generic_desktop() {
    let source = include_str!("../src/app_impl/tab_ai_mode.rs");

    // Find the detect function body
    let fn_start = source
        .find("fn detect_tab_ai_source_type(")
        .expect("detect_tab_ai_source_type must exist");
    let fn_body = &source[fn_start..];
    let fn_end = fn_body[1..]
        .find("\nfn ")
        .or_else(|| fn_body[1..].find("\n    fn "))
        .unwrap_or(fn_body.len());
    let fn_body = &fn_body[..fn_end];

    // The selected_text check must come BEFORE the match on source_view
    let selected_text_pos = fn_body
        .find("selected_text")
        .expect("must check selected_text");
    let desktop_selection_pos = fn_body
        .find("DesktopSelection")
        .expect("must return DesktopSelection for selected text");
    let match_source_pos = fn_body
        .find("match source_view")
        .expect("must match on source_view for view-based detection");
    let desktop_fallback_pos = fn_body
        .rfind("Desktop)")
        .expect("must have Desktop fallback");

    assert!(
        selected_text_pos < match_source_pos,
        "selected_text check must come before match source_view"
    );
    assert!(
        desktop_selection_pos < match_source_pos,
        "DesktopSelection return must come before match source_view"
    );
    assert!(
        match_source_pos < desktop_fallback_pos,
        "Desktop fallback must come after match source_view (in the _ arm)"
    );

    // The selected_text check must be an early return
    let selection_block = &fn_body[selected_text_pos..match_source_pos];
    assert!(
        selection_block.contains("return Some("),
        "selected_text branch must early-return so it wins over any source_view match"
    );
}
