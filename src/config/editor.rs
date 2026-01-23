//! TypeScript config file editor
//!
//! Provides robust utilities for programmatically modifying TypeScript config files
//! like `~/.scriptkit/kit/config.ts`.
//!
//! # Design
//!
//! Rather than using fragile regex, this module:
//! 1. Parses the config structure to find insertion points
//! 2. Handles trailing commas, comments, and whitespace properly
//! 3. Preserves formatting and comments in the original file

use std::path::Path;

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

/// Check if the config contains a property with the given name
pub fn contains_property(content: &str, property_name: &str) -> bool {
    // Look for the property name followed by a colon (accounting for whitespace)
    // This is a simple check - we look for the pattern at the start of a line or after whitespace
    let pattern = format!(r"(?m)^\s*{}\s*:", regex::escape(property_name));
    regex::Regex::new(&pattern)
        .map(|re| re.is_match(content))
        .unwrap_or(false)
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

/// Find the end of the `export default { ... }` object
fn find_object_end(content: &str) -> Option<InsertInfo> {
    // Find "export default {" to locate the config object
    let export_start = content.find("export default")?;
    let open_brace = content[export_start..].find('{')? + export_start;

    // Track brace depth to find the matching close brace
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = ' ';
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut last_significant_char = ' ';
    let mut last_significant_pos = open_brace;

    let chars: Vec<char> = content.chars().collect();
    let mut i = open_brace;

    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied().unwrap_or(' ');

        // Handle line comments
        if !in_string && !in_block_comment && c == '/' && next == '/' {
            in_line_comment = true;
            i += 2;
            continue;
        }
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }

        // Handle block comments
        if !in_string && !in_line_comment && c == '/' && next == '*' {
            in_block_comment = true;
            i += 2;
            continue;
        }
        if in_block_comment {
            if c == '*' && next == '/' {
                in_block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }

        // Handle strings
        if !in_string && (c == '"' || c == '\'' || c == '`') {
            in_string = true;
            string_char = c;
            i += 1;
            continue;
        }
        if in_string {
            if c == string_char && (i == 0 || chars[i - 1] != '\\') {
                in_string = false;
            }
            i += 1;
            continue;
        }

        // Track braces
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                // Found the closing brace
                let has_trailing_comma = last_significant_char == ',';

                // Determine indentation (typically 2 spaces for this config style)
                let indent = detect_indent(content, open_brace);

                return Some(InsertInfo {
                    close_brace_pos: i,
                    has_trailing_comma,
                    indent,
                });
            }
        }

        // Track last significant (non-whitespace) character
        if !c.is_whitespace() {
            last_significant_char = c;
            last_significant_pos = i;
        }

        i += 1;
    }

    // Also check if we need to track last_significant_pos for unused warning
    let _ = last_significant_pos;

    None
}

/// Detect the indentation used in the config file
fn detect_indent(content: &str, after_pos: usize) -> String {
    // Look for the first property after the opening brace to detect indent
    let rest = &content[after_pos + 1..];
    for line in rest.lines().skip(1) {
        // Skip empty lines and comments
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }
        // Found a content line - extract leading whitespace
        let leading: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        if !leading.is_empty() {
            return leading;
        }
    }
    // Default to 2 spaces
    "  ".to_string()
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
pub fn enable_claude_code(content: &str) -> EditResult {
    // Use inline format for cleaner insertion
    // The trailing comma is added by insert_property
    let property = ConfigProperty::new("claudeCode", "{ enabled: true }");
    add_property(content, &property)
}

/// Read, modify, and write a config file
#[allow(dead_code)]
pub fn modify_config_file(path: &Path, property: &ConfigProperty) -> Result<EditResult, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let result = add_property(&content, property);

    if let EditResult::Modified(ref new_content) = result {
        std::fs::write(path, new_content)
            .map_err(|e| format!("Failed to write config: {}", e))?;
    }

    Ok(result)
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
        // Note: This simple check doesn't distinguish nesting levels.
        // For our use case (checking top-level config properties), this is acceptable
        // since we only check for properties we know should be top-level.
        // A full TypeScript parser would be needed for proper nesting detection.
        assert!(contains_property(content, "enabled")); // It matches, but we don't use it this way
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
        let config_path = std::path::PathBuf::from(
            shellexpand::tilde("~/.scriptkit/kit/config.ts").as_ref(),
        );

        if !config_path.exists() {
            println!("Skipping test: config file does not exist at {:?}", config_path);
            return;
        }

        let content = std::fs::read_to_string(&config_path).expect("Failed to read config");
        println!("Config file: {} bytes, {} lines", content.len(), content.lines().count());

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
                assert!(new_content.contains("claudeCode:"), "Should contain claudeCode");
                assert!(new_content.contains("enabled: true"), "Should contain enabled: true");
                assert!(!new_content.contains("}{"), "Should not have adjacent braces");
                assert!(
                    new_content.contains("} satisfies Config;") || new_content.contains("} as Config;"),
                    "Should end with valid Config type assertion"
                );

                // Print last 20 lines for verification
                println!("\n=== Last 20 lines of modified config ===");
                for line in new_content.lines().rev().take(20).collect::<Vec<_>>().into_iter().rev() {
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
}
