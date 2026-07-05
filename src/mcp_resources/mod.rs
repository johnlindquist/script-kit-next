//! MCP Resources Handler
//!
//! Implements MCP resources for Script Kit:
//! - `kit://state` - Current app state as JSON
//! - `scripts://` - List of available scripts
//! - `scriptlets://` - List of available scriptlets
//!
//! Resources are read-only data that clients can access without tool calls.

mod transaction_resources;

// --- merged from part_000.rs ---
use crate::scripts::Script;
use crate::scripts::Scriptlet;
use crate::scripts::{FailedScript, ScriptValidationIssue, ValidationReport};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

const NOTES_RESOURCE_URI: &str = "kit://notes";
const NOTES_RESOURCE_SCHEMA_VERSION: u32 = 1;
const AUDIT_RESOURCE_URI: &str = "kit://audit";
const AUDIT_RESOURCE_SCHEMA_VERSION: u32 = 1;
const AUDIT_DEFAULT_LIMIT: usize = 100;
const AUDIT_HARD_LIMIT: usize = 500;
const GIT_DIFF_DEFAULT_LIMIT_BYTES: usize = 1024 * 1024;
const GIT_DIFF_HARD_CAP_BYTES: usize = 8 * 1024 * 1024;
/// MCP Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    /// Unique URI for this resource (e.g., "scripts://", "kit://state")
    pub uri: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this resource provides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// MIME type of the resource content
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}
/// Resource content returned by resources/read
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    /// The URI of the resource
    pub uri: String,
    /// MIME type of the content
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    /// The actual content (typically JSON stringified)
    pub text: String,
}
/// Application state exposed via kit://state resource
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppStateResource {
    /// Whether the app window is visible
    pub visible: bool,
    /// Whether the app window is focused
    pub focused: bool,
    /// Number of loaded scripts
    pub script_count: usize,
    /// Number of loaded scriptlets
    pub scriptlet_count: usize,
    /// Current filter text (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_text: Option<String>,
    /// Currently selected index (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_index: Option<usize>,
}
/// Script metadata for the scripts:// resource
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptResourceEntry {
    /// Script name
    pub name: String,
    /// File path
    pub path: String,
    /// File extension (ts, js)
    pub extension: String,
    /// Description (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether script has a schema (makes it an MCP tool)
    pub has_schema: bool,
}
impl From<&Script> for ScriptResourceEntry {
    fn from(script: &Script) -> Self {
        Self {
            name: script.name.clone(),
            path: script.path.to_string_lossy().to_string(),
            extension: script.extension.clone(),
            description: script.description.clone(),
            has_schema: script.schema.is_some(),
        }
    }
}
/// Scriptlet metadata for the scriptlets:// resource  
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptletResourceEntry {
    /// Scriptlet name
    pub name: String,
    /// Tool type (bash, ts, paste, etc.)
    pub tool: String,
    /// Description (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Group name (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Expand trigger (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    /// Keyboard shortcut (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
}
impl From<&Scriptlet> for ScriptletResourceEntry {
    fn from(scriptlet: &Scriptlet) -> Self {
        Self {
            name: scriptlet.name.clone(),
            tool: scriptlet.tool.clone(),
            description: scriptlet.description.clone(),
            group: scriptlet.group.clone(),
            keyword: scriptlet.keyword.clone(),
            shortcut: scriptlet.shortcut.clone(),
        }
    }
}
/// Get all available MCP resources
pub fn get_resource_definitions() -> Vec<McpResource> {
    let mut resources = vec![
        McpResource {
            uri: "kit://state".to_string(),
            name: "App State".to_string(),
            description: Some(
                "Current Script Kit application state including visibility, focus, and counts"
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "scripts://".to_string(),
            name: "Scripts".to_string(),
            description: Some("List of all available scripts discovered from installed plugins (plugins/main/scripts/ is the default personal plugin). Scripts are loaded from all plugin roots under ~/.scriptkit/plugins/*/scripts/.".to_string()),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "scriptlets://".to_string(),
            name: "Scriptlets".to_string(),
            description: Some("List of all available scriptlets from markdown files".to_string()),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: NOTES_RESOURCE_URI.to_string(),
            name: "Notes".to_string(),
            description: Some(
                "Active Script Kit notes. Read kit://notes for a bounded list with metadata, kit://notes?tag=... to filter organized notes, add &full=true for full bodies, or kit://notes/{id} for a full note."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: crate::brain::resources::BRAIN_RESOURCE_URI.to_string(),
            name: "Brain".to_string(),
            description: Some(
                "Script Kit's local memory. kit://brain for status, kit://brain/recall?q=... for hybrid retrieval, add &format=json for source refs, kit://brain/doc?source=...&sourceId=... for one doc, kit://brain/docs?refs=... for batch doc reads, and kit://brain/signals for recent attention signals."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: AUDIT_RESOURCE_URI.to_string(),
            name: "MCP Audit Log".to_string(),
            description: Some(
                "Recent MCP mutation audit events from ~/.scriptkit/mcp-audit.jsonl. Supports ?limit=100 and ?traceId=..."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: crate::computer_use::COMPUTER_USE_READINESS_RESOURCE_URI.to_string(),
            name: "Computer Use Readiness".to_string(),
            description: Some(
                "Read-only fail-closed preflight receipt for third-party GUI Computer Use readiness."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://context".to_string(),
            name: "Current Context".to_string(),
            description: Some(
                "Deterministic snapshot of AI-relevant desktop context. Supports ?profile=minimal, ?diagnostics=1, and per-field flags: selectedText, frontmostApp, menuBar, browserUrl, focusedWindow, screenshot, panelScreenshot. See kit://context/schema for the full contract."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://context/schema".to_string(),
            name: "Current Context Schema".to_string(),
            description: Some(
                "Self-describing schema for kit://context profiles, flags, diagnostics output, and example URIs."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://scripts".to_string(),
            name: "Scripts (versioned)".to_string(),
            description: Some(
                "Schema-versioned list of all scripts discovered from installed plugins with metadata. plugins/main/scripts/ is the default personal plugin. Safe for repeated reads."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://scriptlets".to_string(),
            name: "Scriptlets (versioned)".to_string(),
            description: Some(
                "Schema-versioned list of all scriptlets from markdown extension files with metadata."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://sdk-reference".to_string(),
            name: "SDK Reference".to_string(),
            description: Some(
                "Concise Script Kit SDK function reference, script metadata format, and directory conventions for harness script creation."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: FAILED_SCRIPTS_RESOURCE_URI.to_string(),
            name: "Failed Scripts".to_string(),
            description: Some(
                "Scripts that were excluded from the kept catalog at startup because they fail validation — currently duplicate `shortcut` / `alias` / `keyword` / `trigger` bindings. Each entry names the offending path, the fatal issues, and related colliding scripts so authors can repair the metadata. Backed by `crate::scripts::read_scripts_report()`."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: SCRIPT_TEMPLATES_RESOURCE_URI.to_string(),
            name: "Script Templates".to_string(),
            description: Some(
                "Curated starter-script templates for the launcher's New Script from Template catalog. Same Rust-owned data the in-launcher catalog renders, so templates cannot drift between the UI and any MCP harness. v1 templates omit binding fields (`alias`, `shortcut`, `keyword`, `trigger`) so newly-created scripts cannot be immediately hidden by validation."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://clipboard-history".to_string(),
            name: "Clipboard History".to_string(),
            description: Some(
                "Most recent clipboard entries in newest-first order with content type, preview, OCR text, timestamps, and image dimensions. Supports ?limit=N (default 10) and ?diagnostics=1."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://focused-item".to_string(),
            name: "Focused Item".to_string(),
            description: Some(
                "Precise focused or selected item metadata for the active surface. Includes source, kind, semantic ID, label, and surface-specific metadata. Supports ?diagnostics=1."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://git-status".to_string(),
            name: "Git Status".to_string(),
            description: Some(
                "Current git status output from the working directory."
                    .to_string(),
            ),
            mime_type: "text/plain".to_string(),
        },
        McpResource {
            uri: "kit://git-diff".to_string(),
            name: "Git Diff".to_string(),
            description: Some(
                "Current git diff output (staged and unstaged) from the working directory."
                    .to_string(),
            ),
            mime_type: "text/plain".to_string(),
        },
        McpResource {
            uri: "kit://processes".to_string(),
            name: "Processes".to_string(),
            description: Some(
                "Top running processes by CPU usage."
                    .to_string(),
            ),
            mime_type: "text/plain".to_string(),
        },
        McpResource {
            uri: "kit://system".to_string(),
            name: "System Info".to_string(),
            description: Some(
                "Basic system information: hostname, OS version, architecture, uptime, and shell."
                    .to_string(),
            ),
            mime_type: "text/plain".to_string(),
        },
        McpResource {
            uri: "kit://dictation".to_string(),
            name: "Dictation".to_string(),
            description: Some(
                "Most recent dictated text captured by Script Kit. Returns a stable JSON envelope and never fails when no provider is configured."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://dictation-history".to_string(),
            name: "Dictation History".to_string(),
            description: Some(
                "Saved dictation history. Supports ?id=<entry-id> for a single transcript and ?limit=N (default 10) for newest-first JSON summaries."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://calendar".to_string(),
            name: "Calendar".to_string(),
            description: Some(
                "Upcoming calendar events in a prompt-safe JSON envelope."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://notifications".to_string(),
            name: "Notifications".to_string(),
            description: Some(
                "Recent notifications in newest-first order, capped and summarized for prompt use."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: STDIN_COMMANDS_REFERENCE_URI.to_string(),
            name: "Stdin JSONL Commands".to_string(),
            description: Some(
                "Canonical list of stdin JSONL `type` verbs accepted by the ExternalCommand parser. Payload is audited against `stdin_commands::all_external_command_verbs()` so documentation and runtime cannot drift."
                    .to_string(),
            ),
            mime_type: "text/markdown".to_string(),
        },
        McpResource {
            uri: TRIGGER_BUILTINS_REFERENCE_URI.to_string(),
            name: "Trigger Built-ins".to_string(),
            description: Some(
                "Canonical `builtin/...` command IDs accepted by `triggerBuiltin`. Payload is audited against `trigger_registry::all_trigger_builtin_command_ids()` to guarantee the list never goes stale."
                    .to_string(),
            ),
            mime_type: "text/markdown".to_string(),
        },
        McpResource {
            uri: PROTOCOL_STATS_DIAGNOSTICS_URI.to_string(),
            name: "Protocol Stats".to_string(),
            description: Some(
                "Rust↔Bun protocol-boundary counters plus a machine-readable `health.ok` / `health.flags` summary. Exposes `snapshot` (per-counter totals), `health` (threshold-crossed flags), and `thresholds` so MCP consumers can render a boundary health chip without hardcoding limits."
                    .to_string(),
            ),
            mime_type: "application/json".to_string(),
        },
    ];
    resources.extend(transaction_resources::transaction_resource_definitions());
    resources
}

pub const PROTOCOL_STATS_DIAGNOSTICS_URI: &str = "kit://diagnostics/protocol-stats";
/// Read a specific resource by URI
///
/// # Arguments
/// * `uri` - The resource URI to read
/// * `scripts` - Available scripts for scripts:// resource
/// * `scriptlets` - Available scriptlets for scriptlets:// resource
/// * `app_state` - Current app state for kit://state resource
///
/// # Returns
/// * `Ok(ResourceContent)` - The resource content
/// * `Err(String)` - Error message if resource not found
pub fn read_resource(
    uri: &str,
    scripts: &[Arc<Script>],
    scriptlets: &[Arc<Scriptlet>],
    app_state: Option<&AppStateResource>,
) -> Result<ResourceContent, String> {
    match uri {
        "kit://state" => read_state_resource(app_state),
        "scripts://" => read_scripts_resource(scripts),
        "scriptlets://" => read_scriptlets_resource(scriptlets),
        "kit://scripts" => read_kit_scripts_resource(scripts),
        "kit://scriptlets" => read_kit_scriptlets_resource(scriptlets),
        "kit://sdk-reference" => read_sdk_reference_resource(),
        FAILED_SCRIPTS_RESOURCE_URI => read_kit_failed_scripts_resource(),
        SCRIPT_TEMPLATES_RESOURCE_URI => read_kit_script_templates_resource(),
        _ if uri == "kit://context"
            || uri.starts_with("kit://context?")
            || uri == "kit://context/schema"
            || uri.starts_with("kit://context/schema?") =>
        {
            read_context_resource(uri)
        }
        _ if uri == "kit://clipboard-history" || uri.starts_with("kit://clipboard-history?") => {
            read_clipboard_history_resource(uri)
        }
        _ if uri == "kit://focused-item" || uri.starts_with("kit://focused-item?") => {
            read_focused_item_resource(uri)
        }
        _ if uri == "kit://dictation" || uri.starts_with("kit://dictation?") => {
            read_dictation_resource(uri)
        }
        _ if uri == "kit://dictation-history" || uri.starts_with("kit://dictation-history?") => {
            read_dictation_history_resource(uri)
        }
        _ if uri == "kit://calendar" || uri.starts_with("kit://calendar?") => {
            read_calendar_resource(uri)
        }
        _ if uri == "kit://notifications" || uri.starts_with("kit://notifications?") => {
            read_notifications_resource(uri)
        }
        "kit://git-status" => read_git_status_resource(),
        _ if is_notes_resource_uri(uri) => read_notes_resource(uri),
        _ if crate::brain::resources::is_brain_resource_uri(uri) => {
            let (mime_type, text) = crate::brain::resources::read_brain_resource(uri)?;
            Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type,
                text,
            })
        }
        _ if is_audit_resource_uri(uri) => read_audit_resource(uri),
        crate::computer_use::COMPUTER_USE_READINESS_RESOURCE_URI => {
            read_computer_use_readiness_resource()
        }
        _ if uri == "kit://git-diff" || uri.starts_with("kit://git-diff?") => {
            read_git_diff_resource(uri)
        }
        "kit://processes" => read_processes_resource(),
        "kit://system" => read_system_info_resource(),
        STDIN_COMMANDS_REFERENCE_URI => read_stdin_commands_resource(),
        TRIGGER_BUILTINS_REFERENCE_URI => read_trigger_builtins_resource(),
        PROTOCOL_STATS_DIAGNOSTICS_URI => read_protocol_stats_resource(),
        _ if transaction_resources::is_transaction_resource_uri(uri) => {
            transaction_resources::read_transaction_resource(uri)
        }
        _ => Err(format!("Resource not found: {}", uri)),
    }
}

pub(crate) fn is_context_resource_uri(uri: &str) -> bool {
    uri == "kit://context"
        || uri.starts_with("kit://context?")
        || uri == "kit://context/schema"
        || uri.starts_with("kit://context/schema?")
}

pub(crate) fn is_notes_resource_uri(uri: &str) -> bool {
    uri == NOTES_RESOURCE_URI || uri.starts_with("kit://notes?") || uri.starts_with("kit://notes/")
}

pub(crate) fn is_audit_resource_uri(uri: &str) -> bool {
    uri == AUDIT_RESOURCE_URI || uri.starts_with("kit://audit?")
}

fn read_notes_resource(uri: &str) -> Result<ResourceContent, String> {
    crate::notes::init_notes_db()
        .map_err(|error| format!("Failed to initialize notes database: {error}"))?;

    if uri == NOTES_RESOURCE_URI || uri.starts_with("kit://notes?") {
        return read_notes_list_resource(uri);
    }

    read_single_note_resource(uri)
}

fn read_notes_list_resource(uri: &str) -> Result<ResourceContent, String> {
    let include_deleted = query_bool(uri, "includeDeleted");
    let list_query = notes_list_search_query(uri);
    let mut notes = if let Some(query) = &list_query {
        crate::notes::search_notes(query)
            .map_err(|error| format!("Failed to search notes: {error}"))?
    } else if include_deleted {
        let mut active = crate::notes::get_all_notes()
            .map_err(|error| format!("Failed to read active notes: {error}"))?;
        let mut deleted = crate::notes::get_deleted_notes()
            .map_err(|error| format!("Failed to read deleted notes: {error}"))?;
        active.append(&mut deleted);
        active
    } else {
        crate::notes::get_all_notes().map_err(|error| format!("Failed to read notes: {error}"))?
    };

    let original_len = notes.len();
    // full=true swaps the 240-char preview for the full note body, bounded
    // tighter so instruction-note loads stay a sane context size.
    let full_content = query_bool(uri, "full");
    let default_limit = if full_content { 20 } else { 100 };
    let max_limit = if full_content { 50 } else { 500 };
    let limit = parse_u64_query_param(uri, "limit")
        .unwrap_or(default_limit)
        .clamp(1, max_limit) as usize;
    notes.truncate(limit);

    let summaries: Vec<Value> = notes
        .iter()
        .map(|note| {
            if full_content {
                note_full_json(note)
            } else {
                note_summary_json(note)
            }
        })
        .collect();
    let json = serde_json::json!({
        "schemaVersion": NOTES_RESOURCE_SCHEMA_VERSION,
        "uri": NOTES_RESOURCE_URI,
        "query": list_query,
        "count": summaries.len(),
        "truncated": original_len > summaries.len(),
        "notes": summaries,
    });

    Ok(ResourceContent {
        uri: NOTES_RESOURCE_URI.to_string(),
        mime_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&json)
            .map_err(|error| format!("Failed to serialize notes resource: {error}"))?,
    })
}

fn read_single_note_resource(uri: &str) -> Result<ResourceContent, String> {
    let raw_id = uri
        .strip_prefix("kit://notes/")
        .and_then(|rest| rest.split('?').next())
        .filter(|id| !id.trim().is_empty())
        .ok_or_else(|| format!("Invalid notes resource URI: {uri}"))?;
    let note_id = crate::notes::NoteId::parse(raw_id)
        .ok_or_else(|| format!("Invalid note id in URI: {raw_id}"))?;
    let note = crate::notes::get_note(note_id)
        .map_err(|error| format!("Failed to read note {note_id}: {error}"))?
        .ok_or_else(|| format!("Note not found: {note_id}"))?;

    let resource_uri = format!("kit://notes/{note_id}");
    let json = serde_json::json!({
        "schemaVersion": NOTES_RESOURCE_SCHEMA_VERSION,
        "uri": resource_uri,
        "note": note,
        "metadata": note_metadata_json(note_id),
    });

    Ok(ResourceContent {
        uri: resource_uri,
        mime_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&json)
            .map_err(|error| format!("Failed to serialize note resource: {error}"))?,
    })
}

fn note_summary_json(note: &crate::notes::Note) -> Value {
    let preview: String = note.content.chars().take(240).collect();
    let metadata = note_metadata_json(note.id);
    serde_json::json!({
        "id": note.id.as_str(),
        "uri": format!("kit://notes/{}", note.id),
        "title": note.title,
        "preview": preview,
        "charCount": note.content.chars().count(),
        "createdAt": note.created_at.to_rfc3339(),
        "updatedAt": note.updated_at.to_rfc3339(),
        "deletedAt": note.deleted_at.map(|dt| dt.to_rfc3339()),
        "isPinned": note.is_pinned,
        "sortOrder": note.sort_order,
        "metadata": metadata,
    })
}

/// Per-note body cap when `full=true` is requested on the notes list resource.
const NOTE_FULL_CONTENT_MAX_CHARS: usize = 20_000;

fn note_full_json(note: &crate::notes::Note) -> Value {
    let mut json = note_summary_json(note);
    let content: String = note
        .content
        .chars()
        .take(NOTE_FULL_CONTENT_MAX_CHARS)
        .collect();
    if let Some(object) = json.as_object_mut() {
        object.insert(
            "contentTruncated".to_string(),
            Value::Bool(note.content.chars().count() > NOTE_FULL_CONTENT_MAX_CHARS),
        );
        object.insert("content".to_string(), Value::String(content));
        object.remove("preview");
    }
    json
}

fn note_metadata_json(note_id: crate::notes::NoteId) -> Value {
    let tags = crate::notes::get_note_tags(note_id).unwrap_or_default();
    let aliases = crate::notes::get_note_aliases(note_id).unwrap_or_default();
    let tag_count = tags.len();
    let alias_count = aliases.len();
    let outbound_link_count = crate::notes::get_note_outbound_link_count(note_id).unwrap_or(0);
    let backlink_count = crate::notes::get_note_backlink_count(note_id).unwrap_or(0);
    serde_json::json!({
        "tags": tags,
        "aliases": aliases,
        "tagCount": tag_count,
        "aliasCount": alias_count,
        "outboundLinkCount": outbound_link_count,
        "backlinkCount": backlink_count,
    })
}

fn notes_list_search_query(uri: &str) -> Option<String> {
    if let Some(tag) = query_string_param(uri, "tag").filter(|value| !value.trim().is_empty()) {
        return Some(format!("tag:{tag}"));
    }
    if let Some(alias) = query_string_param(uri, "alias").filter(|value| !value.trim().is_empty()) {
        return Some(format!("alias:{alias}"));
    }
    if let Some(link) = query_string_param(uri, "link").filter(|value| !value.trim().is_empty()) {
        return Some(format!("link:{link}"));
    }
    query_string_param(uri, "q").filter(|value| !value.trim().is_empty())
}

pub(crate) fn parse_u64_query_param(uri: &str, key: &str) -> Option<u64> {
    let query = uri.split_once('?')?.1;
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find_map(|(k, v)| {
            if k == key {
                v.parse::<u64>().ok()
            } else {
                None
            }
        })
}

fn query_bool(uri: &str, key: &str) -> bool {
    let Some(query) = uri.split_once('?').map(|(_, query)| query) else {
        return false;
    };
    query.split('&').any(|pair| {
        let Some((k, v)) = pair.split_once('=') else {
            return pair == key;
        };
        k == key && matches!(v, "1" | "true" | "TRUE" | "yes")
    })
}

fn query_string_param(uri: &str, key: &str) -> Option<String> {
    let query = uri.split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (percent_decode_query_component(k) == key).then(|| percent_decode_query_component(v))
    })
}

fn percent_decode_query_component(input: &str) -> String {
    let mut bytes = Vec::with_capacity(input.len());
    let raw = input.as_bytes();
    let mut index = 0;
    while index < raw.len() {
        match raw[index] {
            b'+' => {
                bytes.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < raw.len() => {
                let hi = hex_value(raw[index + 1]);
                let lo = hex_value(raw[index + 2]);
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    bytes.push((hi << 4) | lo);
                    index += 3;
                } else {
                    bytes.push(raw[index]);
                    index += 1;
                }
            }
            byte => {
                bytes.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn read_audit_resource(uri: &str) -> Result<ResourceContent, String> {
    let limit = parse_u64_query_param(uri, "limit")
        .map(|value| value as usize)
        .unwrap_or(AUDIT_DEFAULT_LIMIT)
        .clamp(1, AUDIT_HARD_LIMIT);
    let trace_id_filter = query_string_param(uri, "traceId");

    let audit_path = dirs::home_dir()
        .ok_or_else(|| "Failed to resolve home directory for MCP audit log".to_string())?
        .join(".scriptkit")
        .join("mcp-audit.jsonl");

    let text = match std::fs::read_to_string(&audit_path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(format!(
                "Failed to read MCP audit log {}: {error}",
                audit_path.display()
            ));
        }
    };

    let mut matched = Vec::new();
    for line in text.lines().rev() {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if let Some(trace_id) = &trace_id_filter {
            if event.get("traceId").and_then(|value| value.as_str()) != Some(trace_id.as_str()) {
                continue;
            }
        }
        matched.push(event);
        if matched.len() == limit {
            break;
        }
    }
    matched.reverse();

    let json = serde_json::json!({
        "schemaVersion": AUDIT_RESOURCE_SCHEMA_VERSION,
        "uri": uri,
        "count": matched.len(),
        "truncated": text.lines().count() > matched.len(),
        "events": matched,
    });

    Ok(ResourceContent {
        uri: uri.to_string(),
        mime_type: "application/json".to_string(),
        text: serde_json::to_string_pretty(&json)
            .map_err(|error| format!("Failed to serialize MCP audit resource: {error}"))?,
    })
}
/// Read kit://state resource
fn read_state_resource(app_state: Option<&AppStateResource>) -> Result<ResourceContent, String> {
    let state = app_state.cloned().unwrap_or_default();
    let json = serde_json::to_string_pretty(&state)
        .map_err(|e| format!("Failed to serialize app state: {}", e))?;

    Ok(ResourceContent {
        uri: "kit://state".to_string(),
        mime_type: "application/json".to_string(),
        text: json,
    })
}

fn read_computer_use_readiness_resource() -> Result<ResourceContent, String> {
    let receipt = crate::computer_use::current_computer_use_readiness_receipt();
    let text = serde_json::to_string_pretty(&receipt)
        .map_err(|error| format!("Failed to serialize computer-use readiness: {error}"))?;

    Ok(ResourceContent {
        uri: crate::computer_use::COMPUTER_USE_READINESS_RESOURCE_URI.to_string(),
        mime_type: "application/json".to_string(),
        text,
    })
}

/// Read scripts:// resource
fn read_scripts_resource(scripts: &[Arc<Script>]) -> Result<ResourceContent, String> {
    let entries: Vec<ScriptResourceEntry> = scripts
        .iter()
        .map(|s| ScriptResourceEntry::from(s.as_ref()))
        .collect();
    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| format!("Failed to serialize scripts: {}", e))?;

    Ok(ResourceContent {
        uri: "scripts://".to_string(),
        mime_type: "application/json".to_string(),
        text: json,
    })
}
/// Read scriptlets:// resource
fn read_scriptlets_resource(scriptlets: &[Arc<Scriptlet>]) -> Result<ResourceContent, String> {
    let entries: Vec<ScriptletResourceEntry> = scriptlets
        .iter()
        .map(|s| ScriptletResourceEntry::from(s.as_ref()))
        .collect();
    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| format!("Failed to serialize scriptlets: {}", e))?;

    Ok(ResourceContent {
        uri: "scriptlets://".to_string(),
        mime_type: "application/json".to_string(),
        text: json,
    })
}

// ---------------------------------------------------------------
// Schema-versioned script/scriptlet/sdk-reference resources
// ---------------------------------------------------------------

/// URI for the stdin JSONL verb reference resource.
///
/// Declared payload entries live inside
/// `<!-- drift-audit:stdin-verbs:start -->` / `<!-- drift-audit:stdin-verbs:end -->`
/// and are audited against
/// [`crate::stdin_commands::all_external_command_verbs`] by
/// `tests/mcp_resource_drift.rs`.
pub const STDIN_COMMANDS_REFERENCE_URI: &str = "kit://stdin-commands";

/// URI for the canonical triggerBuiltin command-id reference resource.
///
/// Declared payload entries live inside
/// `<!-- drift-audit:trigger-builtin-ids:start -->` /
/// `<!-- drift-audit:trigger-builtin-ids:end -->` and are audited against
/// [`crate::builtins::trigger_registry::all_trigger_builtin_command_ids`]
/// by `tests/mcp_resource_drift.rs`.
pub const TRIGGER_BUILTINS_REFERENCE_URI: &str = "kit://trigger-builtins";

/// Schema version for the `kit://clipboard-history` resource envelope.
pub const CLIPBOARD_HISTORY_RESOURCE_SCHEMA_VERSION: u32 = 1;

/// Schema version for the `kit://focused-item` resource envelope.
pub const FOCUSED_ITEM_RESOURCE_SCHEMA_VERSION: u32 = 1;

/// Schema version for the `kit://scripts` resource envelope.
pub const SCRIPTS_RESOURCE_SCHEMA_VERSION: u32 = 1;

/// Schema version for the `kit://scriptlets` resource envelope.
pub const SCRIPTLETS_RESOURCE_SCHEMA_VERSION: u32 = 1;

/// Schema version for the `kit://sdk-reference` resource.
/// Bumped to 5: adds per-function GPUI support status (`support`,
/// `unsupportedNote`) so agents can refuse to generate calls into
/// SDK APIs the current GPUI shell does not implement.
pub const SDK_REFERENCE_SCHEMA_VERSION: u32 = 5;

/// URI for the `kit://failed-scripts` resource.
///
/// Surfaces the `ValidationReport.failed_scripts` list so authors can see
/// which scripts were excluded from the kept catalog (today: duplicate
/// `shortcut` / `alias` / `keyword` / `trigger` bindings) instead of
/// silently disappearing from the launcher. Backed by
/// [`crate::scripts::read_scripts_report`].
pub const FAILED_SCRIPTS_RESOURCE_URI: &str = "kit://failed-scripts";

/// Schema version for the `kit://failed-scripts` resource envelope.
pub const FAILED_SCRIPTS_RESOURCE_SCHEMA_VERSION: u32 = 1;

/// URI for the `kit://script-templates` resource.
///
/// Surfaces curated starter templates for newly created scripts so the
/// launcher's template catalog and any MCP harness share one Rust-owned
/// source of truth. v1 templates intentionally omit collision-bearing
/// binding fields (`alias`, `shortcut`, `keyword`, `trigger`) so a
/// newly-created script cannot be immediately hidden by
/// [`crate::scripts::validation::validate_script_catalog`].
pub const SCRIPT_TEMPLATES_RESOURCE_URI: &str = "kit://script-templates";

/// Schema version for the `kit://script-templates` resource envelope.
pub const SCRIPT_TEMPLATES_RESOURCE_SCHEMA_VERSION: u32 = 1;

/// Schema-versioned envelope for script metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptsResourceDocument {
    pub schema_version: u32,
    pub count: usize,
    pub scripts: Vec<ScriptResourceEntry>,
}

/// Schema-versioned envelope for scriptlet metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptletsResourceDocument {
    pub schema_version: u32,
    pub count: usize,
    pub scriptlets: Vec<ScriptletResourceEntry>,
}

/// A single failed-script entry for the `kit://failed-scripts` resource.
///
/// Mirrors [`crate::scripts::FailedScript`] but uses `Vec` for the fatal-issue
/// list so the resource envelope round-trips cleanly through
/// `serde_json::from_str` without Arc-slice deserialization surprises.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FailedScriptEntry {
    pub path: std::path::PathBuf,
    pub name: String,
    pub fatal: Vec<ScriptValidationIssue>,
}

impl From<&FailedScript> for FailedScriptEntry {
    fn from(failed: &FailedScript) -> Self {
        Self {
            path: failed.path.clone(),
            name: failed.name.clone(),
            fatal: failed.fatal.iter().cloned().collect(),
        }
    }
}

/// Schema-versioned envelope for the `kit://failed-scripts` resource.
///
/// Carries both an envelope `schema_version` (this document format) and the
/// inner `validation_schema_version` from [`crate::scripts::VALIDATION_SCHEMA_VERSION`]
/// so consumers can detect changes at either layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedScriptsResourceDocument {
    pub schema_version: u32,
    pub validation_schema_version: u32,
    pub total_candidates: usize,
    pub valid_count: usize,
    pub fatal_count: usize,
    pub warning_count: usize,
    pub failed_scripts: Vec<FailedScriptEntry>,
    pub warnings: Vec<ScriptValidationIssue>,
}

/// GPUI support status for a single SDK function.
///
/// `Supported` is the default; absent JSON fields deserialize as
/// `Supported` so older clients that do not know about this enum
/// continue to round-trip. `Unsupported` entries are documented in
/// `scripts/kit-sdk.ts` but the GPUI app does not currently handle
/// their message (typically they `console.warn` and resolve to nothing
/// or throw). `Experimental` is reserved for partially-implemented
/// APIs so the next marking wave does not require another schema bump.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SdkSupport {
    #[default]
    Supported,
    Unsupported,
    Experimental,
}

/// A single SDK function reference entry.
///
/// `support` is always serialized so agents can rely on a stable
/// `"support": "supported" | "unsupported" | "experimental"` field
/// rather than inferring state from absence. `unsupported_note` is
/// skipped when `None` to keep the envelope lean for the common case.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SdkFunctionRef {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub category: String,
    #[serde(default)]
    pub support: SdkSupport,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unsupported_note: Option<String>,
}

impl SdkFunctionRef {
    fn supported(
        name: impl Into<String>,
        signature: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            signature: signature.into(),
            description: description.into(),
            category: category.into(),
            support: SdkSupport::Supported,
            unsupported_note: None,
        }
    }

    fn unsupported(
        name: impl Into<String>,
        signature: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            signature: signature.into(),
            description: description.into(),
            category: category.into(),
            support: SdkSupport::Unsupported,
            unsupported_note: Some(note.into()),
        }
    }

    fn experimental(
        name: impl Into<String>,
        signature: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            signature: signature.into(),
            description: description.into(),
            category: category.into(),
            support: SdkSupport::Experimental,
            unsupported_note: Some(note.into()),
        }
    }
}

/// SDK inventory flagged unsupported by `scripts/kit-sdk.ts`. Sourced
/// from explicit unsupported throws, `console.warn(...)` lines, and NOTE
/// comments in that file. Only entries that also appear in
/// [`build_sdk_function_refs`] are marked in the generated SDK
/// reference — adding missing entries is deliberately out of scope
/// for this pass so the reference does not silently grow. The list
/// is consumed by
/// [`tests::sdk_reference_marks_every_documented_unsupported_api`]
/// as a canonical test input; it is deliberately kept in the non-test
/// binary so future agents can `rg` for it.
#[allow(dead_code)]
const SDK_NOT_YET_IMPLEMENTED_IN_GPUI: &[&str] = &[
    "setStatus",
    "keyboard.type",
    "keyboard.tap",
    "mouse.move",
    "mouse.click",
    "setPanel",
    "setPreview",
    "setPrompt",
    "mini",
    "micro",
    "hotkey",
    "widget",
    "find",
    "menu",
];

/// Default explanation for "not yet implemented" SDK APIs. The
/// [`SdkFunctionRef::unsupported`] constructor takes a custom note so
/// a function can point the user at a working alternative — this
/// constant is exposed for tests that want to pin the generic wording.
#[allow(dead_code)]
const SDK_UNSUPPORTED_IN_GPUI_NOTE: &str = "Defined in scripts/kit-sdk.ts, but GPUI does not handle this behavior yet; the SDK fails explicitly instead of sending a misleading fire-and-forget message.";

/// Needles for scanning starter-template bodies for references to
/// unsupported SDK APIs. A template that contains any of these
/// substrings is rejected by
/// [`tests::script_templates_do_not_reference_unsupported_sdk_apis`].
#[cfg(test)]
fn unsupported_sdk_reference_scan_needles() -> Vec<String> {
    build_sdk_function_refs()
        .into_iter()
        .filter(|entry| entry.support == SdkSupport::Unsupported)
        .flat_map(|entry| {
            if entry.name.contains('.') {
                vec![format!("{}(", entry.name), format!("{}.", entry.name)]
            } else {
                vec![format!("{}(", entry.name)]
            }
        })
        .collect()
}

/// Mandatory Bun verification contract for final user-authored scripts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessVerificationContract {
    /// Whether verification is mandatory before the agent can report success.
    pub required: bool,
    /// Canonical skill file that defines the verification loop.
    pub skill_path: String,
    /// Exact Bun syntax-check / transpile command for the final script.
    pub build_command: String,
    /// Exact Bun execution command for the final script.
    pub run_command: String,
    /// Observable result the agent must confirm after execution.
    pub success_criteria: String,
    /// What the agent must do if either Bun command fails.
    pub failure_policy: String,
}

/// Describes how a harness can create and verify scripts non-interactively.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessWorkflow {
    /// Dedicated directory for test/temp scripts that won't pollute the user's
    /// main `~/.scriptkit/plugins/main/scripts/` collection.
    pub test_script_directory: String,
    /// Dedicated directory for test scriptlet extension files.
    pub test_scriptlet_directory: String,
    /// Shell command to execute a script via the app stdin bridge.
    /// The harness replaces `{path}` with the absolute script path.
    pub run_command: String,
    /// JSONL message the app sends to its stdin to trigger a script run.
    /// Harnesses that communicate over the stdin bridge use this shape.
    pub stdin_run_message: String,
    /// Shape of a successful execution result on stdout (JSONL).
    pub success_output_shape: String,
    /// Shape of an error execution result on stdout (JSONL).
    pub error_output_shape: String,
    /// Mandatory Bun verification contract for the final user-authored script.
    pub verification: HarnessVerificationContract,
    /// Example minimal test script content (TypeScript).
    pub example_test_script: String,
    /// Example scriptlet (Markdown) content.
    pub example_scriptlet: String,
}

/// Schema-versioned SDK reference document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SdkReferenceDocument {
    pub schema_version: u32,
    pub sdk_package: String,
    pub script_directory: String,
    pub scriptlet_pattern: String,
    pub metadata_format: String,
    pub functions: Vec<SdkFunctionRef>,
    /// Non-interactive workflow for harness-driven script creation and execution.
    pub harness_workflow: HarnessWorkflow,
}

/// Optional metadata defaults written into a newly-created script.
///
/// v1 templates intentionally omit collision-bearing binding fields
/// (`alias`, `shortcut`, `keyword`, `trigger`) — those are what
/// [`crate::scripts::validation::detect_binding_collisions`] uses to
/// fatally exclude duplicates. A starter script should never land on
/// disk in a hidden state.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTemplateMetadataDefaults {
    /// `description:` value in the `export const metadata = { … }` block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A single starter-script template.
///
/// The `body_template` body is a fully-formed TypeScript file with a
/// `{{NAME}}` placeholder substituted by [`render_script_template_file`]
/// at write time — so the `metadata.name` in the on-disk file matches
/// the friendly name the user typed into the naming prompt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTemplateRef {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub filename_hint: String,
    pub body_template: String,
    #[serde(default)]
    pub metadata_defaults: ScriptTemplateMetadataDefaults,
}

/// Schema-versioned envelope for `kit://script-templates`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptTemplatesResourceDocument {
    pub schema_version: u32,
    pub count: usize,
    pub templates: Vec<ScriptTemplateRef>,
}

// ---------------------------------------------------------------
// Clipboard history resource types
// ---------------------------------------------------------------

/// A single clipboard history entry in the MCP resource.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardHistoryEntry {
    pub id: String,
    pub content_type: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocr_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_height: Option<u32>,
    pub pinned: bool,
}

/// Schema-versioned envelope for clipboard history.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardHistoryDocument {
    pub schema_version: u32,
    pub count: usize,
    pub entries: Vec<ClipboardHistoryEntry>,
}

/// Diagnostics wrapper for clipboard history.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ClipboardHistoryDiagnosticsDocument {
    kind: &'static str,
    uri: String,
    document: ClipboardHistoryDocument,
    meta: ClipboardHistoryDiagnosticsMeta,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ClipboardHistoryDiagnosticsMeta {
    duration_ms: u128,
    entry_count: usize,
    source: &'static str,
}

// ---------------------------------------------------------------
// Focused item resource types
// ---------------------------------------------------------------

/// The focused/selected item from the active surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FocusedItemInfo {
    pub source: String,
    pub kind: String,
    pub semantic_id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Schema-versioned envelope for focused item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FocusedItemDocument {
    pub schema_version: u32,
    pub has_focused_item: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused_item: Option<FocusedItemInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// Diagnostics wrapper for focused item.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct FocusedItemDiagnosticsDocument {
    kind: &'static str,
    uri: String,
    document: FocusedItemDocument,
    meta: FocusedItemDiagnosticsMeta,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct FocusedItemDiagnosticsMeta {
    duration_ms: u128,
    has_focused_item: bool,
    warning_count: usize,
    source: String,
}

fn build_sdk_function_refs() -> Vec<SdkFunctionRef> {
    vec![
        SdkFunctionRef::supported(
            "arg",
            "await arg(prompt: string, choices?: Choice[]): Promise<string>",
            "Prompt the user with an input field, optionally with a list of choices.",
            "prompts",
        ),
        SdkFunctionRef::supported(
            "div",
            "await div(html: string): Promise<void>",
            "Display HTML content in a panel.",
            "prompts",
        ),
        SdkFunctionRef::supported(
            "editor",
            "await editor(options?: EditorOptions): Promise<string>",
            "Open a full-screen code editor and return the content.",
            "prompts",
        ),
        SdkFunctionRef::supported(
            "term",
            "await term(command?: string): Promise<string>",
            "Open an interactive terminal, optionally running a command.",
            "prompts",
        ),
        SdkFunctionRef::supported(
            "drop",
            "await drop(options?: DropOptions): Promise<DroppedItem[]>",
            "Accept drag-and-drop files from the user.",
            "prompts",
        ),
        SdkFunctionRef::supported(
            "template",
            "await template(template: string): Promise<string>",
            "Fill in a template string with user-provided values.",
            "prompts",
        ),
        SdkFunctionRef::supported(
            "exec",
            "await exec(command: string, args?: string[]): Promise<ExecResult>",
            "Execute a shell command and return its output.",
            "system",
        ),
        SdkFunctionRef::supported(
            "clipboard",
            "await clipboard.readText(): Promise<string>",
            "Read the current clipboard text content.",
            "clipboard",
        ),
        SdkFunctionRef::supported(
            "copy",
            "await copy(text: string): Promise<void>",
            "Copy text to the clipboard.",
            "clipboard",
        ),
        SdkFunctionRef::supported(
            "paste",
            "await paste(text: string): Promise<void>",
            "Paste text to the focused application.",
            "clipboard",
        ),
        SdkFunctionRef::supported(
            "notify",
            "await notify(message: string | { title?: string; body?: string }): Promise<SystemFeedbackResult>",
            "Request an OS-level system notification (macOS Notification Center). Returns a dispatch receipt; delivery remains OS dependent. Distinct from hud(message), which is an in-launcher overlay.",
            "feedback",
        ),
        SdkFunctionRef::experimental(
            "beep",
            "await beep(): Promise<SystemFeedbackResult>",
            "Request a macOS system beep through afplay.",
            "feedback",
            "beep() returns a dispatch receipt when the feedback process is spawned; audible delivery is not verified and non-macOS platforms return unsupported.",
        ),
        SdkFunctionRef::experimental(
            "say",
            "await say(text: string, voice?: string): Promise<SystemFeedbackResult>",
            "Request macOS text-to-speech through the say command.",
            "feedback",
            "say() returns a dispatch receipt when the feedback process is spawned; speech delivery is not verified and non-macOS platforms return unsupported.",
        ),
        SdkFunctionRef::supported(
            "setSelectedText",
            "await setSelectedText(text: string): Promise<void>",
            "Replace the selected text in the focused application.",
            "system",
        ),
        SdkFunctionRef::supported(
            "getSelectedText",
            "await getSelectedText(): Promise<string>",
            "Read the selected text from the focused application.",
            "system",
        ),
        SdkFunctionRef::supported(
            "readFile",
            "await readFile(path: string): Promise<string>",
            "Read a file's contents as UTF-8 text.",
            "filesystem",
        ),
        SdkFunctionRef::supported(
            "writeFile",
            "await writeFile(path: string, content: string): Promise<void>",
            "Write UTF-8 text content to a file.",
            "filesystem",
        ),
        SdkFunctionRef::supported(
            "home",
            "home(...paths: string[]): string",
            "Resolve a path relative to the user's home directory.",
            "filesystem",
        ),
        SdkFunctionRef::unsupported(
            "find",
            "find(placeholder, options?)",
            "Legacy interactive find prompt. GPUI does not currently implement a Rust find prompt route, renderer, submit contract, or onlyin prompt semantics.",
            "filesystem",
            "Use fileSearch(query, { onlyin }) for non-interactive Spotlight/mdfind results, or path({ startPath }) / arg(...) for supported prompt-driven selection.",
        ),
        SdkFunctionRef::supported(
            "getState",
            "await getState(): Promise<PromptState>",
            "Read the current Script Kit prompt state without mutating the UI.",
            "automation",
        ),
        SdkFunctionRef::supported(
            "getElements",
            "await getElements(limit?: number): Promise<ElementsSnapshot>",
            "Return visible UI elements with semantic IDs, focus, selection, truncation, and warnings.",
            "automation",
        ),
        SdkFunctionRef::supported(
            "waitFor",
            "await waitFor(condition: WaitCondition, options?: WaitForOptions): Promise<WaitForResult>",
            "Poll until a UI condition is satisfied or the timeout expires. Returns { success, elapsed, error?, trace? }. On failure, error contains a stable code (wait_condition_timeout | element_not_found | unsupported_prompt | action_failed), a human message, and an optional suggestion. Pass trace: 'onFailure' in options to get poll-by-poll diagnostics on timeout.",
            "automation",
        ),
        SdkFunctionRef::supported(
            "batch",
            "await batch(commands: BatchCommand[], options?: BatchOptions): Promise<BatchResult>",
            "Execute a deterministic sequence of UI commands. Returns { success, results, failedAt?, totalElapsed, trace? }. Each result entry includes index, success, command, elapsed, value?, and a structured error with stable code on failure. Pass trace: 'onFailure' at the top-level message (not inside options) for per-command diagnostics. Error codes: wait_condition_timeout, element_not_found, selection_not_found, unsupported_command, unsupported_prompt, action_failed.",
            "automation",
        ),
        SdkFunctionRef::supported(
            "computer.listNativeWindows",
            "await computer.listNativeWindows(options?: ComputerUseListNativeWindowsOptions): Promise<ComputerUseListNativeWindowsResult>",
            "List native macOS windows grouped by running app through Script Kit's own local MCP server. Observation-only: does not focus, activate, move, resize, capture screenshots, or send input.",
            "computer-use",
        ),
        SdkFunctionRef::supported(
            "computer.captureNativeWindow",
            "await computer.captureNativeWindow(options: ComputerUseCaptureNativeWindowOptions): Promise<ComputerUseCaptureNativeWindowResult>",
            "Capture one exact native macOS window after PID/nativeWindowId ownership and capture-candidate validation. Returns the structured computer/capture_native_window receipt, optionally including pngBase64 when includeImage is true.",
            "computer-use",
        ),
        SdkFunctionRef::unsupported(
            "setStatus",
            "await setStatus(options: { status: 'busy' | 'idle' | 'error'; message: string }): Promise<SystemFeedbackResult>",
            "Set application status text.",
            "feedback",
            "setStatus(...) currently has no visible GPUI status surface or receipt. The SDK returns ERR_UNSUPPORTED_SDK_FEATURE before sending; use hud(message) for visible feedback, or render progress in a prompt.",
        ),
        SdkFunctionRef::unsupported(
            "menu",
            "await menu(icon: string, scripts?: string[]): Promise<SystemFeedbackResult>",
            "Show a menu-bar icon with quick-access scripts.",
            "system",
            "menu(...) currently has no GPUI tray/menu mutation handler. The SDK returns ERR_UNSUPPORTED_SDK_FEATURE before sending; use the built-in tray icon (System Actions) or prompt-scoped setActions(...) today.",
        ),
    ]
}

/// Cheap UI-facing slice of the SDK reference document.
///
/// Callers reuse the same Rust data that powers `kit://sdk-reference`
/// so the in-product SDK Reference view never drifts from the MCP
/// resource or hand-authors a second API list.
pub fn sdk_reference_entries_for_ui() -> std::sync::Arc<[SdkFunctionRef]> {
    std::sync::Arc::from(build_sdk_reference_document().functions)
}

/// Case-insensitive substring match across `name`, `signature`, `description`,
/// and `category`. Returns the indices (into `entries`) of matching rows, in
/// source order. An empty or whitespace-only filter returns every row.
pub fn filter_sdk_reference_entries(entries: &[SdkFunctionRef], filter: &str) -> Vec<usize> {
    let q = filter.trim().to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter_map(|(idx, entry)| {
            if q.is_empty()
                || entry.name.to_lowercase().contains(&q)
                || entry.signature.to_lowercase().contains(&q)
                || entry.description.to_lowercase().contains(&q)
                || entry.category.to_lowercase().contains(&q)
            {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}

/// A visible SDK Reference row projected from the shared MCP-backed catalog.
#[derive(Debug, Clone, Copy)]
pub struct SdkReferenceVisibleRow<'a> {
    pub display_index: usize,
    pub source_index: usize,
    pub entry: &'a SdkFunctionRef,
}

/// Visible SDK Reference rows in display order.
pub fn sdk_reference_visible_rows<'a>(
    entries: &'a [SdkFunctionRef],
    filter: &str,
) -> Vec<SdkReferenceVisibleRow<'a>> {
    filter_sdk_reference_entries(entries, filter)
        .into_iter()
        .enumerate()
        .filter_map(|(display_index, source_index)| {
            entries
                .get(source_index)
                .map(|entry| SdkReferenceVisibleRow {
                    display_index,
                    source_index,
                    entry,
                })
        })
        .collect()
}

pub fn sdk_reference_visible_row_names(entries: &[SdkFunctionRef], filter: &str) -> Vec<String> {
    sdk_reference_visible_rows(entries, filter)
        .into_iter()
        .map(|row| row.entry.name.clone())
        .collect()
}

pub fn sdk_reference_dataset_and_visible_counts(
    entries: &[SdkFunctionRef],
    filter: &str,
) -> (usize, usize) {
    (
        entries.len(),
        sdk_reference_visible_rows(entries, filter).len(),
    )
}

pub fn sdk_reference_selected_visible_entry<'a>(
    entries: &'a [SdkFunctionRef],
    filter: &str,
    selected_index: usize,
) -> Option<SdkReferenceVisibleRow<'a>> {
    sdk_reference_visible_rows(entries, filter)
        .get(selected_index)
        .copied()
}

pub fn sdk_reference_visible_target_rows<'a>(
    entries: &'a [SdkFunctionRef],
    filter: &str,
    limit: usize,
) -> Vec<SdkReferenceVisibleRow<'a>> {
    sdk_reference_visible_rows(entries, filter)
        .into_iter()
        .take(limit)
        .collect()
}

/// Markdown preview for a single SDK function — used by the in-product
/// SDK Reference view (preview pane + Cmd+C clipboard copy).
///
/// Unsupported entries are prepended with a blockquote warning so a
/// snippet pasted into an editor still carries the "this will no-op or
/// throw" signal even after it leaves the launcher.
pub fn format_sdk_reference_entry_markdown(entry: &SdkFunctionRef) -> String {
    let mut out = String::new();
    if entry.support == SdkSupport::Unsupported {
        out.push_str("> ⚠ Unsupported in GPUI — this function is defined in kit-sdk.ts but currently no-ops or throws.\n");
        if let Some(note) = entry.unsupported_note.as_deref() {
            out.push_str("> ");
            out.push_str(note);
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "# {name}\n\n`{signature}`\n\n_{category}_\n\n{description}\n",
        name = entry.name,
        signature = entry.signature,
        category = entry.category,
        description = entry.description,
    ));
    out
}

/// Curated v1 starter templates.
///
/// Ordering is load-bearing: `blank-starter` is row #1 so the fastest
/// "new script" path (Enter → Enter → name → editor) feels identical
/// to the pre-catalog experience.
///
/// **Invariant:** no template may emit `alias:`, `shortcut:`, `keyword:`,
/// or `trigger:` in its body. `detect_binding_collisions` would mark a
/// fresh duplicate as fatal and hide the script from the launcher —
/// defeating the whole "first useful automation" purpose of templates.
fn build_script_templates() -> Vec<ScriptTemplateRef> {
    vec![
        ScriptTemplateRef {
            id: "blank-starter".into(),
            title: "Blank Starter".into(),
            description: "An empty script shape with an arg prompt and div output — the fastest path from naming to a working script.".into(),
            category: "starter".into(),
            filename_hint: "my-script".into(),
            body_template: concat!(
                "import \"@scriptkit/sdk\";\n",
                "\n",
                "export const metadata = {\n",
                "  name: \"{{NAME}}\",\n",
                "  description: \"{{DESCRIPTION}}\",\n",
                "};\n",
                "\n",
                "const value = await arg(\"Enter a value\");\n",
                "\n",
                "await div(md(`## You typed\\n\\n${value}`));\n",
            ).into(),
            metadata_defaults: ScriptTemplateMetadataDefaults {
                description: Some("A blank starter script".into()),
            },
        },
        ScriptTemplateRef {
            id: "choice-list".into(),
            title: "Choice List".into(),
            description: "Prompt the user to pick one option from a fixed list, then show the selection.".into(),
            category: "prompts".into(),
            filename_hint: "pick-one".into(),
            body_template: concat!(
                "import \"@scriptkit/sdk\";\n",
                "\n",
                "export const metadata = {\n",
                "  name: \"{{NAME}}\",\n",
                "  description: \"{{DESCRIPTION}}\",\n",
                "};\n",
                "\n",
                "const choice = await arg(\"Pick one\", [\"A\", \"B\", \"C\"]);\n",
                "\n",
                "await div(md(`## Selected\\n\\n${choice}`));\n",
            ).into(),
            metadata_defaults: ScriptTemplateMetadataDefaults {
                description: Some("Prompt the user to pick one option from a list".into()),
            },
        },
    ]
}

/// Pure builder for the `kit://script-templates` resource envelope.
pub fn build_script_templates_document() -> ScriptTemplatesResourceDocument {
    let templates = build_script_templates();
    ScriptTemplatesResourceDocument {
        schema_version: SCRIPT_TEMPLATES_RESOURCE_SCHEMA_VERSION,
        count: templates.len(),
        templates,
    }
}

/// Cheap UI-facing slice of the template catalog. Same objects the MCP
/// resource returns, so the in-launcher catalog and any agent reading
/// `kit://script-templates` cannot drift.
pub fn script_template_entries_for_ui() -> std::sync::Arc<[ScriptTemplateRef]> {
    std::sync::Arc::from(build_script_templates())
}

/// Case-insensitive substring match across `title`, `description`, and
/// `category`. Returns the indices (into `entries`) of matching rows in
/// source order. An empty or whitespace-only filter returns every row.
pub fn filter_script_template_entries(entries: &[ScriptTemplateRef], filter: &str) -> Vec<usize> {
    let q = filter.trim().to_lowercase();
    entries
        .iter()
        .enumerate()
        .filter_map(|(idx, entry)| {
            if q.is_empty()
                || entry.title.to_lowercase().contains(&q)
                || entry.description.to_lowercase().contains(&q)
                || entry.category.to_lowercase().contains(&q)
            {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}

/// A visible starter-template row projected from the shared MCP-backed catalog.
#[derive(Debug, Clone, Copy)]
pub struct ScriptTemplateCatalogVisibleRow<'a> {
    pub display_index: usize,
    pub source_index: usize,
    pub template: &'a ScriptTemplateRef,
}

/// Visible starter-template rows in display order.
pub fn script_template_catalog_visible_rows<'a>(
    entries: &'a [ScriptTemplateRef],
    filter: &str,
) -> Vec<ScriptTemplateCatalogVisibleRow<'a>> {
    filter_script_template_entries(entries, filter)
        .into_iter()
        .enumerate()
        .filter_map(|(display_index, source_index)| {
            entries
                .get(source_index)
                .map(|template| ScriptTemplateCatalogVisibleRow {
                    display_index,
                    source_index,
                    template,
                })
        })
        .collect()
}

pub fn script_template_catalog_visible_row_names(
    entries: &[ScriptTemplateRef],
    filter: &str,
) -> Vec<String> {
    script_template_catalog_visible_rows(entries, filter)
        .into_iter()
        .map(|row| row.template.title.clone())
        .collect()
}

pub fn script_template_catalog_dataset_and_visible_counts(
    entries: &[ScriptTemplateRef],
    filter: &str,
) -> (usize, usize) {
    (
        entries.len(),
        script_template_catalog_visible_rows(entries, filter).len(),
    )
}

pub fn script_template_catalog_selected_visible_template<'a>(
    entries: &'a [ScriptTemplateRef],
    filter: &str,
    selected_index: usize,
) -> Option<ScriptTemplateCatalogVisibleRow<'a>> {
    script_template_catalog_visible_rows(entries, filter)
        .get(selected_index)
        .copied()
}

pub fn script_template_catalog_visible_target_rows<'a>(
    entries: &'a [ScriptTemplateRef],
    filter: &str,
    limit: usize,
) -> Vec<ScriptTemplateCatalogVisibleRow<'a>> {
    script_template_catalog_visible_rows(entries, filter)
        .into_iter()
        .take(limit)
        .collect()
}

/// Instantiate a template's `body_template` for on-disk write.
///
/// Substitutes `{{NAME}}` with `friendly_name` and `{{DESCRIPTION}}` with
/// the template's `metadata_defaults.description` (falling back to the
/// template title). The returned string is the exact content that
/// [`crate::app_impl::naming_dialog::ScriptListApp::handle_naming_dialog_completion`]
/// writes over the freshly-created script file between
/// [`crate::script_creation::create_new_script`] and
/// [`crate::script_creation::open_in_editor`].
pub fn render_script_template_file(template: &ScriptTemplateRef, friendly_name: &str) -> String {
    let description = template
        .metadata_defaults
        .description
        .as_deref()
        .unwrap_or(&template.title);
    template
        .body_template
        .replace("{{NAME}}", friendly_name)
        .replace("{{DESCRIPTION}}", description)
}

/// Markdown preview for a single template — used by the catalog view's
/// preview pane and Cmd+C clipboard copy.
pub fn format_script_template_markdown(template: &ScriptTemplateRef) -> String {
    format!(
        "# {title}\n\n_{category}_\n\n{description}\n\n```ts\n{body}```\n",
        title = template.title,
        category = template.category,
        description = template.description,
        body = template.body_template,
    )
}

/// Resolve a template by `id`. Used by
/// [`crate::app_impl::naming_dialog::ScriptListApp::handle_naming_dialog_completion`]
/// to turn the `template_id` carried through [`crate::prompts::NamingSubmitResult`]
/// back into the in-memory [`ScriptTemplateRef`] consumed by
/// [`render_script_template_file`].
pub fn find_script_template(id: &str) -> Option<ScriptTemplateRef> {
    build_script_templates_document()
        .templates
        .into_iter()
        .find(|template| template.id == id)
}

pub(crate) fn build_sdk_reference_document() -> SdkReferenceDocument {
    SdkReferenceDocument {
        schema_version: SDK_REFERENCE_SCHEMA_VERSION,
        sdk_package: "@scriptkit/sdk".into(),
        script_directory: "~/.scriptkit/plugins/main/scripts/ (default personal plugin; all plugins under plugins/*/scripts/ are discovered)".into(),
        scriptlet_pattern: "~/.scriptkit/plugins/*/scriptlets/*.md".into(),
        metadata_format:
            "export const metadata = { name: \"My Script\", description: \"What it does\" }".into(),
        functions: build_sdk_function_refs(),
        harness_workflow: build_harness_workflow(),
    }
}

/// Shell-quote a literal value for safe embedding in a command string.
fn shell_quote_literal(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "/._-:=@".contains(ch))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', r#"'"'"'"#))
    }
}

/// Resolve the absolute path to the running binary, falling back to bare name.
fn resolve_harness_run_binary() -> String {
    match std::env::current_exe() {
        Ok(path) => {
            let text = path.to_string_lossy().into_owned();
            shell_quote_literal(&text)
        }
        Err(_) => "script-kit-gpui".to_string(),
    }
}

/// Build the harness run command using the resolved absolute binary path.
fn build_harness_run_command() -> String {
    format!(
        "echo '{{\"type\":\"run\",\"path\":\"{{path}}\"}}' | {}",
        resolve_harness_run_binary()
    )
}

fn build_harness_workflow() -> HarnessWorkflow {
    HarnessWorkflow {
        test_script_directory: "~/.scriptkit/tmp/test-scripts/".into(),
        test_scriptlet_directory: "~/.scriptkit/tmp/test-scriptlets/".into(),
        run_command: build_harness_run_command(),
        stdin_run_message: r#"{"type":"run","path":"/absolute/path/to/script.ts"}"#.into(),
        success_output_shape: "No dedicated success envelope is emitted for stdin `run`; successful scripts communicate through their normal stdout JSONL protocol and app logs. The mandatory Bun gate for final user scripts is published in `verification`.".into(),
        error_output_shape: "No dedicated error envelope is emitted for stdin `run`; failures surface through script error protocol messages, app logs, and HUD/toast feedback. If either Bun verification command fails, the agent must fix the script and rerun both commands before reporting success.".into(),
        verification: HarnessVerificationContract {
            required: true,
            skill_path: crate::ai::harness::SCRIPT_AUTHORING_SKILL_MARKER.into(),
            build_command: crate::ai::harness::BUN_BUILD_VERIFICATION_MARKER.into(),
            run_command: crate::ai::harness::BUN_EXECUTE_VERIFICATION_MARKER.into(),
            success_criteria: crate::ai::harness::BUN_VERIFICATION_SUCCESS_CRITERIA.into(),
            failure_policy: crate::ai::harness::BUN_VERIFICATION_FAILURE_POLICY.into(),
        },
        example_test_script: concat!(
            "import \"@scriptkit/sdk\";\n",
            "\n",
            "export const metadata = {\n",
            "  name: \"Harness Test\",\n",
            "  description: \"Automated test script\",\n",
            "};\n",
            "\n",
            "const isVerify = process.env.SK_VERIFY === \"1\";\n",
            "\n",
            "const result = isVerify\n",
            "  ? \"a\"\n",
            "  : await arg(\"Pick one\", [\"a\", \"b\", \"c\"]);\n",
            "\n",
            "if (isVerify) {\n",
            "  console.log(JSON.stringify({ ok: true, result }));\n",
            "} else {\n",
            "  await div(md(`## ${result}`));\n",
            "}\n",
        ).into(),
        example_scriptlet: concat!(
            "---\n",
            "name: Date Tools\n",
            "description: Helpful date utilities\n",
            "icon: calendar-days\n",
            "---\n",
            "\n",
            "## Copy Date\n",
            "\n",
            "```metadata\n",
            "description: Copy today's date\n",
            "shortcut: opt d\n",
            "```\n",
            "\n",
            "```tool:copy-date\n",
            "import \"@scriptkit/sdk\";\n",
            "\n",
            "await copy(new Date().toISOString().slice(0, 10));\n",
            "hud(\"Copied today's date\");\n",
            "```\n",
        ).into(),
    }
}

/// Read kit://scripts schema-versioned resource
fn read_kit_scripts_resource(scripts: &[Arc<Script>]) -> Result<ResourceContent, String> {
    let entries: Vec<ScriptResourceEntry> = scripts
        .iter()
        .map(|s| ScriptResourceEntry::from(s.as_ref()))
        .collect();
    let doc = ScriptsResourceDocument {
        schema_version: SCRIPTS_RESOURCE_SCHEMA_VERSION,
        count: entries.len(),
        scripts: entries,
    };
    let json = serde_json::to_string_pretty(&doc)
        .map_err(|e| format!("Failed to serialize scripts document: {e}"))?;
    Ok(ResourceContent {
        uri: "kit://scripts".to_string(),
        mime_type: "application/json".to_string(),
        text: json,
    })
}

/// Pure builder for [`FailedScriptsResourceDocument`]. Split from the
/// resource handler so tests can exercise envelope shape against hand-built
/// [`ValidationReport`]s without touching the filesystem.
pub(crate) fn build_failed_scripts_document(
    report: &ValidationReport,
) -> FailedScriptsResourceDocument {
    FailedScriptsResourceDocument {
        schema_version: FAILED_SCRIPTS_RESOURCE_SCHEMA_VERSION,
        validation_schema_version: report.schema_version,
        total_candidates: report.total_candidates,
        valid_count: report.valid_count,
        fatal_count: report.fatal_count,
        warning_count: report.warning_count,
        failed_scripts: report
            .failed_scripts
            .iter()
            .map(FailedScriptEntry::from)
            .collect(),
        warnings: report.warnings.iter().cloned().collect(),
    }
}

/// Read kit://failed-scripts schema-versioned resource.
///
/// Calls [`crate::scripts::read_scripts_report`] at read time (rather than
/// requiring a cached report threaded through [`read_resource`]), so the
/// response always reflects the current disk state. This is cheap relative
/// to MCP request cadence — script loading already runs at startup.
fn read_kit_failed_scripts_resource() -> Result<ResourceContent, String> {
    let report = crate::scripts::read_scripts_report();
    let doc = build_failed_scripts_document(&report.validation);
    let json = serde_json::to_string_pretty(&doc)
        .map_err(|e| format!("Failed to serialize failed-scripts document: {e}"))?;
    Ok(ResourceContent {
        uri: FAILED_SCRIPTS_RESOURCE_URI.to_string(),
        mime_type: "application/json".to_string(),
        text: json,
    })
}

/// Read kit://script-templates schema-versioned resource.
fn read_kit_script_templates_resource() -> Result<ResourceContent, String> {
    let doc = build_script_templates_document();
    let json = serde_json::to_string_pretty(&doc)
        .map_err(|e| format!("Failed to serialize script-templates document: {e}"))?;
    Ok(ResourceContent {
        uri: SCRIPT_TEMPLATES_RESOURCE_URI.to_string(),
        mime_type: "application/json".to_string(),
        text: json,
    })
}

/// Read kit://scriptlets schema-versioned resource
fn read_kit_scriptlets_resource(scriptlets: &[Arc<Scriptlet>]) -> Result<ResourceContent, String> {
    let entries: Vec<ScriptletResourceEntry> = scriptlets
        .iter()
        .map(|s| ScriptletResourceEntry::from(s.as_ref()))
        .collect();
    let doc = ScriptletsResourceDocument {
        schema_version: SCRIPTLETS_RESOURCE_SCHEMA_VERSION,
        count: entries.len(),
        scriptlets: entries,
    };
    let json = serde_json::to_string_pretty(&doc)
        .map_err(|e| format!("Failed to serialize scriptlets document: {e}"))?;
    Ok(ResourceContent {
        uri: "kit://scriptlets".to_string(),
        mime_type: "application/json".to_string(),
        text: json,
    })
}

/// Read kit://sdk-reference resource
fn read_sdk_reference_resource() -> Result<ResourceContent, String> {
    let doc = build_sdk_reference_document();
    tracing::info!(
        category = "MCP",
        schema_version = doc.schema_version,
        function_count = doc.functions.len(),
        "Built kit://sdk-reference document"
    );
    let json = serde_json::to_string_pretty(&doc)
        .map_err(|e| format!("Failed to serialize SDK reference: {e}"))?;
    Ok(ResourceContent {
        uri: "kit://sdk-reference".to_string(),
        mime_type: "application/json".to_string(),
        text: json,
    })
}
// ---------------------------------------------------------------
// Clipboard history resource
// ---------------------------------------------------------------

/// Default limit for clipboard history entries returned.
const CLIPBOARD_HISTORY_DEFAULT_LIMIT: usize = 10;

/// Maximum limit for clipboard history entries.
const CLIPBOARD_HISTORY_MAX_LIMIT: usize = 50;

/// Default limit for dictation history entries returned.
const DICTATION_HISTORY_DEFAULT_LIMIT: usize = 10;

/// Maximum limit for dictation history entries.
const DICTATION_HISTORY_MAX_LIMIT: usize = 50;

/// Parsed clipboard history request — either a list query or a single-entry lookup.
#[derive(Debug)]
enum ClipboardHistoryRequest {
    /// List mode: fetch up to `limit` entries, optionally with diagnostics wrapper.
    List { limit: usize, diagnostics: bool },
    /// Single-entry mode: fetch the entry with the given ID.
    SingleEntry { id: String },
}

fn parse_clipboard_history_request(uri: &str) -> Result<ClipboardHistoryRequest, String> {
    if uri == "kit://clipboard-history" {
        return Ok(ClipboardHistoryRequest::List {
            limit: CLIPBOARD_HISTORY_DEFAULT_LIMIT,
            diagnostics: false,
        });
    }

    let (_base, query) = uri
        .split_once('?')
        .ok_or_else(|| format!("Resource not found: {uri}"))?;

    let mut limit = CLIPBOARD_HISTORY_DEFAULT_LIMIT;
    let mut diagnostics = false;
    let mut entry_id: Option<String> = None;

    for pair in query.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, "1"));
        match key {
            "id" => {
                entry_id = Some(value.to_string());
            }
            "limit" => {
                limit = value
                    .parse::<usize>()
                    .map_err(|_| {
                        format!("Invalid limit value: {value}. Expected a positive integer.")
                    })?
                    .min(CLIPBOARD_HISTORY_MAX_LIMIT);
            }
            "diagnostics" => diagnostics = parse_bool_param(value)?,
            _ => {
                return Err(format!(
                    "Invalid kit://clipboard-history parameter: {key}. Supported parameters: id, limit, diagnostics."
                ));
            }
        }
    }

    if let Some(id) = entry_id {
        Ok(ClipboardHistoryRequest::SingleEntry { id })
    } else {
        Ok(ClipboardHistoryRequest::List { limit, diagnostics })
    }
}

fn read_clipboard_history_entries(limit: usize) -> Vec<ClipboardHistoryEntry> {
    let cached = crate::clipboard_history::get_cached_entries(limit);
    cached
        .into_iter()
        .map(|entry| ClipboardHistoryEntry {
            id: entry.id,
            content_type: entry.content_type.as_str().to_string(),
            timestamp: entry.timestamp,
            text_preview: if entry.text_preview.is_empty() || entry.text_preview == "[Image]" {
                None
            } else {
                Some(entry.text_preview)
            },
            ocr_text: entry.ocr_text,
            image_width: entry.image_width,
            image_height: entry.image_height,
            pinned: entry.pinned,
        })
        .collect()
}

/// Read kit://clipboard-history resource
fn read_clipboard_history_resource(uri: &str) -> Result<ResourceContent, String> {
    let request = parse_clipboard_history_request(uri)?;

    match request {
        ClipboardHistoryRequest::SingleEntry { id } => {
            // Single-entry mode: return the entry's text content directly.
            let content = crate::clipboard_history::get_entry_content(&id)
                .ok_or_else(|| format!("Clipboard entry not found: {id}"))?;
            Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: "text/plain".to_string(),
                text: content,
            })
        }
        ClipboardHistoryRequest::List { limit, diagnostics } => {
            let started = Instant::now();
            let entries = read_clipboard_history_entries(limit);
            let duration_ms = started.elapsed().as_millis();

            let doc = ClipboardHistoryDocument {
                schema_version: CLIPBOARD_HISTORY_RESOURCE_SCHEMA_VERSION,
                count: entries.len(),
                entries,
            };

            let json = if diagnostics {
                let diag = ClipboardHistoryDiagnosticsDocument {
                    kind: "clipboard_history_diagnostics",
                    uri: uri.to_string(),
                    meta: ClipboardHistoryDiagnosticsMeta {
                        duration_ms,
                        entry_count: doc.count,
                        source: "cached_entries",
                    },
                    document: doc,
                };
                serde_json::to_string_pretty(&diag).map_err(|e| {
                    format!("Failed to serialize clipboard history diagnostics: {e}")
                })?
            } else {
                serde_json::to_string_pretty(&doc)
                    .map_err(|e| format!("Failed to serialize clipboard history: {e}"))?
            };

            Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: "application/json".to_string(),
                text: json,
            })
        }
    }
}

#[derive(Debug)]
enum DictationHistoryRequest {
    List { limit: usize },
    SingleEntry { id: String },
}

fn parse_dictation_history_request(uri: &str) -> Result<DictationHistoryRequest, String> {
    if uri == "kit://dictation-history" {
        return Ok(DictationHistoryRequest::List {
            limit: DICTATION_HISTORY_DEFAULT_LIMIT,
        });
    }

    let (_base, query) = uri
        .split_once('?')
        .ok_or_else(|| format!("Resource not found: {uri}"))?;

    let mut limit = DICTATION_HISTORY_DEFAULT_LIMIT;
    let mut entry_id: Option<String> = None;

    for pair in query.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, "1"));
        match key {
            "id" => entry_id = Some(value.to_string()),
            "limit" => {
                limit = value
                    .parse::<usize>()
                    .map_err(|_| {
                        format!("Invalid limit value: {value}. Expected a positive integer.")
                    })?
                    .min(DICTATION_HISTORY_MAX_LIMIT);
            }
            _ => {
                return Err(format!(
                    "Invalid kit://dictation-history parameter: {key}. Supported parameters: id, limit."
                ));
            }
        }
    }

    if let Some(id) = entry_id {
        Ok(DictationHistoryRequest::SingleEntry { id })
    } else {
        Ok(DictationHistoryRequest::List { limit })
    }
}

fn read_dictation_history_resource(uri: &str) -> Result<ResourceContent, String> {
    match parse_dictation_history_request(uri)? {
        DictationHistoryRequest::SingleEntry { id } => {
            let entry = crate::dictation::get_history_entry(&id)
                .ok_or_else(|| format!("Dictation history entry not found: {id}"))?;
            Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: "text/plain".to_string(),
                text: entry.transcript,
            })
        }
        DictationHistoryRequest::List { limit } => {
            let entries: Vec<crate::dictation::DictationHistoryEntry> =
                crate::dictation::load_history()
                    .into_iter()
                    .take(limit)
                    .collect();
            let json = serde_json::to_string_pretty(&entries)
                .map_err(|e| format!("Failed to serialize dictation history: {e}"))?;
            Ok(ResourceContent {
                uri: uri.to_string(),
                mime_type: "application/json".to_string(),
                text: json,
            })
        }
    }
}

// ---------------------------------------------------------------
// Focused item resource
// ---------------------------------------------------------------

fn parse_focused_item_request(uri: &str) -> Result<bool, String> {
    if uri == "kit://focused-item" {
        return Ok(false);
    }

    let (_base, query) = uri
        .split_once('?')
        .ok_or_else(|| format!("Resource not found: {uri}"))?;

    let mut diagnostics = false;

    for pair in query.split('&').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, "1"));
        match key {
            "diagnostics" => diagnostics = parse_bool_param(value)?,
            _ => {
                return Err(format!(
                    "Invalid kit://focused-item parameter: {key}. Supported parameters: diagnostics."
                ));
            }
        }
    }

    Ok(diagnostics)
}

/// Read the focused item from the global focused-item slot.
///
/// The slot is populated by surfaces (e.g., Tab AI orchestration) when they
/// resolve the focused/selected item. Outside of those flows, the slot is empty
/// and the resource returns `hasFocusedItem: false`.
fn read_focused_item_data() -> (Option<FocusedItemInfo>, Vec<String>) {
    let guard = FOCUSED_ITEM_SLOT.lock();
    match guard.as_ref() {
        Some(item) => (Some(item.clone()), Vec::new()),
        None => (
            None,
            vec!["no_active_surface: No surface has published a focused item.".to_string()],
        ),
    }
}

/// Read kit://focused-item resource
fn read_focused_item_resource(uri: &str) -> Result<ResourceContent, String> {
    let diagnostics = parse_focused_item_request(uri)?;

    let started = Instant::now();
    let (focused_item, warnings) = read_focused_item_data();
    let duration_ms = started.elapsed().as_millis();

    let doc = FocusedItemDocument {
        schema_version: FOCUSED_ITEM_RESOURCE_SCHEMA_VERSION,
        has_focused_item: focused_item.is_some(),
        focused_item,
        warnings: warnings.clone(),
    };

    let json = if diagnostics {
        let diag = FocusedItemDiagnosticsDocument {
            kind: "focused_item_diagnostics",
            uri: uri.to_string(),
            meta: FocusedItemDiagnosticsMeta {
                duration_ms,
                has_focused_item: doc.has_focused_item,
                warning_count: warnings.len(),
                source: "focused_item_slot".to_string(),
            },
            document: doc,
        };
        serde_json::to_string_pretty(&diag)
            .map_err(|e| format!("Failed to serialize focused item diagnostics: {e}"))?
    } else {
        serde_json::to_string_pretty(&doc)
            .map_err(|e| format!("Failed to serialize focused item: {e}"))?
    };

    Ok(ResourceContent {
        uri: uri.to_string(),
        mime_type: "application/json".to_string(),
        text: json,
    })
}

/// Global slot for the currently focused item, populated by surface resolvers.
static FOCUSED_ITEM_SLOT: parking_lot::Mutex<Option<FocusedItemInfo>> =
    parking_lot::Mutex::new(None);

/// Publish a focused item to the global slot so `kit://focused-item` can serve it.
#[allow(dead_code)] // Public API surface — called by Tab AI orchestration at runtime
pub fn publish_focused_item(item: FocusedItemInfo) {
    *FOCUSED_ITEM_SLOT.lock() = Some(item);
}

/// Clear the focused item slot (e.g., when the surface is dismissed).
#[allow(dead_code)] // Public API surface — called when surfaces are dismissed at runtime
pub fn clear_focused_item() {
    *FOCUSED_ITEM_SLOT.lock() = None;
}

// ---------------------------------------------------------------
// Provider-backed JSON resources: dictation, calendar, notifications
//
// Resolution priority:
// 1. In-process JSON slot (published by app features at runtime)
// 2. Environment variable (legacy / external script bridge)
// 3. Static empty fallback envelope
// ---------------------------------------------------------------

static DICTATION_JSON_SLOT: parking_lot::Mutex<Option<String>> = parking_lot::Mutex::new(None);
static CALENDAR_JSON_SLOT: parking_lot::Mutex<Option<String>> = parking_lot::Mutex::new(None);
static NOTIFICATIONS_JSON_SLOT: parking_lot::Mutex<Option<String>> = parking_lot::Mutex::new(None);

/// Publish dictation data into the in-process slot for `kit://dictation`.
pub fn publish_dictation_json(json: impl Into<String>) {
    *DICTATION_JSON_SLOT.lock() = Some(json.into());
}

/// Publish calendar data into the in-process slot for `kit://calendar`.
pub fn publish_calendar_json(json: impl Into<String>) {
    *CALENDAR_JSON_SLOT.lock() = Some(json.into());
}

/// Publish notifications data into the in-process slot for `kit://notifications`.
pub fn publish_notifications_json(json: impl Into<String>) {
    *NOTIFICATIONS_JSON_SLOT.lock() = Some(json.into());
}

/// Clear all provider JSON slots (e.g. on app reset).
pub fn clear_provider_json_slots() {
    *DICTATION_JSON_SLOT.lock() = None;
    *CALENDAR_JSON_SLOT.lock() = None;
    *NOTIFICATIONS_JSON_SLOT.lock() = None;
}

/// Provider-backed resource kinds that may or may not have real data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderJsonResourceKind {
    Dictation,
    Calendar,
    Notifications,
}

/// Returns `true` when the raw JSON text represents a provider payload
/// with real data, not just a placeholder envelope.
fn provider_json_text_has_real_data(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return false;
    };
    let Some(object) = value.as_object() else {
        return false;
    };
    if object.get("available").and_then(|v| v.as_bool()) == Some(false) {
        return false;
    }

    let envelope_only = object.keys().all(|key| {
        matches!(
            key.as_str(),
            "schemaVersion"
                | "type"
                | "ok"
                | "available"
                | "source"
                | "items"
                | "note"
                | "nextStep"
        )
    });

    if let Some(items) = object.get("items").and_then(|v| v.as_array()) {
        if !items.is_empty() {
            return true;
        }
        if envelope_only {
            return false;
        }
    }

    if object.get("available").and_then(|v| v.as_bool()) == Some(true) && !envelope_only {
        return true;
    }

    // Treat any other non-empty provider object as real data too. Some
    // callers seed legacy payloads like {"transcription":"test"} or valid
    // empty-state payloads like {"events":[]} that should still surface the
    // provider-backed picker entries.
    !object.is_empty()
}

/// Resolve the raw JSON candidate text and its source label for a provider kind.
fn provider_json_candidate(kind: ProviderJsonResourceKind) -> (Option<String>, &'static str) {
    match kind {
        ProviderJsonResourceKind::Dictation => {
            if let Some(text) = DICTATION_JSON_SLOT.lock().clone() {
                (Some(text), "slot")
            } else {
                (std::env::var("SCRIPT_KIT_DICTATION_JSON").ok(), "env")
            }
        }
        ProviderJsonResourceKind::Calendar => {
            if let Some(text) = CALENDAR_JSON_SLOT.lock().clone() {
                (Some(text), "slot")
            } else {
                (std::env::var("SCRIPT_KIT_CALENDAR_JSON").ok(), "env")
            }
        }
        ProviderJsonResourceKind::Notifications => {
            if let Some(text) = NOTIFICATIONS_JSON_SLOT.lock().clone() {
                (Some(text), "slot")
            } else {
                (std::env::var("SCRIPT_KIT_NOTIFICATIONS_JSON").ok(), "env")
            }
        }
    }
}

/// Returns `true` when the provider has real data (parsed payload truth),
/// as opposed to only a placeholder or empty envelope.
pub fn has_provider_json_resource(kind: ProviderJsonResourceKind) -> bool {
    let (candidate, source) = provider_json_candidate(kind);
    let has_real_data = candidate
        .as_deref()
        .map(provider_json_text_has_real_data)
        .unwrap_or(false);
    tracing::info!(
        target: "ai",
        event = "mcp_provider_json_availability_checked",
        kind = ?kind,
        source,
        has_candidate = candidate.is_some(),
        has_real_data,
    );
    has_real_data
}

pub struct ProviderJsonItem {
    pub title: String,
    pub subtitle: Option<String>,
}

pub fn read_provider_json_items(kind: ProviderJsonResourceKind) -> Vec<ProviderJsonItem> {
    let (candidate, _source) = provider_json_candidate(kind);
    let Some(text) = candidate else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    let Some(items) = value.get("items").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let title = item
                .get("title")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())?;
            let subtitle = item
                .get("subtitle")
                .or_else(|| item.get("app"))
                .or_else(|| item.get("source"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Some(ProviderJsonItem {
                title: title.to_string(),
                subtitle,
            })
        })
        .collect()
}

/// Determine the resolution source for a provider-backed JSON resource.
fn provider_json_source(slot_value: &Option<String>, env_key: &str) -> &'static str {
    if slot_value.is_some() {
        "slot"
    } else if std::env::var_os(env_key).is_some() {
        "env"
    } else {
        "empty-fallback"
    }
}

/// Build an explicit empty-fallback JSON envelope with stable fields.
fn empty_provider_json(kind: &str, note: &str, next_step: &str, source: &str) -> String {
    format!(
        r#"{{"schemaVersion":1,"type":"{kind}","ok":true,"available":false,"source":"{source}","items":[],"note":"{note}","nextStep":"{next_step}"}}"#
    )
}

/// Read a JSON resource from an in-process slot, falling back to an environment
/// variable, then to a static empty envelope with explicit `source` tracking.
fn read_slot_or_env_backed_json_resource(
    uri: &str,
    slot_value: Option<String>,
    env_key: &str,
    kind: &str,
    note: &str,
    next_step: &str,
    event_name: &'static str,
) -> Result<ResourceContent, String> {
    let source = provider_json_source(&slot_value, env_key);
    let raw = slot_value.or_else(|| std::env::var(env_key).ok());
    let text = match raw {
        Some(text) if provider_json_text_has_real_data(&text) => text,
        Some(text) => {
            tracing::info!(
                target: "ai",
                event = "mcp_provider_json_placeholder_normalized",
                %uri,
                env_key,
                source,
                bytes = text.len(),
            );
            empty_provider_json(kind, note, next_step, source)
        }
        None => empty_provider_json(kind, note, next_step, source),
    };
    tracing::info!(
        target: "ai",
        event = %event_name,
        %uri,
        env_key,
        source,
        bytes = text.len(),
        "mcp_provider_json_resource_read"
    );
    Ok(ResourceContent {
        uri: uri.to_string(),
        mime_type: "application/json".to_string(),
        text,
    })
}

fn read_dictation_resource(uri: &str) -> Result<ResourceContent, String> {
    read_slot_or_env_backed_json_resource(
        uri,
        DICTATION_JSON_SLOT.lock().clone(),
        "SCRIPT_KIT_DICTATION_JSON",
        "dictation",
        "No dictation provider configured.",
        "Publish dictation JSON or set SCRIPT_KIT_DICTATION_JSON.",
        "mcp_dictation_resource_read",
    )
}

fn read_calendar_resource(uri: &str) -> Result<ResourceContent, String> {
    read_slot_or_env_backed_json_resource(
        uri,
        CALENDAR_JSON_SLOT.lock().clone(),
        "SCRIPT_KIT_CALENDAR_JSON",
        "calendar",
        "No calendar provider configured.",
        "Publish calendar JSON or set SCRIPT_KIT_CALENDAR_JSON.",
        "mcp_calendar_resource_read",
    )
}

fn read_notifications_resource(uri: &str) -> Result<ResourceContent, String> {
    read_slot_or_env_backed_json_resource(
        uri,
        NOTIFICATIONS_JSON_SLOT.lock().clone(),
        "SCRIPT_KIT_NOTIFICATIONS_JSON",
        "notifications",
        "No notifications provider configured.",
        "Publish notifications JSON or set SCRIPT_KIT_NOTIFICATIONS_JSON.",
        "mcp_notifications_resource_read",
    )
}

// ---------------------------------------------------------------
// Shell-backed resources: git-status, git-diff, processes, system
// ---------------------------------------------------------------

/// Run a shell command and capture stdout, returning a fallback on failure.
fn run_shell_resource(program: &str, args: &[&str], uri: &str) -> Result<ResourceContent, String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {program}: {e}"))?;

    let text = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        format!("Command exited with {}: {}", output.status, stderr.trim())
    };

    Ok(ResourceContent {
        uri: uri.to_string(),
        mime_type: "text/plain".to_string(),
        text,
    })
}

/// Read `kit://git-status` — runs `git status` in the current directory.
fn read_git_status_resource() -> Result<ResourceContent, String> {
    run_shell_resource("git", &["status"], "kit://git-status")
}

/// Read `kit://git-diff` — runs `git diff` (combined staged + unstaged).
fn read_git_diff_resource(uri: &str) -> Result<ResourceContent, String> {
    // Show both staged and unstaged changes
    let staged = std::process::Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .map_err(|e| format!("Failed to run git diff --cached: {e}"))?;
    let unstaged = std::process::Command::new("git")
        .args(["diff"])
        .output()
        .map_err(|e| format!("Failed to run git diff: {e}"))?;

    let mut text = String::new();
    let staged_out = String::from_utf8_lossy(&staged.stdout);
    let unstaged_out = String::from_utf8_lossy(&unstaged.stdout);
    let staged_err = String::from_utf8_lossy(&staged.stderr);
    let unstaged_err = String::from_utf8_lossy(&unstaged.stderr);

    if !staged.status.success() {
        text.push_str("=== Staged changes ===\n");
        text.push_str(&format!(
            "Command exited with {}: {}\n",
            staged.status,
            staged_err.trim()
        ));
    }

    if staged.status.success() && !staged_out.is_empty() {
        text.push_str("=== Staged changes ===\n");
        text.push_str(&staged_out);
    }
    if !unstaged.status.success() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str("=== Unstaged changes ===\n");
        text.push_str(&format!(
            "Command exited with {}: {}\n",
            unstaged.status,
            unstaged_err.trim()
        ));
    }
    if unstaged.status.success() && !unstaged_out.is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str("=== Unstaged changes ===\n");
        text.push_str(&unstaged_out);
    }
    if text.is_empty() {
        text.push_str("No changes.");
    }
    let total_bytes = text.len();
    let limit = parse_u64_query_param(uri, "limitBytes")
        .map(|value| value as usize)
        .unwrap_or(GIT_DIFF_DEFAULT_LIMIT_BYTES)
        .clamp(1, GIT_DIFF_HARD_CAP_BYTES);
    let offset = parse_u64_query_param(uri, "offsetBytes")
        .map(|value| value as usize)
        .unwrap_or(0)
        .min(total_bytes);
    let end = next_char_boundary(&text, (offset + limit).min(total_bytes));
    let start = next_char_boundary(&text, offset);
    let truncated = end < total_bytes || start > 0;
    let mut bounded_text = text[start..end].to_string();
    if truncated {
        bounded_text.push_str(&format!(
            "\n\n[kit://git-diff truncated: offsetBytes={start}, limitBytes={limit}, totalBytes={total_bytes}]"
        ));
    }

    Ok(ResourceContent {
        uri: uri.to_string(),
        mime_type: "text/plain".to_string(),
        text: bounded_text,
    })
}

fn next_char_boundary(text: &str, mut idx: usize) -> usize {
    idx = idx.min(text.len());
    while idx < text.len() && !text.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}

/// Read `kit://processes` — top processes by CPU.
fn read_processes_resource() -> Result<ResourceContent, String> {
    run_shell_resource("ps", &["aux", "--sort=-%cpu"], "kit://processes").or_else(|_| {
        // macOS ps doesn't support --sort; fall back to piped sort
        let ps = std::process::Command::new("ps")
            .args(["aux"])
            .output()
            .map_err(|e| format!("Failed to run ps: {e}"))?;
        let text = String::from_utf8_lossy(&ps.stdout).to_string();
        Ok(ResourceContent {
            uri: "kit://processes".to_string(),
            mime_type: "text/plain".to_string(),
            text,
        })
    })
}

/// Read `kit://system` — basic system info.
fn read_system_info_resource() -> Result<ResourceContent, String> {
    let mut lines = Vec::new();

    if let Ok(output) = std::process::Command::new("uname").args(["-a"]).output() {
        if output.status.success() {
            lines.push(format!(
                "System: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            ));
        }
    }
    if let Ok(output) = std::process::Command::new("hostname").output() {
        if output.status.success() {
            lines.push(format!(
                "Hostname: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            ));
        }
    }
    if let Ok(output) = std::process::Command::new("uptime").output() {
        if output.status.success() {
            lines.push(format!(
                "Uptime: {}",
                String::from_utf8_lossy(&output.stdout).trim()
            ));
        }
    }
    if let Ok(shell) = std::env::var("SHELL") {
        lines.push(format!("Shell: {shell}"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        lines.push(format!("CWD: {}", cwd.display()));
    }

    let text = if lines.is_empty() {
        "System info unavailable.".to_string()
    } else {
        lines.join("\n")
    };

    Ok(ResourceContent {
        uri: "kit://system".to_string(),
        mime_type: "text/plain".to_string(),
        text,
    })
}

/// Read the `kit://stdin-commands` drift-audited reference resource.
///
/// Emits markdown prose documenting the stdin JSONL envelope, with a
/// `<!-- drift-audit:stdin-verbs:start -->` … `:end` block that enumerates
/// every accepted `type` verb in the shape `- \`verbName\`: description`.
/// `tests/mcp_resource_drift.rs` pins the block against
/// [`crate::stdin_commands::all_external_command_verbs`].
fn read_stdin_commands_resource() -> Result<ResourceContent, String> {
    let mut body = String::new();
    body.push_str(
        "# Stdin JSONL Commands\n\n\
         Script Kit GPUI accepts one JSON object per line on stdin. Each \
         command is dispatched through `ExternalCommand` after the optional \
         `protocolVersion` gate in `src/stdin_commands/mod.rs`.\n\n\
         Example:\n\n\
         ```json\n\
         {\"type\":\"triggerBuiltin\",\"builtinId\":\"builtin/clipboard-history\"}\n\
         ```\n\n\
         The list below is the only source agents should trust for the \
         accepted `type` verb spelling. It is kept in sync with the \
         `ExternalCommand::command_type` match by the drift-audit in \
         `tests/mcp_resource_drift.rs`.\n\n\
         ## Verbs\n\n\
         <!-- drift-audit:stdin-verbs:start -->\n",
    );
    for verb in crate::stdin_commands::all_external_command_verbs() {
        body.push_str(&format!(
            "- `{verb}`: Dispatched as `ExternalCommand::{variant}` in `src/stdin_commands/mod.rs`.\n",
            variant = stdin_verb_variant_hint(verb),
        ));
    }
    body.push_str("<!-- drift-audit:stdin-verbs:end -->\n");

    Ok(ResourceContent {
        uri: STDIN_COMMANDS_REFERENCE_URI.to_string(),
        mime_type: "text/markdown".to_string(),
        text: body,
    })
}

/// Map a stdin verb back to its `ExternalCommand` variant name for the
/// resource prose. Purely cosmetic — the drift audit is on the verb, not
/// the hint string.
fn stdin_verb_variant_hint(verb: &str) -> &'static str {
    match verb {
        "run" => "Run",
        "show" => "Show",
        "hide" => "Hide",
        "setFilter" => "SetFilter",
        "triggerBuiltin" => "TriggerBuiltin",
        "simulateKey" => "SimulateKey",
        "openNotes" => "OpenNotes",
        "openAbout" => "OpenAbout",
        "openAi" => "OpenAi",
        "openMiniAi" => "OpenMiniAi",
        "openAiWithMockData" => "OpenAiWithMockData",
        "openMiniAiWithMockData" => "OpenMiniAiWithMockData",
        "showAiCommandBar" => "ShowAiCommandBar",
        "simulateAiKey" => "SimulateAiKey",
        "captureWindow" => "CaptureWindow",
        "setAiSearch" => "SetAiSearch",
        "setAiInput" => "SetAiInput",
        "setAgentChatInput" => "SetAgentChatInput",
        "getAiWindowState" => "GetAiWindowState",
        "showGrid" => "ShowGrid",
        "hideGrid" => "HideGrid",
        "showShortcutRecorder" => "ShowShortcutRecorder",
        "executeFallback" => "ExecuteFallback",
        "triggerAction" => "TriggerAction",
        "pasteClipboardIntoAgentChat" => "PasteClipboardIntoAgentChat",
        "pushDictationResult" => "PushDictationResult",
        "getConfigFingerprint" => "GetConfigFingerprint",
        _ => "(unknown)",
    }
}

/// Read the `kit://trigger-builtins` drift-audited reference resource.
///
/// Emits markdown prose listing every canonical `builtin/...` command id
/// accepted by the `triggerBuiltin` stdin verb, wrapped in a
/// `<!-- drift-audit:trigger-builtin-ids:start -->` block.
/// `tests/mcp_resource_drift.rs` pins the block against
/// [`crate::builtins::trigger_registry::all_trigger_builtin_command_ids`].
fn read_trigger_builtins_resource() -> Result<ResourceContent, String> {
    let mut body = String::new();
    body.push_str(
        "# Trigger Built-ins\n\n\
         Canonical `builtin/...` command IDs accepted by the `triggerBuiltin` \
         stdin verb. Legacy lowercase aliases (e.g. `clipboard`, `apps`) are \
         still resolved via the registry in \
         `src/builtins/trigger_registry.rs`, but new callers should use the \
         canonical IDs below.\n\n\
         Example:\n\n\
         ```json\n\
         {\"type\":\"triggerBuiltin\",\"builtinId\":\"builtin/clipboard-history\"}\n\
         ```\n\n\
         The list below is the only source agents should trust. It is kept \
         in sync with `TriggerBuiltin::ALL` by the drift-audit in \
         `tests/mcp_resource_drift.rs`.\n\n\
         ## Command IDs\n\n\
         <!-- drift-audit:trigger-builtin-ids:start -->\n",
    );
    for id in crate::builtins::trigger_registry::all_trigger_builtin_command_ids() {
        body.push_str(&format!(
            "- `{id}`: Canonical trigger-builtin command id.\n",
        ));
    }
    body.push_str("<!-- drift-audit:trigger-builtin-ids:end -->\n");

    Ok(ResourceContent {
        uri: TRIGGER_BUILTINS_REFERENCE_URI.to_string(),
        mime_type: "text/markdown".to_string(),
        text: body,
    })
}

/// Read the `kit://diagnostics/protocol-stats` resource
/// (Oracle-Session `protocol-builtin-boundary-refactor-plan` PR4).
///
/// Returns a serialized [`crate::protocol_stats::ProtocolStatsReport`]
/// so MCP consumers can render a live protocol-boundary health chip
/// without shelling out to logs. camelCase field names are baked into
/// the struct via `serde(rename_all = "camelCase")` so the wire shape
/// is stable.
fn read_protocol_stats_resource() -> Result<ResourceContent, String> {
    let report = crate::protocol_stats::current_report();
    let text = serde_json::to_string_pretty(&report)
        .map_err(|e| format!("failed to serialize protocol stats report: {e}"))?;
    Ok(ResourceContent {
        uri: PROTOCOL_STATS_DIAGNOSTICS_URI.to_string(),
        mime_type: "application/json".to_string(),
        text,
    })
}

/// Convert resource content to JSON-RPC result format
pub fn resource_content_to_value(content: ResourceContent) -> Value {
    serde_json::json!({
        "contents": [{
            "uri": content.uri,
            "mimeType": content.mime_type,
            "text": content.text
        }]
    })
}
/// Convert resource list to JSON-RPC result format
pub fn resource_list_to_value(resources: &[McpResource]) -> Value {
    serde_json::to_value(serde_json::json!({
        "resources": resources
    }))
    .unwrap_or(serde_json::json!({"resources": []}))
}

// ---------------------------------------------------------------
// Context resource types and helpers
// ---------------------------------------------------------------

use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextResourceKind {
    Snapshot,
    Schema,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContextResourceRequest {
    kind: ContextResourceKind,
    options: crate::context_snapshot::CaptureContextOptions,
    effective_profile: String,
    diagnostics: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ContextProfileDescriptor {
    name: &'static str,
    description: &'static str,
    options: crate::context_snapshot::CaptureContextOptions,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ContextParameterDescriptor {
    name: &'static str,
    value_type: &'static str,
    description: &'static str,
    default_value: &'static str,
    allowed_values: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ContextSchemaDocument {
    kind: &'static str,
    schema_version: u32,
    default_profile: &'static str,
    diagnostics_supported: bool,
    profiles: Vec<ContextProfileDescriptor>,
    parameters: Vec<ContextParameterDescriptor>,
    examples: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ContextFieldCaptureState {
    Disabled,
    Captured,
    Empty,
    Failed,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ContextFieldStatus {
    field: &'static str,
    enabled: bool,
    present: bool,
    state: ContextFieldCaptureState,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ContextWarningDescriptor {
    field: String,
    code: String,
    message: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ContextDiagnosticsStatus {
    Ok,
    Partial,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ContextDiagnosticsMeta {
    effective_profile: String,
    options: crate::context_snapshot::CaptureContextOptions,
    status: ContextDiagnosticsStatus,
    duration_ms: u128,
    snapshot_bytes: usize,
    enabled_field_count: usize,
    warning_count: usize,
    field_statuses: Vec<ContextFieldStatus>,
    warnings: Vec<ContextWarningDescriptor>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ContextDiagnosticsDocument {
    kind: &'static str,
    uri: String,
    snapshot: crate::context_snapshot::AiContextSnapshot,
    meta: ContextDiagnosticsMeta,
}

fn supported_context_examples() -> Vec<&'static str> {
    vec![
        "kit://context",
        "kit://context?profile=minimal",
        "kit://context?profile=minimal&diagnostics=1",
        "kit://context?selectedText=0&browserUrl=1&focusedWindow=1",
        "kit://context?screenshot=1",
        "kit://context?panelScreenshot=1",
        "kit://context?screenshot=1&panelScreenshot=1",
        "kit://context?screenshot=1&panelScreenshot=1&diagnostics=1",
        "kit://context/schema",
    ]
}

fn supported_context_param_names() -> &'static [&'static str] {
    &[
        "profile",
        "diagnostics",
        "selectedText",
        "frontmostApp",
        "menuBar",
        "browserUrl",
        "focusedWindow",
        "screenshot",
        "panelScreenshot",
    ]
}

fn invalid_context_param(key: &str, _value: &str) -> String {
    format!(
        "Invalid kit://context parameter: {key}. Supported parameters: {}. See kit://context/schema for the full contract and examples.",
        supported_context_param_names().join(", ")
    )
}

fn parse_bool_param(value: &str) -> Result<bool, String> {
    match value {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err(format!(
            "Invalid boolean value: {value}. Expected one of: 1, 0, true, false. See kit://context/schema."
        )),
    }
}

fn parse_context_resource_request(uri: &str) -> Result<ContextResourceRequest, String> {
    use crate::context_snapshot::CaptureContextOptions;

    if uri == "kit://context/schema" {
        return Ok(ContextResourceRequest {
            kind: ContextResourceKind::Schema,
            options: CaptureContextOptions::default(),
            effective_profile: "full".to_string(),
            diagnostics: false,
        });
    }

    if uri == "kit://context" {
        return Ok(ContextResourceRequest {
            kind: ContextResourceKind::Snapshot,
            options: CaptureContextOptions::default(),
            effective_profile: "full".to_string(),
            diagnostics: false,
        });
    }

    let (base, query) = uri
        .split_once('?')
        .ok_or_else(|| format!("Resource not found: {uri}"))?;

    if base == "kit://context/schema" {
        return Err(
            "kit://context/schema does not accept query parameters. Use plain kit://context/schema."
                .to_string(),
        );
    }

    if base != "kit://context" {
        return Err(format!("Resource not found: {uri}"));
    }

    let mut options = CaptureContextOptions::default();
    let mut selected_profile: Option<&str> = None;
    let mut diagnostics = false;
    let mut saw_override = false;
    let mut saw_explicit_screenshot = false;
    let mut saw_explicit_panel_screenshot = false;

    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, "1"));

        match (key, value) {
            ("profile", "full") => {
                options = CaptureContextOptions::all();
                selected_profile = Some("full");
            }
            ("profile", "minimal") => {
                options = CaptureContextOptions::minimal();
                selected_profile = Some("minimal");
            }
            ("profile", other) => {
                return Err(format!(
                    "Unknown profile: {other}. Supported profiles: full, minimal. See kit://context/schema."
                ));
            }
            ("diagnostics", v) => diagnostics = parse_bool_param(v)?,
            ("selectedText", v) => {
                options.include_selected_text = parse_bool_param(v)?;
                saw_override = true;
            }
            ("frontmostApp", v) => {
                options.include_frontmost_app = parse_bool_param(v)?;
                saw_override = true;
            }
            ("menuBar", v) => {
                options.include_menu_bar = parse_bool_param(v)?;
                saw_override = true;
            }
            ("browserUrl", v) => {
                options.include_browser_url = parse_bool_param(v)?;
                saw_override = true;
            }
            ("focusedWindow", v) => {
                options.include_focused_window = parse_bool_param(v)?;
                saw_override = true;
            }
            ("screenshot", v) => {
                options.include_screenshot = parse_bool_param(v)?;
                saw_override = true;
                saw_explicit_screenshot = true;
            }
            ("panelScreenshot", v) => {
                options.include_panel_screenshot = parse_bool_param(v)?;
                saw_override = true;
                saw_explicit_panel_screenshot = true;
            }
            _ => return Err(invalid_context_param(key, value)),
        }
    }

    // Pixel data must be opted into explicitly on custom (per-field) queries.
    // The baseline for overrides is `all()`, which includes the focused-window
    // screenshot — inherited silently, a metadata query like `?selectedText=1&
    // focusedWindow=0` used to embed a full-window base64 PNG and blow the
    // model's context window (758KB observed from the @selection attachment).
    // An explicit `profile=` keeps its documented pixel semantics.
    if (saw_override || diagnostics) && selected_profile.is_none() {
        if !saw_explicit_screenshot {
            options.include_screenshot = false;
        }
        if !saw_explicit_panel_screenshot {
            options.include_panel_screenshot = false;
        }
    }

    let effective_profile = if saw_override {
        "custom".to_string()
    } else {
        selected_profile.unwrap_or("full").to_string()
    };

    Ok(ContextResourceRequest {
        kind: ContextResourceKind::Snapshot,
        options,
        effective_profile,
        diagnostics,
    })
}

fn build_context_schema_document() -> ContextSchemaDocument {
    ContextSchemaDocument {
        kind: "context_schema",
        schema_version: crate::context_snapshot::AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
        default_profile: "full",
        diagnostics_supported: true,
        profiles: vec![
            ContextProfileDescriptor {
                name: "full",
                description: "Capture every currently supported context provider.",
                options: crate::context_snapshot::CaptureContextOptions::all(),
            },
            ContextProfileDescriptor {
                name: "minimal",
                description: "Lower-cost profile that omits selected text and menu bar.",
                options: crate::context_snapshot::CaptureContextOptions::minimal(),
            },
        ],
        parameters: vec![
            ContextParameterDescriptor {
                name: "profile",
                value_type: "enum",
                description: "Named bundle of capture flags.",
                default_value: "full",
                allowed_values: vec!["full", "minimal"],
            },
            ContextParameterDescriptor {
                name: "diagnostics",
                value_type: "boolean",
                description: "Wrap the snapshot in machine-readable metadata, warnings, and field-level status.",
                default_value: "false",
                allowed_values: vec!["1", "0", "true", "false"],
            },
            ContextParameterDescriptor {
                name: "selectedText",
                value_type: "boolean",
                description: "Include the current selection, if the platform/provider can read it.",
                default_value: "1",
                allowed_values: vec!["1", "0", "true", "false"],
            },
            ContextParameterDescriptor {
                name: "frontmostApp",
                value_type: "boolean",
                description: "Include the frontmost application identity.",
                default_value: "1",
                allowed_values: vec!["1", "0", "true", "false"],
            },
            ContextParameterDescriptor {
                name: "menuBar",
                value_type: "boolean",
                description: "Include summarized menu bar items for the frontmost app.",
                default_value: "1",
                allowed_values: vec!["1", "0", "true", "false"],
            },
            ContextParameterDescriptor {
                name: "browserUrl",
                value_type: "boolean",
                description: "Include the focused browser tab URL when available.",
                default_value: "1",
                allowed_values: vec!["1", "0", "true", "false"],
            },
            ContextParameterDescriptor {
                name: "focusedWindow",
                value_type: "boolean",
                description: "Include focused-window metadata derived from window capture.",
                default_value: "1",
                allowed_values: vec!["1", "0", "true", "false"],
            },
            ContextParameterDescriptor {
                name: "screenshot",
                value_type: "boolean",
                description: "Include focused-window screenshot bytes as base64 PNG in focusedWindowImage.",
                default_value: "false",
                allowed_values: vec!["1", "0", "true", "false"],
            },
            ContextParameterDescriptor {
                name: "panelScreenshot",
                value_type: "boolean",
                description: "Include Script Kit's visible panel screenshot as base64 PNG in scriptKitPanelImage.",
                default_value: "false",
                allowed_values: vec!["1", "0", "true", "false"],
            },
        ],
        examples: supported_context_examples(),
    }
}

fn context_warning_code(field: &str) -> &'static str {
    match field {
        "selectedText" => "selected_text_capture_failed",
        "frontmostApp" => "frontmost_app_capture_failed",
        "menuBar" => "menu_bar_capture_failed",
        "browserUrl" => "browser_url_capture_failed",
        "focusedWindow" => "focused_window_capture_failed",
        "screenshot" => "screenshot_capture_failed",
        "panelScreenshot" => "panel_screenshot_capture_failed",
        _ => "capture_failed",
    }
}

fn parse_context_warning(raw: &str) -> ContextWarningDescriptor {
    let (field, message) = raw
        .split_once(':')
        .map(|(field, message)| (field.trim(), message.trim()))
        .unwrap_or(("unknown", raw.trim()));

    ContextWarningDescriptor {
        field: field.to_string(),
        code: context_warning_code(field).to_string(),
        message: message.to_string(),
    }
}

fn build_context_field_status(
    field: &'static str,
    enabled: bool,
    present: bool,
    warnings_by_field: &HashMap<String, ContextWarningDescriptor>,
) -> ContextFieldStatus {
    let state = if !enabled {
        ContextFieldCaptureState::Disabled
    } else if warnings_by_field.contains_key(field) {
        ContextFieldCaptureState::Failed
    } else if present {
        ContextFieldCaptureState::Captured
    } else {
        ContextFieldCaptureState::Empty
    };

    ContextFieldStatus {
        field,
        enabled,
        present,
        state,
    }
}

fn build_context_field_statuses(
    options: &crate::context_snapshot::CaptureContextOptions,
    snapshot: &crate::context_snapshot::AiContextSnapshot,
    warnings_by_field: &HashMap<String, ContextWarningDescriptor>,
) -> Vec<ContextFieldStatus> {
    vec![
        build_context_field_status(
            "selectedText",
            options.include_selected_text,
            snapshot.selected_text.is_some(),
            warnings_by_field,
        ),
        build_context_field_status(
            "frontmostApp",
            options.include_frontmost_app,
            snapshot.frontmost_app.is_some(),
            warnings_by_field,
        ),
        build_context_field_status(
            "menuBar",
            options.include_menu_bar,
            !snapshot.menu_bar_items.is_empty(),
            warnings_by_field,
        ),
        build_context_field_status(
            "browserUrl",
            options.include_browser_url,
            snapshot.browser.is_some(),
            warnings_by_field,
        ),
        build_context_field_status(
            "focusedWindow",
            options.include_focused_window,
            snapshot.focused_window.is_some(),
            warnings_by_field,
        ),
        build_context_field_status(
            "screenshot",
            options.include_screenshot,
            snapshot.focused_window_image.is_some(),
            warnings_by_field,
        ),
        build_context_field_status(
            "panelScreenshot",
            options.include_panel_screenshot,
            snapshot.script_kit_panel_image.is_some(),
            warnings_by_field,
        ),
    ]
}

fn build_context_diagnostics_document(
    uri: &str,
    request: &ContextResourceRequest,
    snapshot: &crate::context_snapshot::AiContextSnapshot,
    duration_ms: u128,
) -> ContextDiagnosticsDocument {
    let warnings: Vec<ContextWarningDescriptor> = snapshot
        .warnings
        .iter()
        .map(|warning| parse_context_warning(warning))
        .collect();

    let warnings_by_field: HashMap<String, ContextWarningDescriptor> = warnings
        .iter()
        .cloned()
        .map(|warning| (warning.field.clone(), warning))
        .collect();

    let enabled_field_count = [
        request.options.include_selected_text,
        request.options.include_frontmost_app,
        request.options.include_menu_bar,
        request.options.include_browser_url,
        request.options.include_focused_window,
        request.options.include_screenshot,
        request.options.include_panel_screenshot,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();

    let snapshot_bytes = serde_json::to_vec(snapshot)
        .map(|bytes| bytes.len())
        .unwrap_or_default();

    let warning_count = warnings.len();
    let field_statuses =
        build_context_field_statuses(&request.options, snapshot, &warnings_by_field);

    ContextDiagnosticsDocument {
        kind: "context_diagnostics",
        uri: uri.to_string(),
        snapshot: snapshot.clone(),
        meta: ContextDiagnosticsMeta {
            effective_profile: request.effective_profile.clone(),
            options: request.options.clone(),
            status: if warnings.is_empty() {
                ContextDiagnosticsStatus::Ok
            } else {
                ContextDiagnosticsStatus::Partial
            },
            duration_ms,
            snapshot_bytes,
            enabled_field_count,
            warning_count,
            field_statuses,
            warnings,
        },
    }
}

fn serialize_context_resource(
    uri: &str,
    request: &ContextResourceRequest,
    snapshot: Option<&crate::context_snapshot::AiContextSnapshot>,
    duration_ms: u128,
) -> Result<String, String> {
    match request.kind {
        ContextResourceKind::Schema => {
            serde_json::to_string_pretty(&build_context_schema_document())
                .map_err(|error| format!("Failed to serialize context schema: {error}"))
        }
        ContextResourceKind::Snapshot => {
            let snapshot = snapshot.ok_or_else(|| {
                "Context snapshot missing while serializing response.".to_string()
            })?;

            if request.diagnostics {
                serde_json::to_string_pretty(&build_context_diagnostics_document(
                    uri,
                    request,
                    snapshot,
                    duration_ms,
                ))
                .map_err(|error| format!("Failed to serialize context diagnostics: {error}"))
            } else {
                serde_json::to_string_pretty(snapshot)
                    .map_err(|error| format!("Failed to serialize context snapshot: {error}"))
            }
        }
    }
}

/// Read kit://context or kit://context/schema resource
fn read_context_resource(uri: &str) -> Result<ResourceContent, String> {
    let request = parse_context_resource_request(uri).map_err(|error| {
        tracing::warn!(
            target: "script_kit::mcp_context_resource",
            uri = %uri,
            error = %error,
            "context_resource_read_invalid_request"
        );
        error
    })?;

    if matches!(request.kind, ContextResourceKind::Schema) {
        tracing::info!(
            target: "script_kit::mcp_context_resource",
            uri = %uri,
            "context_resource_schema_read"
        );

        return Ok(ResourceContent {
            uri: uri.to_string(),
            mime_type: "application/json".to_string(),
            text: serialize_context_resource(uri, &request, None, 0)?,
        });
    }

    tracing::info!(
        target: "script_kit::mcp_context_resource",
        uri = %uri,
        diagnostics = request.diagnostics,
        effective_profile = %request.effective_profile,
        selected_text = request.options.include_selected_text,
        frontmost_app = request.options.include_frontmost_app,
        menu_bar = request.options.include_menu_bar,
        browser_url = request.options.include_browser_url,
        focused_window = request.options.include_focused_window,
        "context_resource_read_start"
    );

    let started = Instant::now();
    let snapshot = crate::context_snapshot::capture_context_snapshot(&request.options);
    let duration_ms = started.elapsed().as_millis();

    tracing::info!(
        target: "script_kit::mcp_context_resource",
        uri = %uri,
        diagnostics = request.diagnostics,
        effective_profile = %request.effective_profile,
        duration_ms = duration_ms,
        warning_count = snapshot.warnings.len(),
        status = if snapshot.warnings.is_empty() { "ok" } else { "partial" },
        "context_resource_read_complete"
    );

    Ok(ResourceContent {
        uri: uri.to_string(),
        mime_type: "application/json".to_string(),
        text: serialize_context_resource(uri, &request, Some(&snapshot), duration_ms)?,
    })
}

// --- merged from part_001.rs ---
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// Helper to wrap Vec<Script> into Vec<Arc<Script>> for tests
    fn wrap_scripts(scripts: Vec<Script>) -> Vec<Arc<Script>> {
        scripts.into_iter().map(Arc::new).collect()
    }

    /// Helper to wrap Vec<Scriptlet> into Vec<Arc<Scriptlet>> for tests
    fn wrap_scriptlets(scriptlets: Vec<Scriptlet>) -> Vec<Arc<Scriptlet>> {
        scriptlets.into_iter().map(Arc::new).collect()
    }

    fn provider_json_test_lock() -> &'static std::sync::Mutex<()> {
        crate::test_utils::PROVIDER_JSON_TEST_LOCK.get_or_init(|| std::sync::Mutex::new(()))
    }

    fn unique_notes_resource_token(prefix: &str) -> String {
        format!("{}_{}", prefix, uuid::Uuid::new_v4().simple())
    }

    // =======================================================
    // TDD Tests - Written FIRST per spec requirements
    // =======================================================

    /// Helper to create a test script
    fn test_script(name: &str, description: Option<&str>) -> Script {
        Script {
            name: name.to_string(),
            path: PathBuf::from(format!(
                "/test/{}.ts",
                name.to_lowercase().replace(' ', "-")
            )),
            extension: "ts".to_string(),
            description: description.map(|s| s.to_string()),
            icon: None,
            alias: None,
            shortcut: None,
            typed_metadata: None,
            schema: None,
            plugin_id: String::new(),
            plugin_title: None,
            kit_name: None,
            body: None,
        }
    }

    /// Helper to create a test scriptlet
    fn test_scriptlet(name: &str, tool: &str, description: Option<&str>) -> Scriptlet {
        Scriptlet {
            icon: None,
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            code: "echo test".to_string(),
            tool: tool.to_string(),
            shortcut: None,
            keyword: None,
            group: None,
            plugin_id: String::new(),
            plugin_title: None,
            file_path: None,
            command: None,
            alias: None,
        }
    }

    #[test]
    fn test_resources_list_includes_all() {
        // REQUIREMENT: resources/list returns the full MCP resource registry.
        let resources = get_resource_definitions();

        assert_eq!(
            resources.len(),
            29,
            "Resource registry count should be updated when new MCP resources land"
        );

        let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&"kit://state"), "Should include kit://state");
        assert!(uris.contains(&"kit://notes"), "Should include kit://notes");
        assert!(uris.contains(&"kit://brain"), "Should include kit://brain");
        assert!(uris.contains(&"kit://audit"), "Should include kit://audit");
        assert!(uris.contains(&"scripts://"), "Should include scripts://");
        assert!(
            uris.contains(&"scriptlets://"),
            "Should include scriptlets://"
        );
        assert!(
            uris.contains(&"kit://transactions/latest"),
            "Should include kit://transactions/latest"
        );
        assert!(
            uris.contains(&"kit://transactions/schema"),
            "Should include kit://transactions/schema"
        );

        // Verify all have required fields
        for resource in &resources {
            assert!(!resource.name.is_empty(), "Resource should have a name");
            assert!(
                resource.mime_type == "application/json"
                    || resource.mime_type == "text/plain"
                    || resource.mime_type == "text/markdown",
                "Should be JSON, text, or markdown mime type, got: {}",
                resource.mime_type
            );
            assert!(resource.description.is_some(), "Should have a description");
        }
    }

    #[test]
    fn brain_resource_description_lists_provenance_reads() {
        let resources = get_resource_definitions();
        let brain = resources
            .iter()
            .find(|resource| resource.uri == "kit://brain")
            .expect("brain resource definition");
        let description = brain.description.as_deref().unwrap_or("");
        assert!(description.contains("format=json"));
        assert!(description.contains("kit://brain/doc"));
        assert!(description.contains("kit://brain/docs"));
    }

    #[test]
    fn test_scripts_resource_read() {
        // REQUIREMENT: scripts:// returns array of script metadata
        let scripts = wrap_scripts(vec![
            test_script("My Script", Some("Does something")),
            test_script("Another Script", None),
        ]);

        let result = read_resource("scripts://", &scripts, &[], None);
        assert!(result.is_ok(), "Should successfully read scripts resource");

        let content = result.unwrap();
        assert_eq!(content.uri, "scripts://");
        assert_eq!(content.mime_type, "application/json");

        // Parse the JSON and verify structure
        let parsed: Vec<ScriptResourceEntry> =
            serde_json::from_str(&content.text).expect("Should be valid JSON array");

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "My Script");
        assert_eq!(parsed[0].description, Some("Does something".to_string()));
        assert_eq!(parsed[1].name, "Another Script");
        assert_eq!(parsed[1].description, None);
    }

    #[test]
    fn test_scriptlets_resource_read() {
        // REQUIREMENT: scriptlets:// returns array of scriptlet metadata
        let scriptlets = wrap_scriptlets(vec![
            test_scriptlet("Open URL", "open", Some("Opens a URL")),
            test_scriptlet("Paste Text", "paste", None),
        ]);

        let result = read_resource("scriptlets://", &[], &scriptlets, None);
        assert!(
            result.is_ok(),
            "Should successfully read scriptlets resource"
        );

        let content = result.unwrap();
        assert_eq!(content.uri, "scriptlets://");
        assert_eq!(content.mime_type, "application/json");

        // Parse the JSON and verify structure
        let parsed: Vec<ScriptletResourceEntry> =
            serde_json::from_str(&content.text).expect("Should be valid JSON array");

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "Open URL");
        assert_eq!(parsed[0].tool, "open");
        assert_eq!(parsed[0].description, Some("Opens a URL".to_string()));
        assert_eq!(parsed[1].name, "Paste Text");
        assert_eq!(parsed[1].tool, "paste");
    }

    #[test]
    fn test_state_resource_read() {
        // REQUIREMENT: kit://state returns current app state
        let app_state = AppStateResource {
            visible: true,
            focused: true,
            script_count: 10,
            scriptlet_count: 5,
            filter_text: Some("test".to_string()),
            selected_index: Some(3),
        };

        let result = read_resource("kit://state", &[], &[], Some(&app_state));
        assert!(result.is_ok(), "Should successfully read state resource");

        let content = result.unwrap();
        assert_eq!(content.uri, "kit://state");
        assert_eq!(content.mime_type, "application/json");

        // Parse and verify
        let parsed: AppStateResource =
            serde_json::from_str(&content.text).expect("Should be valid JSON");

        assert!(parsed.visible);
        assert!(parsed.focused);
        assert_eq!(parsed.script_count, 10);
        assert_eq!(parsed.scriptlet_count, 5);
        assert_eq!(parsed.filter_text, Some("test".to_string()));
        assert_eq!(parsed.selected_index, Some(3));
    }

    #[test]
    fn test_state_resource_read_default() {
        // When no app state is provided, should return defaults
        let result = read_resource("kit://state", &[], &[], None);
        assert!(result.is_ok());

        let content = result.unwrap();
        let parsed: AppStateResource = serde_json::from_str(&content.text).unwrap();

        assert!(!parsed.visible);
        assert!(!parsed.focused);
        assert_eq!(parsed.script_count, 0);
        assert_eq!(parsed.scriptlet_count, 0);
        assert_eq!(parsed.filter_text, None);
        assert_eq!(parsed.selected_index, None);
    }

    #[test]
    fn test_unknown_resource_returns_error() {
        // REQUIREMENT: Unknown URI returns error
        let result = read_resource("unknown://resource", &[], &[], None);

        assert!(result.is_err(), "Unknown resource should return error");
        let error = result.unwrap_err();
        assert!(
            error.contains("Resource not found"),
            "Error should mention resource not found"
        );
        assert!(
            error.contains("unknown://resource"),
            "Error should include the URI"
        );
    }

    #[test]
    fn test_resource_content_to_value() {
        let content = ResourceContent {
            uri: "test://uri".to_string(),
            mime_type: "application/json".to_string(),
            text: r#"{"foo":"bar"}"#.to_string(),
        };

        let value = resource_content_to_value(content);

        // Should have contents array
        let contents = value.get("contents").and_then(|c| c.as_array());
        assert!(contents.is_some());

        let contents = contents.unwrap();
        assert_eq!(contents.len(), 1);

        let first = &contents[0];
        assert_eq!(
            first.get("uri").and_then(|u| u.as_str()),
            Some("test://uri")
        );
        assert_eq!(
            first.get("mimeType").and_then(|m| m.as_str()),
            Some("application/json")
        );
    }

    #[test]
    fn test_resource_list_to_value() {
        let resources = get_resource_definitions();
        let value = resource_list_to_value(&resources);

        // Should have resources array
        let resource_array = value.get("resources").and_then(|r| r.as_array());
        assert!(resource_array.is_some());

        let resource_array = resource_array.unwrap();
        assert_eq!(resource_array.len(), resources.len());

        // First resource should have expected fields
        let first = &resource_array[0];
        assert!(first.get("uri").is_some());
        assert!(first.get("name").is_some());
        assert!(first.get("mimeType").is_some());
    }

    // =======================================================
    // Additional Unit Tests
    // =======================================================

    #[test]
    fn test_script_resource_entry_from_script() {
        use crate::schema_parser::{FieldDef, FieldType, Schema};
        use std::collections::HashMap;

        // Script without schema
        let script_no_schema = test_script("No Schema", Some("Test"));
        let entry: ScriptResourceEntry = (&script_no_schema).into();
        assert!(!entry.has_schema);

        // Script with schema
        let mut input = HashMap::new();
        input.insert(
            "name".to_string(),
            FieldDef {
                field_type: FieldType::String,
                required: true,
                ..Default::default()
            },
        );

        let script_with_schema = Script {
            name: "With Schema".to_string(),
            path: PathBuf::from("/test/with-schema.ts"),
            extension: "ts".to_string(),
            description: None,
            icon: None,
            alias: None,
            shortcut: None,
            typed_metadata: None,
            schema: Some(Schema {
                input,
                output: HashMap::new(),
            }),
            plugin_id: String::new(),
            plugin_title: None,
            kit_name: None,
            body: None,
        };

        let entry: ScriptResourceEntry = (&script_with_schema).into();
        assert!(entry.has_schema);
    }

    #[test]
    fn test_scriptlet_resource_entry_from_scriptlet() {
        let scriptlet = Scriptlet {
            icon: None,
            name: "Full Scriptlet".to_string(),
            description: Some("Test description".to_string()),
            code: "echo test".to_string(),
            tool: "bash".to_string(),
            shortcut: Some("cmd k".to_string()),
            keyword: Some(":test".to_string()),
            group: Some("My Group".to_string()),
            plugin_id: String::new(),
            plugin_title: None,
            file_path: None,
            command: None,
            alias: None,
        };

        let entry: ScriptletResourceEntry = (&scriptlet).into();

        assert_eq!(entry.name, "Full Scriptlet");
        assert_eq!(entry.description, Some("Test description".to_string()));
        assert_eq!(entry.tool, "bash");
        assert_eq!(entry.shortcut, Some("cmd k".to_string()));
        assert_eq!(entry.keyword, Some(":test".to_string()));
        assert_eq!(entry.group, Some("My Group".to_string()));
    }

    #[test]
    fn test_mcp_resource_serialization() {
        let resource = McpResource {
            uri: "test://".to_string(),
            name: "Test".to_string(),
            description: Some("Test description".to_string()),
            mime_type: "application/json".to_string(),
        };

        let json = serde_json::to_string(&resource).unwrap();

        // Should have mimeType (camelCase)
        assert!(json.contains("\"mimeType\""));
        assert!(!json.contains("\"mime_type\""));

        // Deserialize back
        let parsed: McpResource = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.uri, "test://");
        assert_eq!(parsed.mime_type, "application/json");
    }

    #[test]
    fn test_empty_scripts_resource() {
        let result = read_resource("scripts://", &[], &[], None);
        assert!(result.is_ok());

        let content = result.unwrap();
        let parsed: Vec<ScriptResourceEntry> = serde_json::from_str(&content.text).unwrap();
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_empty_scriptlets_resource() {
        let result = read_resource("scriptlets://", &[], &[], None);
        assert!(result.is_ok());

        let content = result.unwrap();
        let parsed: Vec<ScriptletResourceEntry> = serde_json::from_str(&content.text).unwrap();
        assert!(parsed.is_empty());
    }

    // =======================================================
    // Context resource URI parsing tests
    // =======================================================

    #[test]
    fn parse_context_bare_uri_returns_default() {
        let request = parse_context_resource_request("kit://context").unwrap();
        assert_eq!(
            request.options,
            crate::context_snapshot::CaptureContextOptions::default()
        );
        assert_eq!(request.effective_profile, "full");
        assert!(!request.diagnostics);
    }

    #[test]
    fn parse_context_resource_options_supports_minimal_profile() {
        let request = parse_context_resource_request("kit://context?profile=minimal").unwrap();
        assert_eq!(
            request.options,
            crate::context_snapshot::CaptureContextOptions::minimal()
        );
        assert_eq!(request.effective_profile, "minimal");
    }

    #[test]
    fn parse_context_resource_options_allows_profile_overrides() {
        let request = parse_context_resource_request(
            "kit://context?profile=minimal&menuBar=1&selectedText=0",
        )
        .unwrap();

        assert!(!request.options.include_selected_text);
        assert!(request.options.include_menu_bar);
        assert!(request.options.include_frontmost_app);
        assert!(request.options.include_browser_url);
        assert!(request.options.include_focused_window);
        assert_eq!(request.effective_profile, "custom");
    }

    #[test]
    fn parse_context_resource_options_rejects_unknown_flags() {
        let error = parse_context_resource_request("kit://context?nope=1").unwrap_err();
        assert!(
            error.contains("Invalid kit://context parameter: nope"),
            "Error should mention the invalid parameter"
        );
    }

    #[test]
    fn parse_context_rejects_unknown_profile() {
        let error = parse_context_resource_request("kit://context?profile=heavy").unwrap_err();
        assert!(error.contains("Unknown profile"), "Error: {error}");
    }

    #[test]
    fn context_resource_preserves_query_uri() {
        crate::context_snapshot::enable_deterministic_context_capture();
        let content =
            read_resource("kit://context?profile=minimal", &[], &[], None).expect("should resolve");
        assert_eq!(content.uri, "kit://context?profile=minimal");
    }

    #[test]
    fn is_context_resource_uri_only_matches_supported_forms() {
        assert!(is_context_resource_uri("kit://context"));
        assert!(is_context_resource_uri("kit://context?profile=minimal"));
        assert!(is_context_resource_uri("kit://context/schema"));
        assert!(!is_context_resource_uri("kit://contextual"));
        assert!(!is_context_resource_uri("kit://context-schema"));
        assert!(!is_context_resource_uri("unknown://context"));
    }

    // =======================================================
    // Context resource: diagnostics, schema, and self-describing tests
    // =======================================================

    #[test]
    fn parse_context_resource_request_supports_diagnostics_flag() {
        let request =
            parse_context_resource_request("kit://context?profile=minimal&diagnostics=1").unwrap();

        assert!(matches!(request.kind, ContextResourceKind::Snapshot));
        assert_eq!(
            request.options,
            crate::context_snapshot::CaptureContextOptions::minimal()
        );
        assert_eq!(request.effective_profile, "minimal");
        assert!(request.diagnostics);
    }

    #[test]
    fn parse_context_resource_request_marks_profile_override_as_custom() {
        let request =
            parse_context_resource_request("kit://context?profile=minimal&selectedText=1").unwrap();

        assert_eq!(request.effective_profile, "custom");
        assert!(request.options.include_selected_text);
    }

    #[test]
    fn parse_context_resource_request_supports_schema_uri() {
        let request = parse_context_resource_request("kit://context/schema").unwrap();
        assert!(matches!(request.kind, ContextResourceKind::Schema));
    }

    /// Per-field queries inherit their baseline from `all()`, which includes
    /// pixel capture. Pixel data must be explicit opt-in: the `@selection`
    /// attachment URI once inherited `include_screenshot` silently and shipped
    /// a 758KB base64 PNG as prompt text, overflowing the model's context.
    #[test]
    fn parse_context_resource_request_field_overrides_disable_pixels_unless_explicit() {
        let request = parse_context_resource_request(
            "kit://context?selectedText=1&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0",
        )
        .unwrap();
        assert!(request.options.include_selected_text);
        assert!(!request.options.include_screenshot);
        assert!(!request.options.include_panel_screenshot);

        let diagnostics = parse_context_resource_request("kit://context?diagnostics=1").unwrap();
        assert!(!diagnostics.options.include_screenshot);
        assert!(!diagnostics.options.include_panel_screenshot);

        let explicit = parse_context_resource_request(
            "kit://context?screenshot=1&selectedText=0&frontmostApp=0&menuBar=0&browserUrl=0&focusedWindow=0",
        )
        .unwrap();
        assert!(explicit.options.include_screenshot);
        assert!(!explicit.options.include_panel_screenshot);

        // An explicit profile keeps its documented pixel semantics.
        let minimal = parse_context_resource_request("kit://context?profile=minimal").unwrap();
        assert_eq!(
            minimal.options,
            crate::context_snapshot::CaptureContextOptions::minimal()
        );
    }

    #[test]
    fn serialize_context_resource_diagnostics_includes_machine_readable_meta() {
        let request =
            parse_context_resource_request("kit://context?profile=minimal&diagnostics=1").unwrap();

        let snapshot = crate::context_snapshot::AiContextSnapshot {
            schema_version: crate::context_snapshot::AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
            frontmost_app: Some(crate::context_snapshot::FrontmostAppContext {
                pid: 42,
                bundle_id: "com.example.App".to_string(),
                name: "Example App".to_string(),
            }),
            browser: Some(crate::context_snapshot::BrowserContext::from_url(
                "https://example.com".to_string(),
            )),
            warnings: vec!["focusedWindow: permission denied".to_string()],
            ..Default::default()
        };

        let json = serialize_context_resource(
            "kit://context?profile=minimal&diagnostics=1",
            &request,
            Some(&snapshot),
            12,
        )
        .unwrap();

        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["kind"], "context_diagnostics");
        assert_eq!(value["meta"]["effectiveProfile"], "minimal");
        assert_eq!(value["meta"]["status"], "partial");
        assert_eq!(value["meta"]["durationMs"], 12);
        // minimal() enables frontmostApp, browserUrl, focusedWindow, and (since
        // 19db0e0e5, "Enable screenshots in @here (minimal) ... profiles")
        // screenshot — 4 fields total.
        assert_eq!(value["meta"]["enabledFieldCount"], 4);
        assert_eq!(value["meta"]["warningCount"], 1);
        assert_eq!(value["meta"]["fieldStatuses"][0]["field"], "selectedText");
        assert_eq!(value["meta"]["fieldStatuses"][0]["state"], "disabled");
        assert_eq!(value["meta"]["fieldStatuses"][4]["field"], "focusedWindow");
        assert_eq!(value["meta"]["fieldStatuses"][4]["state"], "failed");
        assert_eq!(
            value["meta"]["warnings"][0]["code"],
            "focused_window_capture_failed"
        );
        assert_eq!(value["meta"]["warnings"][0]["message"], "permission denied");
    }

    #[test]
    fn serialize_context_schema_includes_diagnostics_parameter() {
        let request = parse_context_resource_request("kit://context/schema").unwrap();

        let json = serialize_context_resource("kit://context/schema", &request, None, 0).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["kind"], "context_schema");
        assert_eq!(value["diagnosticsSupported"], true);

        let parameter_names: Vec<&str> = value["parameters"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|param| param["name"].as_str())
            .collect();

        assert!(parameter_names.contains(&"diagnostics"));

        let has_diagnostics_example =
            value["examples"].as_array().unwrap().iter().any(|example| {
                example.as_str() == Some("kit://context?profile=minimal&diagnostics=1")
            });

        assert!(has_diagnostics_example);
    }

    // =======================================================
    // Schema-versioned script/scriptlet/sdk-reference resources
    // =======================================================

    #[test]
    fn kit_scripts_resource_returns_schema_versioned_envelope() {
        let scripts = wrap_scripts(vec![
            test_script("Hello World", Some("A greeting script")),
            test_script("Fetch Data", None),
        ]);

        let content = read_resource("kit://scripts", &scripts, &[], None).expect("should resolve");
        assert_eq!(content.uri, "kit://scripts");
        assert_eq!(content.mime_type, "application/json");

        let doc: ScriptsResourceDocument = serde_json::from_str(&content.text).expect("valid JSON");
        assert_eq!(doc.schema_version, SCRIPTS_RESOURCE_SCHEMA_VERSION);
        assert_eq!(doc.count, 2);
        assert_eq!(doc.scripts.len(), 2);
        assert_eq!(doc.scripts[0].name, "Hello World");
        assert_eq!(
            doc.scripts[0].description,
            Some("A greeting script".to_string())
        );
    }

    #[test]
    fn kit_scripts_resource_empty_returns_zero_count() {
        let content = read_resource("kit://scripts", &[], &[], None).expect("should resolve");
        let doc: ScriptsResourceDocument = serde_json::from_str(&content.text).expect("valid JSON");
        assert_eq!(doc.schema_version, SCRIPTS_RESOURCE_SCHEMA_VERSION);
        assert_eq!(doc.count, 0);
        assert!(doc.scripts.is_empty());
    }

    #[test]
    fn kit_scriptlets_resource_returns_schema_versioned_envelope() {
        let scriptlets = wrap_scriptlets(vec![
            test_scriptlet("Open URL", "open", Some("Opens a URL")),
            test_scriptlet("Paste Text", "paste", None),
        ]);

        let content =
            read_resource("kit://scriptlets", &[], &scriptlets, None).expect("should resolve");
        assert_eq!(content.uri, "kit://scriptlets");

        let doc: ScriptletsResourceDocument =
            serde_json::from_str(&content.text).expect("valid JSON");
        assert_eq!(doc.schema_version, SCRIPTLETS_RESOURCE_SCHEMA_VERSION);
        assert_eq!(doc.count, 2);
        assert_eq!(doc.scriptlets.len(), 2);
        assert_eq!(doc.scriptlets[0].name, "Open URL");
        assert_eq!(doc.scriptlets[0].tool, "open");
    }

    #[test]
    fn kit_scriptlets_resource_empty_returns_zero_count() {
        let content = read_resource("kit://scriptlets", &[], &[], None).expect("should resolve");
        let doc: ScriptletsResourceDocument =
            serde_json::from_str(&content.text).expect("valid JSON");
        assert_eq!(doc.count, 0);
        assert!(doc.scriptlets.is_empty());
    }

    #[test]
    fn sdk_reference_resource_returns_valid_document() {
        let content = read_resource("kit://sdk-reference", &[], &[], None).expect("should resolve");
        assert_eq!(content.uri, "kit://sdk-reference");

        let doc: SdkReferenceDocument = serde_json::from_str(&content.text).expect("valid JSON");
        assert_eq!(doc.schema_version, SDK_REFERENCE_SCHEMA_VERSION);
        assert_eq!(doc.sdk_package, "@scriptkit/sdk");
        assert!(!doc.functions.is_empty());

        // Verify key functions are present
        let names: Vec<&str> = doc.functions.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"arg"), "should include arg()");
        assert!(names.contains(&"div"), "should include div()");
        assert!(names.contains(&"exec"), "should include exec()");
        assert!(names.contains(&"copy"), "should include copy()");
    }

    #[test]
    fn sdk_reference_has_categories() {
        let doc = build_sdk_reference_document();
        let categories: Vec<&str> = doc
            .functions
            .iter()
            .map(|f| f.category.as_str())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        assert!(categories.contains(&"prompts"));
        assert!(categories.contains(&"system"));
        assert!(categories.contains(&"clipboard"));
        assert!(categories.contains(&"filesystem"));
    }

    #[test]
    fn kit_scripts_resource_json_uses_camel_case() {
        let scripts = wrap_scripts(vec![test_script("Test", None)]);
        let content = read_resource("kit://scripts", &scripts, &[], None).unwrap();
        assert!(content.text.contains("\"schemaVersion\""));
        assert!(!content.text.contains("\"schema_version\""));
    }

    #[test]
    fn resource_definitions_include_new_resources() {
        let resources = get_resource_definitions();
        let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&"kit://scripts"));
        assert!(uris.contains(&"kit://scriptlets"));
        assert!(uris.contains(&"kit://sdk-reference"));
    }

    #[test]
    fn sdk_reference_includes_metadata_format() {
        let doc = build_sdk_reference_document();
        assert!(doc.metadata_format.contains("export const metadata"));
        assert!(doc.script_directory.contains("plugins/main/scripts"));
        assert!(doc.scriptlet_pattern.contains("scriptlets"));
    }

    #[test]
    fn sdk_reference_roundtrips_through_json() {
        let doc = build_sdk_reference_document();
        let json = serde_json::to_string(&doc).expect("serialize");
        let parsed: SdkReferenceDocument = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(doc, parsed);
    }

    // =======================================================
    // kit://failed-scripts resource tests
    // =======================================================

    #[test]
    fn failed_scripts_resource_is_listed() {
        let resources = get_resource_definitions();
        let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(
            uris.contains(&FAILED_SCRIPTS_RESOURCE_URI),
            "{FAILED_SCRIPTS_RESOURCE_URI} should be in resource definitions"
        );
    }

    #[test]
    fn failed_scripts_resource_lists_validation_failures() {
        use crate::scripts::{
            BindingKind, FailedScript, MetadataField, RelatedScript, ScriptValidationIssue,
            ScriptValidationKind, ValidationReport, ValidationSeverity, VALIDATION_SCHEMA_VERSION,
        };
        use std::path::PathBuf;

        // Two scripts colliding on `cmd k` — mirrors what `validate_script_catalog`
        // would emit for real duplicate-shortcut metadata on disk.
        let issue_for = |path: &str, peer: &str| ScriptValidationIssue {
            severity: ValidationSeverity::Fatal,
            path: PathBuf::from(path),
            script_name: path.into(),
            field: Some(MetadataField::Shortcut),
            message: "Shortcut `cmd k` is declared by 2 scripts".into(),
            kind: ScriptValidationKind::DuplicateBinding {
                binding: BindingKind::Shortcut,
                value: "cmd k".into(),
            },
            related: vec![RelatedScript {
                path: PathBuf::from(peer),
                name: peer.into(),
            }],
        };
        let failed = vec![
            FailedScript {
                path: PathBuf::from("/tmp/a.ts"),
                name: "a".into(),
                fatal: Arc::from(vec![issue_for("/tmp/a.ts", "/tmp/b.ts")]),
            },
            FailedScript {
                path: PathBuf::from("/tmp/b.ts"),
                name: "b".into(),
                fatal: Arc::from(vec![issue_for("/tmp/b.ts", "/tmp/a.ts")]),
            },
        ];
        let report = ValidationReport {
            schema_version: VALIDATION_SCHEMA_VERSION,
            total_candidates: 2,
            valid_count: 0,
            fatal_count: 2,
            warning_count: 0,
            failed_scripts: Arc::from(failed),
            warnings: Arc::from(Vec::<ScriptValidationIssue>::new()),
        };

        let doc = build_failed_scripts_document(&report);
        assert_eq!(doc.schema_version, FAILED_SCRIPTS_RESOURCE_SCHEMA_VERSION);
        assert_eq!(doc.validation_schema_version, VALIDATION_SCHEMA_VERSION);
        assert_eq!(doc.total_candidates, 2);
        assert_eq!(doc.valid_count, 0);
        assert_eq!(doc.fatal_count, 2);
        assert_eq!(doc.failed_scripts.len(), 2);

        // Each failure must name its peer so the author can repair both sides.
        for entry in &doc.failed_scripts {
            assert_eq!(entry.fatal.len(), 1);
            assert_eq!(entry.fatal[0].related.len(), 1);
            assert!(matches!(
                entry.fatal[0].kind,
                ScriptValidationKind::DuplicateBinding {
                    binding: BindingKind::Shortcut,
                    ..
                }
            ));
        }

        let json = serde_json::to_string(&doc).expect("serialize");
        assert!(json.contains("\"schemaVersion\""));
        assert!(!json.contains("\"schema_version\""));
        assert!(json.contains("\"duplicateBinding\""));
        let parsed: FailedScriptsResourceDocument =
            serde_json::from_str(&json).expect("round-trip");
        assert_eq!(parsed.failed_scripts.len(), 2);
    }

    #[test]
    fn failed_scripts_resource_empty_report_serializes_cleanly() {
        use crate::scripts::{ValidationReport, VALIDATION_SCHEMA_VERSION};

        let report = ValidationReport {
            schema_version: VALIDATION_SCHEMA_VERSION,
            total_candidates: 0,
            valid_count: 0,
            fatal_count: 0,
            warning_count: 0,
            failed_scripts: Arc::from(Vec::new()),
            warnings: Arc::from(Vec::new()),
        };
        let doc = build_failed_scripts_document(&report);
        assert_eq!(doc.fatal_count, 0);
        assert!(doc.failed_scripts.is_empty());
        assert!(doc.warnings.is_empty());

        let json = serde_json::to_string(&doc).expect("serialize");
        let parsed: FailedScriptsResourceDocument =
            serde_json::from_str(&json).expect("round-trip");
        assert_eq!(
            parsed.schema_version,
            FAILED_SCRIPTS_RESOURCE_SCHEMA_VERSION
        );
    }

    #[test]
    fn failed_scripts_resource_read_returns_parseable_envelope() {
        // End-to-end: resolves the URI through `read_resource` which calls
        // `read_scripts_report()` internally. Machine state may be non-empty,
        // so assert envelope shape, not failure count.
        let content = read_resource(FAILED_SCRIPTS_RESOURCE_URI, &[], &[], None)
            .expect("resource should resolve");
        assert_eq!(content.uri, FAILED_SCRIPTS_RESOURCE_URI);
        assert_eq!(content.mime_type, "application/json");

        let doc: FailedScriptsResourceDocument =
            serde_json::from_str(&content.text).expect("valid envelope JSON");
        assert_eq!(doc.schema_version, FAILED_SCRIPTS_RESOURCE_SCHEMA_VERSION);
        // If any script failed, its fatal-issue total must be at least as large
        // as the distinct failed-script count (each failed script has ≥1 issue).
        assert!(doc.fatal_count >= doc.failed_scripts.len());
    }

    #[test]
    fn parse_context_request_accepts_panel_screenshot_flag() {
        let request = parse_context_resource_request(
            "kit://context?screenshot=1&panelScreenshot=1&diagnostics=1",
        )
        .expect("request");
        assert!(request.options.include_screenshot);
        assert!(request.options.include_panel_screenshot);
        assert!(request.diagnostics);
    }

    #[test]
    fn diagnostics_surface_reports_panel_screenshot_state() {
        let request =
            parse_context_resource_request("kit://context?panelScreenshot=1&diagnostics=1")
                .expect("request");

        let snapshot = crate::context_snapshot::AiContextSnapshot {
            schema_version: crate::context_snapshot::AI_CONTEXT_SNAPSHOT_SCHEMA_VERSION,
            script_kit_panel_image: Some(crate::context_snapshot::Base64PngContext {
                mime_type: "image/png".to_string(),
                width: 700,
                height: 520,
                base64_data: "cGFuZWw=".to_string(),
                title: Some("Script Kit - Clipboard History".to_string()),
            }),
            ..Default::default()
        };

        let doc = build_context_diagnostics_document(
            "kit://context?panelScreenshot=1&diagnostics=1",
            &request,
            &snapshot,
            1,
        );
        assert!(doc
            .meta
            .field_statuses
            .iter()
            .any(|field| field.field == "panelScreenshot"
                && field.enabled
                && field.present
                && matches!(field.state, ContextFieldCaptureState::Captured)));
    }

    #[test]
    fn schema_document_includes_panel_screenshot_parameter() {
        let schema = build_context_schema_document();
        assert!(
            schema
                .parameters
                .iter()
                .any(|p| p.name == "panelScreenshot"),
            "schema must list panelScreenshot parameter"
        );
    }

    // =======================================================
    // Clipboard history resource tests
    // =======================================================

    #[test]
    fn clipboard_history_resource_is_listed() {
        let resources = get_resource_definitions();
        assert!(
            resources.iter().any(|r| r.uri == "kit://clipboard-history"),
            "kit://clipboard-history should be in resource definitions"
        );
    }

    #[test]
    fn clipboard_history_resource_resolves_with_valid_schema() {
        let content =
            read_resource("kit://clipboard-history", &[], &[], None).expect("should resolve");
        assert_eq!(content.uri, "kit://clipboard-history");
        assert_eq!(content.mime_type, "application/json");

        let doc: ClipboardHistoryDocument =
            serde_json::from_str(&content.text).expect("valid JSON");
        assert_eq!(
            doc.schema_version,
            CLIPBOARD_HISTORY_RESOURCE_SCHEMA_VERSION
        );
        assert_eq!(doc.count, doc.entries.len());
    }

    #[test]
    fn clipboard_history_parse_accepts_limit_param() {
        let req = parse_clipboard_history_request("kit://clipboard-history?limit=5").unwrap();
        match req {
            ClipboardHistoryRequest::List { limit, diagnostics } => {
                assert_eq!(limit, 5);
                assert!(!diagnostics);
            }
            other => panic!("Expected List, got {other:?}"),
        }
    }

    #[test]
    fn clipboard_history_parse_clamps_limit_to_max() {
        let req = parse_clipboard_history_request("kit://clipboard-history?limit=999").unwrap();
        match req {
            ClipboardHistoryRequest::List { limit, .. } => {
                assert_eq!(limit, CLIPBOARD_HISTORY_MAX_LIMIT);
            }
            other => panic!("Expected List, got {other:?}"),
        }
    }

    #[test]
    fn clipboard_history_parse_rejects_unknown_param() {
        let err = parse_clipboard_history_request("kit://clipboard-history?foo=1").unwrap_err();
        assert!(err.contains("Invalid kit://clipboard-history parameter"));
    }

    #[test]
    fn clipboard_history_parse_accepts_id_param() {
        let req = parse_clipboard_history_request("kit://clipboard-history?id=abc123").unwrap();
        match req {
            ClipboardHistoryRequest::SingleEntry { id } => {
                assert_eq!(id, "abc123");
            }
            other => panic!("Expected SingleEntry, got {other:?}"),
        }
    }

    #[test]
    fn clipboard_history_diagnostics_returns_wrapper() {
        let content = read_resource("kit://clipboard-history?diagnostics=1", &[], &[], None)
            .expect("should resolve");

        let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");
        assert_eq!(value["kind"], "clipboard_history_diagnostics");
        assert_eq!(
            value["document"]["schemaVersion"],
            CLIPBOARD_HISTORY_RESOURCE_SCHEMA_VERSION
        );
        assert_eq!(value["meta"]["source"], "cached_entries");
    }

    #[test]
    fn clipboard_history_entry_serialization_roundtrip() {
        let entry = ClipboardHistoryEntry {
            id: "abc-123".to_string(),
            content_type: "text".to_string(),
            timestamp: 1711700000,
            text_preview: Some("Hello world".to_string()),
            ocr_text: None,
            image_width: None,
            image_height: None,
            pinned: false,
        };
        let json = serde_json::to_string(&entry).expect("serialize");
        let parsed: ClipboardHistoryEntry = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(entry, parsed);
    }

    // =======================================================
    // Focused item resource tests
    // =======================================================

    #[test]
    fn focused_item_resource_is_listed() {
        let resources = get_resource_definitions();
        assert!(
            resources.iter().any(|r| r.uri == "kit://focused-item"),
            "kit://focused-item should be in resource definitions"
        );
    }

    #[test]
    fn focused_item_resource_returns_empty_when_no_slot() {
        // Ensure slot is clear
        clear_focused_item();

        let content = read_resource("kit://focused-item", &[], &[], None).expect("should resolve");
        assert_eq!(content.uri, "kit://focused-item");

        let doc: FocusedItemDocument = serde_json::from_str(&content.text).expect("valid JSON");
        assert_eq!(doc.schema_version, FOCUSED_ITEM_RESOURCE_SCHEMA_VERSION);
        assert!(!doc.has_focused_item);
        assert!(doc.focused_item.is_none());
        assert!(
            !doc.warnings.is_empty(),
            "should have a warning when no item"
        );
    }

    #[test]
    fn focused_item_resource_returns_published_item() {
        publish_focused_item(FocusedItemInfo {
            source: "ClipboardHistory".to_string(),
            kind: "clipboard_entry".to_string(),
            semantic_id: "choice:0:hello".to_string(),
            label: "hello world".to_string(),
            metadata: Some(serde_json::json!({"contentType": "text"})),
        });

        let content = read_resource("kit://focused-item", &[], &[], None).expect("should resolve");

        let doc: FocusedItemDocument = serde_json::from_str(&content.text).expect("valid JSON");
        assert!(doc.has_focused_item);
        let item = doc.focused_item.expect("item present");
        assert_eq!(item.source, "ClipboardHistory");
        assert_eq!(item.semantic_id, "choice:0:hello");
        assert!(doc.warnings.is_empty());

        // Clean up
        clear_focused_item();
    }

    #[test]
    fn focused_item_parse_rejects_unknown_param() {
        let err = parse_focused_item_request("kit://focused-item?foo=1").unwrap_err();
        assert!(err.contains("Invalid kit://focused-item parameter"));
    }

    #[test]
    fn focused_item_diagnostics_returns_wrapper() {
        clear_focused_item();

        let content = read_resource("kit://focused-item?diagnostics=1", &[], &[], None)
            .expect("should resolve");

        let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");
        assert_eq!(value["kind"], "focused_item_diagnostics");
        assert_eq!(
            value["document"]["schemaVersion"],
            FOCUSED_ITEM_RESOURCE_SCHEMA_VERSION
        );
        assert_eq!(value["meta"]["source"], "focused_item_slot");
        assert_eq!(value["meta"]["hasFocusedItem"], false);
        assert!(value["meta"]["warningCount"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn focused_item_info_serialization_roundtrip() {
        let item = FocusedItemInfo {
            source: "FileSearch".to_string(),
            kind: "file".to_string(),
            semantic_id: "choice:2:readme".to_string(),
            label: "README.md".to_string(),
            metadata: Some(serde_json::json!({"path": "/tmp/README.md"})),
        };
        let json = serde_json::to_string(&item).expect("serialize");
        let parsed: FocusedItemInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(item, parsed);
    }

    #[test]
    fn test_notes_list_resource_full_param_returns_full_content() {
        let _guard = provider_json_test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        crate::notes::init_notes_db().expect("notes db should initialize before resource test");
        let token = unique_notes_resource_token("resource_full");
        let body: String = format!(
            "---\ntags: [{token}]\n---\n# Full Body\n{}",
            "x".repeat(600)
        );
        let note = crate::notes::Note::with_content(body.clone());
        let note_id = note.id;
        crate::notes::save_note(&note).expect("failed to save notes full-content test note");

        let content = read_notes_list_resource(&format!("kit://notes?tag={token}&full=true"))
            .expect("full-content notes resource should resolve");
        let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");
        let notes = value["notes"].as_array().expect("notes array");
        let entry = notes
            .iter()
            .find(|candidate| candidate["id"] == note_id.as_str())
            .expect("created note should be returned by full-content resource");
        let entry = entry.clone();

        crate::notes::delete_note_permanently(note_id)
            .expect("cleanup failed for notes full-content test");

        assert_eq!(
            entry["content"].as_str().expect("content string"),
            body,
            "full=true should return the complete note body, not a preview"
        );
        assert!(entry.get("preview").is_none(), "full entries drop preview");
        assert_eq!(entry["contentTruncated"], serde_json::Value::Bool(false));
    }

    #[test]
    fn test_notes_list_resource_can_filter_and_report_metadata() {
        let _guard = provider_json_test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        crate::notes::init_notes_db().expect("notes db should initialize before resource test");
        let token = unique_notes_resource_token("resource_tag");
        let note = crate::notes::Note::with_content(format!(
            "---\ntags: [{token}]\naliases: [{token} Alias]\n---\n# Resource Metadata\nBody [[{token} Target]]"
        ));
        let note_id = note.id;
        crate::notes::save_note(&note).expect("failed to save notes resource test note");

        let content = read_notes_list_resource(&format!("kit://notes?tag={token}&limit=10"))
            .expect("tag-filtered notes resource should resolve");
        let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");
        let notes = value["notes"].as_array().expect("notes array");
        let summary = notes
            .iter()
            .find(|candidate| candidate["id"] == note_id.as_str())
            .expect("created note should be returned by tag-filtered resource");

        crate::notes::delete_note_permanently(note_id)
            .expect("cleanup failed for notes resource metadata test");

        assert_eq!(value["query"], format!("tag:{token}"));
        assert!(
            summary["metadata"]["tags"]
                .as_array()
                .expect("tags array")
                .iter()
                .any(|tag| tag == token.as_str()),
            "summary metadata should include indexed tags"
        );
        assert!(
            summary["metadata"]["aliases"]
                .as_array()
                .expect("aliases array")
                .iter()
                .any(|alias| alias == format!("{token} Alias").as_str()),
            "summary metadata should include indexed aliases"
        );
        assert_eq!(summary["metadata"]["outboundLinkCount"], 1);
    }

    #[test]
    fn test_single_note_resource_reports_metadata() {
        let _guard = provider_json_test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        crate::notes::init_notes_db().expect("notes db should initialize before resource test");
        let token = unique_notes_resource_token("single_resource");
        let note = crate::notes::Note::with_content(format!(
            "---\ntags: [{token}]\naliases: [{token} Alias]\n---\n# Single Resource\nBody [[{token} Target]]"
        ));
        let note_id = note.id;
        crate::notes::save_note(&note).expect("failed to save single notes resource test note");

        let content = read_single_note_resource(&format!("kit://notes/{note_id}"))
            .expect("single notes resource should resolve");
        let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");

        crate::notes::delete_note_permanently(note_id)
            .expect("cleanup failed for single notes resource metadata test");

        assert_eq!(value["note"]["id"], note_id.as_str());
        assert!(
            value["metadata"]["tags"]
                .as_array()
                .expect("tags array")
                .iter()
                .any(|tag| tag == token.as_str()),
            "single note metadata should include indexed tags"
        );
        assert!(
            value["metadata"]["aliases"]
                .as_array()
                .expect("aliases array")
                .iter()
                .any(|alias| alias == format!("{token} Alias").as_str()),
            "single note metadata should include indexed aliases"
        );
        assert_eq!(value["metadata"]["outboundLinkCount"], 1);
    }

    #[test]
    fn test_notes_resource_query_params_are_url_decoded() {
        assert_eq!(
            query_string_param("kit://notes?q=project%20plan", "q"),
            Some("project plan".to_string())
        );
        assert_eq!(
            query_string_param("kit://notes?alias=Project+Plan", "alias"),
            Some("Project Plan".to_string())
        );
        assert_eq!(
            notes_list_search_query("kit://notes?tag=projects%2Fscript-kit"),
            Some("tag:projects/script-kit".to_string())
        );
    }

    #[test]
    fn test_notes_list_resource_filters_alias_link_q_and_plus_decoding() {
        let _guard = provider_json_test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        crate::notes::init_notes_db().expect("notes db should initialize before resource test");
        let token = unique_notes_resource_token("resource_query");
        let alias = format!("{token} Project Plan");
        let target_title = format!("{token} Target Note");
        let body_token = format!("{token}_body");
        let note = crate::notes::Note::with_content(format!(
            "---\naliases: [{alias}]\n---\n# Resource Query\n{body_token} links to [[{target_title}]]"
        ));
        let note_id = note.id;
        crate::notes::save_note(&note).expect("failed to save notes resource query test note");

        let alias_uri = format!("kit://notes?alias={}&limit=10", alias.replace(' ', "+"));
        let link_uri = format!(
            "kit://notes?link={}&limit=10",
            target_title.replace(' ', "+")
        );
        let text_uri = format!("kit://notes?q={body_token}&limit=10");
        let alias_content =
            read_notes_list_resource(&alias_uri).expect("alias-filtered notes should resolve");
        let link_content =
            read_notes_list_resource(&link_uri).expect("link-filtered notes should resolve");
        let text_content =
            read_notes_list_resource(&text_uri).expect("text-filtered notes should resolve");

        crate::notes::delete_note_permanently(note_id)
            .expect("cleanup failed for notes resource query test");

        for (label, content) in [
            ("alias", alias_content),
            ("link", link_content),
            ("q", text_content),
        ] {
            let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");
            let notes = value["notes"].as_array().expect("notes array");
            assert!(
                notes
                    .iter()
                    .any(|candidate| candidate["id"] == note_id.as_str()),
                "{label} resource filter should return the created note"
            );
        }
    }

    // ── Provider-backed JSON resource tests ───────────────────────

    #[test]
    fn dictation_resource_empty_fallback_has_explicit_envelope() {
        let _guard = provider_json_test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_provider_json_slots();
        std::env::remove_var("SCRIPT_KIT_DICTATION_JSON");

        let scripts = Vec::new();
        let scriptlets = Vec::new();
        let content =
            read_resource("kit://dictation", &scripts, &scriptlets, None).expect("should read");
        let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");

        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["type"], "dictation");
        assert_eq!(value["ok"], true);
        assert_eq!(value["available"], false);
        assert_eq!(value["source"], "empty-fallback");
        assert!(value["items"].is_array());
        assert!(value["note"].is_string());
        assert!(value["nextStep"].is_string());
    }

    #[test]
    fn calendar_resource_empty_fallback_has_explicit_envelope() {
        let _guard = provider_json_test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_provider_json_slots();
        std::env::remove_var("SCRIPT_KIT_CALENDAR_JSON");

        let scripts = Vec::new();
        let scriptlets = Vec::new();
        let content =
            read_resource("kit://calendar", &scripts, &scriptlets, None).expect("should read");
        let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");

        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["type"], "calendar");
        assert_eq!(value["ok"], true);
        assert_eq!(value["available"], false);
        assert_eq!(value["source"], "empty-fallback");
        assert!(value["items"].is_array());
        assert!(value["note"].is_string());
        assert!(value["nextStep"].is_string());
    }

    #[test]
    fn notifications_resource_empty_fallback_has_explicit_envelope() {
        let _guard = provider_json_test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_provider_json_slots();
        std::env::remove_var("SCRIPT_KIT_NOTIFICATIONS_JSON");

        let scripts = Vec::new();
        let scriptlets = Vec::new();
        let content =
            read_resource("kit://notifications", &scripts, &scriptlets, None).expect("should read");
        let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");

        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["type"], "notifications");
        assert_eq!(value["ok"], true);
        assert_eq!(value["available"], false);
        assert_eq!(value["source"], "empty-fallback");
        assert!(value["items"].is_array());
        assert!(value["note"].is_string());
        assert!(value["nextStep"].is_string());
    }

    #[test]
    fn dictation_resource_prefers_slot_data() {
        let _guard = provider_json_test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_provider_json_slots();
        publish_dictation_json(
            r#"{"schemaVersion":1,"type":"dictation","ok":true,"available":true,"source":"slot","items":[{"text":"hello"}]}"#,
        );

        let scripts = Vec::new();
        let scriptlets = Vec::new();
        let content =
            read_resource("kit://dictation", &scripts, &scriptlets, None).expect("should read");
        let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");

        assert_eq!(value["available"], true);
        assert_eq!(value["source"], "slot");
        assert_eq!(value["items"].as_array().expect("items array").len(), 1);

        clear_provider_json_slots();
    }

    #[test]
    fn calendar_resource_prefers_slot_data() {
        let _guard = provider_json_test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_provider_json_slots();
        publish_calendar_json(
            r#"{"schemaVersion":1,"type":"calendar","ok":true,"available":true,"source":"slot","items":[{"title":"Demo"}]}"#,
        );

        let scripts = Vec::new();
        let scriptlets = Vec::new();
        let content =
            read_resource("kit://calendar", &scripts, &scriptlets, None).expect("should read");
        let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");

        assert_eq!(value["available"], true);
        assert_eq!(value["source"], "slot");
        assert_eq!(value["items"].as_array().expect("items array").len(), 1);

        clear_provider_json_slots();
    }

    #[test]
    fn notifications_resource_prefers_slot_data() {
        let _guard = provider_json_test_lock()
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        clear_provider_json_slots();
        publish_notifications_json(
            r#"{"schemaVersion":1,"type":"notifications","ok":true,"available":true,"source":"slot","items":[{"title":"Build complete"}]}"#,
        );

        let scripts = Vec::new();
        let scriptlets = Vec::new();
        let content =
            read_resource("kit://notifications", &scripts, &scriptlets, None).expect("should read");
        let value: serde_json::Value = serde_json::from_str(&content.text).expect("valid JSON");

        assert_eq!(value["available"], true);
        assert_eq!(value["source"], "slot");
        assert_eq!(value["items"].as_array().expect("items array").len(), 1);

        clear_provider_json_slots();
    }

    fn sdk_ref(name: &str, signature: &str, description: &str, category: &str) -> SdkFunctionRef {
        SdkFunctionRef::supported(name, signature, description, category)
    }

    #[test]
    fn filter_sdk_reference_entries_empty_filter_returns_all_indices() {
        let entries = vec![
            sdk_ref("arg", "arg(prompt)", "Prompt user", "input"),
            sdk_ref("div", "div(html)", "Render HTML", "output"),
        ];
        let indices = filter_sdk_reference_entries(&entries, "");
        assert_eq!(indices, vec![0, 1]);
    }

    #[test]
    fn filter_sdk_reference_entries_whitespace_filter_returns_all_indices() {
        let entries = vec![sdk_ref("arg", "arg(p)", "Prompt", "input")];
        let indices = filter_sdk_reference_entries(&entries, "   ");
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn filter_sdk_reference_entries_matches_case_insensitively_across_fields() {
        let entries = vec![
            sdk_ref("arg", "arg(prompt)", "Prompts the user", "input"),
            sdk_ref("div", "div(html)", "Renders HTML content", "output"),
            sdk_ref("path", "path(opts)", "File picker", "input"),
        ];
        assert_eq!(filter_sdk_reference_entries(&entries, "INPUT"), vec![0, 2]);
        assert_eq!(filter_sdk_reference_entries(&entries, "html"), vec![1]);
        assert_eq!(filter_sdk_reference_entries(&entries, "picker"), vec![2]);
        assert_eq!(
            filter_sdk_reference_entries(&entries, "no-such-thing"),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn format_sdk_reference_entry_markdown_contains_all_fields() {
        let entry = sdk_ref(
            "arg",
            "arg(prompt: string)",
            "Prompts the user for input",
            "input",
        );
        let md = format_sdk_reference_entry_markdown(&entry);
        assert!(md.contains("# arg"), "missing heading: {md}");
        assert!(
            md.contains("`arg(prompt: string)`"),
            "missing signature: {md}"
        );
        assert!(md.contains("_input_"), "missing category: {md}");
        assert!(
            md.contains("Prompts the user for input"),
            "missing description: {md}"
        );
    }

    #[test]
    fn sdk_support_serde_roundtrips_lowercase() {
        // Pins the wire shape: lowercase strings, not PascalCase.
        let supported = serde_json::to_string(&SdkSupport::Supported).expect("serialize");
        let unsupported = serde_json::to_string(&SdkSupport::Unsupported).expect("serialize");
        let experimental = serde_json::to_string(&SdkSupport::Experimental).expect("serialize");
        assert_eq!(supported, "\"supported\"");
        assert_eq!(unsupported, "\"unsupported\"");
        assert_eq!(experimental, "\"experimental\"");

        for raw in [&supported, &unsupported, &experimental] {
            let parsed: SdkSupport = serde_json::from_str(raw).expect("deserialize");
            let again = serde_json::to_string(&parsed).expect("re-serialize");
            assert_eq!(&again, raw, "round-trip mismatch for {raw}");
        }
    }

    #[test]
    fn sdk_function_ref_deserializes_old_shape_as_supported() {
        // Pins backward compatibility: older JSON without `support` still
        // parses, defaulting to Supported with no note.
        let json = r#"{
            "name": "arg",
            "signature": "arg(prompt)",
            "description": "Prompt",
            "category": "prompts"
        }"#;
        let parsed: SdkFunctionRef = serde_json::from_str(json).expect("legacy shape must parse");
        assert_eq!(parsed.support, SdkSupport::Supported);
        assert!(parsed.unsupported_note.is_none());
    }

    #[test]
    fn sdk_function_ref_always_serializes_support_field() {
        // Agents should not have to infer support from field absence.
        let entry = SdkFunctionRef::supported("arg", "arg(p)", "Prompt", "prompts");
        let json = serde_json::to_string(&entry).expect("serialize");
        assert!(
            json.contains("\"support\":\"supported\""),
            "support field must be serialized for Supported entries: {json}"
        );
        assert!(
            !json.contains("unsupportedNote"),
            "Option::None should not emit unsupportedNote: {json}"
        );
    }

    #[test]
    fn sdk_reference_marks_notify_as_supported_system_notification_api() {
        // Pins the user's correction: notify() is intentional OS-level
        // feedback (macOS Notification Center via notify-rust), distinct
        // from hud(message) which is in-launcher. Both must coexist, and
        // kit://sdk-reference must not treat notify() as a dead end.
        let doc = build_sdk_reference_document();
        let notify = doc
            .functions
            .iter()
            .find(|entry| entry.name == "notify")
            .expect("notify must appear in the SDK reference");
        assert_eq!(notify.support, SdkSupport::Supported);
        assert!(
            notify.unsupported_note.is_none(),
            "notify is Supported; it must not carry an unsupported_note"
        );
        let description = notify.description.as_str();
        assert!(
            description.to_lowercase().contains("system notification")
                || description.to_lowercase().contains("notification center"),
            "notify description must advertise it as an OS-level notification API: {description}"
        );
        assert!(
            description.contains("hud"),
            "notify description must contrast itself with hud(message) so readers can pick the right API: {description}"
        );
    }

    #[test]
    fn sdk_reference_marks_every_documented_unsupported_api() {
        // Pins the expanded/contracted list safely: every name in the
        // SDK_NOT_YET_IMPLEMENTED_IN_GPUI inventory that also appears in the
        // reference MUST be labeled Unsupported with a note. Names NOT in the
        // reference are skipped deliberately (this PR does not expand the
        // reference inventory).
        let doc = build_sdk_reference_document();
        for unsupported_name in SDK_NOT_YET_IMPLEMENTED_IN_GPUI {
            if let Some(entry) = doc
                .functions
                .iter()
                .find(|entry| entry.name == *unsupported_name)
            {
                assert_eq!(
                    entry.support,
                    SdkSupport::Unsupported,
                    "`{unsupported_name}` appears in kit-sdk.ts's 'not yet implemented' inventory but is marked Supported in the SDK reference"
                );
                assert!(
                    entry.unsupported_note.is_some(),
                    "`{unsupported_name}` must carry an unsupported_note explaining the status"
                );
            }
        }
    }

    #[test]
    fn sdk_reference_marks_find_as_unsupported_prompt_gap() {
        let doc = build_sdk_reference_document();
        let find = doc
            .functions
            .iter()
            .find(|entry| entry.name == "find")
            .expect("find must appear in the SDK reference");
        assert_eq!(find.support, SdkSupport::Unsupported);
        let note = find
            .unsupported_note
            .as_deref()
            .expect("find must explain its unsupported GPUI boundary");
        assert!(
            note.contains("fileSearch") && note.contains("onlyin"),
            "find unsupported note must point users to the supported onlyin-capable fileSearch API: {note}"
        );
        assert!(
            find.description
                .to_lowercase()
                .contains("does not currently implement"),
            "find description must not imply a working GPUI prompt: {}",
            find.description
        );
    }

    #[test]
    fn filter_sdk_reference_entries_includes_unsupported_results() {
        // Pins: unsupported entries stay discoverable. Filtering does NOT
        // skip them — the label is the only thing that changes.
        let entries = vec![
            sdk_ref("arg", "arg(prompt)", "Prompt user", "prompts"),
            SdkFunctionRef::unsupported(
                "notify",
                "notify(message)",
                "Show notification",
                "feedback",
                "Use hud(...) in GPUI today.",
            ),
        ];
        assert_eq!(filter_sdk_reference_entries(&entries, "notify"), vec![1]);
        assert_eq!(
            filter_sdk_reference_entries(&entries, "hud"),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn format_sdk_reference_entry_markdown_warns_for_unsupported() {
        let entry = SdkFunctionRef::unsupported(
            "notify",
            "notify(message)",
            "Show notification",
            "feedback",
            "Use hud(message) instead.",
        );
        let md = format_sdk_reference_entry_markdown(&entry);
        assert!(
            md.starts_with("> ⚠ Unsupported in GPUI"),
            "unsupported entry markdown must lead with a blockquote warning: {md}"
        );
        assert!(
            md.contains("Use hud(message) instead."),
            "unsupported entry markdown must surface the note: {md}"
        );
        // Body sections still present.
        assert!(md.contains("# notify"), "missing heading: {md}");
        assert!(md.contains("`notify(message)`"), "missing signature: {md}");
        assert!(md.contains("_feedback_"), "missing category: {md}");
        assert!(
            md.contains("Show notification"),
            "missing description: {md}"
        );
    }

    #[test]
    fn format_sdk_reference_entry_markdown_does_not_warn_for_supported() {
        let entry = sdk_ref("arg", "arg(p)", "Prompt", "prompts");
        let md = format_sdk_reference_entry_markdown(&entry);
        assert!(
            !md.contains("Unsupported in GPUI"),
            "supported entry markdown must not carry an unsupported warning: {md}"
        );
    }

    #[test]
    fn sdk_reference_supported_count_exceeds_unsupported_count() {
        let doc = build_sdk_reference_document();
        let supported = doc
            .functions
            .iter()
            .filter(|f| f.support == SdkSupport::Supported)
            .count();
        let unsupported = doc
            .functions
            .iter()
            .filter(|f| f.support == SdkSupport::Unsupported)
            .count();
        assert!(
            unsupported > 0,
            "at least one SDK entry (notify) must be labeled unsupported"
        );
        assert!(
            supported > unsupported,
            "SDK reference is meant to guide authors to working APIs: supported ({supported}) should exceed unsupported ({unsupported})"
        );
    }

    #[test]
    fn sdk_reference_schema_version_is_five() {
        // Pin the current schema version so any accidental bump is visible
        // in the diff and stays paired with an envelope-shape change.
        assert_eq!(SDK_REFERENCE_SCHEMA_VERSION, 5);
    }

    #[test]
    fn script_templates_do_not_reference_unsupported_sdk_apis() {
        // Starter templates cannot silently depend on a stub SDK API. If a
        // future template calls e.g. `notify(...)` or `keyboard.type(...)`,
        // this test must fail so the template author either chooses a
        // working API or we intentionally upgrade the SDK entry's support
        // status first.
        let templates = build_script_templates_document().templates;
        let needles = unsupported_sdk_reference_scan_needles();
        assert!(
            !needles.is_empty(),
            "needle list must be non-empty — if every SDK entry becomes Supported, the needle builder drifted and this test becomes a no-op"
        );
        for template in &templates {
            let rendered = render_script_template_file(template, "Demo");
            for needle in &needles {
                assert!(
                    !rendered.contains(needle.as_str()),
                    "Template `{}` references unsupported SDK API `{needle}`. Rendered body:\n{rendered}",
                    template.id
                );
            }
        }
    }

    #[test]
    fn harness_workflow_examples_do_not_reference_unsupported_sdk_apis() {
        // The kit://sdk-reference harness workflow ships concrete example
        // scripts (test-script + scriptlet) that agents and users copy
        // verbatim. After i008 started flagging `notify` as Unsupported in
        // kit://sdk-reference, any example that still calls `notify(...)`
        // contradicts the product. This test pins the invariant.
        let workflow = build_harness_workflow();
        let examples: [(&str, &str); 2] = [
            ("example_test_script", workflow.example_test_script.as_str()),
            ("example_scriptlet", workflow.example_scriptlet.as_str()),
        ];
        let needles = unsupported_sdk_reference_scan_needles();
        assert!(
            !needles.is_empty(),
            "needle list must be non-empty — if every SDK entry becomes Supported, the needle builder drifted and this test becomes a no-op"
        );
        for (label, body) in &examples {
            for needle in &needles {
                assert!(
                    !body.contains(needle.as_str()),
                    "Harness workflow `{label}` references unsupported SDK API `{needle}`.\nBody:\n{body}"
                );
            }
        }
    }

    #[test]
    fn harness_workflow_example_scriptlet_uses_hud_for_feedback() {
        // Pins the intent of the copy-today's-date scriptlet: because the
        // desired feedback is launcher-local (flash a confirmation while the
        // launcher is the active surface), the canonical example uses
        // `hud(...)` rather than `notify(...)`. `notify(...)` is a
        // Supported, real OS-notification API — equally legitimate when the
        // caller wants Notification Center delivery that lasts past a dismiss
        // — but mixing it into this example would misinform authors about
        // when to pick each one.
        let workflow = build_harness_workflow();
        assert!(
            workflow
                .example_scriptlet
                .contains("hud(\"Copied today's date\")"),
            "example_scriptlet must give launcher-local feedback via `hud(...)`; reach for `notify(...)` only when you want OS Notification Center delivery.\nBody:\n{}",
            workflow.example_scriptlet
        );
        assert!(
            !workflow.example_scriptlet.contains("notify("),
            "example_scriptlet must not call `notify(...)`; this copy-date scriptlet is a launcher-local feedback example — `hud(message)` is the right choice here.\nBody:\n{}",
            workflow.example_scriptlet
        );
    }

    // =======================================================
    // kit://script-templates resource tests
    // =======================================================

    fn template_ref(id: &str, title: &str, description: &str, category: &str) -> ScriptTemplateRef {
        ScriptTemplateRef {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            filename_hint: id.to_string(),
            body_template: "// placeholder for {{NAME}}\n".to_string(),
            metadata_defaults: ScriptTemplateMetadataDefaults::default(),
        }
    }

    #[test]
    fn script_templates_document_has_schema_version_and_templates() {
        let doc = build_script_templates_document();
        assert_eq!(doc.schema_version, SCRIPT_TEMPLATES_RESOURCE_SCHEMA_VERSION);
        assert_eq!(doc.count, doc.templates.len());
        assert!(
            !doc.templates.is_empty(),
            "v1 should ship at least one starter template"
        );
        // Blank Starter must stay in row #1 so the fast path feels identical
        // to the pre-catalog experience.
        assert_eq!(
            doc.templates[0].id, "blank-starter",
            "Blank Starter must be the first row"
        );
    }

    #[test]
    fn script_template_ids_are_unique() {
        let doc = build_script_templates_document();
        let mut ids: Vec<&str> = doc.templates.iter().map(|t| t.id.as_str()).collect();
        ids.sort();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(
            ids.len(),
            original_len,
            "Template ids must be unique: {ids:?}"
        );
    }

    #[test]
    fn filter_script_template_entries_matches_title_description_and_category() {
        let entries = vec![
            template_ref("t-1", "Blank Starter", "Empty shape", "starter"),
            template_ref("t-2", "Choice List", "Pick one from a list", "prompts"),
            template_ref("t-3", "Daily Note", "Writes today's text", "files"),
        ];
        let all = filter_script_template_entries(&entries, "");
        assert_eq!(all, vec![0, 1, 2]);
        let whitespace = filter_script_template_entries(&entries, "   ");
        assert_eq!(whitespace, vec![0, 1, 2]);

        // Title match (case-insensitive).
        assert_eq!(filter_script_template_entries(&entries, "CHOICE"), vec![1]);
        // Description match.
        assert_eq!(filter_script_template_entries(&entries, "today"), vec![2]);
        // Category match.
        assert_eq!(filter_script_template_entries(&entries, "starter"), vec![0]);
        // No matches.
        assert_eq!(
            filter_script_template_entries(&entries, "no-such-thing"),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn render_script_template_file_includes_metadata_name() {
        let template = ScriptTemplateRef {
            id: "demo".into(),
            title: "Demo".into(),
            description: "test".into(),
            category: "starter".into(),
            filename_hint: "demo".into(),
            body_template: concat!(
                "export const metadata = {\n",
                "  name: \"{{NAME}}\",\n",
                "  description: \"{{DESCRIPTION}}\",\n",
                "};\n",
            )
            .into(),
            metadata_defaults: ScriptTemplateMetadataDefaults {
                description: Some("seeded description".into()),
            },
        };
        let rendered = render_script_template_file(&template, "My Friendly Name");
        assert!(
            rendered.contains("name: \"My Friendly Name\""),
            "friendly name should be substituted into metadata.name: {rendered}"
        );
        assert!(
            rendered.contains("description: \"seeded description\""),
            "description default should be substituted: {rendered}"
        );
        assert!(
            !rendered.contains("{{NAME}}"),
            "all placeholders should be replaced: {rendered}"
        );
        assert!(
            !rendered.contains("{{DESCRIPTION}}"),
            "all placeholders should be replaced: {rendered}"
        );
    }

    #[test]
    fn render_script_template_file_falls_back_to_title_when_no_description_default() {
        let mut template = ScriptTemplateRef {
            id: "demo".into(),
            title: "Demo Title".into(),
            description: "card text".into(),
            category: "starter".into(),
            filename_hint: "demo".into(),
            body_template: "{{DESCRIPTION}}".into(),
            metadata_defaults: ScriptTemplateMetadataDefaults::default(),
        };
        template.metadata_defaults.description = None;
        let rendered = render_script_template_file(&template, "unused");
        assert_eq!(
            rendered, "Demo Title",
            "missing description_default should fall back to title"
        );
    }

    #[test]
    fn find_script_template_returns_template_by_id() {
        let found = find_script_template("blank-starter").expect("blank-starter must exist");
        assert_eq!(found.id, "blank-starter");
    }

    #[test]
    fn find_script_template_returns_none_for_unknown_id() {
        assert!(find_script_template("no-such-template-id").is_none());
    }

    #[test]
    fn starter_templates_do_not_emit_collision_binding_fields() {
        let doc = build_script_templates_document();
        for template in &doc.templates {
            let rendered = render_script_template_file(template, "Demo");
            for banned in ["alias:", "shortcut:", "keyword:", "trigger:"] {
                assert!(
                    !rendered.contains(banned),
                    "Template `{}` must not emit `{}` (would be fatally hidden by validate_script_catalog). Rendered:\n{}",
                    template.id,
                    banned,
                    rendered
                );
            }
        }
    }

    #[test]
    fn script_templates_resource_is_listed_and_readable() {
        let resources = get_resource_definitions();
        let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(
            uris.contains(&SCRIPT_TEMPLATES_RESOURCE_URI),
            "{SCRIPT_TEMPLATES_RESOURCE_URI} should be in resource definitions"
        );

        let content = read_resource(SCRIPT_TEMPLATES_RESOURCE_URI, &[], &[], None)
            .expect("script-templates resource should be readable");
        assert_eq!(content.uri, SCRIPT_TEMPLATES_RESOURCE_URI);
        assert_eq!(content.mime_type, "application/json");
        let doc: ScriptTemplatesResourceDocument =
            serde_json::from_str(&content.text).expect("valid JSON envelope");
        assert_eq!(doc.schema_version, SCRIPT_TEMPLATES_RESOURCE_SCHEMA_VERSION);
        assert_eq!(doc.count, doc.templates.len());
    }
}
