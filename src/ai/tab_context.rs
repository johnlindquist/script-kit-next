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
}
