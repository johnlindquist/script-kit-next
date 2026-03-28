//! Tab AI context assembly types.
//!
//! Defines the schema-versioned context blob sent to the AI model when the
//! user submits an intent from the Tab AI overlay.  The blob combines a UI
//! snapshot (current view, focused element, visible elements) with a desktop
//! context snapshot (frontmost app, selected text, browser URL) and recent
//! input history.

use serde::{Deserialize, Serialize};

/// Schema version for `TabAiContextBlob`. Bump when adding/removing/renaming fields.
pub const TAB_AI_CONTEXT_SCHEMA_VERSION: u32 = 1;

/// Snapshot of the Script Kit UI state at the moment Tab AI was invoked.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TabAiUiSnapshot {
    /// The `AppView` variant name (e.g. "ScriptList", "ArgPrompt").
    pub prompt_type: String,
    /// Current text in the filter / input field, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_text: Option<String>,
    /// Semantic ID of the focused element (e.g. "input:filter").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_semantic_id: Option<String>,
    /// Semantic ID of the selected element (e.g. "choice:0:slack").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_semantic_id: Option<String>,
    /// Top visible elements (capped to keep token cost low).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visible_elements: Vec<crate::protocol::ElementInfo>,
}

/// Complete context blob sent alongside the user's natural-language intent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TabAiContextBlob {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// ISO-8601 timestamp of when the context was assembled.
    pub timestamp: String,
    /// UI state at invocation time.
    pub ui: TabAiUiSnapshot,
    /// Desktop context (frontmost app, selected text, browser URL).
    pub desktop: crate::context_snapshot::AiContextSnapshot,
    /// Recent input-history entries (most recent first).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent_inputs: Vec<String>,
    /// Preview of the current clipboard text (truncated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clipboard_preview: Option<String>,
}

impl TabAiContextBlob {
    /// Build a context blob from provided parts — no system calls, fully
    /// deterministic.  Intended for tests and for callers that already hold
    /// resolved data.
    pub fn from_parts(
        ui: TabAiUiSnapshot,
        desktop: crate::context_snapshot::AiContextSnapshot,
        recent_inputs: Vec<String>,
        clipboard_preview: Option<String>,
        timestamp: String,
    ) -> Self {
        Self {
            schema_version: TAB_AI_CONTEXT_SCHEMA_VERSION,
            timestamp,
            ui,
            desktop,
            recent_inputs,
            clipboard_preview,
        }
    }
}

/// Build the user prompt sent to the AI model for Tab AI script generation.
///
/// Combines the user's natural-language intent with a JSON context blob and
/// instructions to return a fenced TypeScript code block.
pub fn build_tab_ai_user_prompt(intent: &str, context_json: &str) -> String {
    format!(
        "User intent:\n{intent}\n\n\
         Current context JSON:\n{context_json}\n\n\
         Generate a minimal Script Kit TypeScript script that acts immediately on this context. \
         Return only runnable code in a single fenced code block."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_ai_context_blob_default_roundtrip() {
        let blob = TabAiContextBlob {
            schema_version: TAB_AI_CONTEXT_SCHEMA_VERSION,
            timestamp: "2026-03-28T00:00:00Z".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_string(&blob).expect("serialize");
        let parsed: TabAiContextBlob = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.schema_version, TAB_AI_CONTEXT_SCHEMA_VERSION);
        assert_eq!(parsed.timestamp, "2026-03-28T00:00:00Z");
    }

    #[test]
    fn tab_ai_ui_snapshot_skips_empty_fields() {
        let snap = TabAiUiSnapshot {
            prompt_type: "ScriptList".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_string(&snap).expect("serialize");
        // Empty optional fields should be omitted
        assert!(!json.contains("inputText"));
        assert!(!json.contains("focusedSemanticId"));
        assert!(!json.contains("visibleElements"));
    }

    #[test]
    fn tab_ai_context_blob_from_parts_deterministic() {
        let ui = TabAiUiSnapshot {
            prompt_type: "ArgPrompt".to_string(),
            input_text: Some("Slack".to_string()),
            focused_semantic_id: Some("input:filter".to_string()),
            selected_semantic_id: Some("choice:0:slack".to_string()),
            visible_elements: vec![crate::protocol::ElementInfo::choice(
                0,
                "Slack",
                "slack",
                true,
            )],
        };
        let desktop = crate::context_snapshot::AiContextSnapshot {
            frontmost_app: Some(crate::context_snapshot::FrontmostAppContext {
                name: "Slack".to_string(),
                bundle_id: "com.tinyspeck.slackmacgap".to_string(),
                pid: 1234,
            }),
            ..Default::default()
        };
        let recent_inputs = vec!["copy url".to_string(), "open finder".to_string()];
        let ts = "2026-03-28T12:00:00Z".to_string();

        let blob =
            TabAiContextBlob::from_parts(ui, desktop, recent_inputs, None, ts.clone());

        assert_eq!(blob.schema_version, TAB_AI_CONTEXT_SCHEMA_VERSION);
        assert_eq!(blob.timestamp, ts);
        assert_eq!(blob.ui.prompt_type, "ArgPrompt");
        assert_eq!(blob.ui.input_text.as_deref(), Some("Slack"));
        assert_eq!(blob.ui.visible_elements.len(), 1);
        assert_eq!(
            blob.desktop.frontmost_app.as_ref().map(|a| a.name.as_str()),
            Some("Slack")
        );
        assert_eq!(blob.recent_inputs.len(), 2);
        assert!(blob.clipboard_preview.is_none());
    }

    #[test]
    fn tab_ai_context_blob_camel_case_json_fields() {
        let ui = TabAiUiSnapshot {
            prompt_type: "ScriptList".to_string(),
            input_text: Some("test".to_string()),
            focused_semantic_id: Some("input:filter".to_string()),
            selected_semantic_id: None,
            visible_elements: vec![],
        };
        let blob = TabAiContextBlob::from_parts(
            ui,
            Default::default(),
            vec!["recent".to_string()],
            Some("clipboard text".to_string()),
            "2026-03-28T00:00:00Z".to_string(),
        );
        let json = serde_json::to_string(&blob).expect("serialize");

        // Verify camelCase field names in JSON output
        assert!(json.contains("schemaVersion"));
        assert!(json.contains("promptType"));
        assert!(json.contains("inputText"));
        assert!(json.contains("focusedSemanticId"));
        assert!(json.contains("recentInputs"));
        assert!(json.contains("clipboardPreview"));

        // Verify snake_case is NOT present
        assert!(!json.contains("schema_version"));
        assert!(!json.contains("prompt_type"));
        assert!(!json.contains("input_text"));
        assert!(!json.contains("recent_inputs"));
    }

    #[test]
    fn tab_ai_context_blob_json_roundtrip_with_all_fields() {
        let ui = TabAiUiSnapshot {
            prompt_type: "ClipboardHistory".to_string(),
            input_text: Some("search term".to_string()),
            focused_semantic_id: Some("choice:2:item".to_string()),
            selected_semantic_id: Some("choice:2:item".to_string()),
            visible_elements: vec![
                crate::protocol::ElementInfo::input("filter", Some("search term"), true),
                crate::protocol::ElementInfo::choice(0, "Item A", "a", false),
                crate::protocol::ElementInfo::choice(1, "Item B", "b", false),
                crate::protocol::ElementInfo::choice(2, "Item C", "item", true),
            ],
        };
        let desktop = crate::context_snapshot::AiContextSnapshot {
            frontmost_app: Some(crate::context_snapshot::FrontmostAppContext {
                name: "Chrome".to_string(),
                bundle_id: "com.google.Chrome".to_string(),
                pid: 5678,
            }),
            selected_text: Some("selected words".to_string()),
            browser: Some(crate::context_snapshot::BrowserContext {
                url: "https://example.com".to_string(),
            }),
            ..Default::default()
        };
        let blob = TabAiContextBlob::from_parts(
            ui,
            desktop,
            vec!["cmd1".to_string(), "cmd2".to_string(), "cmd3".to_string()],
            Some("clipboard preview".to_string()),
            "2026-03-28T18:30:00Z".to_string(),
        );

        let json = serde_json::to_string_pretty(&blob).expect("serialize");
        let parsed: TabAiContextBlob = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.schema_version, TAB_AI_CONTEXT_SCHEMA_VERSION);
        assert_eq!(parsed.ui.prompt_type, "ClipboardHistory");
        assert_eq!(parsed.ui.visible_elements.len(), 4);
        assert_eq!(parsed.desktop.selected_text.as_deref(), Some("selected words"));
        assert_eq!(
            parsed.desktop.browser.as_ref().map(|b| b.url.as_str()),
            Some("https://example.com")
        );
        assert_eq!(parsed.recent_inputs.len(), 3);
        assert_eq!(parsed.clipboard_preview.as_deref(), Some("clipboard preview"));
    }

    #[test]
    fn tab_ai_context_schema_version_is_one() {
        assert_eq!(TAB_AI_CONTEXT_SCHEMA_VERSION, 1);
    }
}
