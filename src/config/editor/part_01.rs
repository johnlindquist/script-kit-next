use std::path::Path;
use tree_sitter::Parser;

/// Result of a config edit operation
#[derive(Debug, Clone, PartialEq)]
pub enum EditResult {
    /// Successfully modified the config
    Modified(String),
    /// The property already exists with the desired value
    AlreadySet,
    /// Could not parse or modify the config
    Failed(String),
}

/// A property to add to the config
#[derive(Debug, Clone)]
pub struct ConfigProperty {
    /// Property name (e.g., "claudeCode")
    pub name: String,
    /// Property value as TypeScript code (e.g., "{\n    enabled: true\n  }")
    pub value: String,
}

impl ConfigProperty {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Error type for config write operations
#[derive(Debug)]
pub enum ConfigWriteError {
    /// The edited content failed TypeScript validation
    ValidationFailed(String),
    /// File system operation failed
    IoError(String),
    /// The editor could not modify the content
    EditFailed(String),
}

impl std::fmt::Display for ConfigWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::EditFailed(msg) => write!(f, "Edit failed: {}", msg),
        }
    }
}

/// Result of a successful config write
#[derive(Debug, PartialEq)]
pub enum WriteOutcome {
    /// File was modified and written
    Written,
    /// Property already existed, no write needed
    AlreadySet,
    /// File was created from scratch (was empty or missing)
    Created,
}

/// Add or update a property in a TypeScript config file
///
/// # Arguments
/// * `content` - The current file content
/// * `property` - The property to add/update
///
/// # Returns
/// * `EditResult::Modified(new_content)` - Successfully modified
/// * `EditResult::AlreadySet` - Property already exists with desired value
/// * `EditResult::Failed(reason)` - Could not modify
pub fn add_property(content: &str, property: &ConfigProperty) -> EditResult {
    // Check if property already exists
    if contains_property(content, &property.name) {
        // TODO: Could update the value if different
        return EditResult::AlreadySet;
    }

    // Find the closing brace of the default export object
    match find_object_end(content) {
        Some(insert_info) => {
            let new_content = insert_property(content, &insert_info, property);
            EditResult::Modified(new_content)
        }
        None => EditResult::Failed("Could not find config object closing brace".to_string()),
    }
}

/// Check if the config contains a top-level property with the given name.
///
/// Uses tree-sitter AST to find `pair` nodes within the export default object,
/// correctly ignoring properties in comments, strings, or nested objects.
pub fn contains_property(content: &str, property_name: &str) -> bool {
    let tree = match parse_typescript(content) {
        Ok(t) => t,
        Err(_) => return false,
    };
    let root = tree.root_node();

    let export = match find_export_statement(root) {
        Some(e) => e,
        None => return false,
    };

    let object = match find_object_in_export(export, content) {
        Some(o) => o,
        None => return false,
    };

    // Check only top-level pairs (direct children of the export object)
    for i in 0..object.named_child_count() {
        if let Some(pair) = object.named_child(i) {
            if pair.kind() == "pair" {
                if let Some(key) = pair.child_by_field_name("key") {
                    let key_text = &content[key.start_byte()..key.end_byte()];
                    if key_text == property_name {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Information about where to insert a new property
#[derive(Debug)]
struct InsertInfo {
    /// Position of the closing brace `}`
    close_brace_pos: usize,
    /// Whether the last property has a trailing comma
    has_trailing_comma: bool,
    /// The indentation to use for the new property
    indent: String,
}

// ==========================================================================
// Tree-sitter AST helpers
// ==========================================================================

/// Parse TypeScript content into a tree-sitter AST.
fn parse_typescript(content: &str) -> Result<tree_sitter::Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
        .map_err(|e| format!("Failed to set TypeScript language: {}", e))?;
    parser
        .parse(content, None)
        .ok_or_else(|| "Failed to parse TypeScript content".to_string())
}

/// Find the `export_statement` node in the root of the AST.
fn find_export_statement(root: tree_sitter::Node) -> Option<tree_sitter::Node> {
    (0..root.named_child_count())
        .filter_map(|i| root.named_child(i))
        .find(|n| n.kind() == "export_statement")
}

/// Find the default export object node within an export_statement.
///
/// Handles three forms:
/// - `export default { ... } satisfies Config;` → satisfies_expression → object
/// - `export default { ... } as Config;` → as_expression → object
/// - `export default { ... };` → object directly
fn find_object_in_export<'a>(
    export_node: tree_sitter::Node<'a>,
    _content: &str,
) -> Option<tree_sitter::Node<'a>> {
    for i in 0..export_node.named_child_count() {
        if let Some(child) = export_node.named_child(i) {
            match child.kind() {
                "satisfies_expression" | "as_expression" => {
                    // The object is the first child of kind "object"
                    for j in 0..child.named_child_count() {
                        if let Some(grandchild) = child.named_child(j) {
                            if grandchild.kind() == "object" {
                                return Some(grandchild);
                            }
                        }
                    }
                }
                "object" => return Some(child),
                _ => {}
            }
        }
    }
    None
}

/// Walk AST to collect ERROR node descriptions for diagnostics.
fn collect_ast_errors(node: tree_sitter::Node, content: &str, out: &mut String) {
    if node.is_error() || node.is_missing() {
        let start = node.start_position();
        let end_byte = node.end_byte().min(node.start_byte() + 30);
        let text = &content[node.start_byte()..end_byte];
        if !out.is_empty() {
            out.push_str("; ");
        }
        out.push_str(&format!(
            "line {}:{} near '{}'",
            start.row + 1,
            start.column,
            text.replace('\n', "\\n")
        ));
    }
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            collect_ast_errors(child, content, out);
        }
    }
}

/// Find the export default object and return insertion info using tree-sitter AST.
///
/// Returns byte offsets suitable for direct `&str` slicing.
fn find_object_end(content: &str) -> Option<InsertInfo> {
    let tree = parse_typescript(content).ok()?;
    let root = tree.root_node();

    let export = find_export_statement(root)?;
    let object = find_object_in_export(export, content)?;

    // The object node spans { ... }. end_byte() is exclusive (byte after }).
    let close_brace_byte = object.end_byte() - 1;

    // Verify it's actually a closing brace
    if content.as_bytes().get(close_brace_byte) != Some(&b'}') {
        return None;
    }

    // Collect pair children (actual properties, not comments)
    let pairs: Vec<_> = (0..object.named_child_count())
        .filter_map(|i| object.named_child(i))
        .filter(|n| n.kind() == "pair")
        .collect();

    // Check for trailing comma between last pair and closing brace
    let has_trailing_comma = if let Some(last_pair) = pairs.last() {
        let between = &content[last_pair.end_byte()..close_brace_byte];
        between.contains(',')
    } else {
        false
    };

    // Detect indent from first pair's column position
    let indent = if let Some(first_pair) = pairs.first() {
        " ".repeat(first_pair.start_position().column)
    } else {
        "  ".to_string()
    };

    Some(InsertInfo {
        close_brace_pos: close_brace_byte,
        has_trailing_comma,
        indent,
    })
}

/// Insert a property into the config content
fn insert_property(content: &str, info: &InsertInfo, property: &ConfigProperty) -> String {
    let before = &content[..info.close_brace_pos];
    let after = &content[info.close_brace_pos..];

    // Build the property string with trailing comma for safety
    let comma_prefix = if info.has_trailing_comma { "" } else { "," };
    let property_str = format!(
        "{}\n\n{}{}: {},",
        comma_prefix, info.indent, property.name, property.value
    );

    // Find where to insert (right before the closing brace)
    // We want to maintain nice formatting, so add a newline if needed
    let needs_newline_before = !before.ends_with('\n');
    let newline_before = if needs_newline_before { "\n" } else { "" };

    // Build the result
    let result = format!("{}{}{}\n{}", before, newline_before, property_str, after);

    // VALIDATION: Check for corruption patterns
    // If we detect }{, it means something went wrong with brace matching
    if result.contains("}{") {
        tracing::error!(
            "Config editor detected potential corruption: }}{{ pattern found. \
             close_brace_pos={}, content_len={}, before_len={}, after_starts_with={:?}",
            info.close_brace_pos,
            content.len(),
            before.len(),
            after.chars().take(10).collect::<String>()
        );
    }

    result
}

/// Enable Claude Code in a config file
///
/// This is a convenience function that adds `claudeCode: { enabled: true }` to the config.
/// Used in tests; production code uses `enable_claude_code_safely`.
#[allow(dead_code)]
pub fn enable_claude_code(content: &str) -> EditResult {
    // Use inline format for cleaner insertion
    // The trailing comma is added by insert_property
    let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
    add_property(content, &property)
}

