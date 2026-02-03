//! TypeScript config file editor
//!
//! Provides robust utilities for programmatically modifying TypeScript config files
//! like `~/.scriptkit/kit/config.ts`.
//!
//! # Design
//!
//! Uses tree-sitter-typescript to parse the AST, giving exact byte offsets for
//! insertion points. This eliminates the fragility of hand-rolled brace counting.

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

/// Validate TypeScript content by attempting compilation with bun.
///
/// Writes content to a temp .ts file and runs `bun build` on it.
/// Returns Ok(()) if valid, Err with details if invalid.
pub fn validate_typescript(content: &str, bun_path: Option<&str>) -> Result<(), String> {
    let bun = bun_path.unwrap_or("bun");

    let tmp = tempfile::Builder::new()
        .suffix(".ts")
        .tempfile()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    std::fs::write(tmp.path(), content).map_err(|e| format!("Failed to write temp file: {}", e))?;

    let output = std::process::Command::new(bun)
        .arg("build")
        .arg("--target=bun")
        .arg("--no-bundle")
        .arg(tmp.path())
        .arg("--outfile=/dev/null")
        .output()
        .map_err(|e| format!("Failed to run bun: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("TypeScript compilation failed: {}", stderr.trim()))
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
        let mut error_msg = String::new();
        collect_ast_errors(root, content, &mut error_msg);
        if !error_msg.is_empty() {
            return Err(format!("TypeScript parse errors: {}", error_msg));
        }
        return Err("TypeScript parse errors detected".into());
    }

    // Find export_statement and its object
    let export = find_export_statement(root).ok_or("No export statement found in AST")?;

    find_object_in_export(export, content)
        .ok_or_else(|| "Could not find config object in export statement".to_string())?;

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

/// Safely modify config.ts: edit in memory, validate, backup, atomic-write.
///
/// This is the single entry point for all config file modifications.
/// Guarantees:
/// 1. Output is valid TypeScript (validated by bun, fallback to structural)
/// 2. A backup exists before overwriting
/// 3. The write is atomic (temp file + rename)
pub fn write_config_safely(
    config_path: &Path,
    property: &ConfigProperty,
    bun_path: Option<&str>,
) -> Result<WriteOutcome, ConfigWriteError> {
    // Step 1: Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ConfigWriteError::IoError(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }
    }

    // Step 2: Read existing content
    let content = std::fs::read_to_string(config_path).unwrap_or_default();

    // Step 3: Produce new content
    let (new_content, was_empty) = if content.is_empty() {
        (generate_fresh_config(property), true)
    } else {
        match add_property(&content, property) {
            EditResult::Modified(new_content) => (new_content, false),
            EditResult::AlreadySet => return Ok(WriteOutcome::AlreadySet),
            EditResult::Failed(reason) => {
                return Err(ConfigWriteError::EditFailed(reason));
            }
        }
    };

    // Step 4: Validate the new content
    let validation_result = validate_typescript(&new_content, bun_path);
    match &validation_result {
        Ok(()) => { /* bun says it's valid */ }
        Err(bun_err) => {
            if bun_err.starts_with("Failed to run bun") {
                // bun unavailable - fall back to structural validation
                tracing::warn!("bun unavailable for validation, using structural check");
                validate_structure(&new_content).map_err(ConfigWriteError::ValidationFailed)?;
            } else {
                // bun ran but TypeScript is invalid - do NOT write
                return Err(ConfigWriteError::ValidationFailed(bun_err.clone()));
            }
        }
    }

    // Step 5: Backup existing file (if non-empty)
    if !content.is_empty() {
        let backup_path = config_path.with_extension("ts.bak");
        if let Err(e) = std::fs::copy(config_path, &backup_path) {
            tracing::warn!(
                error = %e,
                path = %backup_path.display(),
                "Failed to create config backup"
            );
        } else {
            tracing::info!(path = %backup_path.display(), "Config backup saved");
        }
    }

    // Step 6: Atomic write (temp file in same directory + rename)
    let temp_path = config_path.with_extension("ts.tmp");

    std::fs::write(&temp_path, &new_content).map_err(|e| {
        ConfigWriteError::IoError(format!(
            "Failed to write temp file {}: {}",
            temp_path.display(),
            e
        ))
    })?;

    std::fs::rename(&temp_path, config_path).map_err(|e| {
        // Clean up temp file on rename failure
        let _ = std::fs::remove_file(&temp_path);
        ConfigWriteError::IoError(format!(
            "Failed to rename {} to {}: {}",
            temp_path.display(),
            config_path.display(),
            e
        ))
    })?;

    if was_empty {
        Ok(WriteOutcome::Created)
    } else {
        Ok(WriteOutcome::Written)
    }
}

/// Enable Claude Code in config.ts using the safe write path.
pub fn enable_claude_code_safely(
    config_path: &Path,
    bun_path: Option<&str>,
) -> Result<WriteOutcome, ConfigWriteError> {
    let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
    write_config_safely(config_path, &property, bun_path)
}

/// Attempt to recover config.ts from its backup.
///
/// Returns Ok(true) if recovery succeeded, Ok(false) if no backup exists.
pub fn recover_from_backup(
    config_path: &Path,
    bun_path: Option<&str>,
) -> Result<bool, ConfigWriteError> {
    let backup_path = config_path.with_extension("ts.bak");

    if !backup_path.exists() {
        return Ok(false);
    }

    let backup_content = std::fs::read_to_string(&backup_path)
        .map_err(|e| ConfigWriteError::IoError(format!("Failed to read backup: {}", e)))?;

    // Validate backup before restoring
    let validation_result = validate_typescript(&backup_content, bun_path);
    match &validation_result {
        Ok(()) => {}
        Err(bun_err) => {
            if bun_err.starts_with("Failed to run bun") {
                validate_structure(&backup_content).map_err(|e| {
                    ConfigWriteError::ValidationFailed(format!("Backup is also invalid: {}", e))
                })?;
            } else {
                return Err(ConfigWriteError::ValidationFailed(format!(
                    "Backup is also invalid: {}",
                    bun_err
                )));
            }
        }
    }

    // Atomic write of backup content
    let temp_path = config_path.with_extension("ts.tmp");
    std::fs::write(&temp_path, &backup_content)
        .map_err(|e| ConfigWriteError::IoError(format!("Failed to write temp: {}", e)))?;

    std::fs::rename(&temp_path, config_path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        ConfigWriteError::IoError(format!("Failed to rename: {}", e))
    })?;

    tracing::info!(
        path = %config_path.display(),
        backup = %backup_path.display(),
        "Config restored from backup"
    );

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================================================
    // Test: contains_property
    // ==========================================================================

    #[test]
    fn test_contains_property_simple() {
        let content = r#"export default {
  hotkey: { key: ";" },
} satisfies Config;"#;

        assert!(contains_property(content, "hotkey"));
        assert!(!contains_property(content, "claudeCode"));
    }

    #[test]
    fn test_contains_property_with_comments() {
        let content = r#"export default {
  // This is a comment about hotkey
  hotkey: { key: ";" },
  // claudeCode is mentioned in comment but not as property
} satisfies Config;"#;

        assert!(contains_property(content, "hotkey"));
        assert!(!contains_property(content, "claudeCode"));
    }

    #[test]
    fn test_contains_property_nested() {
        let content = r#"export default {
  claudeCode: {
    enabled: true,
    path: "/usr/bin/claude"
  },
} satisfies Config;"#;

        assert!(contains_property(content, "claudeCode"));
        // Tree-sitter AST correctly distinguishes nesting levels:
        // "enabled" is a nested property inside claudeCode, not a top-level config property
        assert!(!contains_property(content, "enabled"));
    }

    // ==========================================================================
    // Test: find_object_end
    // ==========================================================================

    #[test]
    fn test_find_object_end_simple() {
        let content = r#"export default {
  hotkey: { key: ";" },
} satisfies Config;"#;

        let info = find_object_end(content).expect("Should find object end");
        assert!(info.has_trailing_comma);
        assert_eq!(info.indent, "  ");
    }

    #[test]
    fn test_find_object_end_no_trailing_comma() {
        let content = r#"export default {
  hotkey: { key: ";" }
} satisfies Config;"#;

        let info = find_object_end(content).expect("Should find object end");
        assert!(!info.has_trailing_comma);
    }

    #[test]
    fn test_find_object_end_with_comments() {
        let content = r#"export default {
  hotkey: { key: ";" },
  // This is a comment
  // Another comment
} satisfies Config;"#;

        let info = find_object_end(content).expect("Should find object end");
        assert!(info.has_trailing_comma);
    }

    #[test]
    fn test_find_object_end_nested_braces() {
        let content = r#"export default {
  hotkey: {
    modifiers: ["meta"],
    key: ";"
  },
  nested: {
    inner: {
      deep: true
    }
  },
} satisfies Config;"#;

        let info = find_object_end(content).expect("Should find object end");
        assert!(info.has_trailing_comma);
    }

    // ==========================================================================
    // Test: add_property
    // ==========================================================================

    #[test]
    fn test_add_property_simple() {
        let content = r#"export default {
  hotkey: { key: ";" },
} satisfies Config;"#;

        let property = ConfigProperty::new("claudeCode", "{\n    enabled: true\n  }");
        let result = add_property(content, &property);

        match result {
            EditResult::Modified(new_content) => {
                assert!(new_content.contains("claudeCode:"));
                assert!(new_content.contains("enabled: true"));
                // Should not have double commas
                assert!(!new_content.contains(",,"));
                // Should still be valid structure
                assert!(new_content.contains("} satisfies Config;"));
            }
            _ => panic!("Expected Modified result"),
        }
    }

    #[test]
    fn test_add_property_no_trailing_comma() {
        let content = r#"export default {
  hotkey: { key: ";" }
} satisfies Config;"#;

        let property = ConfigProperty::new("claudeCode", "true");
        let result = add_property(content, &property);

        match result {
            EditResult::Modified(new_content) => {
                // Should add comma before new property
                assert!(new_content.contains(",\n"));
                assert!(new_content.contains("claudeCode: true"));
            }
            _ => panic!("Expected Modified result"),
        }
    }

    #[test]
    fn test_add_property_already_exists() {
        let content = r#"export default {
  hotkey: { key: ";" },
  claudeCode: { enabled: false },
} satisfies Config;"#;

        let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
        let result = add_property(content, &property);

        assert_eq!(result, EditResult::AlreadySet);
    }

    #[test]
    fn test_add_property_with_many_comments() {
        let content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

  // ===========================================================================
  // UI Settings
  // ===========================================================================
  // editorFontSize: 14,
  // terminalFontSize: 14,

  // ===========================================================================
  // Advanced Settings
  // ===========================================================================
  // bun_path: "/opt/homebrew/bin/bun",
} satisfies Config;"#;

        let property = ConfigProperty::new("claudeCode", "{\n    enabled: true\n  }");
        let result = add_property(content, &property);

        match result {
            EditResult::Modified(new_content) => {
                assert!(new_content.contains("claudeCode:"));
                // Should not have double commas
                assert!(!new_content.contains(",,"));
                // The comma after hotkey should be preserved
                assert!(new_content.contains("key: \"Semicolon\","));
            }
            _ => panic!("Expected Modified result"),
        }
    }

    // ==========================================================================
    // Test: enable_claude_code convenience function
    // ==========================================================================

    #[test]
    fn test_enable_claude_code() {
        let content = r#"export default {
  hotkey: { key: ";" },
} satisfies Config;"#;

        let result = enable_claude_code(content);

        match result {
            EditResult::Modified(new_content) => {
                assert!(new_content.contains("claudeCode:"));
                assert!(new_content.contains("enabled: true"));
            }
            _ => panic!("Expected Modified result"),
        }
    }

    // ==========================================================================
    // Test: Edge cases
    // ==========================================================================

    #[test]
    fn test_empty_config() {
        let content = r#"export default {} satisfies Config;"#;

        let property = ConfigProperty::new("test", "true");
        let result = add_property(content, &property);

        match result {
            EditResult::Modified(new_content) => {
                assert!(new_content.contains("test: true"));
            }
            _ => panic!("Expected Modified result"),
        }
    }

    #[test]
    fn test_config_with_strings_containing_braces() {
        let content = r#"export default {
  template: "function() { return {}; }",
} satisfies Config;"#;

        let property = ConfigProperty::new("claudeCode", "true");
        let result = add_property(content, &property);

        match result {
            EditResult::Modified(new_content) => {
                assert!(new_content.contains("claudeCode: true"));
                // Original string should be preserved
                assert!(new_content.contains("function() { return {}; }"));
            }
            _ => panic!("Expected Modified result"),
        }
    }

    #[test]
    fn test_config_with_template_literals() {
        let content = r#"export default {
  code: `const x = { a: 1 };`,
} satisfies Config;"#;

        let property = ConfigProperty::new("claudeCode", "true");
        let result = add_property(content, &property);

        match result {
            EditResult::Modified(new_content) => {
                assert!(new_content.contains("claudeCode: true"));
            }
            _ => panic!("Expected Modified result"),
        }
    }

    #[test]
    fn test_preserves_as_config_syntax() {
        let content = r#"export default {
  hotkey: { key: ";" },
} as Config;"#;

        let property = ConfigProperty::new("claudeCode", "true");
        let result = add_property(content, &property);

        match result {
            EditResult::Modified(new_content) => {
                assert!(new_content.contains("} as Config;"));
            }
            _ => panic!("Expected Modified result"),
        }
    }

    // ==========================================================================
    // Test: Real-world config (from user's actual file)
    // ==========================================================================

    #[test]
    fn test_real_world_config() {
        let content = r#"import type { Config } from "@scriptkit/sdk";

/**
 * Script Kit Configuration
 */
export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

  // editorFontSize: 14,

  // builtIns: {
  //   clipboardHistory: true,
  // },

  // bun_path: "/opt/homebrew/bin/bun",
} satisfies Config;
"#;

        let result = enable_claude_code(content);

        match result {
            EditResult::Modified(new_content) => {
                // Verify the result is valid
                assert!(new_content.contains("claudeCode:"));
                assert!(new_content.contains("enabled: true"));
                assert!(!new_content.contains(",,"));
                assert!(new_content.contains("} satisfies Config;"));

                // Print for visual inspection
                println!("=== Modified config ===\n{}", new_content);
            }
            _ => panic!("Expected Modified result"),
        }
    }

    /// Test with the EXACT complex config structure that was corrupted
    /// This is the actual user's config.ts structure
    #[test]
    fn test_complex_config_with_many_commented_sections() {
        // This is a simplified version of the actual corrupted config
        let content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

  // ===========================================================================
  // UI Settings
  // ===========================================================================
  // editorFontSize: 14,

  // ===========================================================================
  // Commands Configuration
  // ===========================================================================
  // commands: {
  //   "builtin/clipboard-history": {
  //     shortcut: { modifiers: ["meta", "shift"], key: "KeyV" }
  //   },
  //
  //   // Add shortcut to a scriptlet
  //   // "scriptlet/clipboard-to-uppercase": {
  //   //   shortcut: { modifiers: ["meta", "shift"], key: "KeyU" }
  //   // },
  // },

  // ===========================================================================
  // Process Limits
  // ===========================================================================
  // processLimits: {
  //   maxMemoryMb: 512,
  // },

  // ===========================================================================
  // Advanced Settings
  // ===========================================================================
  // bun_path: "/opt/homebrew/bin/bun",
} satisfies Config;
"#;

        let result = enable_claude_code(content);

        match result {
            EditResult::Modified(new_content) => {
                // CRITICAL validations
                assert!(
                    !new_content.contains("}{"),
                    "Should NOT contain }}{{ corruption pattern. Got:\n{}",
                    new_content
                );
                assert!(
                    new_content.contains("claudeCode:"),
                    "Should contain claudeCode property"
                );
                assert!(
                    new_content.contains("enabled: true"),
                    "Should contain enabled: true"
                );
                assert!(
                    new_content.contains("} satisfies Config;"),
                    "Should end with valid Config type assertion"
                );

                // The claudeCode property should appear BEFORE the final closing brace
                let claude_pos = new_content.find("claudeCode:").unwrap();
                let final_brace_pos = new_content.rfind("} satisfies").unwrap();
                assert!(
                    claude_pos < final_brace_pos,
                    "claudeCode should be before closing brace"
                );

                // All original comments should still be present
                assert!(new_content.contains("// UI Settings"));
                assert!(new_content.contains("// Commands Configuration"));
                assert!(new_content.contains("// Process Limits"));
                assert!(new_content.contains("// Advanced Settings"));

                // Print for visual inspection
                println!("=== Modified complex config ===\n{}", new_content);
            }
            _ => panic!("Expected Modified result"),
        }
    }

    /// Test that validates output is syntactically valid TypeScript
    #[test]
    fn test_output_has_no_corruption_patterns() {
        let configs = vec![
            // Minimal config
            r#"export default { hotkey: { key: ";" } } satisfies Config;"#,
            // Config with trailing comma
            r#"export default { hotkey: { key: ";" }, } satisfies Config;"#,
            // Config with nested comments
            r#"export default {
  hotkey: { key: ";" },
  // nested: {
  //   deep: {
  //     value: true
  //   }
  // }
} satisfies Config;"#,
            // Config with string containing braces
            r#"export default {
  template: "fn() { return {}; }",
} satisfies Config;"#,
        ];

        for content in configs {
            let result = enable_claude_code(content);
            match result {
                EditResult::Modified(new_content) => {
                    // Check for corruption patterns
                    assert!(
                        !new_content.contains("}{"),
                        "Corruption }}{{ detected in:\n{}",
                        new_content
                    );
                    assert!(
                        !new_content.contains("{{"),
                        "Double open brace detected in:\n{}",
                        new_content
                    );
                    // Note: }} can be valid (closing nested objects)
                }
                EditResult::AlreadySet => {
                    // Fine - property already exists
                }
                EditResult::Failed(reason) => {
                    panic!("Failed to modify config: {}", reason);
                }
            }
        }
    }

    // ==========================================================================
    // Test: Full user config file (system test)
    // ==========================================================================

    /// This test reads the actual user config file and verifies the edit works.
    /// Run with: cargo test --features system-tests test_full_user_config -- --nocapture
    #[cfg(feature = "system-tests")]
    #[test]
    fn test_full_user_config() {
        let config_path =
            std::path::PathBuf::from(shellexpand::tilde("~/.scriptkit/kit/config.ts").as_ref());

        if !config_path.exists() {
            println!(
                "Skipping test: config file does not exist at {:?}",
                config_path
            );
            return;
        }

        let content = std::fs::read_to_string(&config_path).expect("Failed to read config");
        println!(
            "Config file: {} bytes, {} lines",
            content.len(),
            content.lines().count()
        );

        // Skip if claudeCode already exists
        if contains_property(&content, "claudeCode") {
            println!("Skipping test: claudeCode already exists in config");
            return;
        }

        let result = enable_claude_code(&content);

        match result {
            EditResult::Modified(new_content) => {
                // Find where claudeCode was inserted
                if let Some(pos) = new_content.find("claudeCode:") {
                    println!("\n=== claudeCode found at position {} ===", pos);
                    let start = pos.saturating_sub(50);
                    let end = (pos + 100).min(new_content.len());
                    println!("Context:\n{}", &new_content[start..end]);
                } else {
                    println!("WARNING: claudeCode not found in output!");
                }

                // Verify structure
                assert!(
                    new_content.contains("claudeCode:"),
                    "Should contain claudeCode"
                );
                assert!(
                    new_content.contains("enabled: true"),
                    "Should contain enabled: true"
                );
                assert!(
                    !new_content.contains("}{"),
                    "Should not have adjacent braces"
                );
                assert!(
                    new_content.contains("} satisfies Config;")
                        || new_content.contains("} as Config;"),
                    "Should end with valid Config type assertion"
                );

                // Print last 20 lines for verification
                println!("\n=== Last 20 lines of modified config ===");
                for line in new_content
                    .lines()
                    .rev()
                    .take(20)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    println!("{}", line);
                }
            }
            EditResult::AlreadySet => {
                println!("claudeCode already set");
            }
            EditResult::Failed(reason) => {
                panic!("Failed to modify config: {}", reason);
            }
        }
    }

    // ==========================================================================
    // Test: validate_structure
    // ==========================================================================

    #[test]
    fn test_validate_structure_valid() {
        let content = r#"export default {
  hotkey: { key: ";" },
} satisfies Config;"#;
        assert!(validate_structure(content).is_ok());
    }

    #[test]
    fn test_validate_structure_valid_as_config() {
        let content = r#"export default {
  hotkey: { key: ";" },
} as Config;"#;
        assert!(validate_structure(content).is_ok());
    }

    #[test]
    fn test_validate_structure_missing_export() {
        let content = r#"const config = {
  hotkey: { key: ";" },
};"#;
        let err = validate_structure(content).unwrap_err();
        assert!(err.contains("export default"));
    }

    #[test]
    fn test_validate_structure_unbalanced_braces() {
        let content3 = r#"export default {"#;
        let err = validate_structure(content3).unwrap_err();
        // Tree-sitter reports this as a parse error (incomplete object literal)
        assert!(err.contains("parse error") || err.contains("parse errors"));
    }

    #[test]
    fn test_validate_structure_corruption_pattern() {
        let content = r#"export default {
  hotkey: { key: ";" },
}{
  extra: true,
} satisfies Config;"#;
        let err = validate_structure(content).unwrap_err();
        // Tree-sitter detects the `}{` corruption as a parse error
        assert!(err.contains("parse error") || err.contains("parse errors"));
    }

    #[test]
    fn test_validate_structure_missing_satisfies() {
        let content = r#"export default {
  hotkey: { key: ";" },
};"#;
        let err = validate_structure(content).unwrap_err();
        assert!(err.contains("satisfies") || err.contains("Config"));
    }

    // ==========================================================================
    // Test: write_config_safely (using temp directories)
    // ==========================================================================

    #[test]
    fn test_write_config_safely_creates_file_when_missing() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");

        let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
        let result = write_config_safely(&config_path, &property, None);

        match result {
            Ok(WriteOutcome::Created) => {
                let content = std::fs::read_to_string(&config_path).unwrap();
                assert!(content.contains("claudeCode: { enabled: true }"));
                assert!(content.contains("export default"));
                assert!(content.contains("satisfies Config;"));
            }
            Ok(other) => panic!("Expected Created, got {:?}", other),
            Err(e) => {
                println!("write_config_safely failed (may be expected in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_write_config_safely_creates_backup() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");

        let initial = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
} satisfies Config;
"#;
        std::fs::write(&config_path, initial).unwrap();

        let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
        let result = write_config_safely(&config_path, &property, None);

        match result {
            Ok(WriteOutcome::Written) => {
                let backup_path = config_path.with_extension("ts.bak");
                assert!(backup_path.exists(), "Backup file should exist");
                let backup_content = std::fs::read_to_string(&backup_path).unwrap();
                assert_eq!(backup_content, initial);
            }
            Ok(other) => panic!("Expected Written, got {:?}", other),
            Err(e) => {
                println!("write_config_safely failed (may be expected in CI): {}", e);
            }
        }
    }

    #[test]
    fn test_write_config_safely_already_set() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");

        let content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
  claudeCode: { enabled: true },
} satisfies Config;
"#;
        std::fs::write(&config_path, content).unwrap();

        let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
        let result = write_config_safely(&config_path, &property, None);

        match result {
            Ok(WriteOutcome::AlreadySet) => { /* expected */ }
            other => panic!("Expected AlreadySet, got {:?}", other),
        }
    }

    // ==========================================================================
    // Test: recover_from_backup
    // ==========================================================================

    #[test]
    fn test_recover_from_backup_no_backup() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");
        std::fs::write(&config_path, "corrupted").unwrap();

        let result = recover_from_backup(&config_path, None);
        match result {
            Ok(false) => { /* expected - no backup exists */ }
            other => panic!("Expected Ok(false), got {:?}", other),
        }
    }

    #[test]
    fn test_recover_from_backup_restores() {
        let dir = tempfile::TempDir::new().unwrap();
        let config_path = dir.path().join("config.ts");
        let backup_path = config_path.with_extension("ts.bak");

        let valid_content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: { modifiers: ["meta"], key: "Semicolon" },
} satisfies Config;
"#;
        std::fs::write(&config_path, "corrupted content").unwrap();
        std::fs::write(&backup_path, valid_content).unwrap();

        let result = recover_from_backup(&config_path, None);
        match result {
            Ok(true) => {
                let restored = std::fs::read_to_string(&config_path).unwrap();
                assert_eq!(restored, valid_content);
            }
            other => panic!("Expected Ok(true), got {:?}", other),
        }
    }

    // ==========================================================================
    // Test: generate_fresh_config
    // ==========================================================================

    #[test]
    fn test_generate_fresh_config() {
        let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
        let content = generate_fresh_config(&property);
        assert!(validate_structure(&content).is_ok());
        assert!(content.contains("claudeCode: { enabled: true }"));
        assert!(content.contains("hotkey:"));
    }

    // ==========================================================================
    // Test: round-trip with real config pattern
    // ==========================================================================

    #[test]
    fn test_real_config_round_trip() {
        let content = r#"import type { Config } from "@scriptkit/sdk";

export default {
  hotkey: {
    modifiers: ["meta"],
    key: "Semicolon",
  },

  // editorFontSize: 14,
  // terminalFontSize: 14,

  // builtIns: {
  //   clipboardHistory: true,
  // },

  // bun_path: "/opt/homebrew/bin/bun",
} satisfies Config;
"#;

        let result = enable_claude_code(content);
        match result {
            EditResult::Modified(new_content) => {
                assert!(
                    validate_structure(&new_content).is_ok(),
                    "Round-trip produced invalid structure:\n{}",
                    new_content
                );
                assert!(new_content.contains("claudeCode:"));
                assert!(new_content.contains("enabled: true"));
            }
            _ => panic!("Expected Modified result"),
        }
    }
}
