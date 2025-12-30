//! Schema parser for Script Kit scripts
#![allow(dead_code)]
//!
//! Parses the `schema = { input: {...}, output: {...} }` global from scripts.
//! This defines the typed interface for input() and output() functions,
//! enabling MCP tool generation and AI agent integration.
//!
//! Example script with schema:
//! ```typescript
//! schema = {
//!   input: {
//!     title: { type: "string", required: true, description: "Note title" },
//!     tags: { type: "array", items: "string", description: "Tags for the note" }
//!   },
//!   output: {
//!     path: { type: "string", description: "Path to created file" },
//!     wordCount: { type: "number" }
//!   }
//! }
//!
//! const { title, tags } = await input();
//! // ... create note ...
//! output({ path: notePath, wordCount: content.split(' ').length });
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Supported field types for schema definitions
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    #[default]
    String,
    Number,
    Boolean,
    Array,
    Object,
    /// Any type - no validation
    Any,
}

/// Definition of a single field in the schema
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FieldDef {
    /// The type of this field
    #[serde(rename = "type", default)]
    pub field_type: FieldType,
    
    /// Whether this field is required (defaults to false)
    #[serde(default)]
    pub required: bool,
    
    /// Human-readable description for AI agents and documentation
    pub description: Option<String>,
    
    /// Default value if not provided
    pub default: Option<serde_json::Value>,
    
    /// For array types, the type of items
    pub items: Option<String>,
    
    /// For object types, nested field definitions
    pub properties: Option<HashMap<String, FieldDef>>,
    
    /// Enum values (for string fields with limited options)
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    
    /// Minimum value (for numbers) or length (for strings/arrays)
    pub min: Option<f64>,
    
    /// Maximum value (for numbers) or length (for strings/arrays)
    pub max: Option<f64>,
    
    /// Regex pattern for validation (strings only)
    pub pattern: Option<String>,
    
    /// Example value for documentation
    pub example: Option<serde_json::Value>,
}

/// Full schema definition with input and output sections
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Schema {
    /// Input fields - what the script expects to receive
    #[serde(default)]
    pub input: HashMap<String, FieldDef>,
    
    /// Output fields - what the script will produce
    #[serde(default)]
    pub output: HashMap<String, FieldDef>,
}

/// Result of parsing a script file for schema
#[derive(Debug, Clone)]
pub struct SchemaParseResult {
    /// The parsed schema, if found
    pub schema: Option<Schema>,
    /// Any parse errors encountered (non-fatal)
    pub errors: Vec<String>,
    /// The byte range where schema was found
    pub span: Option<(usize, usize)>,
}

/// Extract schema from script content
///
/// Looks for `schema = { ... }` at the top level of the script.
/// The schema object must contain `input` and/or `output` sections.
///
/// Returns `SchemaParseResult` with the parsed schema and any errors.
pub fn extract_schema(content: &str) -> SchemaParseResult {
    let mut result = SchemaParseResult {
        schema: None,
        errors: vec![],
        span: None,
    };

    // Find `schema = ` or `schema=` pattern
    let schema_pattern = find_schema_assignment(content);
    
    if let Some((start_idx, obj_start)) = schema_pattern {
        // Extract the object literal
        match extract_object_literal(content, obj_start) {
            Ok((json_str, end_idx)) => {
                result.span = Some((start_idx, end_idx));
                
                // Normalize and parse
                let normalized = normalize_js_object(&json_str);
                
                match serde_json::from_str::<Schema>(&normalized) {
                    Ok(schema) => {
                        debug!(
                            input_fields = schema.input.len(),
                            output_fields = schema.output.len(),
                            "Parsed schema"
                        );
                        result.schema = Some(schema);
                    }
                    Err(e) => {
                        result.errors.push(format!("Failed to parse schema JSON: {}", e));
                    }
                }
            }
            Err(e) => {
                result.errors.push(e);
            }
        }
    }

    result
}

/// Find the `schema = ` assignment or `defineSchema({` call in the content
fn find_schema_assignment(content: &str) -> Option<(usize, usize)> {
    // First try direct assignment patterns: schema = { ... }
    let assignment_patterns = ["schema=", "schema =", "schema  ="];
    
    for pattern in assignment_patterns {
        if let Some(idx) = content.find(pattern) {
            let after_eq = idx + pattern.len();
            let rest = &content[after_eq..];
            
            for (i, c) in rest.char_indices() {
                if c == '{' {
                    return Some((idx, after_eq + i));
                } else if !c.is_whitespace() {
                    break;
                }
            }
        }
    }
    
    // Then try defineSchema() function pattern: defineSchema({ ... })
    let define_patterns = ["defineSchema({", "defineSchema ({", "defineSchema  ({"];
    
    for pattern in define_patterns {
        if let Some(idx) = content.find(pattern) {
            // Find the opening brace after defineSchema
            let after_define = idx + pattern.len() - 1; // -1 because pattern includes '{'
            return Some((idx, after_define));
        }
    }
    
    None
}

/// Extract a balanced object literal starting at the given index
fn extract_object_literal(content: &str, start: usize) -> Result<(String, usize), String> {
    let bytes = content.as_bytes();
    if start >= bytes.len() || bytes[start] != b'{' {
        return Err("Expected '{' at start of object".to_string());
    }

    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let mut string_char = '"';

    for (i, &byte) in bytes[start..].iter().enumerate() {
        let c = byte as char;
        
        if escape_next {
            escape_next = false;
            continue;
        }

        if in_string {
            if c == '\\' {
                escape_next = true;
            } else if c == string_char {
                in_string = false;
            }
            continue;
        }

        match c {
            '"' | '\'' | '`' => {
                in_string = true;
                string_char = c;
            }
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let end = start + i + 1;
                    return Ok((content[start..end].to_string(), end));
                }
            }
            _ => {}
        }
    }

    Err("Unbalanced braces in schema object".to_string())
}

/// Normalize JavaScript object literal to valid JSON
fn normalize_js_object(js: &str) -> String {
    let mut result = String::with_capacity(js.len());
    let chars: Vec<char> = js.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;
    let mut string_char = '"';

    while i < len {
        let c = chars[i];

        if in_string {
            if c == '\\' && i + 1 < len {
                result.push(c);
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if c == string_char {
                in_string = false;
                result.push('"');
                i += 1;
                continue;
            }
            result.push(c);
            i += 1;
            continue;
        }

        if c == '"' || c == '\'' {
            in_string = true;
            string_char = c;
            result.push('"');
            i += 1;
            continue;
        }

        // Skip comments
        if c == '/' && i + 1 < len && chars[i + 1] == '/' {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        if c == '/' && i + 1 < len && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }

        // Handle trailing commas
        if c == ',' {
            let mut j = i + 1;
            while j < len && chars[j].is_whitespace() {
                j += 1;
            }
            if j < len && (chars[j] == ']' || chars[j] == '}') {
                i += 1;
                continue;
            }
        }

        // Handle unquoted keys
        if c.is_alphabetic() || c == '_' || c == '$' {
            let mut key_end = i;
            while key_end < len && (chars[key_end].is_alphanumeric() || chars[key_end] == '_' || chars[key_end] == '$') {
                key_end += 1;
            }
            
            let mut colon_pos = key_end;
            while colon_pos < len && chars[colon_pos].is_whitespace() {
                colon_pos += 1;
            }
            
            if colon_pos < len && chars[colon_pos] == ':' {
                let key: String = chars[i..key_end].iter().collect();
                result.push('"');
                result.push_str(&key);
                result.push('"');
                i = key_end;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

/// Generate JSON Schema from our Schema definition
/// Useful for validation and MCP tool definitions
impl Schema {
    /// Convert to JSON Schema format for the input section
    pub fn to_json_schema_input(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for (name, field) in &self.input {
            properties.insert(name.clone(), field_to_json_schema(field));
            if field.required {
                required.push(serde_json::Value::String(name.clone()));
            }
        }

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required,
        })
    }

    /// Convert to JSON Schema format for the output section
    pub fn to_json_schema_output(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for (name, field) in &self.output {
            properties.insert(name.clone(), field_to_json_schema(field));
            if field.required {
                required.push(serde_json::Value::String(name.clone()));
            }
        }

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required,
        })
    }
}

fn field_to_json_schema(field: &FieldDef) -> serde_json::Value {
    let mut schema = serde_json::Map::new();
    
    let type_str = match field.field_type {
        FieldType::String => "string",
        FieldType::Number => "number",
        FieldType::Boolean => "boolean",
        FieldType::Array => "array",
        FieldType::Object => "object",
        FieldType::Any => "any",
    };
    schema.insert("type".to_string(), serde_json::Value::String(type_str.to_string()));

    if let Some(desc) = &field.description {
        schema.insert("description".to_string(), serde_json::Value::String(desc.clone()));
    }

    if let Some(default) = &field.default {
        schema.insert("default".to_string(), default.clone());
    }

    if let Some(enum_vals) = &field.enum_values {
        let vals: Vec<serde_json::Value> = enum_vals.iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect();
        schema.insert("enum".to_string(), serde_json::Value::Array(vals));
    }

    if let Some(min) = field.min {
        if matches!(field.field_type, FieldType::Number) {
            schema.insert("minimum".to_string(), serde_json::json!(min));
        } else {
            schema.insert("minLength".to_string(), serde_json::json!(min as i64));
        }
    }

    if let Some(max) = field.max {
        if matches!(field.field_type, FieldType::Number) {
            schema.insert("maximum".to_string(), serde_json::json!(max));
        } else {
            schema.insert("maxLength".to_string(), serde_json::json!(max as i64));
        }
    }

    if let Some(pattern) = &field.pattern {
        schema.insert("pattern".to_string(), serde_json::Value::String(pattern.clone()));
    }

    if let Some(items) = &field.items {
        schema.insert("items".to_string(), serde_json::json!({"type": items}));
    }

    if let Some(props) = &field.properties {
        let mut prop_schemas = serde_json::Map::new();
        for (name, prop_field) in props {
            prop_schemas.insert(name.clone(), field_to_json_schema(prop_field));
        }
        schema.insert("properties".to_string(), serde_json::Value::Object(prop_schemas));
    }

    if let Some(example) = &field.example {
        schema.insert("example".to_string(), example.clone());
    }

    serde_json::Value::Object(schema)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_schema() {
        let content = r#"
schema = {
    input: {
        title: { type: "string", required: true, description: "The title" }
    },
    output: {
        result: { type: "string" }
    }
}
"#;
        let result = extract_schema(content);
        assert!(result.schema.is_some(), "Errors: {:?}", result.errors);
        let schema = result.schema.unwrap();
        
        assert_eq!(schema.input.len(), 1);
        assert_eq!(schema.output.len(), 1);
        
        let title_field = schema.input.get("title").unwrap();
        assert_eq!(title_field.field_type, FieldType::String);
        assert!(title_field.required);
        assert_eq!(title_field.description, Some("The title".to_string()));
    }

    #[test]
    fn test_parse_all_field_types() {
        let content = r#"
schema = {
    input: {
        name: { type: "string" },
        count: { type: "number" },
        enabled: { type: "boolean" },
        items: { type: "array", items: "string" },
        config: { type: "object" },
        anything: { type: "any" }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();
        
        assert_eq!(schema.input.get("name").unwrap().field_type, FieldType::String);
        assert_eq!(schema.input.get("count").unwrap().field_type, FieldType::Number);
        assert_eq!(schema.input.get("enabled").unwrap().field_type, FieldType::Boolean);
        assert_eq!(schema.input.get("items").unwrap().field_type, FieldType::Array);
        assert_eq!(schema.input.get("config").unwrap().field_type, FieldType::Object);
        assert_eq!(schema.input.get("anything").unwrap().field_type, FieldType::Any);
    }

    #[test]
    fn test_parse_field_constraints() {
        let content = r#"
schema = {
    input: {
        username: {
            type: "string",
            required: true,
            min: 3,
            max: 20,
            pattern: "^[a-z]+$"
        },
        age: {
            type: "number",
            min: 0,
            max: 150
        },
        status: {
            type: "string",
            enum: ["active", "inactive", "pending"]
        }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();
        
        let username = schema.input.get("username").unwrap();
        assert_eq!(username.min, Some(3.0));
        assert_eq!(username.max, Some(20.0));
        assert_eq!(username.pattern, Some("^[a-z]+$".to_string()));
        
        let status = schema.input.get("status").unwrap();
        assert_eq!(status.enum_values, Some(vec!["active".to_string(), "inactive".to_string(), "pending".to_string()]));
    }

    #[test]
    fn test_parse_with_defaults_and_examples() {
        let content = r#"
schema = {
    input: {
        count: {
            type: "number",
            default: 10,
            example: 42
        }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();
        
        let count = schema.input.get("count").unwrap();
        assert_eq!(count.default, Some(serde_json::json!(10)));
        assert_eq!(count.example, Some(serde_json::json!(42)));
    }

    #[test]
    fn test_parse_no_schema() {
        let content = r#"
// Just a regular script
const x = await arg("Pick");
"#;
        let result = extract_schema(content);
        assert!(result.schema.is_none());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_parse_input_only() {
        let content = r#"
schema = {
    input: {
        name: { type: "string" }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();
        assert_eq!(schema.input.len(), 1);
        assert_eq!(schema.output.len(), 0);
    }

    #[test]
    fn test_parse_output_only() {
        let content = r#"
schema = {
    output: {
        result: { type: "string" }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();
        assert_eq!(schema.input.len(), 0);
        assert_eq!(schema.output.len(), 1);
    }

    #[test]
    fn test_to_json_schema() {
        let content = r#"
schema = {
    input: {
        title: { type: "string", required: true, description: "Title" },
        count: { type: "number", required: false }
    }
}
"#;
        let result = extract_schema(content);
        let schema = result.schema.unwrap();
        
        let json_schema = schema.to_json_schema_input();
        
        assert_eq!(json_schema["type"], "object");
        assert!(json_schema["properties"]["title"].is_object());
        assert_eq!(json_schema["properties"]["title"]["type"], "string");
        
        let required = json_schema["required"].as_array().unwrap();
        assert!(required.contains(&serde_json::json!("title")));
        assert!(!required.contains(&serde_json::json!("count")));
    }

    #[test]
    fn test_parse_trailing_commas() {
        let content = r#"
schema = {
    input: {
        name: { type: "string", },
    },
    output: {
        result: { type: "boolean", },
    },
}
"#;
        let result = extract_schema(content);
        assert!(result.schema.is_some(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_parse_single_quotes() {
        let content = r#"
schema = {
    input: {
        name: { type: 'string', description: 'The name' }
    }
}
"#;
        let result = extract_schema(content);
        assert!(result.schema.is_some());
        let schema = result.schema.unwrap();
        assert_eq!(schema.input.get("name").unwrap().description, Some("The name".to_string()));
    }

    #[test]
    fn test_span_tracking() {
        let content = r#"// Header
schema = { input: { x: { type: "string" } } }
const y = 1;"#;
        let result = extract_schema(content);
        assert!(result.span.is_some());
        let (start, end) = result.span.unwrap();
        let extracted = &content[start..end];
        assert!(extracted.contains("schema"));
        assert!(extracted.contains("input"));
    }

    #[test]
    fn test_invalid_schema_reports_error() {
        let content = r#"
schema = {
    input: {
        bad: { type: "invalid_type" }
    }
}
"#;
        let result = extract_schema(content);
        // serde should error on unknown enum variant
        assert!(result.schema.is_none() || result.schema.as_ref().unwrap().input.is_empty());
    }

    // TDD: Test for defineSchema() function pattern
    // This pattern should also work for MCP tool detection
    #[test]
    fn test_parse_define_schema_function() {
        let content = r#"
import "@johnlindquist/kit"

const { input, output } = defineSchema({
    input: {
        greeting: { type: "string", required: true, description: "Greeting message" },
        count: { type: "number" }
    },
    output: {
        message: { type: "string", description: "Response message" }
    }
} as const)

const { greeting } = await input()
output({ message: `Hello ${greeting}!` })
"#;
        let result = extract_schema(content);
        assert!(result.schema.is_some(), "defineSchema() should be parseable. Errors: {:?}", result.errors);
        
        let schema = result.schema.unwrap();
        assert_eq!(schema.input.len(), 2, "Should have 2 input fields");
        assert_eq!(schema.output.len(), 1, "Should have 1 output field");
        
        // Verify input fields
        let greeting = schema.input.get("greeting").expect("Should have greeting field");
        assert!(greeting.required, "greeting should be required");
        assert_eq!(greeting.description, Some("Greeting message".to_string()));
        
        let count = schema.input.get("count").expect("Should have count field");
        assert!(!count.required, "count should not be required");
        
        // Verify output fields
        let message = schema.output.get("message").expect("Should have message field");
        assert_eq!(message.description, Some("Response message".to_string()));
    }

    // TDD: Test that both patterns work (direct assignment and defineSchema)
    #[test]
    fn test_parse_both_schema_patterns() {
        // Direct assignment pattern
        let direct = r#"
schema = {
    input: { name: { type: "string", required: true } }
}
"#;
        let result1 = extract_schema(direct);
        assert!(result1.schema.is_some(), "Direct assignment should work");
        
        // defineSchema function pattern
        let define_fn = r#"
const { input, output } = defineSchema({
    input: { name: { type: "string", required: true } }
} as const)
"#;
        let result2 = extract_schema(define_fn);
        assert!(result2.schema.is_some(), "defineSchema() should work");
        
        // Both should produce same schema
        let schema1 = result1.schema.unwrap();
        let schema2 = result2.schema.unwrap();
        assert_eq!(schema1.input.len(), schema2.input.len());
        assert_eq!(
            schema1.input.get("name").unwrap().required,
            schema2.input.get("name").unwrap().required
        );
    }
}
