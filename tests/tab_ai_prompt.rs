//! Integration tests for Tab AI prompt construction and context serialization.
//!
//! Validates that:
//! 1. `build_tab_ai_user_prompt` produces deterministic, well-structured prompts
//! 2. Context blobs serialize correctly and round-trip through JSON
//! 3. The prompt contains all required sections for downstream script extraction

use script_kit_gpui::ai::{
    build_tab_ai_user_prompt, TabAiContextBlob, TabAiUiSnapshot, TAB_AI_CONTEXT_SCHEMA_VERSION,
};

/// Helper: build a minimal context blob for testing.
fn minimal_context() -> TabAiContextBlob {
    TabAiContextBlob::from_parts(
        TabAiUiSnapshot {
            prompt_type: "ScriptList".to_string(),
            ..Default::default()
        },
        Default::default(),
        vec![],
        None,
        "2026-03-28T00:00:00Z".to_string(),
    )
}

/// Helper: build a rich context blob with all fields populated.
fn rich_context() -> TabAiContextBlob {
    TabAiContextBlob::from_parts(
        TabAiUiSnapshot {
            prompt_type: "ArgPrompt".to_string(),
            input_text: Some("Slack".to_string()),
            focused_semantic_id: Some("input:filter".to_string()),
            selected_semantic_id: Some("choice:0:slack".to_string()),
            visible_elements: vec![script_kit_gpui::protocol::ElementInfo::choice(
                0, "Slack", "slack", true,
            )],
        },
        script_kit_gpui::context_snapshot::AiContextSnapshot {
            frontmost_app: Some(script_kit_gpui::context_snapshot::FrontmostAppContext {
                name: "Slack".to_string(),
                bundle_id: "com.tinyspeck.slackmacgap".to_string(),
                pid: 1234,
            }),
            selected_text: Some("hello world".to_string()),
            browser: Some(script_kit_gpui::context_snapshot::BrowserContext {
                url: "https://example.com".to_string(),
            }),
            ..Default::default()
        },
        vec!["copy url".to_string(), "open finder".to_string()],
        Some("clipboard text preview".to_string()),
        "2026-03-28T12:00:00Z".to_string(),
    )
}

#[test]
fn prompt_contains_intent_section() {
    let context_json = serde_json::to_string_pretty(&minimal_context()).unwrap();
    let prompt = build_tab_ai_user_prompt("force quit this app", &context_json);

    assert!(prompt.contains("User intent:"));
    assert!(prompt.contains("force quit this app"));
}

#[test]
fn prompt_contains_context_json_section() {
    let context_json = serde_json::to_string_pretty(&minimal_context()).unwrap();
    let prompt = build_tab_ai_user_prompt("test", &context_json);

    assert!(prompt.contains("Current context JSON:"));
    assert!(prompt.contains("schemaVersion"));
}

#[test]
fn prompt_requests_fenced_code_block() {
    let context_json = serde_json::to_string_pretty(&minimal_context()).unwrap();
    let prompt = build_tab_ai_user_prompt("test", &context_json);

    assert!(
        prompt.contains("fenced code block"),
        "Prompt must request a fenced code block for script extraction"
    );
}

#[test]
fn prompt_mentions_script_kit_typescript() {
    let context_json = serde_json::to_string_pretty(&minimal_context()).unwrap();
    let prompt = build_tab_ai_user_prompt("test", &context_json);

    assert!(prompt.contains("Script Kit TypeScript"));
}

#[test]
fn rich_context_serializes_all_fields() {
    let blob = rich_context();
    let json = serde_json::to_string_pretty(&blob).unwrap();

    // Verify all expected camelCase fields are present
    assert!(json.contains("schemaVersion"));
    assert!(json.contains("promptType"));
    assert!(json.contains("inputText"));
    assert!(json.contains("focusedSemanticId"));
    assert!(json.contains("selectedSemanticId"));
    assert!(json.contains("visibleElements"));
    assert!(json.contains("recentInputs"));
    assert!(json.contains("clipboardPreview"));
    assert!(json.contains("frontmostApp"));
    assert!(json.contains("selectedText"));

    // Verify actual values
    assert!(json.contains("ArgPrompt"));
    assert!(json.contains("Slack"));
    assert!(json.contains("hello world"));
    assert!(json.contains("https://example.com"));
    assert!(json.contains("copy url"));
    assert!(json.contains("clipboard text preview"));
}

#[test]
fn rich_context_prompt_includes_all_context_data() {
    let blob = rich_context();
    let context_json = serde_json::to_string_pretty(&blob).unwrap();
    let prompt = build_tab_ai_user_prompt("force quit this app", &context_json);

    // Intent is present
    assert!(prompt.contains("force quit this app"));

    // Context data flows through
    assert!(prompt.contains("ArgPrompt"));
    assert!(prompt.contains("Slack"));
    assert!(prompt.contains("hello world"));
    assert!(prompt.contains("https://example.com"));
}

#[test]
fn context_blob_json_roundtrip() {
    let blob = rich_context();
    let json = serde_json::to_string(&blob).unwrap();
    let parsed: TabAiContextBlob = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.schema_version, TAB_AI_CONTEXT_SCHEMA_VERSION);
    assert_eq!(parsed.ui.prompt_type, "ArgPrompt");
    assert_eq!(parsed.ui.input_text.as_deref(), Some("Slack"));
    assert_eq!(parsed.recent_inputs.len(), 2);
    assert_eq!(
        parsed.clipboard_preview.as_deref(),
        Some("clipboard text preview")
    );
}

#[test]
fn minimal_context_omits_empty_optional_fields() {
    let blob = minimal_context();
    let json = serde_json::to_string(&blob).unwrap();

    // These should be omitted when None/empty
    assert!(!json.contains("inputText"));
    assert!(!json.contains("focusedSemanticId"));
    assert!(!json.contains("selectedSemanticId"));
    assert!(!json.contains("visibleElements"));
    assert!(!json.contains("recentInputs"));
    assert!(!json.contains("clipboardPreview"));
}

/// Lock the `src/ai/mod.rs` re-export path: if a re-export breaks, this test
/// fails immediately — no need to run the full binary to discover the gap.
#[test]
fn public_ai_exports_cover_tab_ai_prompt_and_context_types() {
    use script_kit_gpui::context_snapshot::{
        AiContextSnapshot, BrowserContext, FrontmostAppContext,
    };

    let prompt = build_tab_ai_user_prompt("force quit", r#"{"ui":{"promptType":"AppLauncher"}}"#);

    let blob = TabAiContextBlob::from_parts(
        TabAiUiSnapshot {
            prompt_type: "AppLauncher".to_string(),
            input_text: Some("Slack".to_string()),
            ..Default::default()
        },
        AiContextSnapshot {
            frontmost_app: Some(FrontmostAppContext {
                name: "Slack".to_string(),
                bundle_id: "com.tinyspeck.slackmacgap".to_string(),
                pid: 1234,
            }),
            browser: Some(BrowserContext {
                url: "https://example.com".to_string(),
            }),
            ..Default::default()
        },
        vec!["force quit".to_string()],
        None,
        "2026-03-28T00:00:00Z".to_string(),
    );

    assert!(prompt.contains("force quit"));
    assert!(prompt.contains("single fenced code block"));
    assert_eq!(blob.schema_version, TAB_AI_CONTEXT_SCHEMA_VERSION);
    assert_eq!(blob.ui.prompt_type, "AppLauncher");
    assert_eq!(
        blob.desktop
            .frontmost_app
            .as_ref()
            .map(|app| app.name.as_str()),
        Some("Slack")
    );
}

/// Multiline intent must flow through the prompt unchanged.
#[test]
fn multiline_intent_preserved_in_prompt() {
    let context_json = serde_json::to_string(&minimal_context()).unwrap();
    let prompt = build_tab_ai_user_prompt("rename selection\nthen copy it", &context_json);

    assert!(prompt.contains("rename selection\nthen copy it"));
    assert!(prompt.contains("Script Kit TypeScript"));
}
