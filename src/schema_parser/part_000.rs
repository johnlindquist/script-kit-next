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
/// Array item schema definition
/// Supports both shorthand (`items: "string"`) and full object syntax (`items: { type: "string", enum: [...] }`)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ItemsDef {
    /// Shorthand: just the type name (e.g., `items: "string"`)
    Type(String),
    /// Full schema object (e.g., `items: { type: "string", enum: ["a", "b"] }`)
    Schema(Box<FieldDef>),
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
    /// Supports both shorthand (`"string"`) and full schema (`{ type: "string", enum: [...] }`)
    #[serde(default)]
    pub items: Option<ItemsDef>,

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
                        result
                            .errors
                            .push(format!("Failed to parse schema JSON: {}", e));
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
            while key_end < len
                && (chars[key_end].is_alphanumeric()
                    || chars[key_end] == '_'
                    || chars[key_end] == '$')
            {
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
