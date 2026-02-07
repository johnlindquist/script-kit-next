use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use crate::mcp_kit_tools::ToolDefinition;
use crate::scripts::Script;
/// Represents a Script Kit script as an MCP tool
#[derive(Debug, Clone)]
pub struct ScriptTool {
    /// The script this tool wraps
    pub script: Script,
    /// Tool name in format: scripts/{script-name}
    pub tool_name: String,
    /// JSON Schema for the tool's input
    pub input_schema: Value,
    /// Tool description from script metadata
    pub description: String,
}
/// Result of a script tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptToolResult {
    pub content: Vec<ScriptToolContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}
/// Content item in script tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}
/// Convert a script name to a tool-friendly slug
/// e.g., "Create Note" -> "create-note"
fn slugify_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
/// Generate an MCP tool definition from a script
///
/// Only scripts with `schema.input` will generate tools.
/// Returns `None` if the script has no schema or no input schema.
///
/// Tool name format: `scripts/{script-name-slug}`
/// Tool description: From script description or fallback to name
/// Input schema: From script's schema.input converted to JSON Schema
pub fn generate_tool_from_script(script: &Script) -> Option<ToolDefinition> {
    // Only scripts with schema.input become tools
    let schema = script.schema.as_ref()?;

    // Skip scripts with empty input schema
    if schema.input.is_empty() {
        return None;
    }

    let tool_name = format!("scripts/{}", slugify_name(&script.name));
    let description = script
        .description
        .clone()
        .unwrap_or_else(|| format!("Run the {} script", script.name));
    let input_schema = schema.to_json_schema_input();

    Some(ToolDefinition {
        name: tool_name,
        description,
        input_schema,
    })
}
/// Generate a ScriptTool from a script (includes the script reference)
pub fn generate_script_tool(script: &Script) -> Option<ScriptTool> {
    let schema = script.schema.as_ref()?;

    if schema.input.is_empty() {
        return None;
    }

    let tool_name = format!("scripts/{}", slugify_name(&script.name));
    let description = script
        .description
        .clone()
        .unwrap_or_else(|| format!("Run the {} script", script.name));
    let input_schema = schema.to_json_schema_input();

    Some(ScriptTool {
        script: script.clone(),
        tool_name,
        input_schema,
        description,
    })
}
/// Get all tool definitions from a list of scripts
/// Only returns scripts that have schema.input defined
pub fn get_script_tool_definitions(scripts: &[Arc<Script>]) -> Vec<ToolDefinition> {
    scripts
        .iter()
        .filter_map(|s| generate_tool_from_script(s.as_ref()))
        .collect()
}
/// Check if a tool name is in the scripts/* namespace
pub fn is_script_tool(name: &str) -> bool {
    name.starts_with("scripts/")
}
/// Find a script by its tool name
/// Returns None if tool is not in scripts/* namespace or script not found
pub fn find_script_by_tool_name<'a>(
    scripts: &'a [Arc<Script>],
    tool_name: &str,
) -> Option<&'a Arc<Script>> {
    if !is_script_tool(tool_name) {
        return None;
    }

    // Extract the slug from "scripts/{slug}"
    let slug = tool_name.strip_prefix("scripts/")?;

    // Find script where slugified name matches
    scripts.iter().find(|s| slugify_name(&s.name) == slug)
}
/// Handle a scripts/* namespace tool call
///
/// This validates the tool exists and returns a placeholder result.
/// Actual script execution should be handled by the caller using the script path.
pub fn handle_script_tool_call(
    scripts: &[Arc<Script>],
    tool_name: &str,
    arguments: &Value,
) -> ScriptToolResult {
    // Find the script
    let script = match find_script_by_tool_name(scripts, tool_name) {
        Some(s) => s,
        None => {
            return ScriptToolResult {
                content: vec![ScriptToolContent {
                    content_type: "text".to_string(),
                    text: format!("Script tool not found: {}", tool_name),
                }],
                is_error: Some(true),
            }
        }
    };

    // Return success with script path for execution
    // The actual execution should be done by the caller
    ScriptToolResult {
        content: vec![ScriptToolContent {
            content_type: "text".to_string(),
            text: serde_json::json!({
                "status": "pending",
                "script_path": script.path.to_string_lossy(),
                "arguments": arguments,
                "message": format!("Script '{}' queued for execution", script.name)
            })
            .to_string(),
        }],
        is_error: None,
    }
}
