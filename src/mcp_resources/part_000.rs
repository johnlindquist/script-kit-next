use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use crate::scripts::Script;
use crate::scripts::Scriptlet;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    vec![
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
    ]
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
        _ => Err(format!("Resource not found: {}", uri)),
    }
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
