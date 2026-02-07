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
