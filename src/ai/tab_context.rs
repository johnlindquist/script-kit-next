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

/// Schema version for `TabAiExecutionRecord`. Bump when adding/removing/renaming fields.
pub const TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION: u32 = 2;

/// Schema version for `TabAiExecutionReceipt`. Bump when adding/removing/renaming fields.
pub const TAB_AI_EXECUTION_RECEIPT_SCHEMA_VERSION: u32 = 1;

/// Execution lifecycle status for append-only audit receipts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TabAiExecutionStatus {
    Dispatched,
    Succeeded,
    Failed,
}

/// Record captured at dispatch time and carried forward until completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabAiExecutionRecord {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// The user's original natural-language intent.
    pub intent: String,
    /// The TypeScript source the AI generated.
    pub generated_source: String,
    /// Path to the temp `.ts` file that was executed.
    pub temp_script_path: String,
    /// Slug derived from the AI response (used for save naming).
    pub slug: String,
    /// The `AppView` variant name at invocation time.
    pub prompt_type: String,
    /// Bundle ID of the frontmost app at invocation time, if captured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    /// AI model identifier used for generation.
    #[serde(default)]
    pub model_id: String,
    /// AI provider identifier used for generation.
    #[serde(default)]
    pub provider_id: String,
    /// Number of context-assembly warnings at build time.
    #[serde(default)]
    pub context_warning_count: usize,
    /// ISO-8601 timestamp when the script was executed.
    pub executed_at: String,
}

impl TabAiExecutionRecord {
    /// Build a record from parts — fully deterministic, no system calls.
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        intent: String,
        generated_source: String,
        temp_script_path: String,
        slug: String,
        prompt_type: String,
        bundle_id: Option<String>,
        model_id: String,
        provider_id: String,
        context_warning_count: usize,
        executed_at: String,
    ) -> Self {
        Self {
            schema_version: TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION,
            intent,
            generated_source,
            temp_script_path,
            slug,
            prompt_type,
            bundle_id,
            model_id,
            provider_id,
            context_warning_count,
            executed_at,
        }
    }
}

/// Append-only audit receipt written on dispatch and again on completion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TabAiExecutionReceipt {
    pub schema_version: u32,
    pub status: TabAiExecutionStatus,
    pub intent: String,
    pub slug: String,
    pub prompt_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    pub model_id: String,
    pub provider_id: String,
    pub temp_script_path: String,
    pub context_warning_count: usize,
    pub save_offer_eligible: bool,
    pub memory_write_eligible: bool,
    pub cleanup_attempted: bool,
    pub cleanup_succeeded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub written_at: String,
}

/// Returns the file path for the Tab AI execution audit log.
///
/// Located at `~/.scriptkit/scripts/.tab-ai-executions.jsonl`.
pub fn tab_ai_execution_audit_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME")
        .map_err(|_| "tab_ai_execution_audit_path: HOME is not set".to_string())?;
    Ok(std::path::Path::new(&home)
        .join(".scriptkit")
        .join("scripts")
        .join(".tab-ai-executions.jsonl"))
}

/// Build an audit receipt from a record and completion metadata.
pub fn build_tab_ai_execution_receipt(
    record: &TabAiExecutionRecord,
    status: TabAiExecutionStatus,
    cleanup_attempted: bool,
    cleanup_succeeded: bool,
    error: Option<String>,
) -> TabAiExecutionReceipt {
    let memory_write_eligible = matches!(status, TabAiExecutionStatus::Succeeded);
    let save_offer_eligible = memory_write_eligible && should_offer_save(record);

    TabAiExecutionReceipt {
        schema_version: TAB_AI_EXECUTION_RECEIPT_SCHEMA_VERSION,
        status,
        intent: record.intent.clone(),
        slug: record.slug.clone(),
        prompt_type: record.prompt_type.clone(),
        bundle_id: record.bundle_id.clone(),
        model_id: record.model_id.clone(),
        provider_id: record.provider_id.clone(),
        temp_script_path: record.temp_script_path.clone(),
        context_warning_count: record.context_warning_count,
        save_offer_eligible,
        memory_write_eligible,
        cleanup_attempted,
        cleanup_succeeded,
        error,
        written_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Append a single audit receipt as one JSON line to the JSONL audit log.
pub fn append_tab_ai_execution_receipt(receipt: &TabAiExecutionReceipt) -> Result<(), String> {
    append_tab_ai_execution_receipt_to_path(receipt, &tab_ai_execution_audit_path()?)
}

/// Append a single audit receipt to a specific JSONL path (test-friendly).
pub fn append_tab_ai_execution_receipt_to_path(
    receipt: &TabAiExecutionReceipt,
    path: &std::path::Path,
) -> Result<(), String> {
    use std::io::Write as _;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "tab_ai_execution_audit_dir_failed: path={} error={}",
                parent.display(),
                e
            )
        })?;
    }

    let line = serde_json::to_string(receipt)
        .map_err(|e| format!("tab_ai_execution_audit_serialize_failed: error={}", e))?;

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| {
            format!(
                "tab_ai_execution_audit_open_failed: path={} error={}",
                path.display(),
                e
            )
        })?;

    writeln!(file, "{}", line).map_err(|e| {
        format!(
            "tab_ai_execution_audit_write_failed: path={} error={}",
            path.display(),
            e
        )
    })?;

    tracing::info!(
        event = "tab_ai_execution_audit_written",
        status = ?receipt.status,
        slug = %receipt.slug,
        prompt_type = %receipt.prompt_type,
        model_id = %receipt.model_id,
        provider_id = %receipt.provider_id,
    );

    Ok(())
}

/// Schema version for `TabAiMemoryEntry`. Bump when adding/removing/renaming fields.
pub const TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION: u32 = 1;

/// Lightweight entry persisted to the Tab AI memory index for future intent matching.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TabAiMemoryEntry {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// The user's original natural-language intent.
    pub intent: String,
    /// The TypeScript source the AI generated.
    pub generated_source: String,
    /// Slug derived from the AI response.
    pub slug: String,
    /// The `AppView` variant name at invocation time.
    pub prompt_type: String,
    /// Bundle ID of the frontmost app, if captured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    /// ISO-8601 timestamp when the entry was written.
    pub written_at: String,
}

/// Returns the file path for the Tab AI memory index.
///
/// Located at `~/.scriptkit/scripts/.tab-ai-memory.json`.
pub fn tab_ai_memory_index_path() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME")
        .map_err(|_| "tab_ai_memory_index_path: HOME is not set".to_string())?;
    Ok(std::path::Path::new(&home)
        .join(".scriptkit")
        .join("scripts")
        .join(".tab-ai-memory.json"))
}

/// Read the Tab AI memory index from an explicit path.
///
/// Returns an empty `Vec` if the index file does not exist.
pub fn read_tab_ai_memory_index_from_path(
    path: &std::path::Path,
) -> Result<Vec<TabAiMemoryEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = std::fs::read_to_string(path).map_err(|e| {
        format!(
            "tab_ai_memory_read_failed: path={} error={}",
            path.display(),
            e
        )
    })?;
    serde_json::from_str(&json).map_err(|e| {
        format!(
            "tab_ai_memory_parse_failed: path={} error={}",
            path.display(),
            e
        )
    })
}

/// Read the Tab AI memory index from the default location.
pub fn read_tab_ai_memory_index() -> Result<Vec<TabAiMemoryEntry>, String> {
    let path = tab_ai_memory_index_path()?;
    read_tab_ai_memory_index_from_path(&path)
}

/// Write a Tab AI memory entry to an explicit path.
///
/// Appends to the existing index (deduplicating by intent + bundle_id),
/// then writes back to disk.  Returns the entry that was written.
pub fn write_tab_ai_memory_entry_to_path(
    record: &TabAiExecutionRecord,
    path: &std::path::Path,
) -> Result<TabAiMemoryEntry, String> {
    let entry = TabAiMemoryEntry {
        schema_version: TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
        intent: record.intent.clone(),
        generated_source: record.generated_source.clone(),
        slug: record.slug.clone(),
        prompt_type: record.prompt_type.clone(),
        bundle_id: record.bundle_id.clone(),
        written_at: record.executed_at.clone(),
    };

    let mut entries = read_tab_ai_memory_index_from_path(path)?;

    // Deduplicate: remove older entry with same intent + bundle_id
    entries.retain(|existing| {
        !(existing.intent == entry.intent && existing.bundle_id == entry.bundle_id)
    });

    entries.push(entry.clone());

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "tab_ai_memory_dir_failed: path={} error={}",
                parent.display(),
                e
            )
        })?;
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| format!("tab_ai_memory_serialize_failed: error={}", e))?;
    std::fs::write(path, json).map_err(|e| {
        format!(
            "tab_ai_memory_write_failed: path={} error={}",
            path.display(),
            e
        )
    })?;

    tracing::info!(
        event = "tab_ai_memory_written",
        intent = %record.intent,
        slug = %record.slug,
        prompt_type = %record.prompt_type,
    );

    Ok(entry)
}

/// Write a Tab AI memory entry to the default location.
pub fn write_tab_ai_memory_entry(
    record: &TabAiExecutionRecord,
) -> Result<TabAiMemoryEntry, String> {
    let path = tab_ai_memory_index_path()?;
    write_tab_ai_memory_entry_to_path(record, &path)
}

/// Clean up a temporary script file created for Tab AI execution.
///
/// Returns `true` if the file was successfully removed (or already absent),
/// `false` if removal failed.
pub fn cleanup_tab_ai_temp_script(path: &str) -> bool {
    let p = std::path::Path::new(path);
    if !p.exists() {
        tracing::info!(
            event = "tab_ai_temp_cleanup_noop",
            path = %path,
            reason = "already_absent",
        );
        return true;
    }
    match std::fs::remove_file(p) {
        Ok(()) => {
            tracing::info!(
                event = "tab_ai_temp_cleanup_success",
                path = %path,
            );
            true
        }
        Err(e) => {
            tracing::warn!(
                event = "tab_ai_temp_cleanup_failed",
                path = %path,
                error = %e,
            );
            false
        }
    }
}

/// Decide whether to offer "Save as script?" after a successful Tab AI execution.
///
/// Requires at least 3 non-empty lines — trivial one-liners are not worth saving.
pub fn should_offer_save(record: &TabAiExecutionRecord) -> bool {
    let non_empty_line_count = record
        .generated_source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let offer = non_empty_line_count >= 3;
    tracing::info!(
        event = "tab_ai_save_offer_decision",
        offer,
        slug = %record.slug,
        model_id = %record.model_id,
        provider_id = %record.provider_id,
        source_len = record.generated_source.len(),
        context_warning_count = record.context_warning_count,
    );
    offer
}

#[cfg(test)]
mod execution_record_compat_tests {
    use super::*;

    #[test]
    fn legacy_v1_execution_record_fixture_still_deserializes() {
        let json = std::fs::read_to_string("tests/fixtures/tab_ai_execution_record_v1.json")
            .expect("missing tests/fixtures/tab_ai_execution_record_v1.json");
        let record: TabAiExecutionRecord =
            serde_json::from_str(&json).expect("legacy v1 record should deserialize");
        assert!(!record.intent.is_empty());
        assert!(!record.generated_source.is_empty());
        assert_eq!(record.context_warning_count, 0);
        assert!(record.model_id.is_empty(), "v1 had no model_id — default should be empty string");
        assert!(
            record.provider_id.is_empty(),
            "v1 had no provider_id — default should be empty string"
        );

        tracing::info!(
            event = "execution_record_compat_test_passed",
            schema_version = record.schema_version,
            intent = %record.intent,
            context_warning_count = record.context_warning_count,
        );
    }

    #[test]
    fn v2_record_with_all_fields_still_deserializes() {
        let json = r#"{
            "schemaVersion": 2,
            "intent": "open browser",
            "generatedSource": "line1\nline2\nline3",
            "tempScriptPath": "/tmp/test.ts",
            "slug": "open-browser",
            "promptType": "ScriptList",
            "modelId": "gpt-4.1",
            "providerId": "vercel",
            "contextWarningCount": 2,
            "executedAt": "2026-03-28T00:00:00Z"
        }"#;
        let record: TabAiExecutionRecord =
            serde_json::from_str(json).expect("v2 record should deserialize");
        assert_eq!(record.schema_version, 2);
        assert_eq!(record.model_id, "gpt-4.1");
        assert_eq!(record.provider_id, "vercel");
        assert_eq!(record.context_warning_count, 2);
    }
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
                0, "Slack", "slack", true,
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

        let blob = TabAiContextBlob::from_parts(ui, desktop, recent_inputs, None, ts.clone());

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
        assert_eq!(
            parsed.desktop.selected_text.as_deref(),
            Some("selected words")
        );
        assert_eq!(
            parsed.desktop.browser.as_ref().map(|b| b.url.as_str()),
            Some("https://example.com")
        );
        assert_eq!(parsed.recent_inputs.len(), 3);
        assert_eq!(
            parsed.clipboard_preview.as_deref(),
            Some("clipboard preview")
        );
    }

    #[test]
    fn tab_ai_context_schema_version_is_one() {
        assert_eq!(TAB_AI_CONTEXT_SCHEMA_VERSION, 1);
    }

    #[test]
    fn tab_ai_context_blob_omits_empty_optional_fields() {
        let blob = TabAiContextBlob::from_parts(
            TabAiUiSnapshot {
                prompt_type: "ScriptList".to_string(),
                ..Default::default()
            },
            Default::default(),
            vec![],
            None,
            "2026-03-28T00:00:00Z".to_string(),
        );
        let json = serde_json::to_string(&blob).expect("serialize");
        assert!(json.contains("\"schemaVersion\":1"));
        assert!(
            !json.contains("recentInputs"),
            "empty Vec should be omitted"
        );
        assert!(!json.contains("clipboardPreview"), "None should be omitted");
    }

    #[test]
    fn tab_ai_user_prompt_preserves_multiline_intent_and_contract() {
        let prompt = build_tab_ai_user_prompt(
            "rename selection\nthen copy it",
            r#"{"ui":{"promptType":"ScriptList"}}"#,
        );
        assert!(prompt.contains("User intent:\nrename selection\nthen copy it"));
        assert!(prompt.contains("Current context JSON:\n{\"ui\":{\"promptType\":\"ScriptList\"}}"));
        assert!(prompt.contains("Script Kit TypeScript"));
        assert!(prompt.contains("single fenced code block"));
    }

    // --- TabAiExecutionRecord tests ---

    fn sample_execution_record() -> TabAiExecutionRecord {
        TabAiExecutionRecord::from_parts(
            "force quit Slack".to_string(),
            "import '@anthropic-ai/sdk';\nawait exec('kill Slack');\nconsole.log('done');"
                .to_string(),
            "/tmp/scriptlet-abc123.ts".to_string(),
            "force-quit-slack".to_string(),
            "AppLauncher".to_string(),
            Some("com.tinyspeck.slackmacgap".to_string()),
            "gpt-4.1".to_string(),
            "vercel".to_string(),
            0,
            "2026-03-28T12:00:00Z".to_string(),
        )
    }

    #[test]
    fn tab_ai_execution_record_from_parts_sets_schema_version() {
        let record = sample_execution_record();
        assert_eq!(
            record.schema_version,
            TAB_AI_EXECUTION_RECORD_SCHEMA_VERSION
        );
        assert_eq!(record.intent, "force quit Slack");
        assert_eq!(record.slug, "force-quit-slack");
        assert_eq!(record.prompt_type, "AppLauncher");
    }

    #[test]
    fn tab_ai_execution_record_serde_roundtrip() {
        let record = sample_execution_record();
        let json = serde_json::to_string(&record).expect("serialize");
        let parsed: TabAiExecutionRecord = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.schema_version, record.schema_version);
        assert_eq!(parsed.intent, record.intent);
        assert_eq!(parsed.slug, record.slug);
        assert_eq!(parsed.bundle_id, record.bundle_id);
    }

    #[test]
    fn tab_ai_execution_record_omits_none_bundle_id() {
        let record = TabAiExecutionRecord::from_parts(
            "test".to_string(),
            "code".to_string(),
            "/tmp/x.ts".to_string(),
            "test".to_string(),
            "ScriptList".to_string(),
            None,
            "gpt-4.1".to_string(),
            "vercel".to_string(),
            0,
            "2026-03-28T00:00:00Z".to_string(),
        );
        let json = serde_json::to_string(&record).expect("serialize");
        assert!(!json.contains("bundleId"));
    }

    #[test]
    fn should_offer_save_returns_true_for_three_plus_lines() {
        let record = sample_execution_record();
        // sample has 3 non-empty lines
        assert!(should_offer_save(&record));
    }

    #[test]
    fn should_offer_save_returns_false_for_fewer_than_three_lines() {
        let record = TabAiExecutionRecord::from_parts(
            "test".to_string(),
            "one\ntwo".to_string(),
            "/tmp/x.ts".to_string(),
            "test".to_string(),
            "ScriptList".to_string(),
            None,
            "gpt-4.1".to_string(),
            "vercel".to_string(),
            0,
            "2026-03-28T00:00:00Z".to_string(),
        );
        assert!(!should_offer_save(&record));
    }

    #[test]
    fn should_offer_save_returns_false_for_empty_source() {
        let record = TabAiExecutionRecord::from_parts(
            "test".to_string(),
            "   ".to_string(),
            "/tmp/x.ts".to_string(),
            "test".to_string(),
            "ScriptList".to_string(),
            None,
            "gpt-4.1".to_string(),
            "vercel".to_string(),
            0,
            "2026-03-28T00:00:00Z".to_string(),
        );
        assert!(!should_offer_save(&record));
    }

    // --- TabAiExecutionReceipt tests ---

    #[test]
    fn append_tab_ai_execution_receipt_writes_one_json_line() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join(".tab-ai-executions.jsonl");

        let record = sample_execution_record();
        let receipt = build_tab_ai_execution_receipt(
            &record,
            TabAiExecutionStatus::Dispatched,
            false,
            false,
            None,
        );
        append_tab_ai_execution_receipt_to_path(&receipt, &path).expect("append");

        let content = std::fs::read_to_string(&path).expect("read");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1, "exactly one line per receipt");

        let parsed: TabAiExecutionReceipt = serde_json::from_str(lines[0]).expect("valid JSON");
        assert_eq!(parsed.status, TabAiExecutionStatus::Dispatched);
        assert_eq!(parsed.slug, "force-quit-slack");
        assert_eq!(parsed.model_id, "gpt-4.1");
        assert_eq!(parsed.provider_id, "vercel");
        assert!(!parsed.save_offer_eligible);
        assert!(!parsed.memory_write_eligible);

        // camelCase check
        assert!(lines[0].contains("modelId"));
        assert!(lines[0].contains("providerId"));
        assert!(lines[0].contains("contextWarningCount"));
        assert!(lines[0].contains("saveOfferEligible"));
        assert!(!lines[0].contains("model_id"));
        assert!(!lines[0].contains("provider_id"));
    }

    #[test]
    fn append_receipt_is_append_only() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join(".tab-ai-executions.jsonl");

        let record = sample_execution_record();

        let r1 = build_tab_ai_execution_receipt(
            &record,
            TabAiExecutionStatus::Dispatched,
            false,
            false,
            None,
        );
        append_tab_ai_execution_receipt_to_path(&r1, &path).expect("append 1");

        let r2 = build_tab_ai_execution_receipt(
            &record,
            TabAiExecutionStatus::Succeeded,
            true,
            true,
            None,
        );
        append_tab_ai_execution_receipt_to_path(&r2, &path).expect("append 2");

        let content = std::fs::read_to_string(&path).expect("read");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2, "two receipts = two lines");

        let p1: TabAiExecutionReceipt = serde_json::from_str(lines[0]).expect("parse line 1");
        let p2: TabAiExecutionReceipt = serde_json::from_str(lines[1]).expect("parse line 2");
        assert_eq!(p1.status, TabAiExecutionStatus::Dispatched);
        assert_eq!(p2.status, TabAiExecutionStatus::Succeeded);
        assert!(p2.save_offer_eligible);
        assert!(p2.memory_write_eligible);
    }

    #[test]
    fn build_receipt_sets_eligibility_based_on_status() {
        let record = sample_execution_record();

        let dispatched = build_tab_ai_execution_receipt(
            &record,
            TabAiExecutionStatus::Dispatched,
            false,
            false,
            None,
        );
        assert!(!dispatched.memory_write_eligible);
        assert!(!dispatched.save_offer_eligible);

        let succeeded = build_tab_ai_execution_receipt(
            &record,
            TabAiExecutionStatus::Succeeded,
            true,
            true,
            None,
        );
        assert!(succeeded.memory_write_eligible);
        assert!(succeeded.save_offer_eligible);

        let failed = build_tab_ai_execution_receipt(
            &record,
            TabAiExecutionStatus::Failed,
            true,
            true,
            Some("exit code 1".to_string()),
        );
        assert!(!failed.memory_write_eligible);
        assert!(!failed.save_offer_eligible);
        assert_eq!(failed.error.as_deref(), Some("exit code 1"));
    }

    #[test]
    fn cleanup_tab_ai_temp_script_returns_true_for_absent_file() {
        assert!(cleanup_tab_ai_temp_script(
            "/tmp/nonexistent-tab-ai-test-12345.ts"
        ));
    }

    #[test]
    fn cleanup_tab_ai_temp_script_removes_existing_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let path = dir.path().join("tab-ai-test-cleanup.ts");
        std::fs::write(&path, "console.log('cleanup test')").expect("write test file");
        assert!(path.exists());
        assert!(cleanup_tab_ai_temp_script(path.to_str().expect("utf8")));
        assert!(!path.exists());
    }

    #[test]
    fn tab_ai_memory_entry_serde_roundtrip() {
        let entry = TabAiMemoryEntry {
            schema_version: TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
            intent: "copy url".to_string(),
            generated_source: "await copy(browser.url)".to_string(),
            slug: "copy-url".to_string(),
            prompt_type: "ScriptList".to_string(),
            bundle_id: Some("com.google.Chrome".to_string()),
            written_at: "2026-03-28T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        let parsed: TabAiMemoryEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, entry);
    }

    #[test]
    fn tab_ai_memory_entry_omits_none_bundle_id() {
        let entry = TabAiMemoryEntry {
            schema_version: TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
            intent: "test".to_string(),
            generated_source: "code".to_string(),
            slug: "test".to_string(),
            prompt_type: "ScriptList".to_string(),
            bundle_id: None,
            written_at: "2026-03-28T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        assert!(!json.contains("bundleId"));
    }
}
