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
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
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
            description: Some("List of all available scripts in ~/.scriptkit/scripts/".to_string()),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "scriptlets://".to_string(),
            name: "Scriptlets".to_string(),
            description: Some("List of all available scriptlets from markdown files".to_string()),
            mime_type: "application/json".to_string(),
        },
        McpResource {
            uri: "kit://context".to_string(),
            name: "Current Context".to_string(),
            description: Some(
                "Deterministic snapshot of AI-relevant desktop context. Supports ?profile=minimal, ?diagnostics=1, and per-field flags: selectedText, frontmostApp, menuBar, browserUrl, focusedWindow, screenshot. See kit://context/schema for the full contract."
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
                "Schema-versioned list of all scripts in ~/.scriptkit/scripts/ with metadata. Safe for repeated reads."
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
    ];
    resources.extend(transaction_resources::transaction_resource_definitions());
    resources
}
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
        _ if uri == "kit://context"
            || uri.starts_with("kit://context?")
            || uri == "kit://context/schema"
            || uri.starts_with("kit://context/schema?") =>
        {
            read_context_resource(uri)
        }
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

/// Schema version for the `kit://scripts` resource envelope.
pub const SCRIPTS_RESOURCE_SCHEMA_VERSION: u32 = 1;

/// Schema version for the `kit://scriptlets` resource envelope.
pub const SCRIPTLETS_RESOURCE_SCHEMA_VERSION: u32 = 1;

/// Schema version for the `kit://sdk-reference` resource.
pub const SDK_REFERENCE_SCHEMA_VERSION: u32 = 1;

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

/// A single SDK function reference entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SdkFunctionRef {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub category: String,
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
}

fn build_sdk_function_refs() -> Vec<SdkFunctionRef> {
    vec![
        SdkFunctionRef {
            name: "arg".into(),
            signature: "await arg(prompt: string, choices?: Choice[]): Promise<string>".into(),
            description: "Prompt the user with an input field, optionally with a list of choices.".into(),
            category: "prompts".into(),
        },
        SdkFunctionRef {
            name: "div".into(),
            signature: "await div(html: string): Promise<void>".into(),
            description: "Display HTML content in a panel.".into(),
            category: "prompts".into(),
        },
        SdkFunctionRef {
            name: "editor".into(),
            signature: "await editor(options?: EditorOptions): Promise<string>".into(),
            description: "Open a full-screen code editor and return the content.".into(),
            category: "prompts".into(),
        },
        SdkFunctionRef {
            name: "term".into(),
            signature: "await term(command?: string): Promise<string>".into(),
            description: "Open an interactive terminal, optionally running a command.".into(),
            category: "prompts".into(),
        },
        SdkFunctionRef {
            name: "drop".into(),
            signature: "await drop(options?: DropOptions): Promise<DroppedItem[]>".into(),
            description: "Accept drag-and-drop files from the user.".into(),
            category: "prompts".into(),
        },
        SdkFunctionRef {
            name: "template".into(),
            signature: "await template(template: string): Promise<string>".into(),
            description: "Fill in a template string with user-provided values.".into(),
            category: "prompts".into(),
        },
        SdkFunctionRef {
            name: "exec".into(),
            signature: "await exec(command: string, args?: string[]): Promise<ExecResult>".into(),
            description: "Execute a shell command and return its output.".into(),
            category: "system".into(),
        },
        SdkFunctionRef {
            name: "clipboard".into(),
            signature: "await clipboard.readText(): Promise<string>".into(),
            description: "Read the current clipboard text content.".into(),
            category: "clipboard".into(),
        },
        SdkFunctionRef {
            name: "copy".into(),
            signature: "await copy(text: string): Promise<void>".into(),
            description: "Copy text to the clipboard.".into(),
            category: "clipboard".into(),
        },
        SdkFunctionRef {
            name: "paste".into(),
            signature: "await paste(text: string): Promise<void>".into(),
            description: "Paste text to the focused application.".into(),
            category: "clipboard".into(),
        },
        SdkFunctionRef {
            name: "notify".into(),
            signature: "await notify(message: string): Promise<void>".into(),
            description: "Show a system notification.".into(),
            category: "feedback".into(),
        },
        SdkFunctionRef {
            name: "setSelectedText".into(),
            signature: "await setSelectedText(text: string): Promise<void>".into(),
            description: "Replace the selected text in the focused application.".into(),
            category: "system".into(),
        },
        SdkFunctionRef {
            name: "getSelectedText".into(),
            signature: "await getSelectedText(): Promise<string>".into(),
            description: "Read the selected text from the focused application.".into(),
            category: "system".into(),
        },
        SdkFunctionRef {
            name: "readFile".into(),
            signature: "await readFile(path: string): Promise<string>".into(),
            description: "Read a file's contents as UTF-8 text.".into(),
            category: "filesystem".into(),
        },
        SdkFunctionRef {
            name: "writeFile".into(),
            signature: "await writeFile(path: string, content: string): Promise<void>".into(),
            description: "Write UTF-8 text content to a file.".into(),
            category: "filesystem".into(),
        },
        SdkFunctionRef {
            name: "home".into(),
            signature: "home(...paths: string[]): string".into(),
            description: "Resolve a path relative to the user's home directory.".into(),
            category: "filesystem".into(),
        },
    ]
}

fn build_sdk_reference_document() -> SdkReferenceDocument {
    SdkReferenceDocument {
        schema_version: SDK_REFERENCE_SCHEMA_VERSION,
        sdk_package: "@johnlindquist/kit".into(),
        script_directory: "~/.scriptkit/scripts/".into(),
        scriptlet_pattern: "~/.scriptkit/kit/*/extensions/*.md".into(),
        metadata_format: "// Name: My Script\n// Description: What it does\n// Shortcut: opt i".into(),
        functions: build_sdk_function_refs(),
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
    let json = serde_json::to_string_pretty(&doc)
        .map_err(|e| format!("Failed to serialize SDK reference: {e}"))?;
    Ok(ResourceContent {
        uri: "kit://sdk-reference".to_string(),
        mime_type: "application/json".to_string(),
        text: json,
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
        "kit://context?screenshot=1&diagnostics=1",
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
            }
            _ => return Err(invalid_context_param(key, value)),
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
            kit_name: None,
        }
    }

    /// Helper to create a test scriptlet
    fn test_scriptlet(name: &str, tool: &str, description: Option<&str>) -> Scriptlet {
        Scriptlet {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            code: "echo test".to_string(),
            tool: tool.to_string(),
            shortcut: None,
            keyword: None,
            group: None,
            file_path: None,
            command: None,
            alias: None,
        }
    }

    #[test]
    fn test_resources_list_includes_all() {
        // REQUIREMENT: resources/list returns all three resources
        let resources = get_resource_definitions();

        assert_eq!(resources.len(), 10, "Should have exactly 10 resources");

        let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(uris.contains(&"kit://state"), "Should include kit://state");
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
            assert_eq!(
                resource.mime_type, "application/json",
                "Should be JSON mime type"
            );
            assert!(resource.description.is_some(), "Should have a description");
        }
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
        assert_eq!(resource_array.len(), 10);

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
            kit_name: None,
        };

        let entry: ScriptResourceEntry = (&script_with_schema).into();
        assert!(entry.has_schema);
    }

    #[test]
    fn test_scriptlet_resource_entry_from_scriptlet() {
        let scriptlet = Scriptlet {
            name: "Full Scriptlet".to_string(),
            description: Some("Test description".to_string()),
            code: "echo test".to_string(),
            tool: "bash".to_string(),
            shortcut: Some("cmd k".to_string()),
            keyword: Some(":test".to_string()),
            group: Some("My Group".to_string()),
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
            browser: Some(crate::context_snapshot::BrowserContext {
                url: "https://example.com".to_string(),
            }),
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
        assert_eq!(value["meta"]["enabledFieldCount"], 3);
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

        let doc: ScriptsResourceDocument =
            serde_json::from_str(&content.text).expect("valid JSON");
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
        let doc: ScriptsResourceDocument =
            serde_json::from_str(&content.text).expect("valid JSON");
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

        let doc: SdkReferenceDocument =
            serde_json::from_str(&content.text).expect("valid JSON");
        assert_eq!(doc.schema_version, SDK_REFERENCE_SCHEMA_VERSION);
        assert_eq!(doc.sdk_package, "@johnlindquist/kit");
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
        assert!(doc.metadata_format.contains("// Name:"));
        assert!(doc.metadata_format.contains("// Description:"));
        assert!(doc.script_directory.contains("scripts"));
        assert!(doc.scriptlet_pattern.contains("extensions"));
    }

    #[test]
    fn sdk_reference_roundtrips_through_json() {
        let doc = build_sdk_reference_document();
        let json = serde_json::to_string(&doc).expect("serialize");
        let parsed: SdkReferenceDocument = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(doc, parsed);
    }
}
