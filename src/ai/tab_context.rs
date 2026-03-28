//! Tab AI context assembly types.
//!
//! Defines the schema-versioned context blob sent to the AI model when the
//! user submits an intent from the Tab AI overlay.  The blob combines a UI
//! snapshot (current view, focused element, visible elements) with a desktop
//! context snapshot (frontmost app, selected text, browser URL) and recent
//! input history.

use serde::{Deserialize, Serialize};

/// Schema version for `TabAiContextBlob`. Bump when adding/removing/renaming fields.
pub const TAB_AI_CONTEXT_SCHEMA_VERSION: u32 = 2;

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

/// Clipboard content summary for Tab AI context.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TabAiClipboardContext {
    /// MIME-like content type (e.g. "text", "image").
    pub content_type: String,
    /// Truncated preview of the clipboard content.
    pub preview: String,
    /// OCR text extracted from clipboard image, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocr_text: Option<String>,
}

/// Truncate a string to at most `limit` characters, appending `…` if truncated.
///
/// Returns an empty string when `limit` is zero.
pub fn truncate_tab_ai_text(value: &str, limit: usize) -> String {
    if limit == 0 {
        return String::new();
    }
    let char_count = value.chars().count();
    if char_count <= limit {
        value.to_string()
    } else {
        let prefix: String = value.chars().take(limit.saturating_sub(1)).collect();
        format!("{prefix}…")
    }
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
    /// Structured clipboard context (content type, preview, optional OCR).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clipboard: Option<TabAiClipboardContext>,
    /// Prior automation suggestions from the Tab AI memory index.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prior_automations: Vec<TabAiMemorySuggestion>,
}

impl TabAiContextBlob {
    /// Build a context blob from provided parts — no system calls, fully
    /// deterministic.  Intended for tests and for callers that already hold
    /// resolved data.
    pub fn from_parts(
        ui: TabAiUiSnapshot,
        desktop: crate::context_snapshot::AiContextSnapshot,
        recent_inputs: Vec<String>,
        clipboard: Option<TabAiClipboardContext>,
        prior_automations: Vec<TabAiMemorySuggestion>,
        timestamp: String,
    ) -> Self {
        Self {
            schema_version: TAB_AI_CONTEXT_SCHEMA_VERSION,
            timestamp,
            ui,
            desktop,
            recent_inputs,
            clipboard,
            prior_automations,
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
         Context JSON:\n\
         ```json\n\
         {context_json}\n\
         ```\n\n\
         Write one valid Script Kit TypeScript script.\n\
         - Use the live context as the source of truth.\n\
         - Prefer ui.selectedSemanticId / ui.focusedSemanticId and visibleElements for in-app targets.\n\
         - Prefer desktop.selectedText, desktop.browser.url, and desktop.frontmostApp for desktop targets.\n\
         - Use clipboard.preview or clipboard.ocrText when the request refers to copied or pasted content.\n\
         - Treat priorAutomations as hints only; borrow their shape if useful, but do not assume they are still correct if live context disagrees.\n\
         - Keep the script short and directly executable.\n\
         - Return only a fenced ```ts block.\n",
        intent = intent.trim(),
        context_json = context_json,
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

// ---------------------------------------------------------------------------
// Tab AI memory suggestion resolver
// ---------------------------------------------------------------------------

/// A suggestion surfaced from the Tab AI memory index for the current intent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TabAiMemorySuggestion {
    pub slug: String,
    pub bundle_id: String,
    pub raw_query: String,
    pub effective_query: String,
    pub prompt_type: String,
    pub written_at: String,
    pub score: f32,
}

/// The reason a memory resolution produced (or failed to produce) suggestions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TabAiMemoryResolutionReason {
    MissingBundleId,
    EmptyQuery,
    ZeroLimit,
    IndexMissing,
    NoCandidatesForBundle,
    BelowThreshold,
    Matched,
}

/// Machine-readable outcome metadata from a memory resolution attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TabAiMemoryResolutionOutcome {
    pub query: String,
    pub normalized_query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_id: Option<String>,
    pub limit: usize,
    pub threshold: f32,
    pub candidate_count: usize,
    pub match_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_score: Option<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_slugs: Vec<String>,
    pub reason: TabAiMemoryResolutionReason,
    pub index_path: String,
}

/// Full resolution result: suggestions plus machine-readable outcome metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TabAiMemoryResolution {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<TabAiMemorySuggestion>,
    pub outcome: TabAiMemoryResolutionOutcome,
}

const TAB_AI_MEMORY_SUGGESTION_MIN_SCORE: f32 = 0.35;

fn normalize_tab_ai_match_text(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    let mut last_was_space = false;

    for ch in input.chars() {
        let ch = if ch == '\u{2192}' { ' ' } else { ch };

        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            last_was_space = false;
        } else if !last_was_space {
            normalized.push(' ');
            last_was_space = true;
        }
    }

    normalized.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tab_ai_token_set(input: &str) -> std::collections::BTreeSet<String> {
    normalize_tab_ai_match_text(input)
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}

fn tab_ai_jaccard_similarity(left: &str, right: &str) -> f32 {
    let left_set = tab_ai_token_set(left);
    let right_set = tab_ai_token_set(right);

    if left_set.is_empty() || right_set.is_empty() {
        return 0.0;
    }

    let intersection = left_set.intersection(&right_set).count() as f32;
    let union = left_set.union(&right_set).count() as f32;

    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

fn score_tab_ai_memory_candidate(query: &str, entry: &TabAiMemoryEntry) -> f32 {
    let query_norm = normalize_tab_ai_match_text(query);
    let intent_norm = normalize_tab_ai_match_text(&entry.intent);

    if query_norm.is_empty() || intent_norm.is_empty() {
        return 0.0;
    }

    if query_norm == intent_norm {
        return 1.0;
    }

    let overlap = tab_ai_jaccard_similarity(&query_norm, &intent_norm);

    // Small bonus when one normalized phrase contains the other.
    // This keeps "force quit app" and "force quit current app" related.
    let contains_bonus = if intent_norm.contains(&query_norm) || query_norm.contains(&intent_norm) {
        0.20
    } else {
        0.0
    };

    (overlap * 0.80) + contains_bonus
}

/// Emit the structured log event for a memory resolution outcome.
fn log_tab_ai_memory_resolution(outcome: &TabAiMemoryResolutionOutcome) {
    tracing::info!(
        event = "tab_ai_memory_resolution",
        query = %outcome.query,
        normalized_query = %outcome.normalized_query,
        bundle_id = ?outcome.bundle_id,
        limit = outcome.limit,
        threshold = outcome.threshold,
        candidate_count = outcome.candidate_count,
        match_count = outcome.match_count,
        top_score = ?outcome.top_score,
        reason = ?outcome.reason,
        matched_slugs = ?outcome.matched_slugs,
        index_path = %outcome.index_path,
    );
}

/// Build the initial outcome template shared by all resolution paths.
fn base_resolution_outcome(
    query: &str,
    normalized_query: &str,
    bundle_id: Option<String>,
    limit: usize,
    index_path: &std::path::Path,
) -> TabAiMemoryResolutionOutcome {
    TabAiMemoryResolutionOutcome {
        query: query.to_string(),
        normalized_query: normalized_query.to_string(),
        bundle_id,
        limit,
        threshold: TAB_AI_MEMORY_SUGGESTION_MIN_SCORE,
        candidate_count: 0,
        match_count: 0,
        top_score: None,
        matched_slugs: Vec::new(),
        reason: TabAiMemoryResolutionReason::Matched,
        index_path: index_path.display().to_string(),
    }
}

/// Canonical, outcome-aware resolver for Tab AI memory suggestions.
/// This is the machine-readable surface callers and tests should prefer.
pub fn resolve_tab_ai_memory_suggestions_with_outcome(
    raw_query: &str,
    bundle_id: Option<&str>,
    limit: usize,
) -> Result<TabAiMemoryResolution, String> {
    resolve_tab_ai_memory_suggestions_with_outcome_from_path(
        raw_query,
        bundle_id,
        limit,
        &tab_ai_memory_index_path()?,
    )
}

/// Outcome-aware resolver against an explicit index path.
pub fn resolve_tab_ai_memory_suggestions_with_outcome_from_path(
    raw_query: &str,
    bundle_id: Option<&str>,
    limit: usize,
    path: &std::path::Path,
) -> Result<TabAiMemoryResolution, String> {
    let query = raw_query.trim().to_string();
    let normalized_query = normalize_tab_ai_match_text(&query);
    let bundle_id_clean = bundle_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);

    let mut outcome = base_resolution_outcome(
        &query,
        &normalized_query,
        bundle_id_clean.clone(),
        limit,
        path,
    );

    // --- Early-exit branches with explicit reasons ---

    if bundle_id_clean.is_none() {
        outcome.reason = TabAiMemoryResolutionReason::MissingBundleId;
        log_tab_ai_memory_resolution(&outcome);
        return Ok(TabAiMemoryResolution {
            suggestions: Vec::new(),
            outcome,
        });
    }

    if query.is_empty() {
        outcome.reason = TabAiMemoryResolutionReason::EmptyQuery;
        log_tab_ai_memory_resolution(&outcome);
        return Ok(TabAiMemoryResolution {
            suggestions: Vec::new(),
            outcome,
        });
    }

    if limit == 0 {
        outcome.reason = TabAiMemoryResolutionReason::ZeroLimit;
        log_tab_ai_memory_resolution(&outcome);
        return Ok(TabAiMemoryResolution {
            suggestions: Vec::new(),
            outcome,
        });
    }

    if !path.exists() {
        outcome.reason = TabAiMemoryResolutionReason::IndexMissing;
        log_tab_ai_memory_resolution(&outcome);
        return Ok(TabAiMemoryResolution {
            suggestions: Vec::new(),
            outcome,
        });
    }

    // --- Read and filter candidates ---

    let bundle_id_norm =
        normalize_tab_ai_match_text(bundle_id_clean.as_deref().unwrap_or_default());

    let bundle_entries: Vec<TabAiMemoryEntry> = read_tab_ai_memory_index_from_path(path)?
        .into_iter()
        .filter(|entry| {
            entry
                .bundle_id
                .as_ref()
                .map(|value| normalize_tab_ai_match_text(value) == bundle_id_norm)
                .unwrap_or(false)
        })
        .collect();

    outcome.candidate_count = bundle_entries.len();

    if bundle_entries.is_empty() {
        outcome.reason = TabAiMemoryResolutionReason::NoCandidatesForBundle;
        log_tab_ai_memory_resolution(&outcome);
        return Ok(TabAiMemoryResolution {
            suggestions: Vec::new(),
            outcome,
        });
    }

    // --- Score and rank ---

    let mut matches: Vec<TabAiMemorySuggestion> = bundle_entries
        .into_iter()
        .filter_map(|entry| {
            let score = score_tab_ai_memory_candidate(&query, &entry);
            if score < TAB_AI_MEMORY_SUGGESTION_MIN_SCORE {
                return None;
            }
            Some(TabAiMemorySuggestion {
                slug: entry.slug,
                bundle_id: entry.bundle_id.unwrap_or_default(),
                raw_query: entry.intent.clone(),
                effective_query: entry.intent,
                prompt_type: entry.prompt_type,
                written_at: entry.written_at,
                score,
            })
        })
        .collect();

    matches.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| right.written_at.cmp(&left.written_at))
            .then_with(|| left.slug.cmp(&right.slug))
    });

    if matches.is_empty() {
        outcome.reason = TabAiMemoryResolutionReason::BelowThreshold;
        log_tab_ai_memory_resolution(&outcome);
        return Ok(TabAiMemoryResolution {
            suggestions: Vec::new(),
            outcome,
        });
    }

    matches.truncate(limit);

    outcome.reason = TabAiMemoryResolutionReason::Matched;
    outcome.match_count = matches.len();
    outcome.top_score = matches.first().map(|item| item.score);
    outcome.matched_slugs = matches.iter().map(|item| item.slug.clone()).collect();

    log_tab_ai_memory_resolution(&outcome);

    Ok(TabAiMemoryResolution {
        suggestions: matches,
        outcome,
    })
}

/// Back-compat wrapper: existing callers can keep asking for just the suggestions.
pub fn resolve_tab_ai_memory_suggestions(
    raw_query: &str,
    bundle_id: Option<&str>,
    limit: usize,
) -> Result<Vec<TabAiMemorySuggestion>, String> {
    Ok(resolve_tab_ai_memory_suggestions_with_outcome(raw_query, bundle_id, limit)?.suggestions)
}

/// Back-compat wrapper against an explicit path.
pub fn resolve_tab_ai_memory_suggestions_from_path(
    raw_query: &str,
    bundle_id: Option<&str>,
    limit: usize,
    path: &std::path::Path,
) -> Result<Vec<TabAiMemorySuggestion>, String> {
    Ok(
        resolve_tab_ai_memory_suggestions_with_outcome_from_path(
            raw_query, bundle_id, limit, path,
        )?
        .suggestions,
    )
}

// ---------------------------------------------------------------------------
// Tab AI invocation receipt — machine-readable richness/degradation signal
// ---------------------------------------------------------------------------

/// Schema version for `TabAiInvocationReceipt`. Bump when adding/removing/renaming fields.
pub const TAB_AI_INVOCATION_RECEIPT_SCHEMA_VERSION: u32 = 1;

/// Tri-state field status used in invocation receipts.
///
/// - `Captured` — data was successfully extracted from the surface.
/// - `Degraded` — the surface structurally supports the data but it could not
///   be extracted (e.g. panel-only element collection, terminal input).
/// - `Unavailable` — the surface has no concept of this data (e.g. webcam has
///   no input text).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TabAiFieldStatus {
    Captured,
    Degraded,
    Unavailable,
}

impl std::fmt::Display for TabAiFieldStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Captured => f.write_str("captured"),
            Self::Degraded => f.write_str("degraded"),
            Self::Unavailable => f.write_str("unavailable"),
        }
    }
}

/// Stable, machine-readable reason code explaining why a field is degraded or
/// unavailable.  These are enumerated so downstream consumers (tests, agents,
/// dashboards) can match on them without parsing free-form strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TabAiDegradationReason {
    /// `collect_visible_elements` returned only `panel:*` placeholders.
    PanelOnlyElements,
    /// `collect_visible_elements` returned zero elements and no warnings.
    NoSemanticElements,
    /// No focused or selected semantic ID was found.
    MissingFocusTarget,
    /// `current_input_text()` returned `None` on a surface that structurally
    /// supports input (e.g. terminal where content exists but is not
    /// user-typed text).
    InputNotExtractable,
    /// The surface has no user-editable text concept at all (e.g. webcam,
    /// drop zone).
    InputNotApplicable,
}

impl std::fmt::Display for TabAiDegradationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PanelOnlyElements => f.write_str("panel_only_elements"),
            Self::NoSemanticElements => f.write_str("no_semantic_elements"),
            Self::MissingFocusTarget => f.write_str("missing_focus_target"),
            Self::InputNotExtractable => f.write_str("input_not_extractable"),
            Self::InputNotApplicable => f.write_str("input_not_applicable"),
        }
    }
}

/// Machine-readable receipt emitted on every Tab AI invocation.
///
/// Identifies the prompt/view type and whether UI context was rich or
/// degraded, with explicit reasons for each degradation.  Designed to be
/// inspectable in tests and parseable from structured logs without human
/// interpretation of free-form strings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TabAiInvocationReceipt {
    /// Schema version for forward compatibility.
    pub schema_version: u32,
    /// `AppView` variant name at invocation time.
    pub prompt_type: String,
    /// Tri-state status for input text extraction.
    pub input_status: TabAiFieldStatus,
    /// Tri-state status for focus/selection target.
    pub focus_status: TabAiFieldStatus,
    /// Tri-state status for semantic element collection.
    pub elements_status: TabAiFieldStatus,
    /// Number of semantic elements collected.
    pub element_count: usize,
    /// Number of element-collection warnings.
    pub warning_count: usize,
    /// Whether any focused or selected semantic ID was captured.
    pub has_focus_target: bool,
    /// Whether input text was captured.
    pub has_input_text: bool,
    /// Machine-readable reason codes for any degraded or unavailable fields.
    /// Empty when all fields are `Captured`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degradation_reasons: Vec<TabAiDegradationReason>,
    /// Overall richness: `true` when all three statuses are `Captured`.
    pub rich: bool,
}

impl TabAiInvocationReceipt {
    /// Build a receipt from snapshot extraction results.
    ///
    /// `input_status_str` / `focus_status_str` are the tri-state strings
    /// already computed in `snapshot_tab_ai_ui` ("captured", "degraded",
    /// "unavailable").  `warnings` are from `ElementCollectionOutcome`.
    pub fn from_snapshot(
        prompt_type: &str,
        input_text: &Option<String>,
        focused_id: &Option<String>,
        selected_id: &Option<String>,
        element_count: usize,
        warnings: &[String],
    ) -> Self {
        // --- input_status ---
        let input_status = if input_text.is_some() {
            TabAiFieldStatus::Captured
        } else {
            // Check if the surface structurally lacks input
            // Names must match what `app_view_name()` returns at runtime.
            let no_input_surfaces = [
                "DivPrompt",
                "DropPrompt",
                "Webcam",
                "CreationFeedback",
                "ActionsDialog",
                "Settings",
                "InstalledKits",
            ];
            if no_input_surfaces.contains(&prompt_type) {
                TabAiFieldStatus::Unavailable
            } else {
                TabAiFieldStatus::Degraded
            }
        };

        // --- focus_status ---
        let has_focus_target = focused_id.is_some() || selected_id.is_some();
        let focus_status = if has_focus_target {
            TabAiFieldStatus::Captured
        } else if warnings.is_empty() {
            TabAiFieldStatus::Unavailable
        } else {
            TabAiFieldStatus::Degraded
        };

        // --- elements_status ---
        let has_panel_only_warning =
            warnings.iter().any(|w| w.starts_with("panel_only_"));
        let elements_status = if element_count > 0 && !has_panel_only_warning {
            TabAiFieldStatus::Captured
        } else if has_panel_only_warning {
            TabAiFieldStatus::Degraded
        } else if element_count == 0 && warnings.is_empty() {
            TabAiFieldStatus::Unavailable
        } else {
            TabAiFieldStatus::Degraded
        };

        // --- degradation_reasons ---
        let mut degradation_reasons = Vec::new();
        if has_panel_only_warning {
            degradation_reasons.push(TabAiDegradationReason::PanelOnlyElements);
        }
        if element_count == 0 && warnings.is_empty() {
            degradation_reasons.push(TabAiDegradationReason::NoSemanticElements);
        }
        if !has_focus_target && (focus_status != TabAiFieldStatus::Unavailable) {
            degradation_reasons.push(TabAiDegradationReason::MissingFocusTarget);
        }
        match input_status {
            TabAiFieldStatus::Degraded => {
                degradation_reasons.push(TabAiDegradationReason::InputNotExtractable);
            }
            TabAiFieldStatus::Unavailable => {
                degradation_reasons.push(TabAiDegradationReason::InputNotApplicable);
            }
            TabAiFieldStatus::Captured => {}
        }

        let rich = input_status == TabAiFieldStatus::Captured
            && focus_status == TabAiFieldStatus::Captured
            && elements_status == TabAiFieldStatus::Captured;

        Self {
            schema_version: TAB_AI_INVOCATION_RECEIPT_SCHEMA_VERSION,
            prompt_type: prompt_type.to_string(),
            input_status,
            focus_status,
            elements_status,
            element_count,
            warning_count: warnings.len(),
            has_focus_target,
            has_input_text: input_text.is_some(),
            degradation_reasons,
            rich,
        }
    }
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
        assert!(
            record.model_id.is_empty(),
            "v1 had no model_id — default should be empty string"
        );
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

        let blob =
            TabAiContextBlob::from_parts(ui, desktop, recent_inputs, None, vec![], ts.clone());

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
        assert!(blob.clipboard.is_none());
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
            Some(TabAiClipboardContext {
                content_type: "text".to_string(),
                preview: "clipboard text".to_string(),
                ocr_text: None,
            }),
            vec![],
            "2026-03-28T00:00:00Z".to_string(),
        );
        let json = serde_json::to_string(&blob).expect("serialize");

        // Verify camelCase field names in JSON output
        assert!(json.contains("schemaVersion"));
        assert!(json.contains("promptType"));
        assert!(json.contains("inputText"));
        assert!(json.contains("focusedSemanticId"));
        assert!(json.contains("recentInputs"));
        assert!(json.contains("contentType"));

        // Verify snake_case is NOT present
        assert!(!json.contains("schema_version"));
        assert!(!json.contains("prompt_type"));
        assert!(!json.contains("input_text"));
        assert!(!json.contains("recent_inputs"));
        assert!(!json.contains("content_type"));
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
            Some(TabAiClipboardContext {
                content_type: "text".to_string(),
                preview: "clipboard preview".to_string(),
                ocr_text: None,
            }),
            vec![],
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
            parsed.clipboard.as_ref().map(|c| c.preview.as_str()),
            Some("clipboard preview")
        );
    }

    #[test]
    fn tab_ai_context_schema_version_is_two() {
        assert_eq!(TAB_AI_CONTEXT_SCHEMA_VERSION, 2);
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
            vec![],
            "2026-03-28T00:00:00Z".to_string(),
        );
        let json = serde_json::to_string(&blob).expect("serialize");
        assert!(json.contains("\"schemaVersion\":2"));
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
    fn tab_ai_user_prompt_preserves_multiline_intent_and_contract() {
        let prompt = build_tab_ai_user_prompt(
            "rename selection\nthen copy it",
            r#"{"ui":{"promptType":"ScriptList"}}"#,
        );
        assert!(prompt.contains("User intent:\nrename selection\nthen copy it"));
        assert!(prompt.contains("Context JSON:"));
        assert!(prompt.contains(r#"{"ui":{"promptType":"ScriptList"}}"#));
        assert!(prompt.contains("Script Kit TypeScript"));
        assert!(prompt.contains("fenced ```ts block"));
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

#[cfg(test)]
mod tab_ai_memory_suggestion_tests {
    use super::*;

    fn memory_entry(
        intent: &str,
        bundle_id: Option<&str>,
        slug: &str,
        written_at: &str,
    ) -> TabAiMemoryEntry {
        TabAiMemoryEntry {
            schema_version: TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
            intent: intent.to_string(),
            generated_source: "import \"@scriptkit/sdk\";\nawait hide();\n".to_string(),
            slug: slug.to_string(),
            prompt_type: "AppLauncher".to_string(),
            bundle_id: bundle_id.map(str::to_string),
            written_at: written_at.to_string(),
        }
    }

    #[test]
    fn resolve_tab_ai_memory_suggestions_returns_similar_non_exact_match() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(".tab-ai-memory.json");
        let entries = vec![memory_entry(
            "force quit current app",
            Some("com.apple.Safari"),
            "force-quit-current-app",
            "2026-03-28T00:00:00Z",
        )];
        std::fs::write(&path, serde_json::to_string_pretty(&entries).expect("ser")).expect("write");

        let results = resolve_tab_ai_memory_suggestions_from_path(
            "force quit app",
            Some("com.apple.Safari"),
            3,
            &path,
        )
        .expect("resolve suggestions");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "force-quit-current-app");
        assert_eq!(results[0].effective_query, "force quit current app");
        assert!(results[0].score >= TAB_AI_MEMORY_SUGGESTION_MIN_SCORE);
    }

    #[test]
    fn resolve_tab_ai_memory_suggestions_filters_by_bundle_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(".tab-ai-memory.json");
        let entries = vec![
            memory_entry(
                "copy browser url",
                Some("com.apple.Safari"),
                "copy-browser-url",
                "2026-03-28T00:00:00Z",
            ),
            memory_entry(
                "copy browser url",
                Some("com.tinyspeck.slackmacgap"),
                "copy-browser-url-slack",
                "2026-03-28T00:00:01Z",
            ),
        ];
        std::fs::write(&path, serde_json::to_string_pretty(&entries).expect("ser")).expect("write");

        let results = resolve_tab_ai_memory_suggestions_from_path(
            "copy url",
            Some("com.apple.Safari"),
            3,
            &path,
        )
        .expect("resolve suggestions");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "copy-browser-url");
        assert_eq!(results[0].bundle_id, "com.apple.Safari");
    }

    #[test]
    fn resolve_tab_ai_memory_suggestions_prefers_exact_match_then_recency() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(".tab-ai-memory.json");
        let entries = vec![
            memory_entry(
                "force quit current app",
                Some("com.apple.Safari"),
                "older-similar",
                "2026-03-28T00:00:00Z",
            ),
            memory_entry(
                "force quit app",
                Some("com.apple.Safari"),
                "exact-match",
                "2026-03-28T00:00:01Z",
            ),
        ];
        std::fs::write(&path, serde_json::to_string_pretty(&entries).expect("ser")).expect("write");

        let results = resolve_tab_ai_memory_suggestions_from_path(
            "force quit app",
            Some("com.apple.Safari"),
            3,
            &path,
        )
        .expect("resolve suggestions");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].slug, "exact-match");
        assert!(results[0].score >= results[1].score);
    }
}

#[cfg(test)]
mod tab_ai_memory_resolution_tests {
    use super::*;

    #[test]
    fn tab_ai_memory_resolution_reports_missing_bundle_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tab-ai-memory.json");

        let resolution = resolve_tab_ai_memory_suggestions_with_outcome_from_path(
            "force quit slack",
            None,
            3,
            &path,
        )
        .expect("resolve");

        assert!(resolution.suggestions.is_empty());
        assert_eq!(
            resolution.outcome.reason,
            TabAiMemoryResolutionReason::MissingBundleId
        );
        assert_eq!(resolution.outcome.candidate_count, 0);
        assert_eq!(resolution.outcome.match_count, 0);
        assert!(resolution.outcome.top_score.is_none());
        assert!(resolution.outcome.matched_slugs.is_empty());
    }

    #[test]
    fn tab_ai_memory_resolution_reports_empty_query() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tab-ai-memory.json");

        let resolution = resolve_tab_ai_memory_suggestions_with_outcome_from_path(
            "   ",
            Some("com.tinyspeck.slackmacgap"),
            3,
            &path,
        )
        .expect("resolve");

        assert!(resolution.suggestions.is_empty());
        assert_eq!(
            resolution.outcome.reason,
            TabAiMemoryResolutionReason::EmptyQuery
        );
    }

    #[test]
    fn tab_ai_memory_resolution_reports_zero_limit() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tab-ai-memory.json");

        let resolution = resolve_tab_ai_memory_suggestions_with_outcome_from_path(
            "force quit",
            Some("com.tinyspeck.slackmacgap"),
            0,
            &path,
        )
        .expect("resolve");

        assert!(resolution.suggestions.is_empty());
        assert_eq!(
            resolution.outcome.reason,
            TabAiMemoryResolutionReason::ZeroLimit
        );
    }

    #[test]
    fn tab_ai_memory_resolution_reports_index_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("missing.json");

        let resolution = resolve_tab_ai_memory_suggestions_with_outcome_from_path(
            "force quit slack",
            Some("com.tinyspeck.slackmacgap"),
            3,
            &path,
        )
        .expect("resolve");

        assert!(resolution.suggestions.is_empty());
        assert_eq!(
            resolution.outcome.reason,
            TabAiMemoryResolutionReason::IndexMissing
        );
        assert!(resolution.outcome.index_path.contains("missing.json"));
    }

    #[test]
    fn tab_ai_memory_resolution_reports_no_candidates_for_bundle() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tab-ai-memory.json");
        let entries = vec![TabAiMemoryEntry {
            schema_version: TAB_AI_MEMORY_ENTRY_SCHEMA_VERSION,
            intent: "force quit".to_string(),
            generated_source: "import \"@scriptkit/sdk\";\n".to_string(),
            slug: "force-quit".to_string(),
            prompt_type: "ScriptList".to_string(),
            bundle_id: Some("com.apple.Safari".to_string()),
            written_at: "2026-03-28T00:00:00Z".to_string(),
        }];
        std::fs::write(&path, serde_json::to_string_pretty(&entries).expect("ser")).expect("write");

        let resolution = resolve_tab_ai_memory_suggestions_with_outcome_from_path(
            "force quit",
            Some("com.tinyspeck.slackmacgap"),
            3,
            &path,
        )
        .expect("resolve");

        assert!(resolution.suggestions.is_empty());
        assert_eq!(
            resolution.outcome.reason,
            TabAiMemoryResolutionReason::NoCandidatesForBundle
        );
        assert_eq!(resolution.outcome.candidate_count, 0);
    }

    #[test]
    fn tab_ai_memory_resolution_prefers_recent_high_score_matches() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tab-ai-memory.json");

        // Both intents share enough tokens with the query "force quit app"
        // to score above the 0.35 threshold.
        let older = TabAiExecutionRecord::from_parts(
            "force quit current app".to_string(),
            "import \"@scriptkit/sdk\";\nawait notify(\"old\");\n".to_string(),
            "/tmp/old.ts".to_string(),
            "force-quit-old".to_string(),
            "ScriptList".to_string(),
            Some("com.tinyspeck.slackmacgap".to_string()),
            "model-a".to_string(),
            "provider-a".to_string(),
            0,
            "2026-03-28T00:00:00Z".to_string(),
        );
        let newer = TabAiExecutionRecord::from_parts(
            "force quit app".to_string(),
            "import \"@scriptkit/sdk\";\nawait notify(\"new\");\n".to_string(),
            "/tmp/new.ts".to_string(),
            "force-quit-new".to_string(),
            "ScriptList".to_string(),
            Some("com.tinyspeck.slackmacgap".to_string()),
            "model-a".to_string(),
            "provider-a".to_string(),
            0,
            "2026-03-28T01:00:00Z".to_string(),
        );

        write_tab_ai_memory_entry_to_path(&older, &path).expect("write older");
        write_tab_ai_memory_entry_to_path(&newer, &path).expect("write newer");

        let resolution = resolve_tab_ai_memory_suggestions_with_outcome_from_path(
            "force quit app",
            Some("com.tinyspeck.slackmacgap"),
            3,
            &path,
        )
        .expect("resolve");

        assert_eq!(
            resolution.outcome.reason,
            TabAiMemoryResolutionReason::Matched
        );
        assert_eq!(resolution.outcome.match_count, 2);
        assert_eq!(resolution.outcome.top_score, Some(1.0));
        // Exact match "force quit app" scores 1.0, should be first
        assert_eq!(
            resolution.suggestions.first().map(|s| s.slug.as_str()),
            Some("force-quit-new")
        );
        assert_eq!(resolution.outcome.candidate_count, 2);
        assert!(!resolution.outcome.matched_slugs.is_empty());
    }

    #[test]
    fn tab_ai_memory_write_dedupes_same_intent_and_bundle() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("tab-ai-memory.json");

        let first = TabAiExecutionRecord::from_parts(
            "copy url".to_string(),
            "import \"@scriptkit/sdk\";\nawait notify(\"a\");\n".to_string(),
            "/tmp/one.ts".to_string(),
            "copy-url-one".to_string(),
            "ScriptList".to_string(),
            Some("com.google.Chrome".to_string()),
            "model-a".to_string(),
            "provider-a".to_string(),
            0,
            "2026-03-28T00:00:00Z".to_string(),
        );
        let second = TabAiExecutionRecord::from_parts(
            "copy url".to_string(),
            "import \"@scriptkit/sdk\";\nawait notify(\"b\");\n".to_string(),
            "/tmp/two.ts".to_string(),
            "copy-url-two".to_string(),
            "ScriptList".to_string(),
            Some("com.google.Chrome".to_string()),
            "model-a".to_string(),
            "provider-a".to_string(),
            0,
            "2026-03-28T01:00:00Z".to_string(),
        );

        write_tab_ai_memory_entry_to_path(&first, &path).expect("write first");
        write_tab_ai_memory_entry_to_path(&second, &path).expect("write second");

        let entries = read_tab_ai_memory_index_from_path(&path).expect("read");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].slug, "copy-url-two");
    }
}
