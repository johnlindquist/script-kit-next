//! MCP Script Namespace Tools
//!
//! Auto-generates MCP tools from Script Kit scripts that have schema definitions.
//! Scripts with `schema = { input: {...} }` are exposed as `scripts/{script-name}` tools.
//!
//! Example script:
//! ```typescript
//! // Name: Create Note
//! // Description: Creates a new note
//! schema = {
//!   input: {
//!     title: { type: "string", required: true, description: "Note title" },
//!     content: { type: "string", description: "Note content" }
//!   }
//! }
//! ```
//!
//! This becomes MCP tool: `scripts/create-note`
//! With inputSchema derived from schema.input

// Allow dead code - ScriptTool struct and generate_script_tool for future tool execution
#![allow(dead_code)]

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema_parser::{FieldDef, FieldType, Schema};
    use std::collections::HashMap;
    use std::path::PathBuf;

    // =======================================================
    // TDD Tests - Written FIRST per spec requirements
    // =======================================================

    /// Helper to create a test script with schema
    fn test_script_with_schema(name: &str, description: Option<&str>, schema: Schema) -> Script {
        Script {
            name: name.to_string(),
            path: PathBuf::from(format!("/test/{}.ts", slugify_name(name))),
            extension: "ts".to_string(),
            description: description.map(|s| s.to_string()),
            icon: None,
            alias: None,
            shortcut: None,
            typed_metadata: None,
            schema: Some(schema),
        }
    }

    /// Helper to create a simple schema with one input field
    fn simple_input_schema(field_name: &str, field_type: FieldType, required: bool) -> Schema {
        let mut input = HashMap::new();
        input.insert(
            field_name.to_string(),
            FieldDef {
                field_type,
                required,
                description: Some(format!("The {} field", field_name)),
                ..Default::default()
            },
        );
        Schema {
            input,
            output: HashMap::new(),
        }
    }

    /// Helper to create a script without schema
    fn test_script_without_schema(name: &str) -> Script {
        Script {
            name: name.to_string(),
            path: PathBuf::from(format!("/test/{}.ts", slugify_name(name))),
            extension: "ts".to_string(),
            description: None,
            icon: None,
            alias: None,
            shortcut: None,
            typed_metadata: None,
            schema: None,
        }
    }

    // =======================================================
    // test_generate_tool_from_script_with_schema
    // =======================================================

    #[test]
    fn test_generate_tool_from_script_with_schema() {
        let schema = simple_input_schema("title", FieldType::String, true);
        let script = test_script_with_schema("Create Note", Some("Creates a new note"), schema);

        let tool = generate_tool_from_script(&script);

        assert!(
            tool.is_some(),
            "Script with schema.input should generate tool"
        );
        let tool = tool.unwrap();

        // Verify tool properties
        assert_eq!(tool.name, "scripts/create-note");
        assert_eq!(tool.description, "Creates a new note");

        // Verify input schema structure
        assert_eq!(tool.input_schema["type"], "object");
        assert!(tool.input_schema["properties"]["title"].is_object());
        assert_eq!(tool.input_schema["properties"]["title"]["type"], "string");

        // Verify required fields
        let required = tool.input_schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("title")));
    }

    #[test]
    fn test_generate_tool_from_script_with_multiple_fields() {
        let mut input = HashMap::new();
        input.insert(
            "title".to_string(),
            FieldDef {
                field_type: FieldType::String,
                required: true,
                description: Some("Note title".to_string()),
                ..Default::default()
            },
        );
        input.insert(
            "content".to_string(),
            FieldDef {
                field_type: FieldType::String,
                required: false,
                description: Some("Note content".to_string()),
                ..Default::default()
            },
        );
        input.insert(
            "priority".to_string(),
            FieldDef {
                field_type: FieldType::Number,
                required: false,
                ..Default::default()
            },
        );

        let schema = Schema {
            input,
            output: HashMap::new(),
        };
        let script = test_script_with_schema("Multi Field", None, schema);

        let tool = generate_tool_from_script(&script);
        assert!(tool.is_some());
        let tool = tool.unwrap();

        // All fields should be in properties
        assert!(tool.input_schema["properties"]["title"].is_object());
        assert!(tool.input_schema["properties"]["content"].is_object());
        assert!(tool.input_schema["properties"]["priority"].is_object());

        // Only required fields should be in required array
        let required = tool.input_schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert!(required.contains(&serde_json::json!("title")));
    }

    // =======================================================
    // test_no_tool_for_script_without_schema
    // =======================================================

    #[test]
    fn test_no_tool_for_script_without_schema() {
        let script = test_script_without_schema("Simple Script");

        let tool = generate_tool_from_script(&script);

        assert!(
            tool.is_none(),
            "Script without schema should not generate tool"
        );
    }

    #[test]
    fn test_no_tool_for_script_with_empty_input_schema() {
        let schema = Schema {
            input: HashMap::new(), // Empty input
            output: HashMap::new(),
        };
        let script = test_script_with_schema("Empty Input", None, schema);

        let tool = generate_tool_from_script(&script);

        assert!(
            tool.is_none(),
            "Script with empty input schema should not generate tool"
        );
    }

    #[test]
    fn test_no_tool_for_script_with_output_only_schema() {
        let mut output = HashMap::new();
        output.insert(
            "result".to_string(),
            FieldDef {
                field_type: FieldType::String,
                ..Default::default()
            },
        );
        let schema = Schema {
            input: HashMap::new(), // No input
            output,
        };
        let script = test_script_with_schema("Output Only", None, schema);

        let tool = generate_tool_from_script(&script);

        assert!(
            tool.is_none(),
            "Script with only output schema should not generate tool"
        );
    }

    // =======================================================
    // test_script_tool_name_format
    // =======================================================

    #[test]
    fn test_script_tool_name_format() {
        let test_cases = vec![
            ("Create Note", "scripts/create-note"),
            ("git-commit", "scripts/git-commit"),
            ("Hello World", "scripts/hello-world"),
            ("test_script", "scripts/test-script"),
            ("UPPERCASE", "scripts/uppercase"),
            ("multi  spaces", "scripts/multi-spaces"),
            ("special@chars!", "scripts/special-chars"),
        ];

        for (name, expected_tool_name) in test_cases {
            let schema = simple_input_schema("x", FieldType::String, false);
            let script = test_script_with_schema(name, None, schema);

            let tool = generate_tool_from_script(&script);
            assert!(tool.is_some(), "Tool should be generated for '{}'", name);
            assert_eq!(
                tool.unwrap().name,
                expected_tool_name,
                "Tool name for '{}' should be '{}'",
                name,
                expected_tool_name
            );
        }
    }

    #[test]
    fn test_script_tool_name_starts_with_scripts_prefix() {
        let schema = simple_input_schema("x", FieldType::String, false);
        let script = test_script_with_schema("Any Script", None, schema);

        let tool = generate_tool_from_script(&script);
        assert!(tool.is_some());
        assert!(
            tool.unwrap().name.starts_with("scripts/"),
            "Tool name must start with 'scripts/' prefix"
        );
    }

    // =======================================================
    // test_script_tool_input_schema
    // =======================================================

    #[test]
    fn test_script_tool_input_schema() {
        let mut input = HashMap::new();
        input.insert(
            "name".to_string(),
            FieldDef {
                field_type: FieldType::String,
                required: true,
                description: Some("User name".to_string()),
                ..Default::default()
            },
        );
        input.insert(
            "age".to_string(),
            FieldDef {
                field_type: FieldType::Number,
                required: false,
                min: Some(0.0),
                max: Some(150.0),
                ..Default::default()
            },
        );
        input.insert(
            "active".to_string(),
            FieldDef {
                field_type: FieldType::Boolean,
                required: false,
                default: Some(serde_json::json!(true)),
                ..Default::default()
            },
        );

        let schema = Schema {
            input,
            output: HashMap::new(),
        };
        let script = test_script_with_schema("User Profile", None, schema);

        let tool = generate_tool_from_script(&script);
        assert!(tool.is_some());
        let input_schema = &tool.unwrap().input_schema;

        // Type should be object
        assert_eq!(input_schema["type"], "object");

        // Check string field with description
        let name_schema = &input_schema["properties"]["name"];
        assert_eq!(name_schema["type"], "string");
        assert_eq!(name_schema["description"], "User name");

        // Check number field with constraints
        let age_schema = &input_schema["properties"]["age"];
        assert_eq!(age_schema["type"], "number");
        assert_eq!(age_schema["minimum"], 0.0);
        assert_eq!(age_schema["maximum"], 150.0);

        // Check boolean field with default
        let active_schema = &input_schema["properties"]["active"];
        assert_eq!(active_schema["type"], "boolean");
        assert_eq!(active_schema["default"], true);

        // Required array should only have "name"
        let required = input_schema["required"].as_array().unwrap();
        assert_eq!(required.len(), 1);
        assert!(required.contains(&serde_json::json!("name")));
    }

    #[test]
    fn test_script_tool_input_schema_with_enum() {
        let mut input = HashMap::new();
        input.insert(
            "status".to_string(),
            FieldDef {
                field_type: FieldType::String,
                required: true,
                enum_values: Some(vec![
                    "pending".to_string(),
                    "active".to_string(),
                    "completed".to_string(),
                ]),
                ..Default::default()
            },
        );

        let schema = Schema {
            input,
            output: HashMap::new(),
        };
        let script = test_script_with_schema("Status Update", None, schema);

        let tool = generate_tool_from_script(&script);
        assert!(tool.is_some());
        let input_schema = &tool.unwrap().input_schema;

        let status_enum = input_schema["properties"]["status"]["enum"].as_array();
        assert!(status_enum.is_some());
        let enum_values = status_enum.unwrap();
        assert_eq!(enum_values.len(), 3);
        assert!(enum_values.contains(&serde_json::json!("pending")));
        assert!(enum_values.contains(&serde_json::json!("active")));
        assert!(enum_values.contains(&serde_json::json!("completed")));
    }

    // =======================================================
    // Additional helper function tests
    // =======================================================

    #[test]
    fn test_is_script_tool() {
        assert!(is_script_tool("scripts/create-note"));
        assert!(is_script_tool("scripts/git-commit"));
        assert!(is_script_tool("scripts/any-name"));

        assert!(!is_script_tool("kit/show"));
        assert!(!is_script_tool("tools/list"));
        assert!(!is_script_tool("scriptsshow")); // No slash
    }

    #[test]
    fn test_get_script_tool_definitions() {
        let schema1 = simple_input_schema("x", FieldType::String, false);
        let schema2 = simple_input_schema("y", FieldType::Number, true);

        let scripts: Vec<Arc<Script>> = vec![
            Arc::new(test_script_with_schema("Script One", None, schema1)),
            Arc::new(test_script_without_schema("Script Two")), // No schema
            Arc::new(test_script_with_schema("Script Three", None, schema2)),
        ];

        let tools = get_script_tool_definitions(&scripts);

        assert_eq!(
            tools.len(),
            2,
            "Only scripts with schema should generate tools"
        );
        assert!(tools.iter().any(|t| t.name == "scripts/script-one"));
        assert!(tools.iter().any(|t| t.name == "scripts/script-three"));
    }

    #[test]
    fn test_find_script_by_tool_name() {
        let schema = simple_input_schema("x", FieldType::String, false);
        let scripts: Vec<Arc<Script>> = vec![
            Arc::new(test_script_with_schema("Create Note", None, schema.clone())),
            Arc::new(test_script_with_schema("Git Commit", None, schema.clone())),
        ];

        // Should find by tool name
        let found = find_script_by_tool_name(&scripts, "scripts/create-note");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Create Note");

        // Should find different script
        let found = find_script_by_tool_name(&scripts, "scripts/git-commit");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Git Commit");

        // Should not find non-existent script
        let found = find_script_by_tool_name(&scripts, "scripts/unknown");
        assert!(found.is_none());

        // Should not find kit/* namespace
        let found = find_script_by_tool_name(&scripts, "kit/show");
        assert!(found.is_none());
    }

    #[test]
    fn test_handle_script_tool_call_success() {
        let schema = simple_input_schema("title", FieldType::String, true);
        let scripts: Vec<Arc<Script>> = vec![Arc::new(test_script_with_schema(
            "Create Note",
            Some("Creates notes"),
            schema,
        ))];

        let result = handle_script_tool_call(
            &scripts,
            "scripts/create-note",
            &serde_json::json!({"title": "Test Note"}),
        );

        assert!(
            result.is_error.is_none() || result.is_error == Some(false),
            "Should succeed for valid tool"
        );
        assert!(!result.content.is_empty());

        // Result should contain script path
        let text = &result.content[0].text;
        assert!(text.contains("create-note"));
    }

    #[test]
    fn test_handle_script_tool_call_not_found() {
        let scripts: Vec<Arc<Script>> = vec![];

        let result = handle_script_tool_call(&scripts, "scripts/unknown", &serde_json::json!({}));

        assert_eq!(result.is_error, Some(true));
        assert!(result.content[0].text.contains("not found"));
    }

    #[test]
    fn test_script_tool_description_fallback() {
        let schema = simple_input_schema("x", FieldType::String, false);
        let script = test_script_with_schema("My Script", None, schema);

        let tool = generate_tool_from_script(&script);
        assert!(tool.is_some());

        // Should have fallback description when none provided
        let tool = tool.unwrap();
        assert!(tool.description.contains("My Script"));
        assert!(tool.description.contains("Run"));
    }

    #[test]
    fn test_generate_script_tool_struct() {
        let schema = simple_input_schema("title", FieldType::String, true);
        let script = test_script_with_schema("Test Script", Some("Description"), schema);

        let script_tool = generate_script_tool(&script);
        assert!(script_tool.is_some());

        let tool = script_tool.unwrap();
        assert_eq!(tool.tool_name, "scripts/test-script");
        assert_eq!(tool.description, "Description");
        assert_eq!(tool.script.name, "Test Script");
    }

    #[test]
    fn test_slugify_name() {
        assert_eq!(slugify_name("Hello World"), "hello-world");
        assert_eq!(slugify_name("git-commit"), "git-commit");
        assert_eq!(slugify_name("test_script"), "test-script");
        assert_eq!(slugify_name("UPPER CASE"), "upper-case");
        assert_eq!(slugify_name("multi  spaces"), "multi-spaces");
        assert_eq!(slugify_name("special@#$chars"), "special-chars");
        assert_eq!(slugify_name("---leading-trailing---"), "leading-trailing");
    }
}
