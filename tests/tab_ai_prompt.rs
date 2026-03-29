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
        vec![],
        vec![],
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
        Some(script_kit_gpui::ai::TabAiClipboardContext {
            content_type: "text".to_string(),
            preview: "clipboard text preview".to_string(),
            ocr_text: None,
        }),
        vec![],
        vec![],
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

    assert!(prompt.contains("Context JSON:"));
    assert!(prompt.contains("schemaVersion"));
}

#[test]
fn prompt_requests_fenced_code_block() {
    let context_json = serde_json::to_string_pretty(&minimal_context()).unwrap();
    let prompt = build_tab_ai_user_prompt("test", &context_json);

    assert!(
        prompt.contains("fenced ```ts block"),
        "Prompt must request a fenced ts code block for script extraction"
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
    assert!(json.contains("contentType"));
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
        parsed.clipboard.as_ref().map(|c| c.preview.as_str()),
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
    assert!(!json.contains("clipboard"));
    assert!(!json.contains("priorAutomations"));
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
        vec![],
        vec![],
        "2026-03-28T00:00:00Z".to_string(),
    );

    assert!(prompt.contains("force quit"));
    assert!(prompt.contains("fenced ```ts block"));
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

#[test]
fn build_tab_ai_user_prompt_mentions_clipboard_and_prior_automations() {
    let prompt = build_tab_ai_user_prompt(
        "copy just the url",
        r#"{"clipboard":{"preview":"https://example.com"},"priorAutomations":[{"slug":"copy-url"}]}"#,
    );
    assert!(prompt.contains("clipboard.preview"));
    assert!(prompt.contains("priorAutomations"));
    assert!(prompt.contains("Return only a fenced ```ts block."));
}

// ── Source-level regression tests ─────────────────────────────────────
//
// These tests use `include_str!` to lock the Tab AI overlay source against
// unintentional regressions in footer hints, placeholder copy, and
// memory-hint rendering.

/// The overlay source included once for all source-level assertions.
const TAB_AI_SOURCE: &str = include_str!("../src/app_impl/tab_ai_mode.rs");

#[test]
fn tab_ai_overlay_uses_canonical_three_key_footer_contract() {
    // Source uses \u{21B5} and \u{2318} escape sequences — match the raw text
    assert!(
        TAB_AI_SOURCE.contains(r#""\u{21B5} Send"#),
        "tab ai chat must expose the Send hint in the footer"
    );
    assert!(
        TAB_AI_SOURCE.contains(r#""\u{2318}K Actions"#),
        "tab ai chat must expose the Actions hint in the footer"
    );
    assert!(
        TAB_AI_SOURCE.contains(r#""Esc Back"#),
        "tab ai chat must expose the Esc Back hint in the footer"
    );
}

#[test]
fn tab_ai_overlay_does_not_use_bespoke_esc_footer() {
    assert!(
        !TAB_AI_SOURCE.contains("\"Esc Cancel\""),
        "tab ai overlay must not contain a bespoke Esc Cancel footer entry"
    );
}

#[test]
fn tab_ai_overlay_preserves_memory_hint_rendering() {
    assert!(
        TAB_AI_SOURCE.contains("Similar prior automation:"),
        "visual cleanup must not silently remove memory-hint behavior"
    );
}

#[test]
fn tab_ai_overlay_idle_placeholder_matches_expected_copy() {
    assert!(
        TAB_AI_SOURCE.contains("Ask AI about the current context"),
        "idle placeholder must contain 'Ask AI about the current context'"
    );
}

#[test]
fn tab_ai_overlay_running_placeholder_matches_expected_copy() {
    assert!(
        TAB_AI_SOURCE.contains("Generating"),
        "running placeholder must contain 'Generating'"
    );
}

#[test]
fn tab_ai_save_offer_uses_named_opacity_constants() {
    // The save-offer overlay must not use raw float literals for opacity
    assert!(
        !TAB_AI_SOURCE.contains("0.85,"),
        "save-offer overlay should use OPACITY_NEAR_FULL, not raw 0.85"
    );
    assert!(
        !TAB_AI_SOURCE.contains("0.4,"),
        "save-offer overlay should use OPACITY_DISABLED, not raw 0.4"
    );
}

#[test]
fn tab_ai_save_offer_uses_shared_hint_strip() {
    assert!(
        TAB_AI_SOURCE.contains("HintStrip::new(vec!["),
        "save-offer overlay must use the shared HintStrip component"
    );
    assert!(
        TAB_AI_SOURCE.contains(r#""\u{21B5} Save"#),
        "save-offer overlay must expose the Save hint via HintStrip"
    );
    assert!(
        TAB_AI_SOURCE.contains(r#""Esc Dismiss""#),
        "save-offer overlay must expose the Dismiss hint via HintStrip"
    );
}

#[test]
fn tab_ai_save_offer_is_not_floating_card() {
    // The old floating card used a fixed width (420px) and centered layout
    assert!(
        !TAB_AI_SOURCE.contains("w(px(420.))"),
        "save-offer overlay must not use the old 420px floating card width"
    );
    assert!(
        !TAB_AI_SOURCE.contains("rounded_b("),
        "save-offer overlay must not use bottom-rounded card corners"
    );
}

#[test]
fn tab_ai_save_offer_uses_ghost_opacity_divider() {
    // The save-offer render path must reference OPACITY_GHOST for its divider
    // (both the main overlay and save-offer overlay use it)
    let save_offer_section = TAB_AI_SOURCE
        .find("render_tab_ai_save_offer_overlay")
        .expect("save-offer render function exists");
    let save_offer_code = &TAB_AI_SOURCE[save_offer_section..];
    assert!(
        save_offer_code.contains("OPACITY_GHOST"),
        "save-offer overlay must use OPACITY_GHOST for its divider"
    );
}
