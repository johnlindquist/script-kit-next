//! TypeScript config file editor
//!
//! Provides robust utilities for programmatically modifying TypeScript config files
//! like `~/.scriptkit/config.ts`.
//!
//! # Design
//!
//! Uses tree-sitter-typescript to parse the AST, giving exact byte offsets for
//! insertion points. This eliminates the fragility of hand-rolled brace counting.

// --- merged from part_01.rs ---
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

fn find_top_level_pair<'a>(
    object: tree_sitter::Node<'a>,
    content: &str,
    property_name: &str,
) -> Option<tree_sitter::Node<'a>> {
    (0..object.named_child_count())
        .filter_map(|i| object.named_child(i as u32))
        .find(|pair| {
            if pair.kind() != "pair" {
                return false;
            }
            pair.child_by_field_name("key")
                .map(|key| &content[key.start_byte()..key.end_byte()] == property_name)
                .unwrap_or(false)
        })
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
    let tree = match parse_typescript(content) {
        Ok(tree) => tree,
        Err(error) => return EditResult::Failed(error),
    };
    let root = tree.root_node();
    let export = match find_export_statement(root) {
        Some(export) => export,
        None => return EditResult::Failed("No export statement found".to_string()),
    };
    let object = match find_object_in_export(export, content) {
        Some(object) => object,
        None => {
            return EditResult::Failed("Could not find config object in export statement".into());
        }
    };

    if let Some(pair) = find_top_level_pair(object, content, &property.name) {
        let Some(value_node) = pair.child_by_field_name("value") else {
            return EditResult::Failed(format!(
                "Could not find existing value node for {}",
                property.name
            ));
        };
        let existing_value = content[value_node.start_byte()..value_node.end_byte()].trim();
        if existing_value == property.value.trim() {
            return EditResult::AlreadySet;
        }

        let mut new_content = String::with_capacity(
            content.len() - (value_node.end_byte() - value_node.start_byte())
                + property.value.len(),
        );
        new_content.push_str(&content[..value_node.start_byte()]);
        new_content.push_str(&property.value);
        new_content.push_str(&content[value_node.end_byte()..]);
        return EditResult::Modified(new_content);
    }

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
        if let Some(pair) = object.named_child(i as u32) {
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
        .filter_map(|i| root.named_child(i as u32))
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
        if let Some(child) = export_node.named_child(i as u32) {
            match child.kind() {
                "satisfies_expression" | "as_expression" => {
                    // The object is the first child of kind "object"
                    for j in 0..child.named_child_count() {
                        if let Some(grandchild) = child.named_child(j as u32) {
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

/// A location is useful; a snippet of a private setting is not. Follow only
/// the first error branch, without slicing UTF-8 at a diagnostic byte limit.
fn error_position(mut node: tree_sitter::Node<'_>) -> tree_sitter::Point {
    while !node.is_error() && !node.is_missing() {
        let mut cursor = node.walk();
        let Some(child) = node
            .children(&mut cursor)
            .find(|n| n.has_error() || n.is_missing())
        else {
            break;
        };
        node = child;
    }
    node.start_position()
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
        .filter_map(|i| object.named_child(i as u32))
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

/// Enable only the provider switch. Its executable, permissions, tools and
/// extra directories are separate decisions, not defaults to reinstall.
pub fn enable_claude_code(content: &str) -> EditResult {
    fn propose(content: &str) -> Result<EditResult, String> {
        if content.is_empty() {
            return Ok(EditResult::Modified(generate_fresh_config(
                &ConfigProperty::new("claudeCode", "{ enabled: true }"),
            )));
        }
        let tree = parse_typescript(content)?;
        let object = literal_default_object(tree.root_node(), content)?;
        let Some(provider) =
            unique_literal_member(object, content, "claudeCode", "duplicate claudeCode")?
        else {
            return Ok(prepend_member(
                content,
                object,
                "claudeCode: { enabled: true }",
            ));
        };
        if provider.kind() != "object" {
            return Err("claudeCode must be a literal object for this edit".into());
        }
        let Some(switch) =
            unique_literal_member(provider, content, "enabled", "duplicate enabled")?
        else {
            return Ok(prepend_member(content, provider, "enabled: true"));
        };
        match switch.kind() {
            "true" => Ok(EditResult::AlreadySet),
            "false" => {
                let mut next = content.to_owned();
                next.replace_range(switch.byte_range(), "true");
                Ok(EditResult::Modified(next))
            }
            _ => Err("claudeCode.enabled is not a literal boolean".into()),
        }
    }
    propose(content).unwrap_or_else(EditResult::Failed)
}

// Insert after the opening brace; never move a comment, consume a trailing
// comma, or add a comma before the first member of an empty object.
fn prepend_member(content: &str, object: tree_sitter::Node, member: &str) -> EditResult {
    let at = object.start_byte() + 1;
    let rest = &content[at..object.end_byte() - 1];
    let suffix = if rest.is_empty() {
        " "
    } else if rest.starts_with(char::is_whitespace) {
        ","
    } else {
        ", "
    };
    let mut next = content.to_owned();
    next.insert_str(at, &format!(" {member}{suffix}"));
    EditResult::Modified(next)
}

// --- merged from part_02.rs ---

#[cfg(unix)]
#[path = "editor/syntax.rs"]
mod syntax;

/// Check the proposed bytes without executing the config or its compile-time macros.
pub fn validate_typescript(content: &str, bun_path: Option<&str>) -> Result<(), String> {
    let bun = super::resolve_bun(bun_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::PermissionDenied {
            error.to_string()
        } else {
            format!("Failed to run bun: {error}")
        }
    })?;

    #[cfg(unix)]
    {
        syntax::check(&bun, content)
    }
    #[cfg(not(unix))]
    {
        // Without a descriptor-backed entry point, use the existing structural
        // check, not an executable build or a mutable source pathname.
        let _ = bun;
        validate_structure(content)
    }
}

/// Structural validation using tree-sitter AST.
///
/// Parses the content with tree-sitter-typescript and checks that:
/// 1. Content contains `export default`
/// 2. The AST has no ERROR nodes (no syntax errors)
/// 3. The export default object can be found
/// 4. Content ends with `satisfies Config;` or `as Config;`
pub fn validate_structure(content: &str) -> Result<(), String> {
    if !content.contains("export default") {
        return Err("Missing 'export default' declaration".into());
    }

    let tree =
        parse_typescript(content).map_err(|e| format!("Failed to parse TypeScript: {}", e))?;
    let root = tree.root_node();

    // Check for parse errors (syntax errors, corruption, etc.)
    if root.has_error() {
        let at = error_position(root);
        return Err(format!(
            "TypeScript parse error at line {}, byte column {}",
            at.row + 1,
            at.column + 1
        ));
    }

    literal_default_object(root, content)?;

    let trimmed = content.trim();
    if !trimmed.contains("satisfies Config;") && !trimmed.contains("as Config;") {
        return Err("Missing 'satisfies Config' or 'as Config' type assertion".into());
    }

    Ok(())
}

/// Generate a fresh config.ts with the given property included.
fn generate_fresh_config(property: &ConfigProperty) -> String {
    format!(
        r#"import type {{ Config }} from "@scriptkit/sdk";

export default {{
  hotkey: {{ modifiers: ["meta"], key: "Semicolon" }},
  {}: {},
}} satisfies Config;
"#,
        property.name, property.value
    )
}

/// Prepare a preference mutation without executing config.ts or touching disk.
/// The caller may commit this together with another file in a recoverable
/// transaction. Ordinary config writes retain their existing validation path.
pub(crate) fn prepare_config_property(
    content: &str,
    property: &ConfigProperty,
) -> Result<String, ConfigWriteError> {
    let prepared = if content.trim().is_empty() {
        generate_fresh_config(property)
    } else {
        match add_property(content, property) {
            EditResult::Modified(prepared) => prepared,
            EditResult::AlreadySet => content.to_owned(),
            EditResult::Failed(reason) => return Err(ConfigWriteError::EditFailed(reason)),
        }
    };
    validate_structure(&prepared).map_err(ConfigWriteError::ValidationFailed)?;
    Ok(prepared)
}

/// Decode a fixture config without evaluating JavaScript, imports or getters.
/// Only a literal default-export object and inert import/type syntax is accepted.
pub(crate) fn parse_static_config(content: &str) -> Result<serde_json::Value, String> {
    if content.len() > 1_048_576 {
        return Err("static config exceeds size limit".into());
    }
    let tree = parse_typescript(content)?;
    let root = tree.root_node();
    if root.has_error() {
        return Err("static config contains syntax errors".into());
    }
    let mut exports = 0;
    for index in 0..root.named_child_count() {
        let child = root
            .named_child(index as u32)
            .ok_or("missing syntax node")?;
        match child.kind() {
            "import_statement" | "comment" | "type_alias_declaration" | "interface_declaration" => {
            }
            "export_statement" => exports += 1,
            _ => return Err(format!("nonliteral config statement: {}", child.kind())),
        }
    }
    if exports != 1 {
        return Err("static config requires one default export".into());
    }
    let export = find_export_statement(root).ok_or("missing default export")?;
    if !(0..export.child_count())
        .filter_map(|i| export.child(i as u32))
        .any(|node| node.kind() == "default")
    {
        return Err("static config requires a default export".into());
    }
    let object =
        find_object_in_export(export, content).ok_or("config export must be a literal object")?;
    decode_static_config_value(object, content, 0)
}

// A named export may precede the default. Only the latter is Kit's config.
fn literal_default_object<'a>(
    root: tree_sitter::Node<'a>,
    content: &str,
) -> Result<tree_sitter::Node<'a>, &'static str> {
    if root.has_error() {
        return Err("invalid TypeScript");
    }
    let mut cursor = root.walk();
    let mut exports = root.named_children(&mut cursor).filter(|n| {
        n.kind() == "export_statement"
            && (0..n.child_count())
                .filter_map(|i| n.child(i as u32))
                .any(|child| child.kind() == "default")
    });
    let export = exports.next().ok_or("no literal default export")?;
    if exports.next().is_some() {
        return Err("ambiguous default export");
    }
    find_object_in_export(export, content).ok_or("nonliteral default export")
}

// Do not guess through spreads, computed names, getters or duplicate target keys.
// Values of unrelated named properties need not be evaluated or rewritten.
fn unique_literal_member<'a>(
    object: tree_sitter::Node<'a>,
    content: &str,
    name: &str,
    duplicate: &'static str,
) -> Result<Option<tree_sitter::Node<'a>>, &'static str> {
    let mut found = None;
    let mut cursor = object.walk();
    for child in object.named_children(&mut cursor) {
        if child.kind() == "comment" {
            continue;
        }
        if child.kind() != "pair" {
            return Err("nonliteral object member");
        }
        let key = child.child_by_field_name("key").ok_or("missing key")?;
        let text = key
            .utf8_text(content.as_bytes())
            .map_err(|_| "invalid key or value")?;
        let key = match key.kind() {
            "property_identifier" if !text.contains('\\') => text.to_owned(),
            "string" => decode_static_config_string(text).map_err(|_| "nonliteral key")?,
            _ => return Err("nonliteral key"),
        };
        if key == name {
            if found.is_some() {
                return Err(duplicate);
            }
            found = Some(child.child_by_field_name("value").ok_or("missing value")?);
        }
    }
    Ok(found)
}

/// Inspect only an unambiguous top-level literal pin before evaluating config.ts.
/// Other values may be expressions. A spread/computed key can redefine the pin;
/// leave those configs to normal Bun evaluation rather than guessing a target.
pub(super) fn bootstrap_bun_pin(content: &str) -> Result<Option<String>, &'static str> {
    let tree = parse_typescript(content).map_err(|_| "invalid TypeScript")?;
    let object = literal_default_object(tree.root_node(), content)?;
    let Some(value) = unique_literal_member(object, content, "bun_path", "duplicate bun_path")?
    else {
        return Ok(None);
    };
    if value.kind() != "string" {
        return Err("nonliteral bun_path");
    }
    let text = value
        .utf8_text(content.as_bytes())
        .map_err(|_| "invalid key or value")?;
    decode_static_config_string(text)
        .map(Some)
        .map_err(|_| "nonliteral bun_path")
}

fn decode_static_config_string(text: &str) -> Result<String, String> {
    if text.starts_with('"') {
        return serde_json::from_str(text)
            .map_err(|error| format!("invalid static string: {error}"));
    }
    if !text.starts_with('\'') || !text.ends_with('\'') {
        return Err("unsupported static string".into());
    }
    let mut json = String::with_capacity(text.len() + 2);
    json.push('"');
    let mut chars = text[1..text.len() - 1].chars();
    while let Some(character) = chars.next() {
        match character {
            '"' => json.push_str("\\\""),
            '\\' => match chars.next().ok_or("unterminated string escape")? {
                '\'' => json.push('\''),
                escaped => {
                    json.push('\\');
                    json.push(escaped);
                }
            },
            other => json.push(other),
        }
    }
    json.push('"');
    serde_json::from_str(&json).map_err(|error| format!("invalid static string: {error}"))
}

fn decode_static_config_value(
    node: tree_sitter::Node,
    content: &str,
    depth: usize,
) -> Result<serde_json::Value, String> {
    use serde_json::Value;
    if depth > 64 {
        return Err("static config exceeds nesting limit".into());
    }
    let text = node
        .utf8_text(content.as_bytes())
        .map_err(|error| error.to_string())?;
    match node.kind() {
        "object" => {
            let mut object = serde_json::Map::new();
            let mut keys = std::collections::BTreeSet::new();
            for index in 0..node.named_child_count() {
                let pair = node
                    .named_child(index as u32)
                    .ok_or("missing object member")?;
                if pair.kind() == "comment" {
                    continue;
                }
                if pair.kind() != "pair" {
                    return Err(format!("nonliteral object member: {}", pair.kind()));
                }
                let key = pair
                    .child_by_field_name("key")
                    .ok_or("missing object key")?;
                let key_text = key
                    .utf8_text(content.as_bytes())
                    .map_err(|error| error.to_string())?;
                let key = match key.kind() {
                    "property_identifier" => key_text.to_string(),
                    "string" => decode_static_config_string(key_text)?,
                    _ => return Err("nonliteral object key".into()),
                };
                let value = pair
                    .child_by_field_name("value")
                    .ok_or("missing object value")?;
                if !keys.insert(key.clone()) {
                    return Err("duplicate static config property".into());
                }
                if matches!(value.kind(), "identifier" | "undefined")
                    && value
                        .utf8_text(content.as_bytes())
                        .map_err(|error| error.to_string())?
                        == "undefined"
                {
                    continue; // Same object omission semantics as JSON.stringify.
                }
                let value = decode_static_config_value(value, content, depth + 1)?;
                object.insert(key, value);
            }
            Ok(Value::Object(object))
        }
        "array" => {
            let mut values = Vec::new();
            let mut needs_value = true;
            for index in 0..node.child_count() {
                let child = node.child(index as u32).ok_or("missing array member")?;
                match child.kind() {
                    "[" | "]" | "comment" => {}
                    "," if needs_value => {
                        return Err("array holes are not static JSON values".into())
                    }
                    "," => needs_value = true,
                    _ => {
                        values.push(decode_static_config_value(child, content, depth + 1)?);
                        needs_value = false;
                    }
                }
            }
            Ok(Value::Array(values))
        }
        "string" => decode_static_config_string(text).map(Value::String),
        "number" | "true" | "false" | "null" => {
            serde_json::from_str(text).map_err(|error| error.to_string())
        }
        "unary_expression" => {
            let argument = node
                .child_by_field_name("argument")
                .ok_or("missing unary argument")?;
            let operator = node
                .child_by_field_name("operator")
                .ok_or("missing unary operator")?;
            if argument.kind() != "number"
                || operator
                    .utf8_text(content.as_bytes())
                    .map_err(|error| error.to_string())?
                    != "-"
            {
                return Err("only negative numeric literals are allowed".into());
            }
            let number = argument
                .utf8_text(content.as_bytes())
                .map_err(|error| error.to_string())?;
            serde_json::from_str(&format!("-{number}")).map_err(|error| error.to_string())
        }
        other => Err(format!("nonliteral static config value: {other}")),
    }
}

// Native edits and the installed settings CLI both own config.ts. Use the same
// exclusive directory; never break another writer's lock on a timer.
struct SettingsLock {
    path: std::path::PathBuf,
    #[cfg(unix)]
    directory: std::fs::File,
}

impl SettingsLock {
    fn acquire(file: &Path) -> std::io::Result<Self> {
        let mut name = file.as_os_str().to_owned();
        name.push(".settings-lock");
        let path = std::path::PathBuf::from(name);
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                std::io::Error::new(
                    e.kind(),
                    "Settings writer busy; inspect the lock if a writer crashed",
                )
            } else {
                e
            }
        })?;
        #[cfg(unix)]
        let directory = {
            use std::os::unix::fs::OpenOptionsExt;
            std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
                .open(&path)?
        };
        Ok(Self {
            path,
            #[cfg(unix)]
            directory,
        })
    }

    fn confirm(&self) -> std::io::Result<()> {
        let entry = std::fs::symlink_metadata(&self.path)?;
        let same = entry.is_dir();
        #[cfg(unix)]
        let same = {
            use std::os::unix::fs::MetadataExt;
            let owner = self.directory.metadata()?;
            same && (entry.dev(), entry.ino()) == (owner.dev(), owner.ino())
        };
        if !same {
            return Err(std::io::Error::other("Settings writer lock was replaced"));
        }
        Ok(())
    }
}

impl Drop for SettingsLock {
    fn drop(&mut self) {
        if self.confirm().is_err() {
            return;
        }
        if let Err(e) = std::fs::remove_dir(&self.path) {
            tracing::warn!(kind = ?e.kind(), "Could not remove settings writer lock");
        }
    }
}

impl From<std::io::Error> for ConfigWriteError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error.to_string())
    }
}

const MAX_CONFIG_BYTES: u64 = 2 * 1024 * 1024; // same limit as scripts/config-document.ts

#[derive(PartialEq, Eq)]
struct ConfigSnapshot {
    bytes: Vec<u8>,
    #[cfg(unix)]
    identity: (u64, u64, u32),
}

fn config_snapshot(path: &Path) -> std::io::Result<Option<ConfigSnapshot>> {
    use std::io::{Error, ErrorKind, Read};
    if !crate::atomic_file::inspect_private_file(path)? {
        return Ok(None);
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() || meta.len() > MAX_CONFIG_BYTES {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Settings must be a regular file of at most 2 MiB",
        ));
    }
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        if meta.nlink() != 1 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Settings must not be hard-linked",
            ));
        }
        (meta.dev(), meta.ino(), meta.mode())
    };
    let mut bytes = Vec::new();
    file.take(MAX_CONFIG_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_CONFIG_BYTES {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Settings grew beyond 2 MiB",
        ));
    }
    Ok(Some(ConfigSnapshot {
        bytes,
        #[cfg(unix)]
        identity,
    }))
}

fn config_text(snapshot: &ConfigSnapshot) -> Result<&str, ConfigWriteError> {
    std::str::from_utf8(&snapshot.bytes)
        .map_err(|_| ConfigWriteError::IoError("Settings are not valid UTF-8".into()))
}

fn confirm_snapshot(
    path: &Path,
    original: Option<&ConfigSnapshot>,
) -> Result<(), ConfigWriteError> {
    if config_snapshot(path)?.as_ref() != original {
        return Err(ConfigWriteError::IoError(
            "Settings changed during edit; reread before saving".into(),
        ));
    }
    Ok(())
}

#[path = "editor/draft.rs"]
mod draft;
use draft::ConfigDraft;

fn validate_proposal(source: &str, bun_path: Option<&str>) -> Result<(), ConfigWriteError> {
    match validate_typescript(source, bun_path) {
        Ok(()) => Ok(()),
        Err(e) if e.starts_with("Failed to run bun") => {
            validate_structure(source).map_err(ConfigWriteError::ValidationFailed)
        }
        Err(e) => Err(ConfigWriteError::ValidationFailed(e)),
    }
}

/// Validate a proposed edit without publishing it. Back up the exact original
/// before replacing a nonempty file. An absent file is created exclusively.
pub fn write_config_safely(
    config_path: &Path,
    property: &ConfigProperty,
    bun_path: Option<&str>,
) -> Result<WriteOutcome, ConfigWriteError> {
    edit_config_file(config_path, property, |next| {
        validate_proposal(next, bun_path)
    })
}

fn edit_config_file(
    config_path: &Path,
    property: &ConfigProperty,
    validate: impl FnOnce(&str) -> Result<(), ConfigWriteError>,
) -> Result<WriteOutcome, ConfigWriteError> {
    edit_config_source(
        config_path,
        |content| {
            if content.is_empty() {
                EditResult::Modified(generate_fresh_config(property))
            } else {
                add_property(content, property)
            }
        },
        validate,
    )
}

fn edit_config_source(
    config_path: &Path,
    propose: impl FnOnce(&str) -> EditResult,
    validate: impl FnOnce(&str) -> Result<(), ConfigWriteError>,
) -> Result<WriteOutcome, ConfigWriteError> {
    if let Some(dir) = config_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    let lock = SettingsLock::acquire(config_path)?;
    let original = config_snapshot(config_path)?;
    let content = original
        .as_ref()
        .map(config_text)
        .transpose()?
        .unwrap_or("");
    let next = match propose(content) {
        EditResult::Modified(text) => text,
        EditResult::AlreadySet => {
            lock.confirm()?;
            confirm_snapshot(config_path, original.as_ref())?;
            return Ok(WriteOutcome::AlreadySet);
        }
        EditResult::Failed(why) => return Err(ConfigWriteError::EditFailed(why)),
    };
    if next.len() as u64 > MAX_CONFIG_BYTES {
        return Err(ConfigWriteError::EditFailed(
            "Settings would exceed 2 MiB".into(),
        ));
    }
    validate(&next)?;
    confirm_snapshot(config_path, original.as_ref())?;
    ConfigDraft::prepare(&lock, next.as_bytes())?.publish_edit(config_path, original.as_ref())?;
    Ok(if content.is_empty() {
        WriteOutcome::Created
    } else {
        WriteOutcome::Written
    })
}

/// Enable Claude Code in config.ts using the safe write path.
pub fn enable_claude_code_safely(
    config_path: &Path,
    bun_path: Option<&str>,
) -> Result<WriteOutcome, ConfigWriteError> {
    edit_config_source(config_path, enable_claude_code, |next| {
        validate_proposal(next, bun_path)
    })
}

pub fn write_json_property_safely<T: serde::Serialize>(
    config_path: &Path,
    property_name: &str,
    value: &T,
    bun_path: Option<&str>,
) -> Result<WriteOutcome, ConfigWriteError> {
    let property = ConfigProperty::new(
        property_name,
        serde_json::to_string(value).map_err(|error| {
            ConfigWriteError::EditFailed(format!(
                "Failed to serialize property {}: {}",
                property_name, error
            ))
        })?,
    );
    write_config_safely(config_path, &property, bun_path)
}

/// Explicitly recover config.ts from its backup. Never call this merely because
/// validation rejected a proposed edit: that edit has not changed the file.
pub fn recover_from_backup(
    config_path: &Path,
    bun_path: Option<&str>,
) -> Result<bool, ConfigWriteError> {
    let backup_path = config_path.with_extension("ts.bak");
    let Some(backup) = config_snapshot(&backup_path)? else {
        return Ok(false);
    };
    let lock = SettingsLock::acquire(config_path)?;
    let original = config_snapshot(config_path)?;
    validate_proposal(config_text(&backup)?, bun_path)?;
    let draft = ConfigDraft::prepare(&lock, &backup.bytes)?;
    confirm_snapshot(&backup_path, Some(&backup))?;
    draft.publish(config_path, original.as_ref())?;
    Ok(true)
}

#[cfg(test)]
#[path = "editor/tests.rs"]
mod tests;
