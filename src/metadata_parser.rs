//! Typed metadata parser for Script Kit scripts
//!
//! Parses the new typed `metadata = { ... }` global from scripts,
//! complementing the existing comment-based metadata parser in scripts.rs.
//!
//! Example script with typed metadata:
//! ```typescript
//! metadata = {
//!   name: "Create Note",
//!   description: "Creates a new note in the notes directory",
//!   author: "John Lindquist",
//!   enter: "Create",
//!   alias: "note",
//!   tags: ["productivity", "notes"],
//!   hidden: false
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Typed metadata extracted from a `metadata = { ... }` global declaration
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TypedMetadata {
    /// Display name for the script
    pub name: Option<String>,
    /// Description shown in the UI
    pub description: Option<String>,
    /// Author of the script
    pub author: Option<String>,
    /// Text shown on the Enter/Submit button
    pub enter: Option<String>,
    /// Short alias for quick triggering (e.g., "gc" for "git-commit")
    pub alias: Option<String>,
    /// Text expansion keyword trigger (e.g., "!testing", ":sig", "addr,,")
    #[serde(alias = "expand", alias = "snippet")]
    pub keyword: Option<String>,
    /// Icon name (e.g., "File", "Terminal", "Star")
    pub icon: Option<String>,
    /// Keyboard shortcut (e.g., "opt i", "cmd shift k")
    pub shortcut: Option<String>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// Whether to hide this script from the main list
    #[serde(default)]
    pub hidden: bool,
    /// Custom placeholder text for the input
    pub placeholder: Option<String>,
    /// Cron expression for scheduled execution
    pub cron: Option<String>,
    /// Natural language schedule (e.g., "every tuesday at 2pm") - converted to cron internally
    pub schedule: Option<String>,
    /// Watch patterns for file-triggered execution
    #[serde(default)]
    pub watch: Vec<String>,
    /// Background script (runs without UI)
    #[serde(default)]
    pub background: bool,
    /// System-level script (higher privileges)
    #[serde(default)]
    pub system: bool,
    /// Whether this script is a fallback handler (shown when no search results match)
    #[serde(default)]
    pub fallback: bool,
    /// Display label for fallback with {input} placeholder (e.g., "Search docs for {input}")
    pub fallback_label: Option<String>,
    /// Any additional custom fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Result of parsing a script file for typed metadata
#[derive(Debug, Clone)]
pub struct MetadataParseResult {
    /// The parsed typed metadata, if found
    pub metadata: Option<TypedMetadata>,
    /// Any parse errors encountered (non-fatal)
    pub errors: Vec<String>,
    /// The byte range where metadata was found (for potential removal/replacement)
    pub span: Option<(usize, usize)>,
}

/// Extract typed metadata from script content
///
/// Looks for `metadata = { ... }` at the top level of the script.
/// The metadata object must be valid JSON5/JavaScript object literal.
///
/// Returns `MetadataParseResult` with the parsed metadata and any errors.
pub fn extract_typed_metadata(content: &str) -> MetadataParseResult {
    let mut result = MetadataParseResult {
        metadata: None,
        errors: vec![],
        span: None,
    };

    // Find `metadata = ` or `metadata=` pattern
    let metadata_pattern = find_metadata_assignment(content);

    if let Some((start_idx, obj_start)) = metadata_pattern {
        // Extract the object literal
        match extract_object_literal(content, obj_start) {
            Ok((json_str, end_idx)) => {
                result.span = Some((start_idx, end_idx));

                // Parse as JSON (JavaScript object literals are mostly JSON-compatible)
                // We do minimal preprocessing to handle JS-specific syntax
                let normalized = normalize_js_object(&json_str);

                match serde_json::from_str::<TypedMetadata>(&normalized) {
                    Ok(metadata) => {
                        debug!(
                            name = ?metadata.name,
                            description = ?metadata.description,
                            "Parsed typed metadata"
                        );
                        result.metadata = Some(metadata);
                    }
                    Err(e) => {
                        result
                            .errors
                            .push(format!("Failed to parse metadata JSON: {}", e));
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

/// Find the `metadata = ` assignment in the content
/// Returns (start_index, object_start_index) if found
fn find_metadata_assignment(content: &str) -> Option<(usize, usize)> {
    // Look for `metadata` followed by optional whitespace and `=`
    let patterns = ["metadata=", "metadata =", "metadata  ="];

    for pattern in patterns {
        if let Some(idx) = content.find(pattern) {
            // Find the `{` after `=`
            let after_eq = idx + pattern.len();
            let rest = &content[after_eq..];

            // Skip whitespace to find `{`
            for (i, c) in rest.char_indices() {
                if c == '{' {
                    return Some((idx, after_eq + i));
                } else if !c.is_whitespace() {
                    // Not an object literal
                    break;
                }
            }
        }
    }

    None
}

/// Extract a balanced object literal starting at the given index
/// Returns (object_string, end_index) on success
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

    Err("Unbalanced braces in metadata object".to_string())
}

/// Normalize JavaScript object literal to valid JSON
/// - Converts unquoted keys to quoted keys
/// - Converts single quotes to double quotes
/// - Removes trailing commas
fn normalize_js_object(js: &str) -> String {
    let mut result = String::with_capacity(js.len());
    let chars: Vec<char> = js.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;
    let mut string_char = '"';

    while i < len {
        let c = chars[i];

        // Handle string state
        if in_string {
            if c == '\\' && i + 1 < len {
                result.push(c);
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if c == string_char {
                in_string = false;
                result.push('"'); // Always output double quotes
                i += 1;
                continue;
            }
            result.push(c);
            i += 1;
            continue;
        }

        // Handle start of string
        if c == '"' || c == '\'' {
            in_string = true;
            string_char = c;
            result.push('"'); // Always use double quotes
            i += 1;
            continue;
        }

        // Skip single-line comments
        if c == '/' && i + 1 < len && chars[i + 1] == '/' {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Skip multi-line comments
        if c == '/' && i + 1 < len && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }

        // Handle trailing commas: ,] or ,}
        if c == ',' {
            // Look ahead for ] or } (skipping whitespace)
            let mut j = i + 1;
            while j < len && chars[j].is_whitespace() {
                j += 1;
            }
            if j < len && (chars[j] == ']' || chars[j] == '}') {
                // Skip the trailing comma
                i += 1;
                continue;
            }
        }

        // Handle unquoted keys (after { or ,)
        if c.is_alphabetic() || c == '_' || c == '$' {
            // Check if this looks like an unquoted key
            let mut key_end = i;
            while key_end < len
                && (chars[key_end].is_alphanumeric()
                    || chars[key_end] == '_'
                    || chars[key_end] == '$')
            {
                key_end += 1;
            }

            // Skip whitespace after key
            let mut colon_pos = key_end;
            while colon_pos < len && chars[colon_pos].is_whitespace() {
                colon_pos += 1;
            }

            // If followed by colon, it's an unquoted key
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_metadata() {
        let content = r#"
metadata = {
    name: "My Script",
    description: "Does something cool"
}

const result = await arg("Pick one");
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("My Script".to_string()));
        assert_eq!(meta.description, Some("Does something cool".to_string()));
    }

    #[test]
    fn test_parse_all_fields() {
        let content = r#"
metadata = {
    name: "Full Script",
    description: "A script with all fields",
    author: "John Doe",
    enter: "Execute",
    alias: "fs",
    icon: "Star",
    shortcut: "cmd shift f",
    tags: ["productivity", "utility"],
    hidden: false,
    placeholder: "Type something...",
    cron: "0 9 * * *",
    watch: ["*.ts", "*.js"],
    background: false,
    system: false
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let meta = result.metadata.unwrap();

        assert_eq!(meta.name, Some("Full Script".to_string()));
        assert_eq!(
            meta.description,
            Some("A script with all fields".to_string())
        );
        assert_eq!(meta.author, Some("John Doe".to_string()));
        assert_eq!(meta.enter, Some("Execute".to_string()));
        assert_eq!(meta.alias, Some("fs".to_string()));
        assert_eq!(meta.icon, Some("Star".to_string()));
        assert_eq!(meta.shortcut, Some("cmd shift f".to_string()));
        assert_eq!(meta.tags, vec!["productivity", "utility"]);
        assert!(!meta.hidden);
        assert_eq!(meta.placeholder, Some("Type something...".to_string()));
        assert_eq!(meta.cron, Some("0 9 * * *".to_string()));
        assert_eq!(meta.watch, vec!["*.ts", "*.js"]);
        assert!(!meta.background);
        assert!(!meta.system);
    }

    #[test]
    fn test_parse_with_single_quotes() {
        let content = r#"
metadata = {
    name: 'Single Quoted',
    description: 'Uses single quotes'
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("Single Quoted".to_string()));
    }

    #[test]
    fn test_parse_with_trailing_comma() {
        let content = r#"
metadata = {
    name: "Trailing Comma",
    description: "Has a trailing comma",
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("Trailing Comma".to_string()));
    }

    #[test]
    fn test_parse_no_metadata() {
        let content = r#"
// Name: Old Style
// Description: Uses comments

const result = await arg("Pick");
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_none());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_parse_metadata_no_spaces() {
        let content = r#"metadata={name:"NoSpaces"}"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("NoSpaces".to_string()));
    }

    #[test]
    fn test_parse_extra_fields() {
        let content = r#"
metadata = {
    name: "With Extras",
    customField: "custom value",
    anotherField: 42
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("With Extras".to_string()));
        assert!(meta.extra.contains_key("customField"));
    }

    #[test]
    fn test_parse_nested_objects_in_string() {
        let content = r#"
metadata = {
    name: "Has JSON in string",
    description: "Contains {nested: \"json\"} in description"
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_some());
        let meta = result.metadata.unwrap();
        assert!(meta.description.unwrap().contains("{nested:"));
    }

    #[test]
    fn test_span_tracking() {
        let content = r#"// Comment
metadata = {
    name: "Test"
}
const x = 1;"#;
        let result = extract_typed_metadata(content);
        assert!(result.span.is_some());
        let (start, end) = result.span.unwrap();
        let extracted = &content[start..end];
        assert!(extracted.contains("metadata"));
        assert!(extracted.contains("name"));
    }

    #[test]
    fn test_invalid_json_reports_error() {
        let content = r#"
metadata = {
    name: "Bad JSON,
    description: missing closing quote
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.metadata.is_none());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_defaults_for_missing_optional_fields() {
        let content = r#"metadata = { name: "Minimal" }"#;
        let result = extract_typed_metadata(content);
        let meta = result.metadata.unwrap();

        assert_eq!(meta.name, Some("Minimal".to_string()));
        assert_eq!(meta.description, None);
        assert_eq!(meta.tags, Vec::<String>::new());
        assert!(!meta.hidden);
        assert!(!meta.background);
    }

    #[test]
    fn test_parse_schedule_field() {
        let content = r#"
metadata = {
    name: "Scheduled Script",
    description: "Runs on a schedule",
    schedule: "every tuesday at 2pm"
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("Scheduled Script".to_string()));
        assert_eq!(meta.schedule, Some("every tuesday at 2pm".to_string()));
    }

    #[test]
    fn test_parse_cron_and_schedule_together() {
        let content = r#"
metadata = {
    name: "Dual Scheduled",
    cron: "0 14 * * 2",
    schedule: "every tuesday at 2pm"
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let meta = result.metadata.unwrap();
        assert_eq!(meta.cron, Some("0 14 * * 2".to_string()));
        assert_eq!(meta.schedule, Some("every tuesday at 2pm".to_string()));
    }

    #[test]
    fn test_parse_fallback_fields() {
        let content = r#"
metadata = {
    name: "Search Docs",
    description: "Search documentation for a term",
    fallback: true,
    fallbackLabel: "Search docs for {input}"
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let meta = result.metadata.unwrap();
        assert_eq!(meta.name, Some("Search Docs".to_string()));
        assert!(meta.fallback, "fallback should be true");
        assert_eq!(
            meta.fallback_label,
            Some("Search docs for {input}".to_string())
        );
    }

    #[test]
    fn test_parse_fallback_without_label() {
        let content = r#"
metadata = {
    name: "Web Search",
    fallback: true
}
"#;
        let result = extract_typed_metadata(content);
        assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
        let meta = result.metadata.unwrap();
        assert!(meta.fallback, "fallback should be true");
        assert_eq!(
            meta.fallback_label, None,
            "fallback_label should be None when not provided"
        );
    }

    #[test]
    fn test_fallback_defaults_to_false() {
        let content = r#"
metadata = {
    name: "Regular Script"
}
"#;
        let result = extract_typed_metadata(content);
        let meta = result.metadata.unwrap();
        assert!(!meta.fallback, "fallback should default to false");
        assert_eq!(meta.fallback_label, None);
    }
}
